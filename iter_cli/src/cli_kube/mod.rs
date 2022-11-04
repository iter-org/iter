use k8s_openapi::api::core::v1::Secret;
use kube::{Client, Api};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;

pub async fn get_client() -> Client {
    let client = Client::try_default().await.unwrap();
    client
}

// pub async fn create_secret<D: Serialize + DeserializeOwned>(secret_name: &str, namespace: &str) -> Result<Option<D>, anyhow::Error> {
//     let client = get_client().await;
//     let secret_api: Api<Secret> = Api::namespaced(client, &namespace);

    
// }

pub async fn create_install_secrets<S: Serialize>(secret: S, name: &str, namespace: &str) -> Secret {
    serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": &name,
            "namespace": &namespace
        },
        "data": {
            "secret": base64::encode(&serde_json::to_string(&secret).unwrap()),
        }
    })).unwrap()
}