use castle_api::types::State;
use mongodb::bson::{oid::ObjectId, doc};
use serde::{Serialize, Deserialize};

use super::{utils::model::Model};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileNickname {
    pub _id: ObjectId,
    name: String,
    profile_id: ObjectId,
    user_id: ObjectId,
}

impl Model for ProfileNickname {
    fn collection_name() ->  & 'static str {
        "profile_nicknames"
    }
}

#[castle_api::castle_macro(Type)]
impl ProfileNickname {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_string()
    }

    pub fn name(&self, _state: &State) -> &str {
        &self.name
    }

    pub fn profile_id(&self, _state: &State) -> String {
        self.profile_id.to_hex()
    }

    pub fn user_id(&self, _state: &State) -> String {
        self.user_id.to_hex()
    }
}