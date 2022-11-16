use castle_api::types::State;
use mongodb::bson::{oid::ObjectId, doc, bson};
use serde::{Serialize, Deserialize};

use super::{utils::model::Model, User};


#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMember {
    pub _id: ObjectId,
    pub project_id: ObjectId,
    pub user_id: ObjectId,
    date_joined: mongodb::bson::DateTime,
    permissions: Vec<String>,
}

impl Model for ProjectMember {
    fn collection_name() ->  & 'static str {
        "project_members"
    }
}


impl ProjectMember {
    pub async fn create_project_member(
        state: &State,
        project_id: ObjectId,
        permissions: Vec<String>
    ) -> Result<ObjectId, anyhow::Error> {
        let user = state.borrow::<User>();

        let project_member_id = ProjectMember::create(state, bson!({
            "project_id": project_id,
            "date_joined": mongodb::bson::DateTime::now(),
            "user_id": user._id.clone(),
            "permissions": permissions,
        })).await?;

        Ok(project_member_id)
    }
}

#[castle_api::castle_macro(Type)]
impl ProjectMember {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_hex()
    }

    pub fn project_id(&self, _state: &State) -> String {
        self.project_id.to_hex()
    }

    pub fn user_id(&self, _state: &State) -> String {
        self.user_id.to_hex()
    }

    pub fn date_joined(&self, _state: &State) -> String {
        self.date_joined.to_string()
    }

    pub fn permissions(&self, _state: &State) -> Vec<String> {
        self.permissions.clone()
    }

    pub async fn update_permissions(&self, _state: &State, permissions: Vec<String>) -> Result<(), anyhow::Error> {
        let user = _state.borrow::<User>();
        ProjectMember::update(_state, &user._id, doc! {
            "&set": {
                "permissions": permissions
            }
        }).await?;
        Ok(())
    }
}