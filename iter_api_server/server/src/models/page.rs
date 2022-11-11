use castle_api::types::State;
use mongodb::bson::{oid::ObjectId, bson, doc};
use serde::{Serialize, Deserialize};
use super::{utils::model::Model, node::Node, Organisation};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Page {
    _id: ObjectId,
    node: ObjectId,
    template: Option<ObjectId>,
    parent_page: Option<ObjectId>,
    pub root_page: ObjectId,
    organisation: ObjectId,
    //view: Object<ObjectId<View>>
    // permissions: some permission data
}

impl Model for Page {
    fn collection_name() ->  &'static str {
        "pages"
    }
}

impl Page {
    
    pub async fn create_root_page(
        state: &State,
        name: &str,
        organisation_id: ObjectId
    ) -> Result<ObjectId, anyhow::Error> {
        let page_id = ObjectId::new();
        let node_id = Node::create_node(
            state,
            name,
            organisation_id,
            page_id,
        ).await?;
        return Ok(Page::create(state, bson!({
            "_id": page_id,
            "node": node_id,
            "template": None::<ObjectId>,
            "parent_page": None::<ObjectId>,
            "root_page": page_id,
            "organisation": organisation_id,
        })).await?)
    }

    pub async fn create_page(
        state: &State,
        node_id: ObjectId,
        template_id: Option<ObjectId>,
        parent_page_id: Option<ObjectId>,
        root_page: ObjectId,
        organisation_id: ObjectId
    ) -> Result<ObjectId, anyhow::Error> {

        return Ok(Page::create(state, bson!({
            "node": node_id,
            "template": template_id,
            "parent_page": parent_page_id,
            "root_page": root_page,
            "organisation": organisation_id,
        })).await?)
    }
}

#[castle_api::castle_macro(Type)]
impl Page {
    pub fn _id(&self, _state: &State) -> String {
        self._id.to_string()
    }

    pub async fn node(&self, _state: &State) -> Result<Node, anyhow::Error> {
        Node::find_by_id(self.node, _state).await
    }

    pub async fn root_page_children_count(&self, _state: &State) -> Result<u64, anyhow::Error> {
        Page::count(_state, doc!{
            "root_page_id": self.root_page,
        }).await
    }


    // pub fn template(&self, _state: &State) -> Option<Tem> {
    //     match self.template {
    //         Some(ref template) => Some(template.to_string()),
    //         None => None
    //     }
    // }

    pub async fn parent_page(&self, _state: &State) -> Result<Option<Page>, anyhow::Error> {
        match self.parent_page {
            Some(ref parent_page) => Ok(Some(Page::find_by_id(parent_page.clone(), _state).await?)),
            None => Ok(None)
        }
    }

    pub async fn root_page(&self, _state: &State) -> Result<Page, anyhow::Error> {
        Page::find_by_id(self.root_page, _state).await
    }

    pub async fn organisation(&self, _state: &State) -> Result<Organisation, anyhow::Error> {
        Organisation::find_by_id(self.organisation, _state).await
    }
}