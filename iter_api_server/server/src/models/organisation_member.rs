use castle_api::types::State;
use mongodb::bson::{oid::ObjectId, doc, bson};
use serde::{Serialize, Deserialize};

use super::{utils::model::Model, User};


#[derive(Debug, Serialize, Deserialize)]
pub struct OrganisationMember {
    pub _id: ObjectId,
    email: String,
    pub profiles: Vec<ObjectId>,
    pub organisation_id: ObjectId,
    user_id: ObjectId,
    date_joined: mongodb::bson::DateTime,
}

impl Model for OrganisationMember {
    fn collection_name() ->  & 'static str {
        "organisation_members"
    }
}


/// ```
/// struct OrganisationMember {
///     _id: ObjectID,
///     organisation_id: ObjectID,
///     date_joined: Date,
///     date_invited: Date,
///     email: String,
///     user_id: ObjectID,
///     profiles: Vec<ObjectID>,
/// }
/// ```
impl OrganisationMember {
    pub async fn create_organisation_member(
        state: &State,
        organisation_id: ObjectId,
        profiles: Vec<ObjectId>
    ) -> Result<ObjectId, anyhow::Error> {
        let user = state.borrow::<User>();

        let org_member_id = OrganisationMember::create(state, bson!({
            "organisation_id": organisation_id,
            "date_joined": mongodb::bson::DateTime::now(),
            "date_invited": mongodb::bson::DateTime::now(), // this needs to use the actual date invited
            "email": user.email.clone(),
            "user_id": user._id.clone(),
            "profiles": profiles
        })).await?;

        Ok(org_member_id)
    }
}

#[castle_api::castle_macro(Type)]
impl OrganisationMember {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_hex()
    }
    pub fn email(&self, _state: &State) -> &str {
        &self.email
    }
    pub fn organisation_id(&self, _state: &State) -> String {
        self.organisation_id.to_hex()
    }

    pub fn user_id(&self, _state: &State) -> String {
        self.user_id.to_hex()
    }
    pub fn date_joined(&self, _state: &State) -> String {
        self.date_joined.to_string()
    }
}