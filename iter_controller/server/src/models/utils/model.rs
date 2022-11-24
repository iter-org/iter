use std::fmt::Debug;

use castle_api::types::State;

use k8s_openapi::NamespaceResourceScope;
use kube::{Resource, Client, Api, api::PostParams};
use serde::de::DeserializeOwned;
use serde_json::Value;
use dialoguer::console::style;

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



#[async_trait::async_trait]
pub(crate) trait Model: Clone + Debug + Send + Resource + Sized + Unpin + Sync + serde::de::DeserializeOwned + serde::Serialize {
    
    async fn create_or_update_namespaced_resource(
        mut resource: serde_json::Value,
    ) -> Result<(), anyhow::Error>
    where
        <Self as Resource>::DynamicType: Default,
        Self: Resource<Scope = NamespaceResourceScope>
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
    
        let api: Api<Self> = Api::namespaced(client, &namespace);
    
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
}
