use std::{time::SystemTime, sync::Arc};

use mongodb::{bson::bson};

use castle_api::types::State;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use mongodb::{bson::{oid::ObjectId, doc}};
use secrets::BackendSecrets;
use serde::{Deserialize, Serialize};

use super::{utils::model::Model, OrganisationMember, Organisation, Profile, sidebar::Sidebar};

/// We only load the basic info from the db and metadata needed for the user for authentication,
/// logging, and other purposes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub _id: ObjectId,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    pub recently_used_icons: Vec<String>, //max 10
}

#[castle_api::castle_macro(Type)]
impl User {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_hex()
    }

    pub fn email(&self, _state: &State) -> &str {
        &self.email
    }

    pub async fn update_email(&self, state: &State, email: &str) -> Result<(), anyhow::Error> {
        User::update(state, &self._id, doc!{
            "$set": {
                "email": email
            }
        }).await?;
        Ok(())
    }

    pub async fn first_name(&self, _state: &State) -> String {
        self.first_name.to_string()
    }

    pub async fn update_first_name(&self, _state: &State, first_name: &str) -> Result<(), anyhow::Error> {
        User::update(_state, &self._id, doc!{
            "$set": {
                "first_name": first_name
            }
        }).await?;
        Ok(())
    }

    pub async fn last_name(&self, _state: &State) -> String {
        self.last_name.to_string()
    }

    pub async fn update_last_name(&self, _state: &State, last_name: &str) -> Result<(), anyhow::Error> {
        User::update(_state, &self._id, doc!{
            "$set": {
                "last_name": last_name
            }
        }).await?;
        Ok(())
    }

    pub async fn update_password(
        &self,
        state: &State,
        new_password: &str
    ) -> Result<(), anyhow::Error> {
        User::update(state, &self._id, doc!{
            "$set": {
                "password": bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
                    .map_err(|_| anyhow::anyhow!{"Failed to hash user password"})?
            }
        }).await?;
        Ok(())
    }

    pub async fn organisations(&self, state: &State) -> Result<Vec<Organisation>, anyhow::Error> {
        let org_members: Vec<OrganisationMember> = OrganisationMember::find_many(
            state,
            doc!{
                "user_id": self._id,
            },
            100
        ).await?;

        Ok(Organisation::find_many(
            state,
            doc!{
                "_id": {
                    "$in": org_members.into_iter().map(|member| member.organisation_id).collect::<Vec<ObjectId>>()
                }
            },
            100
        ).await?)
    }

    pub async fn profiles(&self, state: &State) -> Result<Vec<Profile>, anyhow::Error> {
        let org_members: Vec<OrganisationMember> = OrganisationMember::find_many(
            state,
            doc!{
                "user_id": self._id,
            },
            100
        ).await?;

        Ok(Profile::find_many(
            state,
            doc!{
                "_id": {
                    "$in": org_members.into_iter().map(|member| member.profiles).flatten().collect::<Vec<ObjectId>>()
                }
            },
            100
        ).await?)
    }

    pub fn recently_used_icons(&self, _state: &State) -> Vec<String> {
        self.recently_used_icons.clone()
    }

    pub async fn update_recently_used_icons(&self, _state: &State, icon: &str) -> Result<(), anyhow::Error> {
        let mut recently_used_icons = self.recently_used_icons.clone();
        if recently_used_icons.contains(&icon.to_string()) {
            recently_used_icons.remove(recently_used_icons.iter().position(|x| x == icon).unwrap());
        }
        recently_used_icons.insert(0, icon.to_string());
        if recently_used_icons.len() > 10 {
            recently_used_icons.pop();
        }
        User::update(_state, &self._id, doc!{
            "$set": {
                "recently_used_icons": recently_used_icons
            }
        }).await?;
        Ok(())
    }
}

impl Model for User {
    fn collection_name() -> &'static str {
        "users"
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenData {
    user_id: ObjectId,
    created: SystemTime,
}

impl User {
    /// Authenticates a user from a token and returns the basic user info.
    /// If the token is invalid, returns a AuthenticationError.
    ///
    /// The token is a JWT token that is signed with a secret key.
    pub async fn authenticate_from_token(
        token: &str,
        secret: &str,
        state: &State,
    ) -> Result<User, anyhow::Error> {
        // We want to use a "sliding window" token so there is no direct expiry time.
        // We use the database to store the "last used" time of the token.
        // This means if a user constantly uses the same token they will not be logged out.

        let mut validation = Validation::new(Algorithm::HS256);
        // remove default required_spec_claims
        validation.set_required_spec_claims::<&str>(&[]);
        // disable expiry valiation
        validation.validate_exp = false;

        match jsonwebtoken::decode::<TokenData>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        ) {
            Ok(decoded) => User::find_by_id(decoded.claims.user_id, state).await,
            Err(e) => {
                tracing::error!("{}", e);
                Err(anyhow::anyhow!("Invalid authentication token"))
            }
        }
    }

    pub async fn login(
        state: &State,
        email: &str,
        password: &str,
    ) -> Result<String, anyhow::Error> {
        let secrets = state.borrow::<Arc<BackendSecrets>>();
        #[derive(Serialize, Deserialize)]
        struct PrivateUser {
            _id: ObjectId,
            password: String,
        }
        tracing::info!("Login attempt by: {}", email);

        let user = User::find_one::<PrivateUser>(state, doc!{
            "email": email,
        }).await?;

        match user {
            Some(user) => {
                match bcrypt::verify(password, &user.password) {
                    Ok(true) => tracing::info!("Login Success: {}", email),
                    Ok(false) | Err(_) => {
                        tracing::error!("Login attempt with password mismatch");
                        return Err(anyhow::anyhow!("Invalid password"));
                    }
                }

                match jsonwebtoken::encode(
                    &jsonwebtoken::Header::default(),
                    &TokenData {
                        user_id: user._id,
                        created: SystemTime::now(),
                    },
                    &EncodingKey::from_secret(secrets.jwt_secret.as_bytes()),
                ) {
                    Ok(token) => Ok(token),
                    Err(_) => Err(anyhow::anyhow!("Failed to generate token")),
                }
            }
            None => {
                tracing::info!("User Not Found: {}", email);
                Err(anyhow::anyhow!("User not found"))
            }
        }
    }

    pub async fn create_user(
        state: &State,
        email: &str,
        password: &str,
        first_name: &str,
        last_name: &str
    ) -> Result<ObjectId, anyhow::Error> {
        User::create(state, bson!({
            "email": email,
            "password": bcrypt::hash(password, bcrypt::DEFAULT_COST)
                .map_err(|_| anyhow::anyhow!{"Failed to hash user password"})?,
            "first_name": first_name,
            "last_name": last_name,
            "recently_used_icons": Vec::<String>::new(),
        })).await
    }
}


