use std::{str::FromStr};

use mongodb::{bson::bson};

use castle_api::types::State;
use mongodb::{bson::{oid::ObjectId, doc}};
use serde::{Deserialize, Serialize};

use super::{utils::model::Model, User, page::Page};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sidebar {
    pub _id: ObjectId,
    pub user_id: ObjectId,
    pub items: Vec<SidebarItem>,
    pub current_tab: Option<ObjectId>
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidebarItem {
    root_page: ObjectId,
    pinned: bool,
    tabs: Vec<ObjectId>
}

#[castle_api::castle_macro(Type)]
impl SidebarItem {
    async fn root_page(&self, state: &State) -> Result<Page, anyhow::Error> {
        Ok(Page::find_by_id(self.root_page, state).await?)
    }

    fn pinned(&self, _state: &State) -> bool {
        self.pinned
    }

    async fn tabs(&self, state: &State) -> Result<Vec<Page>, anyhow::Error> {
        let mut vec_of_pages = vec![];
        for object_id in &self.tabs {
            vec_of_pages.push(Page::find_by_id(*object_id, state).await?);
        }
        Ok(vec_of_pages)
    }
}

impl Model for Sidebar {
    fn collection_name() ->  & 'static str {
        "sidebars"
    }
}

impl Sidebar {
    pub async fn create_sidebar(
        state: &State,
    ) -> Result<ObjectId, anyhow::Error> {
        Sidebar::create(state, bson!({
            "user_id": state.borrow::<User>()._id,
            "items": mongodb::bson::to_bson(&Vec::<SidebarItem>::new())?,
            "current_tab": None::<ObjectId>
        })).await
    }
}

#[castle_api::castle_macro(Type)]
impl Sidebar {
    fn user_id(&self, _state: &State) -> Result<String, anyhow::Error> {
        Ok(self.user_id.to_string())
    }

    fn items(&self, _state: &State) -> Result<Vec<SidebarItem>, anyhow::Error> {
        return Ok(self.items.clone())
    }

    async fn switch_tab(&self, state: &State, page_id: &str) -> Result<(), anyhow::Error> {
        let mut items = self.items.clone();

        match self.current_tab {
            Some(current_tab) => {
                let current_tab_page: Page = Page::find_by_id(current_tab, state).await?;
                items = close_tab(
                    self.items.clone(),
                    current_tab_page.root_page,
                    current_tab
                );
            },
            None => {}
        };

        let page: Page = Page::find_by_id(ObjectId::from_str(page_id)?, state).await?;
        let root_page_id = page.root_page;
        items = add_tab(items, root_page_id, ObjectId::from_str(page_id)?);

        // update sidebar
        Sidebar::update(state, &self._id, doc!{
            "$set": {
                "items": mongodb::bson::to_bson(&items)?,
                "current_tab": ObjectId::from_str(page_id)?
            }
        }).await?;

        Ok(())
    }

    // if page's root page is not in sidebar, add sidebar item with page as a tab
    // else, add page as a tab to the sidebar item with the same root page
    async fn add_tab(&self, state: &State, page_id: &str) -> Result<(), anyhow::Error> {
        let page: Page = Page::find_by_id(ObjectId::from_str(page_id)?, state).await?;
        let items = add_tab(self.items.clone(), page.root_page, ObjectId::from_str(page_id)?);
        Sidebar::update(state, &self._id, doc!{
            "$set": {
                "items": mongodb::bson::to_bson(&items)?,
                "current_tab": ObjectId::from_str(page_id)?
            }
        }).await?;
        Ok(())
    }

    async fn close_tab(&self, state: &State, page_id: &str) -> Result<(), anyhow::Error> {
        let page: Page = Page::find_by_id(ObjectId::from_str(page_id)?, state).await?;
        let items = close_tab(
            self.items.clone(),
            page.root_page,
            ObjectId::from_str(page_id)?
        );

        Sidebar::update(state, &self._id, doc!{
            "$set": {
                "items": mongodb::bson::to_bson(&items)?,
            }
        }).await?;

        if self.current_tab == Some(ObjectId::from_str(page_id)?) {
            Sidebar::update(state, &self._id, doc!{
                "$set": {
                    "current_tab": None::<ObjectId>
                }
            }).await?;
        }
        Ok(())
    }

    async fn toggle_pin_workspace(&self, state: &State, root_page_id: &str) -> Result<(), anyhow::Error> {
        let mut items = self.items.clone();

        for mut item in &mut items {
            if item.root_page == ObjectId::from_str(root_page_id)? {
                item.pinned = !item.pinned;
                break;
            }
        }
        Sidebar::update(state, &self._id, doc!{
            "$set": {
                "items": mongodb::bson::to_bson(&items)?
            }
        }).await?;
        Ok(())
    }
}

fn add_tab(mut items: Vec<SidebarItem>, root_page_id: ObjectId, page_id: ObjectId) -> Vec<SidebarItem> {
    // add new tab (if it is not already open)
    let root_page_id = root_page_id;
    let mut found_root_page = false;
    for item in &mut items {
        if item.root_page == root_page_id {
            if !item.tabs.contains(&page_id) {
                item.tabs.push(page_id);
            }
            found_root_page = true;
        }
    }

    if !found_root_page {
        items.push(SidebarItem {
            root_page: root_page_id,
            pinned: false,
            tabs: vec![page_id]
        })
    }

    return items
}

fn close_tab(
    mut items: Vec<SidebarItem>,
    root_page_id: ObjectId,
    page_id: ObjectId
) -> Vec<SidebarItem> {
    let item = items.iter_mut().find(|item| item.root_page == root_page_id);
    match item {
        Some(item) => {
            item.tabs.retain(|tab| *tab != page_id);
            if item.tabs.len() == 0 && !item.pinned {
                items.retain(|item| item.root_page != root_page_id);
            }
        },
        None => unreachable!()
    };

    return items
}