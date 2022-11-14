use std::{str::FromStr};

use castle_api::types::State;
use mongodb::bson::{oid::ObjectId};
use super::{User, Organisation, utils::model::Model, OrganisationMember};

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
}