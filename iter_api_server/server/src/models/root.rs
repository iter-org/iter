use std::{str::FromStr};

use castle_api::types::State;
use super::User;

pub struct Root;

#[castle_api::castle_macro(Type)]
impl Root {
    fn ping(&self, _state: &State) -> String {
        "pong".to_string()
    }

    // async fn create_user(
    //     &self,
    //     state: &State,
    //     email: &str,
    //     password: &str,
    //     first_name: &str,
    //     last_name: &str
    // ) -> Result<String, anyhow::Error> {
    //     let id = User::create_user(state, email, password, first_name, last_name).await?;
    //     Ok(id.to_string())
    // }

    // async fn login(
    //     &self,
    //     state: &State,
    //     email: &str,
    //     password: &str,
    // ) -> Result<String, anyhow::Error> {
    //     User::login(state, email, password)
    //         .await
    //         .map_err(|e| e.into())
    // }

    // #[directive(name = "organisation")]
    async fn me(
        &self,
        state: &State,
    ) -> User {
        state.borrow::<User>().clone()
    }

    // async fn create_project(
    //     &self,
    //     state: &State,
    //     name: &str,
    //     git_url: &str,
    // ) -> Result<String, anyhow::Error> {
    //     Ok(Project::create_project(state, name, git_url).await?.to_string())
    // }

    // async fn project(
    //     &self,
    //     state: &State,
    //     project_id: &str,
    // ) -> Result<Project, anyhow::Error> {
    //     Project::find_by_id(ObjectId::from_str(project_id)?, state).await
    // }

    // async fn project_member(
    //     &self,
    //     state: &State,
    //     user_id: &str
    // ) -> Result<ProjectMember, anyhow::Error> {
    //     ProjectMember::find_by_id(ObjectId::from_str(user_id)?, state).await
    // }

    // async fn create_project_member(
    //     &self,
    //     state: &State,
    //     project_id: &str,
    //     permissions: Vec<&str>
    // ) -> Result<String, anyhow::Error> {
    //     Ok(ProjectMember::create_project_member(
    //         state,
    //         ObjectId::from_str(project_id)?,
    //         permissions.iter().map(|s| s.to_string()).collect()
    //     ).await?.to_string())
    // }

    // async fn remove_project_member(
    //     &self,
    //     state: &State,
    //     user_id: &str
    // ) -> Result<(), anyhow::Error> {
    //     Ok(ProjectMember::delete(state, &ObjectId::from_str(user_id)?).await?)
    // }
}