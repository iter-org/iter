

use castle_api::types::State;
use mongodb::{bson::{doc, oid::ObjectId, bson},};
use rand::Rng;
use serde::{Deserialize, Serialize};
use super::{User, utils::{model::Model}};


#[derive(Debug, Serialize, Deserialize)]
pub struct Deployment {
    pub metadata: apimachinery::pkg::apis::meta::v1::ObjectMeta,
    _id: ObjectId,
    hash: String,
    // domains: Vec<String>,
    kind: DeploymentKind
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
enum DeploymentKind {
    GitCommit(String),
    PullRequest(ObjectId),
    Manual(ObjectId) 
}

impl Model for Deployment {
    fn collection_name() ->  &'static str {
        "deployments"
    }
}

impl Model for Deployment {}

impl Resource for Deployment {
    const GROUP: &'static str = "iter";
    const KIND: &'static str = "Deployment";
    const VERSION: &'static str = "v1";
    const URL_PATH_SEGMENT: &'static str = "deployments";
    const API_VERSION: &'static str ="iter/v1";
    type Scope = NamespaceResourceScope;
}

impl ListableResource for Deployment {
    const LIST_KIND: &'static str = "DeploymentList";
}

impl Metadata for Deployment {
    type Ty = apimachinery::pkg::apis::meta::v1::ObjectMeta;
    
    fn metadata(&self) -> &Self::Ty {
        &self.metadata
    }
    
    fn metadata_mut(&mut self) -> &mut Self::Ty {
        &mut self.metadata
    }
}

impl Deployment {

    pub async fn create_deployment(state: &State, domains: Vec<String>, git_commit: String) -> Result<String, anyhow::Error> {
        let hash: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

        Deployment::create(state, bson!({
            "hash": hash.clone(),
            // "domains": domains,
            "kind": git_commit,
        })).await?;
        Ok(hash)
    }
}

#[castle_api::castle_macro(Type)]
impl Deployment {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_hex()
    }
    pub fn hash(&self, _state: &State) -> &str {
        &self.hash
    }
}


