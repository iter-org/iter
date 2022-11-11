use std::{str::FromStr, sync::Arc};

use castle_api::types::State;
use mongodb::bson::{oid::ObjectId, doc};
use secrets::BackendSecrets;
use zxcvbn::time_estimates::CrackTimeSeconds;

use super::{User, Organisation, utils::model::Model, Profile, OrganisationMember, stripe::product::{Product}, node::Node, page::Page, sidebar::Sidebar};

pub struct Root;

#[castle_api::castle_macro(Type)]
impl Root {
    fn ping(&self, _state: &State) -> String {
        "pong".to_string()
    }

    async fn create_user(
        &self,
        state: &State,
        email: &str,
        password: &str,
        first_name: &str,
        last_name: &str
    ) -> Result<String, anyhow::Error> {
        let id = User::create_user(state, email, password, first_name, last_name).await?;
        Ok(id.to_string())
    }

    async fn login(
        &self,
        state: &State,
        email: &str,
        password: &str,
    ) -> Result<String, anyhow::Error> {
        User::login(state, email, password)
            .await
            .map_err(|e| e.into())
    }

    // #[directive(name = "organisation")]
    async fn me(
        &self,
        state: &State,
    ) -> User {
        state.borrow::<User>().clone()
    }

    async fn create_organisation(
        &self,
        state: &State,
        name: &str,
    ) -> Result<String, anyhow::Error> {
        Ok(Organisation::create_organisation(state, name).await?.to_string())
    }

    async fn organisation(
        &self,
        state: &State,
        organisation_id: &str,
    ) -> Result<Organisation, anyhow::Error> {
        Organisation::find_by_id(ObjectId::from_str(organisation_id)?, state).await
    }


    async fn profile(
        &self,
        state: &State,
        profile_id: &str,
    ) -> Result<Profile, anyhow::Error> {
        Profile::find_by_id(ObjectId::from_str(profile_id)?, state).await
    }

    async fn create_profile(
        &self,
        state: &State,
        name: &str,
        display_picture: Option<&str>,
        organisation_id: &str,
        roles: Vec<String>,
    ) -> Result<String, anyhow::Error> {
        let converted_dp = match display_picture {
            Some(dp) => Some(ObjectId::from_str(dp)?),
            None => None,
        };

        let profile_id = Profile::create_profile(
            state,
            name,
            converted_dp,
            ObjectId::from_str(organisation_id)?,
            roles
        ).await?.to_string();

        Ok(profile_id)
    }

    async fn create_profile_in_org_settings(
        &self,
        state: &State,
        organisation_id: &str,
        name: &str,
        roles: Vec<String>,
        members: Vec<String>,
    ) -> Result<String, anyhow::Error> {
        let members: Vec<ObjectId> = members.iter().map(|m| ObjectId::from_str(m).unwrap()).collect();

        let profile_id = Profile::create_profile_in_org_settings(
            state,
            ObjectId::from_str(organisation_id)?,
            name,
            roles,
            members
        ).await?.to_string();

        Ok(profile_id)
    }

    async fn organisation_member(
        &self,
        state: &State,
        organisation_member_id: &str
    ) -> Result<OrganisationMember, anyhow::Error> {
        OrganisationMember::find_by_id(ObjectId::from_str(organisation_member_id)?, state).await
    }

    async fn create_organisation_member(
        &self,
        state: &State,
        organisation_id: &str,
        profiles: Vec<&str>
    ) -> Result<String, anyhow::Error> {
        Ok(OrganisationMember::create_organisation_member(
            state,
            ObjectId::from_str(organisation_id)?,
            profiles.iter().map(|p| ObjectId::from_str(p).unwrap()).collect()
        ).await?.to_string())
    }

    async fn remove_organisation_member(
        &self,
        state: &State,
        organisation_member_id: &str
    ) -> Result<(), anyhow::Error> {
        Ok(OrganisationMember::delete(state, &ObjectId::from_str(organisation_member_id)?).await?)
    }

    async fn stripe_publishable_key(&self, state: &State) -> String {
        state.borrow::<Arc<BackendSecrets>>().stripe_secrets.publishable_key.clone()
    }

    async fn products(&self, state: &State) -> Result<Vec<Product>, anyhow::Error> {
        Product::get_products(state).await
    }

    async fn crack_time(
        &self,
        _state: &State,
        password: &str,
    ) -> Result<CrackSeconds, anyhow::Error> {
        let guesses = match zxcvbn::zxcvbn(password, &[]) {
            Ok(entropy) => entropy.guesses(),
            Err(_) => 0,
        };

        Ok(CrackSeconds {
            guesses,
            seconds: guesses as f64 / 100_000.0,
            string: CrackTimeSeconds::Float(guesses as f64 / 100_000.0).to_string(),
        })
    }

    async fn create_root_page(&self, state: &State, name: &str, organisation_id: &str) -> Result<String, anyhow::Error> {
        Ok(Page::create_root_page(state, name, ObjectId::from_str(organisation_id)?).await?.to_string())
    }
    async fn create_node(
        &self,
        state: &State,
        name: &str,
        organisation_id: &str,
        root_page_id: &str
    ) -> Result<String, anyhow::Error>{
        Ok(Node::create_node(state, name, ObjectId::from_str(organisation_id)?, ObjectId::from_str(root_page_id)?).await?.to_string())
    }
    async fn node(
        &self,
        state: &State,
        node_id: &str
    ) -> Result<Node, anyhow::Error>
    {
        Node::find_by_id(ObjectId::from_str(node_id)?, state).await
    }

    async fn create_page(
        &self,
        state: &State,
        node_id: &str,
        template_id: Option<&str>,
        parent_page_id: Option<&str>,
        root_page_id: &str,
        organisation_id: &str,
    ) -> Result<String, anyhow::Error> {
        let template_id = match template_id {
            Some(t) => Some(ObjectId::from_str(t)?),
            None => None
        };
        let parent_page_id = match parent_page_id {
            Some(p) => Some(ObjectId::from_str(p)?),
            None => None
        };

        Ok(Page::create_page(
            state,
            ObjectId::from_str(node_id)?,
            template_id,
            parent_page_id,
            ObjectId::from_str(root_page_id)?,
            ObjectId::from_str(organisation_id)?
        ).await?.to_string())
    }

    async fn page(
        &self,
        state: &State,
        page_id: &str
    ) -> Result<Page, anyhow::Error> {
        Page::find_by_id(ObjectId::from_str(page_id)?, state).await
    }

    async fn create_sidebar(
        &self,
        state: &State,
    ) -> Result<String, anyhow::Error> {
        Ok(Sidebar::create_sidebar(state).await?.to_string())
    }

    async fn sidebar(
        &self,
        state: &State
    ) -> Result<Sidebar, anyhow::Error> {
        let sidebar: Option<Sidebar> = Sidebar::find_one(state, doc!{
            "user_id": state.borrow::<User>()._id
        }).await?;

        return match sidebar {
            Some(sidebar) => Ok(sidebar),
            None => Ok(Sidebar::find_by_id(Sidebar::create_sidebar(state).await?, state).await?)
        }
    }
}

#[castle_api::castle_macro(Type)]
#[derive(Clone, Debug)]
struct CrackSeconds {
    guesses: u64,
    seconds: f64,
    string: String,
}

