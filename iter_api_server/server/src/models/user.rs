
use mongodb::{bson::bson};

use castle_api::types::State;
use mongodb::{bson::{oid::ObjectId, doc}};
use serde::{Deserialize, Serialize};

use super::{utils::model::Model, project::Project, project_member::ProjectMember};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub _id: ObjectId,
    pub github_id: String,
    pub joined: mongodb::bson::DateTime,
    pub platform_permissions: Vec<String>,
}

impl Model for User {
    fn collection_name() -> &'static str {
        "users"
    }
}

#[castle_api::castle_macro(Type)]
impl User {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_hex()
    }

    pub fn github_id(&self, _state: &State) -> &str {
        &self.github_id
    }

    pub fn joined(&self, _state: &State) -> String {
        self.joined.to_string()
    }
    
    pub fn platform_permissions(&self, _state: &State) -> Vec<String> {
        self.platform_permissions.clone()
    }

    pub async fn update_platform_permissions(&self, state: &State, permissions: Vec<String>) -> Result<(), anyhow::Error> {
        User::update(state, &self._id, doc! {
            "$set": {
                "platform_permissions": permissions
            }
        }).await?;
        Ok(())
    }

    pub async fn projects(&self, state: &State) -> Result<Vec<Project>, anyhow::Error> {
        let project_members: Vec<ProjectMember> = ProjectMember::find_many(
            state,
            doc!{
                "user_id": self._id,
            },
            100
        ).await?;

        Ok(Project::find_many(
            state,
            doc!{
                "_id": {
                    "$in": project_members.into_iter().map(|member| member.user_id).collect::<Vec<ObjectId>>()
                }
            },
            100
        ).await?)
    }

 
}



