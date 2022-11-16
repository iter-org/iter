

use castle_api::types::State;
use mongodb::{bson::{doc, oid::ObjectId, bson},};
use rand::Rng;
use serde::{Deserialize, Serialize};
use super::{User, utils::{model::Model}};


#[derive(Debug, Serialize, Deserialize)]
pub struct Deployment {
    _id: ObjectId,
    hash: String,
    // domains: Vec<String>,
    kind: DeploymentKind
}

#[derive(Debug, Serialize, Deserialize)]
enum DeploymentKind {
    GitCommit(String),
    PullRequest(ObjectId),
    Manual(ObjectId) // object id to a user
}

impl Model for Deployment {
    fn collection_name() ->  &'static str {
        "deployments"
    }
}

impl Deployment {

    pub async fn create_deployment(state: &State, domains: Vec<String>, git_commit: String) -> Result<String, anyhow::Error> {
        let hash: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(30)
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


