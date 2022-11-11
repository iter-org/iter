use k8s_openapi::{api::core::v1::Secret, serde::{Serialize, Deserialize, de::DeserializeOwned}};
use rand::Rng;
use kube::{Api, Client};
use serde_json::json;

pub const SECRET_NAME: &str = "backend-secrets";

#[derive(Debug, Serialize, Deserialize)]
pub struct BackendSecrets {
    pub jwt_secret: String,
    pub mongo: MongoSecrets,
    pub stripe_secrets: StripeSecrets,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MongoSecrets {
    pub username: String,
    pub password: String,
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StripeSecrets {
    pub secret_key: String,
    pub publishable_key: String,
}


mod optional_secrets {
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BackendSecretsOpt {
        pub jwt_secret: Option<String>,
        pub mongo: Option<MongoSecretsOpt>,
        pub stripe_secrets: Option<StripeSecretsOpt>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MongoSecretsOpt {
        pub username: Option<String>,
        pub password: Option<String>,
        pub host: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StripeSecretsOpt {
        pub secret_key: Option<String>,
        pub publishable_key: Option<String>,
    }
}

/// Get a kubernetes client
async fn get_client() -> Client {
    let client = Client::try_default().await.unwrap();
    client
}

/// Get our pods current namespace
pub async fn get_namespace() -> String {
    let config = kube::config::Config::infer().await.unwrap();
    config.default_namespace
}

/// Panics if the secret does not exist
pub async fn get_secret<D: Serialize + DeserializeOwned>(secret_name: &str, namespace: &str) -> Result<Option<D>, anyhow::Error> {
    let client = get_client().await;
    let secret_api: Api<Secret> = Api::namespaced(client, &namespace);

    match secret_api.get_opt(secret_name).await {
        Ok(Some(Secret { data: Some(data), .. })) => serde_json::from_slice(
            &data.get("secret")
                .ok_or(anyhow::anyhow!("Could not get secret data"))?.0
            ).map_err(|e| anyhow::anyhow!(e)),
        Err(e) => Err(anyhow::anyhow!(e)),
        _ => Ok(None)
    }
}

#[allow(dead_code)]
async fn create_kube_secret_object<S: Serialize>(secret: S, name: &str, namespace: &str) -> Secret {
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

#[allow(dead_code)]
fn generate_random_hex<const LEN: usize>() -> String {
    let mut rand = rand::thread_rng();

    let mut secret = [0u8; LEN];
    rand.fill(&mut secret as &mut [u8]);

    // print the secret
    secret.map(|x| format!("{:02X?}", x)).join("")
}

#[tokio::test]
async fn create_kubernetes_dev_secret() {
    use dialoguer::{theme::ColorfulTheme, console::style, Input};
    use kube::api::PostParams;

    let theme = ColorfulTheme::default();

    let namespace = Input::with_theme(&theme)
        .with_prompt("Kubernetes Namespace")
        .default("dev".to_string())
        .interact()
        .unwrap();

    println!("Checking if secret already exists");
    let secret = get_secret::<optional_secrets::BackendSecretsOpt>(SECRET_NAME, &namespace).await.unwrap().unwrap_or(optional_secrets::BackendSecretsOpt {
        jwt_secret: None,
        mongo: None,
        stripe_secrets: None,
    });

    let secret = BackendSecrets {
        jwt_secret: secret.clone().jwt_secret.unwrap_or_else(|| {
            println!("{} Generating JWT Secret · {}", style("✔").green().bold(), style("·················").green());
            generate_random_hex::<80>()
        }),
        mongo: MongoSecrets {
            username: Input::with_theme(&theme)
                .with_prompt("MongoDB Username")
                .default(secret.clone().mongo.map(|mongo| mongo.username).flatten().unwrap_or(String::new()))
                .interact()
                .unwrap(),
            password: Input::with_theme(&theme)
                .with_prompt("MongoDB Password")
                .default(secret.clone().mongo.map(|mongo| mongo.password).flatten().unwrap_or(String::new()))
                .interact()
                .unwrap(),
            host: Input::with_theme(&theme)
                .with_prompt("MongoDB Host")
                .default(secret.clone().mongo.map(|mongo| mongo.host).flatten().unwrap_or(String::new()))
                .interact()
                .unwrap(),
        },
        stripe_secrets: StripeSecrets {
            secret_key: Input::with_theme(&theme)
                .with_prompt("Stripe Secret Key")
                .default(secret.clone().stripe_secrets.map(|stripe| stripe.secret_key).flatten().unwrap_or(String::new()))
                .interact()
                .unwrap(),
            publishable_key: Input::with_theme(&theme)
                .with_prompt("Stripe Publishable Key")
                .default(secret.clone().stripe_secrets.map(|stripe| stripe.publishable_key).flatten().unwrap_or(String::new()))
                .interact()
                .unwrap(),
        }
    };


    let client = get_client().await;

    let secret = create_kube_secret_object(secret, SECRET_NAME, &namespace).await;

    let secret_api: Api<Secret> = Api::namespaced(client, &namespace);

    match secret_api.create(&PostParams::default(),&secret).await {
        Ok(_) => println!("{} Created secret {} in namespace {}",
            style("✔").green().bold(),
            style(SECRET_NAME).green(),
            style(namespace).green()
        ),
        Err(kube::Error::Api(kube::core::ErrorResponse { reason, .. })) if reason == "AlreadyExists" => {
            secret_api.replace(SECRET_NAME, &PostParams::default(), &secret).await.unwrap();
        }
        Err(e) => panic!("Error getting secret: {:?}", e),
    }
}

#[tokio::test]
async fn can_get_namespace() {
    let namespace = get_namespace().await;

    println!("{}", namespace);
}


