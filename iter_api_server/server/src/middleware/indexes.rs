use castle_api::{async_trait, types::State};
use envoy_http::Middleware;
use mongodb::{bson::doc, options::IndexOptions};
use tokio::sync::RwLock;

use crate::{
    graph::create_state_from_context,
    models::{utils::model::Model, User, OrganisationMember},
};

pub struct IndexMiddleware {
    indexes_created: RwLock<bool>,
}

#[async_trait]
impl Middleware for IndexMiddleware {
    async fn handle(
        &self,
        ctx: &mut envoy_http::Context,
        next: envoy_http::Next,
    ) -> envoy_http::Result {

        if false == self.indexes_created.read().await.clone() {
            let mut write_guard = self.indexes_created.write().await;
            if !*write_guard {
                let state = create_state_from_context(ctx);
                create_indexes(&state).await?;
                *write_guard = true;
            }
        }

        next.run(ctx).await
    }
}

impl IndexMiddleware {
    pub fn new() -> Self {
        Self {
            indexes_created: RwLock::new(false),
        }
    }
}

async fn create_indexes(state: &State) -> Result<(), anyhow::Error> {
    User::create_index(
        &state,
        doc! { "email": 1 },
        IndexOptions::builder().unique(true).build(),
    ).await?;

    OrganisationMember::create_index(
        &state,
        doc! { "user_id": 1 },
        IndexOptions::builder().unique(false).build()
    ).await?;

    OrganisationMember::create_index(
        &state,
        doc! { "organisation_id": 1 },
        IndexOptions::builder().unique(false).build()
    ).await?;

    Ok(())
}
