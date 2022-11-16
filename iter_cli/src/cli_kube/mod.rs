use std::fmt::Debug;

use dialoguer::console::style;
use k8s_openapi::{NamespaceResourceScope};
use kube::{api::PostParams, Api, Client, Resource};
use serde::{Serialize, de::DeserializeOwned};

pub async fn get_client() -> Result<Client, anyhow::Error> {
    let client = Client::try_default().await;
    match client {
        Ok(client) => Ok(client),
        Err(err) => {
            eprintln!(
                "{} {}",
                style("✖").red().bold(),
                style("Failed to connect to Kubernetes").red().bold(),
            );
            eprintln!(
                "{} {} {} {}\n{:#?}",
                style("?").blue().bold(),
                style("Perhaps try").cyan(),
                style("kubectl get pods").blue().bold(),
                style("to see if you have access to the cluster").cyan(),
                style(&err).red(),
            );
            Err(anyhow::anyhow!(err))
        }
    }
}

pub async fn create_or_update_namespaced_resource<R: Clone + DeserializeOwned + Debug + Serialize + Resource<Scope = NamespaceResourceScope>>(
    mut resource: serde_json::Value,
) -> Result<(), anyhow::Error>
where
    <R as Resource>::DynamicType: Default
{
    let client = get_client().await?;
    
    let namespace = match resource["metadata"]["namespace"].as_str() {
        Some(namespace) => namespace.to_string(),
        None => Err(anyhow::anyhow!("No namespace provided in resource"))?,
    };
    let name = match resource["metadata"]["name"].as_str() {
        Some(name) => name.to_string(),
        None => Err(anyhow::anyhow!("No name provided in resource"))?,
    };

    let kind = match resource["kind"].as_str() {
        Some(kind) => kind.to_string(),
        None => Err(anyhow::anyhow!("No kind provided in resource"))?,
    };

    let api: Api<R> = Api::namespaced(client, &namespace);

    // Check if resource exists
    let resource_version = match api.get(&name).await {
        Ok(resource) => serde_json::to_value(resource)?["metadata"]["resourceVersion"].as_str().map(|val| val.to_string()),
        Err(_) => None
    };

    match resource_version {
        Some(resource_version) => {
            resource["metadata"]["resourceVersion"] = serde_json::Value::String(resource_version);
            api.replace(&name, &PostParams::default(), &serde_json::from_value(resource)?).await?;
        },
        None => {
            api.create(&PostParams::default(), &serde_json::from_value(resource)?).await?;
        }
    }

    println!(
        "{} {} {} {} {}",
        style("✔").green().bold(),
        style(format!("Created {}", kind)).white().bold(),
        style(name).green(),
        style("in namespace").white().bold(),
        style(namespace).green()
    );

    Ok(())
}

pub async fn create_or_update_cluster_resource<R: Clone + DeserializeOwned + Debug + Serialize + Resource>(
    mut resource: serde_json::Value,
) -> Result<(), anyhow::Error>
where
    <R as Resource>::DynamicType: Default
{
    let client = get_client().await?;
    
    let name = match resource["metadata"]["name"].as_str() {
        Some(name) => name.to_string(),
        None => Err(anyhow::anyhow!("No name provided in resource"))?,
    };

    let kind = match resource["kind"].as_str() {
        Some(kind) => kind.to_string(),
        None => Err(anyhow::anyhow!("No kind provided in resource"))?,
    };

    let api: Api<R> = Api::all(client);

    // Check if resource exists
    let resource_version = match api.get(&name).await {
        Ok(resource) => serde_json::to_value(resource)?["metadata"]["resourceVersion"].as_str().map(|val| val.to_string()),
        Err(_) => None
    };

    match resource_version {
        Some(resource_version) => {
            resource["metadata"]["resourceVersion"] = serde_json::Value::String(resource_version);
            api.replace(&name, &PostParams::default(), &serde_json::from_value(resource)?).await?;
        },
        None => {
            api.create(&PostParams::default(), &serde_json::from_value(resource)?).await?;
        }
    }


    println!(
        "{} {}: {}",
        style("✔").green().bold(),
        style(format!("Created {}", kind)).white().bold(),
        style(name).green(),
    );

    Ok(())
}