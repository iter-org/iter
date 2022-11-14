use castle_api::types::State;
use mongodb::{bson::{doc, oid::ObjectId, bson},};
use serde::{Deserialize, Serialize};
use super::{User, utils::{model::Model}, OrganisationMember};

#[derive(Debug, Serialize, Deserialize)]
pub struct Organisation {
    _id: ObjectId,
    name: String,
    //date_created: Date,
    //created_by: ObjectID,
    // display_picture: Option<String>,
    legal_name: String,
    billing_address: String,
}

impl Model for Organisation {
    fn collection_name() ->  &'static str {
        "organisations"
    }
}

impl Organisation {
    /// Create a new organisation in the database.
    ///
    /// ## Algorithm
    /// 1. Create the [Organisation] with required information
    /// 2. Create the first [Profile] and attach it to the organisation
    /// 3. Create the first [OrganisationMember] and attach it to the organisation
    /// 4. Return the [Organisation] [ObjectId]
    // directive authenticated
    pub async fn create_organisation(state: &State, name: &str) -> Result<ObjectId, anyhow::Error> {
        let user = state.borrow::<User>();

        let organisation_id = Organisation::create(state, bson!({
            "name": name,
            "created_by": user._id,
            "date_created": mongodb::bson::DateTime::now(),
            "display_picture": None::<ObjectId>,
            "legal_name": "",
            "billing_address": "",
        })).await?;

        OrganisationMember::create(state, bson!({
            "organisation_id": organisation_id,
            "date_joined": mongodb::bson::DateTime::now(),
            "date_invited": mongodb::bson::DateTime::now(),
            "email": user.email.clone(),
            "user_id": user._id,
            "users": vec![user._id(&state)]
        })).await?;

        Ok(organisation_id)
    }

}

#[castle_api::castle_macro(Type)]
impl Organisation {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_hex()
    }
    pub fn name(&self, _state: &State) -> &str {
        &self.name
    }

    pub async fn update_name(&self, state: &State, name: &str) -> Result<(), anyhow::Error> {
        Organisation::update(state, &self._id, doc!{
            "$set": {
                "name": name
            }
        }).await?;
        Ok(())
    }

    pub fn legal_name(&self, _state: &State) -> &str {
        &self.legal_name
    }

    pub async fn update_legal_name(&self, state: &State, legal_name: &str) -> Result<(), anyhow::Error> {
        Organisation::update(state, &self._id, doc!{
            "$set": {
                "legal_name": legal_name
            }
        }).await?;
        Ok(())
    }

    pub fn billing_address(&self, _state: &State) -> &str {
        &self.billing_address
    }

    pub async fn update_billing_address(&self, state: &State, billing_address: &str) -> Result<(), anyhow::Error> {
        Organisation::update(state, &self._id, doc!{
            "$set": {
                "billing_address": billing_address
            }
        }).await?;
        Ok(())
    }

    pub async fn members(&self, state: &State) -> Result<Vec<OrganisationMember>, anyhow::Error> {
        OrganisationMember::find_many(
            state,
            doc!{

                "organisation_id": self._id,
            },
            100
        ).await
    }

    pub async fn member_count(&self, state: &State) -> Result<u64, anyhow::Error> {
        OrganisationMember::count(state, doc!{
            "organisation_id": self._id,
        }).await
    }
    // pub fn profile_picture(&self, _state: &State) -> Option<String> {
    //     self.display_picture.clone()
    // }


    // adding member is done in OrganisationMember

}


