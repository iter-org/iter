
use k8s_openapi::{Resource, NamespaceResourceScope, apimachinery, Metadata, ListableResource};

use castle_api::types::State;
use serde::{Deserialize, Serialize};

use super::{utils::model::Model};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    /// Standard object metadata.
    pub metadata: apimachinery::pkg::apis::meta::v1::ObjectMeta,
    pub github_user_id: String,
    pub platform_permissions: Vec<String>,
}

impl Model for User {}

impl Resource for User {
    const GROUP: &'static str = "iter";
    const KIND: &'static str = "User";
    const VERSION: &'static str = "v1";
    const URL_PATH_SEGMENT: &'static str = "users";
    const API_VERSION: &'static str ="iter/v1";
    type Scope = NamespaceResourceScope;
}

impl ListableResource for User {
    const LIST_KIND: &'static str = "UserList";
}

impl Metadata for User {
    type Ty = apimachinery::pkg::apis::meta::v1::ObjectMeta;
    
    fn metadata(&self) -> &Self::Ty {
        &self.metadata
    }
    
    fn metadata_mut(&mut self) -> &mut Self::Ty {
        &mut self.metadata
    }
}

#[castle_api::castle_macro(Type)]
impl User {
    // pub fn _id(&self, _state: &State) -> String {
    //     self._id.to_hex()
    // }

    // pub fn github_id(&self, _state: &State) -> &str {
    //     &self.github_id
    // }

    // pub fn joined(&self, _state: &State) -> String {
    //     self.joined.to_string()
    // }
    
    // pub fn platform_permissions(&self, _state: &State) -> Vec<String> {
    //     self.platform_permissions.clone()
    // }

    // pub async fn update_platform_permissions(&self, state: &State, permissions: Vec<String>) -> Result<(), anyhow::Error> {
    //     User::update(state, &self._id, doc! {
    //         "$set": {
    //             "platform_permissions": permissions
    //         }
    //     }).await?;
    //     Ok(())
    // }

    // pub async fn projects(&self, state: &State) -> Result<Vec<Project>, anyhow::Error> {
    //     let project_members: Vec<ProjectMember> = ProjectMember::find_many(
    //         state,
    //         doc!{
    //             "user_id": self._id,
    //         },
    //         100
    //     ).await?;

    //     Ok(Project::find_many(
    //         state,
    //         doc!{
    //             "_id": {
    //                 "$in": project_members.into_iter().map(|member| member.user_id).collect::<Vec<ObjectId>>()
    //             }
    //         },
    //         100
    //     ).await?)
    // }
}



