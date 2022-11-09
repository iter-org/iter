use k8s_openapi::api::core::v1::Secret;
use kube::{Client, Api, api::PostParams};
use serde::{Serialize};
use dialoguer::{console::style};
use serde_json::json;

pub async fn get_client() -> Result<Client, anyhow::Error> {
    let client = Client::try_default().await;
    match client {
        Ok(client) => Ok(client),
        Err(err) => {
            eprintln!("{} {}",
                style("✖").red().bold(),
                style("Failed to connect to Kubernetes").red().bold(),
            );
            eprintln!("{} {} {}\n{} {}\n{:#?}",
                style("?").blue().bold(),
                style("Perhaps try").cyan(),
                style("kubectl get pods").blue().bold(),
                style("?").blue().bold(),
                style("to see if you have access to the cluster").cyan(),
                style(&err).red(),
            );
            Err(anyhow::anyhow!(err))
        },
    }
}

// pub async fn create_secret<D: Serialize + DeserializeOwned>(secret_name: &str, namespace: &str) -> Result<Option<D>, anyhow::Error> {
//     let client = get_client().await;
//     let secret_api: Api<Secret> = Api::namespaced(client, &namespace);

    
// }

pub async fn generate_secret_object<S: Serialize>(secret: S, name: &str, namespace: &str) -> Result<Secret, anyhow::Error> {
    serde_json::from_value(json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": &name,
            "namespace": &namespace
        },
        "data": {
            "secret": base64::encode(&serde_json::to_string(&secret)?),
        }
    })).map_err(|e| anyhow::anyhow!(e))
}

// create namespace object
// run create_or_replace_kubernetes_resource(namespace_object: serde_json::Value)

pub async fn create_or_update_kube_secrets<S: Serialize>(secret: S, name: &str, namespace: &str) -> Result<(), anyhow::Error> {
    let client = get_client().await?;
    let secret = generate_secret_object(secret, name, namespace).await?;
    let secret_api: Api<Secret> = Api::namespaced(client, &namespace);

    match secret_api.create(&PostParams::default(),&secret).await {
        Ok(_) => println!("{} Created secret {} in namespace {}",
            style("✔").green().bold(),
            style(name).green(),
            style(namespace).green()
        ),
        Err(kube::Error::Api(kube::core::ErrorResponse { reason, .. })) if reason == "AlreadyExists" => {
            secret_api.replace(name, &PostParams::default(), &secret).await?;
        }
        Err(e) => panic!("Error getting secret: {:?}", e),
    }
    Ok(())
}