// create namespace object
// create secret object
// create daemon set
// create service for node port
// create service for load balancer
// create fission api resources
// create pod for frontend
// create pod for backend
// create pod for database or redis

use dialoguer::console::style;
use k8s_openapi::api::apps::v1::{DaemonSet, DaemonSetSpec};
use k8s_openapi::api::core::v1::{
    Container, ContainerPort, EnvVar, EnvVarSource, Namespace, ObjectFieldSelector, PodSpec,
    PodTemplateSpec, ResourceRequirements, Secret, Service, ServiceAccount, ServicePort,
    ServiceSpec,
};
use k8s_openapi::api::rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, RoleRef, Subject};
use k8s_openapi::apimachinery::pkg::{
    api::resource::Quantity, apis::meta::v1::LabelSelector, util::intstr::IntOrString,
};
use kube::core::ObjectMeta;
use serde_json::json;

use crate::{
    cli_kube::{create_or_update_cluster_resource, create_or_update_namespaced_resource},
    cli_types,
    utils::unwrap_or_prompt,
};

const ITER_NAMESPACE: &str = "iter";
const ITER_INGRESS_POD_NAME: &str = "iter-ingress-pod";
const ITER_SERVICE_ACCOUNT_NAME: &str = "iter-service-account";
const ITER_SERVICE_NAME: &str = "iter-service";
const ITER_INGRESS_ROLE_NAME: &str = "iter-ingress-role";
const ITER_INGRESS_ROLE_BINDING_NAME: &str = "iter-ingress-role-binding";
const ITER_DAEMONSET_NAME: &str = "iter-daemonset";
const INGRESS_DAEMONSET_IMAGE: &str = "public.ecr.aws/k2s9w9h5/iter/ingress:latest";

pub async fn install_command(
    cli_types::InstallCommand {
        domain,
    }: cli_types::InstallCommand,
) -> Result<(), anyhow::Error> {
    let domain = unwrap_or_prompt(domain, "Provide iter domain")?;

    create_or_update_cluster_resource::<Namespace>(json!({
        "apiVersion": "v1",
        "kind": "Namespace",
        "metadata": {
            "name": &ITER_NAMESPACE
        }
    }))
    .await?;

    create_or_update_namespaced_resource::<Secret>(json!({
        "apiVersion": "v1",
        "kind": "Secret",
        "metadata": {
            "name": "iter-secret",
            "namespace": &ITER_NAMESPACE
        },
        "data": {
            "secret": base64::encode(&serde_json::to_string(&json!(
                {
                    "domain": domain,
                }
            ))?),
        }
    }))
    .await?;

    let daemonset = DaemonSet {
        metadata: ObjectMeta {
            name: Some(ITER_DAEMONSET_NAME.to_string()),
            namespace: Some(ITER_NAMESPACE.to_string()),
            ..Default::default()
        },
        spec: Some(DaemonSetSpec {
            selector: LabelSelector {
                match_labels: Some(
                    [("app".to_string(), ITER_INGRESS_POD_NAME.to_string())]
                        .into_iter()
                        .collect(),
                ),
                ..Default::default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(
                        [("app".to_string(), ITER_INGRESS_POD_NAME.to_string())]
                            .into_iter()
                            .collect(),
                    ),
                    ..Default::default()
                }),
                spec: Some(PodSpec {
                    service_account_name: Some(ITER_SERVICE_ACCOUNT_NAME.to_string()),
                    termination_grace_period_seconds: Some(0),
                    containers: vec![Container {
                        name: ITER_INGRESS_POD_NAME.to_string(),
                        image: Some(INGRESS_DAEMONSET_IMAGE.to_string()),
                        env: Some(vec![EnvVar {
                            name: "CURRENT_POD_NAME".to_string(),
                            value_from: Some(EnvVarSource {
                                field_ref: Some(ObjectFieldSelector {
                                    field_path: "metadata.name".to_string(),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }]),
                        resources: Some(ResourceRequirements {
                            limits: Some(
                                [
                                    ("memory".to_string(), Quantity("50Mi".to_string())),
                                    ("cpu".to_string(), Quantity("0.4".to_string())),
                                ]
                                .into_iter()
                                .collect(),
                            ),
                            ..Default::default()
                        }),
                        ports: Some(vec![
                            ContainerPort {
                                container_port: 80,
                                ..Default::default()
                            },
                            ContainerPort {
                                container_port: 443,
                                ..Default::default()
                            },
                        ]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    create_or_update_namespaced_resource::<DaemonSet>(serde_json::to_value(daemonset)?).await?;

    let service = Service {
        metadata: ObjectMeta {
            name: Some(ITER_SERVICE_NAME.to_string()),
            namespace: Some(ITER_NAMESPACE.to_string()),
            ..Default::default()
        },
        spec: Some(ServiceSpec {
            type_: Some("NodePort".to_string()),
            selector: Some(
                [("app".to_string(), ITER_INGRESS_POD_NAME.to_string())]
                    .into_iter()
                    .collect(),
            ),
            ports: Some(vec![
                ServicePort {
                    name: Some("http".to_string()),
                    node_port: Some(30001),
                    protocol: Some("TCP".to_string()),
                    port: 30001,
                    target_port: Some(IntOrString::Int(80)),
                    ..Default::default()
                },
                ServicePort {
                    name: Some("https".to_string()),
                    node_port: Some(30002),
                    protocol: Some("TCP".to_string()),
                    port: 30002,
                    target_port: Some(IntOrString::Int(443)),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }),
        ..Default::default()
    };

    create_or_update_namespaced_resource::<Service>(serde_json::to_value(service)?).await?;

    let cluster_role = ClusterRole {
        metadata: ObjectMeta {
            name: Some(ITER_INGRESS_ROLE_NAME.to_string()),
            ..Default::default()
        },
        rules: Some(vec![
            PolicyRule {
                api_groups: Some(vec![
                    "extensions".to_string(),
                    "networking.k8s.io".to_string(),
                    "".to_string(),
                ]),
                resources: Some(vec![
                    "ingresses".to_string(),
                    "pods".to_string(),
                    "services".to_string(),
                    "secrets".to_string(),
                ]),
                verbs: vec!["get".to_string(), "list".to_string(), "watch".to_string()],
                ..Default::default()
            },
            PolicyRule {
                api_groups: Some(vec![
                    "extensions".to_string(),
                    "networking.k8s.io".to_string(),
                    "".to_string(),
                ]),
                resources: Some(vec!["secrets".to_string()]),
                verbs: vec!["update".to_string(), "create".to_string()],
                ..Default::default()
            },
        ]),
        ..Default::default()
    };

    create_or_update_cluster_resource::<ClusterRole>(serde_json::to_value(cluster_role)?).await?;

    let cluster_role_binding = ClusterRoleBinding {
        metadata: ObjectMeta {
            name: Some(ITER_INGRESS_ROLE_BINDING_NAME.to_string()),
            ..Default::default()
        },
        role_ref: RoleRef {
            api_group: "rbac.authorization.k8s.io".to_string(),
            kind: "ClusterRole".to_string(),
            name: ITER_INGRESS_ROLE_NAME.to_string(),
            ..Default::default()
        },
        subjects: Some(vec![Subject {
            kind: "ServiceAccount".to_string(),
            name: ITER_SERVICE_ACCOUNT_NAME.to_string(),
            namespace: Some(ITER_NAMESPACE.to_string()),
            ..Default::default()
        }]),
        ..Default::default()
    };

    create_or_update_cluster_resource::<ClusterRoleBinding>(serde_json::to_value(
        cluster_role_binding,
    )?)
    .await?;

    let service_account = ServiceAccount {
        metadata: ObjectMeta {
            name: Some(ITER_SERVICE_ACCOUNT_NAME.to_string()),
            namespace: Some(ITER_NAMESPACE.to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    create_or_update_namespaced_resource::<ServiceAccount>(serde_json::to_value(service_account)?)
        .await?;

    println!(
        "{} {}",
        style("✔").green().bold(),
        style("Iter Install Completed").white().bold(),
    );

    Ok(())
}
