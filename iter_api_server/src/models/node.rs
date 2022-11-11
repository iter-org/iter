use castle_api::types::State;
use mongodb::bson::{oid::ObjectId, bson, doc};
use serde::{Serialize, Deserialize};
use super::{utils::model::Model, Organisation, page::Page};

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    _id: ObjectId,
    name: String,
    icon: Option<String>,
    icon_color: Option<String>,
    organisation: ObjectId,
    root_page: ObjectId,
    //created_date: Date,
    //created_by: ObjectId<(Profile, OrganisationMember)>,
    //last_edited: Date,
    //last_edited_by: ObjectId<(Profile, OrganisationMember)>,
}

impl Model for Node {
    fn collection_name() ->  &'static str {
        "nodes"
    }
}

impl Node {
    pub async fn create_node(
        state: &State,
        name: &str,
        organisation_id: ObjectId,
        root_page_id: ObjectId
    ) -> Result<ObjectId, anyhow::Error> {
        return Ok(Node::create(state, bson!({
            "name": name,
            "organisation": organisation_id,
            "root_page": root_page_id,
            "icon": None::<String>,
            "icon_color": None::<String>,
        })).await?)
    }
}

#[castle_api::castle_macro(Type)]
impl Node {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_string()
    }

    pub fn name(&self, _state: &State) -> String {
        self.name.clone()
    }

    pub async fn update_name(&self, _state: &State, name: &str) -> Result<(), anyhow::Error> {
        Node::update(_state, &self._id, doc!{
            "$set": {
                "name": name
            }
        }).await?;
        Ok(())
    }

    pub async fn organisation(&self, _state: &State) -> Result<Organisation, anyhow::Error> {
        Organisation::find_by_id(self.organisation, _state).await
    }


    pub async fn root_page(&self, _state: &State) -> Result<Page, anyhow::Error> {
        Page::find_by_id(self.root_page, _state).await
    }

    pub async fn icon(&self, _state: &State) -> Result<Option<String>, anyhow::Error> {
        Ok(self.icon.clone())
    }

    pub async fn update_icon(&self, _state: &State, icon: &str) -> Result<(), anyhow::Error> {
        Node::update(_state, &self._id, doc!{
            "$set": {
                "icon": icon
            }
        }).await?;
        Ok(())
    }

    pub async fn icon_color(&self, _state: &State) -> Result<Option<String>, anyhow::Error> {
        Ok(self.icon_color.clone())
    }

    pub async fn update_icon_color(&self, _state: &State, color: &str) -> Result<(), anyhow::Error> {
        Node::update(_state, &self._id, doc!{
            "$set": {
                "icon_color": color
            }
        }).await?;
        Ok(())
    }
}