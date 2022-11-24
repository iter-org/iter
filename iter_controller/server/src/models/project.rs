use castle_api::types::State;
use mongodb::{bson::{doc, oid::ObjectId, bson},};
use serde::{Deserialize, Serialize};
use super::{User, utils::{model::Model}, project_member::ProjectMember};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub metadata: apimachinery::pkg::apis::meta::v1::ObjectMeta,
    project_name: String,
    git_url: String,
}

impl Model for Project {
    fn collection_name() ->  &'static str {
        "projects"
    }
}

impl Model for Project {}

impl Resource for Project {
    const GROUP: &'static str = "iter";
    const KIND: &'static str = "Project";
    const VERSION: &'static str = "v1";
    const URL_PATH_SEGMENT: &'static str = "projects";
    const API_VERSION: &'static str ="iter/v1";
    type Scope = NamespaceResourceScope;
}

impl ListableResource for Project {
    const LIST_KIND: &'static str = "ProjectList";
}

impl Metadata for Project {
    type Ty = apimachinery::pkg::apis::meta::v1::ObjectMeta;
    
    fn metadata(&self) -> &Self::Ty {
        &self.metadata
    }
    
    fn metadata_mut(&mut self) -> &mut Self::Ty {
        &mut self.metadata
    }
}

impl Project {

    pub async fn create_project(state: &State, name: &str, git_url: &str) -> Result<ObjectId, anyhow::Error> {
        let user = state.borrow::<User>();

        let project_id = Project::create(state, bson!({
            "name": name,
            "created_by": user._id,
            "date_created": mongodb::bson::DateTime::now(),
        })).await?;

        ProjectMember::create(state, bson!({
            "permissions": vec!["*"],
            "joined": mongodb::bson::DateTime::now(),
            "user_id": user._id,
            "project_id": project_id,
        })).await?;

        Ok(project_id)
    }

}



#[castle_api::castle_macro(Type)]
impl Project {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_hex()
    }
    pub fn name(&self, _state: &State) -> &str {
        &self.name
    }

    pub async fn update_name(&self, state: &State, name: &str) -> Result<(), anyhow::Error> {
        Project::update(state, &self._id, doc!{
            "$set": {
                "name": name
            }
        }).await?;
        Ok(())
    }




    pub async fn members(&self, state: &State) -> Result<Vec<ProjectMember>, anyhow::Error> {
        ProjectMember::find_many(
            state,
            doc!{
                "project_id": self._id,
            },
            100
        ).await
    }

    pub async fn member_count(&self, state: &State) -> Result<u64, anyhow::Error> {
        ProjectMember::count(state, doc!{
            "project_id": self._id,
        }).await
    }
    // pub fn profile_picture(&self, _state: &State) -> Option<String> {
    //     self.display_picture.clone()
    // }


    // adding member is done in OrganisationMember

}


