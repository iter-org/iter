//! # Drawbridge-Ingress
//!
//! A load balancer written in rust made for routing traffic to kubernetes services.
//!
//! ## How it works
//! 1. A task which regularly queries the kubernetes api for the list of services, ingresses, and listens for changes.
//! 2. letsencrypt is used to generate certificates for the hosts configured in the ingress.
//! 3. Constructs a routing rable based on the loaded kubernetes ingress configurations.
//! 4. listen on :80 and :443 for incoming http and https requests. The requests are routed to the appropriate service
//! according to the routing table, and a reverse proxy is used to forward the request to the service.
//! 5. incoming https requests are matched against the appropriate SSL certificate generated by letsencrypt.
//!
#![feature(never_type)]
#![feature(try_blocks)]
use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use kube_config_tracker::{RoutingTable};
use proxy::proxy_request;
use std::net::SocketAddr;
use std::sync::Arc;
use iter_tls_acceptor::tls_acceptor::TlsAcceptor;

use crate::error::{Code, IngressLoadBalancerError};

mod lets_encrypt;
mod error;
mod kube_config_tracker;
mod proxy;
mod certificate_state;

//  Components
//  - Ingress
//      - endpoints
//          - /.well-known/letsencrypt -> letsencrypt challenge contents
//          - /health-check -> static response
//          - {other} -> proxy to service or serverless function
//      - watches
//          - certificates
//          - services
//          - ingresses
//          - serverless deployments

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), IngressLoadBalancerError> {
    let routing_table = Arc::new(RoutingTable::new());
    let certificate_state = Arc::new(certificate_state::CertificateState::new());

    // start a task which listens for changes to the kubernetes api
    // and updates the routing table accordingly
    tokio::spawn(routing_table.clone().start_watching());

    let cert_state = certificate_state.clone();
    let proxy_service_handler = Arc::new(move || {
        let routing_table = routing_table.clone();
        let cert_state = cert_state.clone();
        async move {
            Ok::<_, Error>(service_fn(move |req| {
                proxy_request(routing_table.clone(), req, cert_state.clone())
            }))
        }
    });

    let proxy_service_handler_clone = proxy_service_handler.clone();
    let proxy_service_http = make_service_fn(move |_| proxy_service_handler_clone.clone()());
    let proxy_service_https = make_service_fn(move |_| proxy_service_handler.clone()());

    let https_incoming = AddrIncoming::bind(&SocketAddr::from(([0, 0, 0, 0], 443))).unwrap();
    let incoming_tls_acceptor = TlsAcceptor::new(https_incoming, certificate_state.clone());
    let http_server_task = tokio::task::spawn(Server::bind(&SocketAddr::from(([0, 0, 0, 0], 80))).serve(proxy_service_http));
    let https_server_task = tokio::task::spawn(Server::builder(incoming_tls_acceptor).serve(proxy_service_https));

    tokio::select! {
        http_server = http_server_task => http_server.unwrap().unwrap(),
        https_server = https_server_task => https_server.unwrap().unwrap(),
    }

    Ok(())
}
