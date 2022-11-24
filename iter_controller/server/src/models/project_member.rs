use castle_api::types::State;
use mongodb::bson::{oid::ObjectId, doc, bson};
use serde::{Serialize, Deserialize};

use super::{utils::model::Model, User};


#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMember {
    pub metadata: apimachinery::pkg::apis::meta::v1::ObjectMeta,
    // kubernetes Project resource name
    pub project: String,
    // kubernetes User resource name
    pub user: String,
    permissions: Vec<String>,
}



impl Model for ProjectMember {
    fn collection_name() ->  & 'static str {
        "project_members"
    }
}

impl Model for ProjectMember {}

impl Resource for ProjectMember {
    const GROUP: &'static str = "iter";
    const KIND: &'static str = "ProjectMember";
    const VERSION: &'static str = "v1";
    const URL_PATH_SEGMENT: &'static str = "project-members";
    const API_VERSION: &'static str ="iter/v1";
    type Scope = NamespaceResourceScope;
}

impl ListableResource for ProjectMember {
    const LIST_KIND: &'static str = "ProjectMemberList";
}

impl Metadata for ProjectMember {
    type Ty = apimachinery::pkg::apis::meta::v1::ObjectMeta;
    
    fn metadata(&self) -> &Self::Ty {
        &self.metadata
    }
    
    fn metadata_mut(&mut self) -> &mut Self::Ty {
        &mut self.metadata
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