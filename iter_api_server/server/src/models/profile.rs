use std::str::FromStr;

use castle_api::types::State;
use mongodb::bson::{oid::ObjectId, doc, bson};
use serde::{Serialize, Deserialize};

use super::{utils::model::Model, OrganisationMember, Organisation, profile_nickname::ProfileNickname, User};

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    _id: ObjectId,
    name: String,
    display_picture: Option<ObjectId>,
    organisation_id: ObjectId,
    roles: Vec<String>,
}

impl Model for Profile {
    fn collection_name() ->  & 'static str {
        "profiles"
    }
}

///```
/// struct Profile {
///     _id: ObjectID,
///     name: String,
///     display_picture: Option<ObjectID>,
///     organisation_id: ObjectID,
///     roles: Vec<String>
/// }
///```
impl Profile {
    //create the profile & return object ID
    //we may need to add update the member profile list!!!
    pub async fn create_profile(
        state: &State,
        name: &str,
        display_picture: Option<ObjectId>,
        organisation_id: ObjectId,
        roles: Vec<String>,
    ) -> Result<ObjectId, anyhow::Error> {
        let profile_id = Profile::create(state, bson!({
            "name": name,
            "display_picture": display_picture,
            "organisation_id": organisation_id,
            "roles": roles
        })).await?;

        Ok(profile_id)
    }

    pub async fn create_profile_in_org_settings(
        state: &State,
        organisation_id: ObjectId,
        name: &str,
        roles: Vec<String>,
        members: Vec<ObjectId>,
    ) -> Result<ObjectId, anyhow::Error> {
        let profile_id = Profile::create(state, bson!({
            "name": name,
            "organisation_id": organisation_id,
            "roles": roles
        })).await?;

        for member_id in members {
            let member: OrganisationMember = OrganisationMember::find_by_id(member_id, state).await?;
            let mut member_profiles = member.profiles;
            member_profiles.push(profile_id.clone());
            OrganisationMember::update(state, &member_id, doc! {
                "$set": {
                    "profiles": member_profiles
                }
            }).await?;
        }

        Ok(profile_id)
    }
}


#[castle_api::castle_macro(Type)]
impl Profile {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_hex()
    }
    pub fn name(&self, _state: &State) -> &str {
        &self.name
    }

    pub async fn update_name(&self, state: &State, name: &str) -> Result<(), anyhow::Error> {
        Profile::update(state, &self._id, doc!{
            "$set": {
                "name": name
            }
        }).await?;
        Ok(())
    }

    //TODO: fix option being able to be resolved
    pub fn display_picture(&self, _state: &State) -> String {
        match &self.display_picture {
            Some(id) => id.to_string(),
            None => "".to_string()
        }
    }

    pub fn organisation_id(&self, _state: &State) -> String {
        self.organisation_id.to_string()
    }

    pub fn roles(&self, _state: &State) -> Vec<&str> {
        self.roles.iter().map(|id| id.as_str()).collect()
    }

    pub async fn update_roles(&self, state: &State, roles: Vec<String>) -> Result<(), anyhow::Error> {
        Profile::update(state, &self._id, doc!{
            "$set": {
                "roles": roles
            }
        }).await?;
        Ok(())
    }

    pub async fn members(&self, state: &State) -> Result<Vec<OrganisationMember>, anyhow::Error> {
        let org_members: Vec<OrganisationMember> = OrganisationMember::find_many(state, doc!{
            "organisation_id": self.organisation_id.clone()
        },
        100).await?;

        let profiles_members = org_members.into_iter().
            filter(|member| member.profiles.contains(&self._id)).collect::<Vec<OrganisationMember>>();
        Ok(profiles_members)
    }

    pub async fn update_members(&self, state: &State, members: Vec<String>) -> Result<(), anyhow::Error> {
        let organisation: Organisation = Organisation::find_by_id(self.organisation_id, state).await?;
        let every_member_in_org = Organisation::members(&organisation, state).await?;
        let new_profiles_members: Vec<ObjectId> = members.into_iter().map(|member| ObjectId::from_str(&member).unwrap()).collect();
        for member in every_member_in_org {
            if member.profiles.contains(&self._id) {
                if !new_profiles_members.contains(&member._id) {
                    OrganisationMember::update(state, &member._id, doc!{
                        "$set": {
                            "profiles": member.profiles.iter().filter(|&x| x != &self._id).cloned().collect::<Vec<ObjectId>>()
                        }
                    }).await?;
                }
            } else {
                if new_profiles_members.contains(&member._id) {
                    OrganisationMember::update(state, &member._id, doc!{
                        "$push": {
                            "profiles": self._id.clone()
                        }
                    }).await?;
                }
            }
        }

        Ok(())
    }

    // Gets the profile nickname for the current user
    pub(crate) async fn nickname(&self, state: &State) -> Result<String, anyhow::Error> {
        Ok(
            ProfileNickname::find_one(state, doc!{
                "profile_id": self._id.clone(),
                "user_id": state.borrow::<User>()._id
            }).await?
            .map(|nickname: ProfileNickname| nickname.name(&state).to_string())
            .unwrap_or("".to_string())
        )
    }

    async fn update_nickname(
        &self,
        state: &State,
        name: &str
    ) -> Result<(), anyhow::Error> {
        let user_id = state.borrow::<User>()._id;

        let existing_nickname: Option<ProfileNickname> = ProfileNickname::find_one(state, doc!{
            "profile_id": self._id,
            "user_id": user_id
        }).await?;

        match existing_nickname {
            Some(nickname) => {
                ProfileNickname::update(state, &nickname._id, doc!{
                    "$set": {
                        "name": name
                    }
                }).await?;
            },
            None => {
                ProfileNickname::create(state, bson!({
                    "profile_id": self._id,
                    "user_id": user_id,
                    "name": name
                })).await?;
            }
        };
        Ok(())
    }
}