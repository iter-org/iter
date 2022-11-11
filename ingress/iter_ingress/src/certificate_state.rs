use std::{collections::HashMap, sync::Arc};

use hyper::{Response, Body};
use iter_letsencrypt::challenge::Http01Challenge;
use rustls::{sign::CertifiedKey, ServerConfig};
use serde::{Serialize, Deserialize};
use iter_tls_acceptor::tls_acceptor::ResolvesServerConf;
use tokio::sync::RwLock;


pub type Host = String;
pub type Path = String;

pub struct CertificateState {
    pub certs: RwLock<HashMap<String, CertKey>>,
    pub challenges: RwLock<HashMap<(Host, Path), Http01Challenge>>,
}
#[derive(Clone)]
pub struct CertKey {
    // DER encoded
    pub certs: Vec<Vec<u8>>,
    pub private_key: Vec<u8>,
    pub certified_key: Arc<CertifiedKey>,
    pub server_config: Arc<ServerConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CertData {
    pub private_key: Vec<u8>,
    pub certs: Vec<Vec<u8>>,
}


impl CertificateState {
    pub fn new() -> CertificateState {
        CertificateState {
            certs: RwLock::new(HashMap::new()),
            challenges: RwLock::new(HashMap::new()),
        }
    }

    pub async fn apply_challenge(&self, challenge: Http01Challenge) {
        self.challenges.write().await.insert((challenge.domain.clone(), challenge.path.clone()), challenge.clone());
        println!("applied challenge on: {}{}", challenge.domain, challenge.path);
    }

    #[inline]
    pub async fn handle_if_challenge(&self, host: &str, path: &str) -> Option<Response<Body>> {
        if let Some(challenge) = self
            .challenges
            .read()
            .await
            .get(&(host.to_string(), path.to_string()))
        {
            println!("serving incoming challenge on: {}{}", host, path);
            return Some(Response::new(Body::from(challenge.contents.clone())));
        }
        None
    }
}


#[async_trait::async_trait]
impl ResolvesServerConf for CertificateState {
    async fn resolve_server_config(self: Arc<Self>, hello: &rustls::server::ClientHello) -> Option<Arc<ServerConfig>> {
        let name = hello.server_name();

        match name {
            Some(name) => {
                let state = self.clone();
                let certs = state.certs.read().await;
                let cert = certs.get(name);

                match cert {
                    Some(cert) => {
                        Some(cert.server_config.clone())
                    }
                    None => None,
                }
            }
            None => None,
        }
    }
}

impl Serialize for CertKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct CertKeyMinimal {
            pub certs: Vec<Vec<u8>>,
            pub private_key: Vec<u8>,
        }

        let cert_key_minimal = CertKeyMinimal {
            certs: self.certs.clone(),
            private_key: self.private_key.clone(),
        };

        cert_key_minimal.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CertKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CertKeyMinimal {
            pub certs: Vec<Vec<u8>>,
            pub private_key: Vec<u8>,
        }

        let cert_key_minimal = CertKeyMinimal::deserialize(deserializer)?;

        Ok(cert_key_from(
            cert_key_minimal.certs,
            cert_key_minimal.private_key,
        ))
    }
}


impl std::fmt::Debug for CertKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CertKey {{ certs: {:?}, private_key: {:?} }}",
            self.certs, self.private_key
        )
    }
}

pub fn cert_key_from(certs: Vec<Vec<u8>>, private_key: Vec<u8>) -> CertKey {
    let key = rustls::sign::RsaSigningKey::new(&rustls::PrivateKey(private_key.clone())).unwrap();

    CertKey {
        certified_key: Arc::new(CertifiedKey::new(
            certs
                .iter()
                .map(|cert| rustls::Certificate(cert.clone()))
                .collect(),
            Arc::new(key),
        )),
        server_config: Arc::new(rustls::ServerConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(certs.clone().iter()
            .map(|cert| rustls::Certificate(cert.clone()))
            .collect(), rustls::PrivateKey(private_key.clone()))
            .unwrap()),
        certs,
        private_key,

    }
}
