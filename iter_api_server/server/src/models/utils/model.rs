use castle_api::types::State;
use mongodb::bson::{Bson, Document};
use mongodb::error::{ErrorKind, WriteFailure, WriteError};
use mongodb::{Client as MongoClient, IndexModel};
use mongodb::options::{FindOneOptions, FindOptions, IndexOptions};
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::middleware::KubernetesNamespace;
use futures::stream::{StreamExt};

#[async_trait::async_trait]
pub(crate) trait Model: Send + Sized + Unpin + Sync + serde::de::DeserializeOwned {
    fn model_name() -> String {
        std::any::type_name::<Self>().to_string()
    }

    fn collection<T>(state: &State) -> Collection<T> {
        state
            .borrow::<MongoClient>()
            .database(state.borrow::<KubernetesNamespace>().as_ref())
            .collection(Self::collection_name())
    }

    fn collection_name() -> &'static str;

    async fn find_by_id<T: DeserializeOwned + Unpin + Send + Sync>(
        id: ObjectId,
        state: &State,
    ) -> Result<T, anyhow::Error> {
        Self::collection::<T>(state)
            .find_one(doc! { "_id": id }, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to find {}: {}", Self::model_name(), e))?
            .ok_or(anyhow::anyhow!("{} not found", Self::model_name()))
    }

    async fn get_field<T: DeserializeOwned + Unpin + Send + Sync>(
        id: &ObjectId,
        field: &str,
        state: &State,
    ) -> Result<T, anyhow::Error> {
        Self::get_field_optional(id, field, state)
            .await?
            .ok_or(anyhow::anyhow!(
                "{} had no field named {}",
                Self::model_name(),
                field
            ))
    }

    async fn get_field_optional<T: DeserializeOwned + Unpin + Send + Sync>(
        id: &ObjectId,
        field: &str,
        state: &State,
    ) -> Result<Option<T>, anyhow::Error> {
        Self::find_by_id::<Value>(id.clone(), state)
            .await?
            .get(field)
            .cloned()
            .map(|v| T::deserialize(v))
            .transpose()
            .map_err(|e| anyhow::anyhow!("Failed to deserialize {}: {}", Self::model_name(), e))
    }

    async fn create(state: &State, doc: Bson) -> Result<ObjectId, anyhow::Error> {
        Ok(Self::collection::<Bson>(state)
            .insert_one(doc, None)
            .await
            .map_err(|e| match *e.kind {
                ErrorKind::Write(WriteFailure::WriteError(WriteError {
                    code: 11000, // duplicate key
                    ..
                })) => anyhow::anyhow! {"{} already exists", Self::model_name()},
                _ => anyhow::anyhow! {"Failed to create {}: {}", Self::model_name(), e},
            })?
            .inserted_id
            .as_object_id()
            .expect("Expected ObjectId")
        )
    }

    async fn update(state: &State, id: &ObjectId, update: Document) -> Result<(), anyhow::Error> {
        Self::collection::<Bson>(state)
            .update_one(doc! { "_id": id }, update, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update {}: {}", Self::model_name(), e))?;
        Ok(())
    }

    async fn delete(state: &State, id: &ObjectId) -> Result<(), anyhow::Error> {
        Self::collection::<Bson>(state)
            .delete_one(doc! { "_id": id }, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete {}: {}", Self::model_name(), e))?;
        Ok(())
    }

    async fn find_one<T: DeserializeOwned + Unpin + Send + Sync>(
        state: &State,
        query: Document
    ) -> Result<Option<T>, anyhow::Error> {
        let find_options = FindOneOptions::builder().projection(None).build();

        Self::collection::<T>(state)
            .find_one(query, find_options)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to find {}: {}", Self::model_name(), e))
    }

    async fn find_many<T: DeserializeOwned + Unpin + Send + Sync>(
        state: &State,
        filter: Document,
        limit: impl Into<Option<i64>> + Send
    ) -> Result<Vec<T>, anyhow::Error> {
        let find_options = FindOptions::builder()
            .limit(limit)
            .projection(None)
            .build();

        Self::collection::<T>(state)
            .find(filter, find_options)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to find {}: {}", Self::model_name(), e))?
            .collect::<Vec<Result<T, _>>>()
            .await
            .into_iter()
            .collect::<Result<Vec<T>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to collect {}: {}", Self::model_name(), e))
    }

    async fn count(
        state: &State,
        filter: Document,
    ) -> Result<u64, anyhow::Error> {
        Self::collection::<()>(state)
            .count_documents(filter, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to find {}: {}", Self::model_name(), e))
    }

    /// Create index
    /// This should only be run during startup
    async fn create_index(state: &State, index: Document, options: IndexOptions) -> Result<(), anyhow::Error> {
        Self::collection::<Bson>(state)
            .create_index(IndexModel::builder()
                .keys(index)
                .options(options)
                .build(), None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create index for {}: {}", Self::model_name(), e))?;
        Ok(())
    }
}
