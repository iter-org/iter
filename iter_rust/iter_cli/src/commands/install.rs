// create namespace object
// create secret object
// create daemon set
// create service for node port
// create service for load balancer
// create fission api resources
// create pod for frontend
// create pod for backend
// create pod for database or redis

use std::collections::BTreeMap;

use clap::Parser;
use dialoguer::console::style;
use k8s_openapi::api::apps::v1::{DaemonSet, DaemonSetSpec, Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    Container, ContainerPort, EnvVar, EnvVarSource, Namespace, ObjectFieldSelector, PodSpec,
    PodTemplateSpec, ResourceRequirements, Secret, Service, ServiceAccount, ServicePort,
    ServiceSpec,
};
use k8s_openapi::api::rbac::v1::{ClusterRole, ClusterRoleBinding, PolicyRule, RoleRef, Subject};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::{
    CustomResourceDefinition, CustomResourceDefinitionNames, CustomResourceDefinitionSpec,
    CustomResourceDefinitionVersion, CustomResourceValidation, JSONSchemaProps,
    JSONSchemaPropsOrArray,
};
use k8s_openapi::apimachinery::pkg::{
    api::resource::Quantity, apis::meta::v1::LabelSelector, util::intstr::IntOrString,
};
use k8s_openapi::ByteString;
use kube::core::ObjectMeta;
use serde_json::json;

use crate::{
    cli_kube::{create_or_update_cluster_resource, create_or_update_namespaced_resource},
    utils::unwrap_or_prompt,
};

use super::RunnableCommand;

#[derive(Parser, Debug, Clone)]
pub struct InstallCommand {
    /// the endpoint of the iter api server
    #[arg(short, long)]
    pub endpoint: Option<String>,
}

const ITER_NAMESPACE: &str = "iter";
const ITER_INGRESS_POD_NAME: &str = "iter-ingress-pod";
const ITER_SERVICE_ACCOUNT_NAME: &str = "iter-service-account";
const ITER_SERVICE_NAME: &str = "iter-service";
const ITER_INGRESS_ROLE_NAME: &str = "iter-ingress-role";
const ITER_INGRESS_ROLE_BINDING_NAME: &str = "iter-ingress-role-binding";
const ITER_DAEMONSET_NAME: &str = "iter-daemonset";
const INGRESS_DAEMONSET_IMAGE: &str = "public.ecr.aws/k2s9w9h5/iter/ingress:latest";
const ITER_API_IMAGE_URL: &str = "public.ecr.aws/k2s9w9h5/iter/api:latest";
const ITER_API_DEPLOYMENT_NAME: &str = "iter-api";

#[async_trait::async_trait]
impl RunnableCommand for InstallCommand {
    async fn run(self) -> Result<(), anyhow::Error> {
        let InstallCommand { endpoint } = self;

        let endpoint = unwrap_or_prompt(endpoint, "Provide iter endpoint")?;
    
        create_or_update_cluster_resource(Namespace {
            metadata: ObjectMeta {
                name: Some(ITER_NAMESPACE.to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .await?;
    
        create_or_update_namespaced_resource(Secret {
            metadata: ObjectMeta {
                name: Some("iter-secret".to_string()),
                namespace: Some(ITER_NAMESPACE.to_string()),
                ..Default::default()
            },
            data: Some(BTreeMap::from([(
                "secret".into(),
                ByteString(
                    base64::encode(&serde_json::to_string(&json!(
                        {
                            "endpoint": endpoint,
                        }
                    ))?)
                    .into_bytes()
                    .to_vec(),
                ),
            )])),
            ..Default::default()
        })
        .await?;
    
        create_or_update_namespaced_resource(DaemonSet {
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
        })
        .await?;
    
        create_or_update_namespaced_resource(Service {
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
        })
        .await?;
    
        create_or_update_cluster_resource(ClusterRole {
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
        })
        .await?;
    
        create_or_update_cluster_resource(ClusterRoleBinding {
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
        })
        .await?;
    
        create_or_update_namespaced_resource(ServiceAccount {
            metadata: ObjectMeta {
                name: Some(ITER_SERVICE_ACCOUNT_NAME.to_string()),
                namespace: Some(ITER_NAMESPACE.to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .await?;
    
        create_or_update_namespaced_resource(Deployment {
            metadata: ObjectMeta {
                name: Some(ITER_API_DEPLOYMENT_NAME.to_string()),
                namespace: Some(ITER_NAMESPACE.to_string()),
                ..Default::default()
            },
            spec: Some(DeploymentSpec {
                replicas: Some(1),
                selector: LabelSelector {
                    match_labels: Some(
                        [("app".to_string(), ITER_API_DEPLOYMENT_NAME.to_string())]
                            .into_iter()
                            .collect(),
                    ),
                    ..Default::default()
                },
                template: PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some(
                            [("app".to_string(), ITER_API_DEPLOYMENT_NAME.to_string())]
                                .into_iter()
                                .collect(),
                        ),
                        ..Default::default()
                    }),
                    spec: Some(PodSpec {
                        containers: vec![Container {
                            name: ITER_API_DEPLOYMENT_NAME.to_string(),
                            image: Some(ITER_API_IMAGE_URL.to_string()),
                            ports: Some(vec![ContainerPort {
                                container_port: 80,
                                ..Default::default()
                            }]),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        })
        .await?;
    
        //creates user
        create_or_update_cluster_resource(CustomResourceDefinition {
            metadata: ObjectMeta {
                name: Some("iter-users.iter.earth".to_string()),
                ..Default::default()
            },
            spec: CustomResourceDefinitionSpec {
                group: "iter.earth".to_string(),
                names: CustomResourceDefinitionNames {
                    kind: "User".to_string(),
                    plural: "iter-users".to_string(),
                    singular: Some("iter-user".to_string()),
                    ..Default::default()
                },
                scope: "Namespaced".to_string(),
                versions: vec![CustomResourceDefinitionVersion {
                    name: "v1".to_string(),
                    served: true,
                    storage: true,
                    schema: Some(CustomResourceValidation {
                        open_api_v3_schema: Some(JSONSchemaProps {
                            type_: Some("object".to_string()),
                            properties: Some(BTreeMap::from([
                                (
                                    "github-user-id".to_string(),
                                    JSONSchemaProps {
                                        type_: Some("string".to_string()),
                                        ..Default::default()
                                    },
                                ),
                                (
                                    "platform-permissions".to_string(),
                                    JSONSchemaProps {
                                        type_: Some("array".to_string()),
                                        items: Some(JSONSchemaPropsOrArray::Schema(Box::new(
                                            JSONSchemaProps {
                                                type_: Some("string".to_string()),
                                                ..Default::default()
                                            },
                                        ))),
                                        ..Default::default()
                                    }
                                )
                            ])),
                            ..Default::default()
                        }),
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        })
        .await?;
    
        //creates project
        create_or_update_cluster_resource(CustomResourceDefinition {
            metadata: ObjectMeta {
                name: Some("projects.iter.earth".to_string()),
                ..Default::default()
            },
            spec: CustomResourceDefinitionSpec {
                group: "iter.earth".to_string(),
                names: CustomResourceDefinitionNames {
                    kind: "Project".to_string(),
                    plural: "projects".to_string(),
                    singular: Some("project".to_string()),
                    ..Default::default()
                },
                scope: "Namespaced".to_string(),
                versions: vec![CustomResourceDefinitionVersion {
                    name: "v1".to_string(),
                    served: true,
                    storage: true,
                    schema: Some(CustomResourceValidation {
                        open_api_v3_schema: Some(JSONSchemaProps {
                            type_: Some("object".to_string()),
                            properties: Some(BTreeMap::from([
                                (
                                    "project_name".to_string(),
                                    JSONSchemaProps {
                                        type_: Some("string".to_string()),
                                        ..Default::default()
                                    },
                                ),
                                (
                                    "git_url".to_string(),
                                    JSONSchemaProps {
                                        type_: Some("string".to_string()),
                                        ..Default::default()
                                    }
                                )
                            ])),
                            ..Default::default()
                        }),
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        })
        .await?;
    
        //creates project_member
        create_or_update_cluster_resource(CustomResourceDefinition {
            metadata: ObjectMeta {
                name: Some("project-members.iter.earth".to_string()),
                ..Default::default()
            },
            spec: CustomResourceDefinitionSpec {
                group: "iter.earth".to_string(),
                names: CustomResourceDefinitionNames {
                    kind: "ProjectMember".to_string(),
                    plural: "project-members".to_string(),
                    singular: Some("project-member".to_string()),
                    ..Default::default()
                },
                scope: "Namespaced".to_string(),
                versions: vec![CustomResourceDefinitionVersion {
                    name: "v1".to_string(),
                    served: true,
                    storage: true,
                    schema: Some(CustomResourceValidation {
                        open_api_v3_schema: Some(JSONSchemaProps {
                            type_: Some("object".to_string()),
                            properties: Some(BTreeMap::from([
                                (
                                    "project".to_string(),
                                    JSONSchemaProps {
                                        type_: Some("string".to_string()),
                                        ..Default::default()
                                    },
                                ),
                                (
                                    "user".to_string(),
                                    JSONSchemaProps {
                                        type_: Some("string".to_string()),
                                        ..Default::default()
                                    }
                                ),
                                (
                                    "permissions".to_string(),
                                    JSONSchemaProps {
                                        type_: Some("array".to_string()),
                                        items: Some(JSONSchemaPropsOrArray::Schema(Box::new(
                                            JSONSchemaProps {
                                                type_: Some("string".to_string()),
                                                ..Default::default()
                                            },
                                        ))),
                                        ..Default::default()
                                    }
                                )
                            ])),
                            ..Default::default()
                        }),
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        })
        .await?;
    
        //creates deployments
        create_or_update_cluster_resource(CustomResourceDefinition {
            metadata: ObjectMeta {
                name: Some("deployments.iter.earth".to_string()),
                ..Default::default()
            },
            spec: CustomResourceDefinitionSpec {
                group: "iter.earth".to_string(),
                names: CustomResourceDefinitionNames {
                    kind: "Deployment".to_string(),
                    plural: "deployments".to_string(),
                    singular: Some("deployment".to_string()),
                    ..Default::default()
                },
                scope: "Namespaced".to_string(),
                versions: vec![CustomResourceDefinitionVersion {
                    name: "v1".to_string(),
                    served: true,
                    storage: true,
                    schema: Some(CustomResourceValidation {
                        open_api_v3_schema: Some(JSONSchemaProps {
                            type_: Some("object".to_string()),
                            properties: Some(BTreeMap::from([
                                (
                                    "project".to_string(),
                                    JSONSchemaProps {
                                        type_: Some("string".to_string()),
                                        ..Default::default()
                                    },
                                ),
                                (
                                    "user".to_string(),
                                    JSONSchemaProps {
                                        type_: Some("string".to_string()),
                                        ..Default::default()
                                    }
                                ),
                                (
                                    "permissions".to_string(),
                                    JSONSchemaProps {
                                        type_: Some("array".to_string()),
                                        items: Some(JSONSchemaPropsOrArray::Schema(Box::new(
                                            JSONSchemaProps {
                                                type_: Some("string".to_string()),
                                                ..Default::default()
                                            },
                                        ))),
                                        ..Default::default()
                                    }
                                )
                            ])),
                            ..Default::default()
                        }),
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        })
        .await?;
    
        println!(
            "{} {}",
            style("✔").green().bold(),
            style("Iter Install Completed").white().bold(),
        );
    
        Ok(())
    }
    
}
#[cfg(test)]
mod test {
    use crate::cli;
    #[tokio::test]
    async fn test_install_command() -> Result<(), anyhow::Error> {
        let args = vec![
            "iter".to_string(),
            "install".to_string(),
            "-e".to_string(),
            "endpoint_url".to_string(),
        ];

        cli(args.into_iter()).await
    }
    #[tokio::test]
    async fn test_install_command_without_args() -> Result<(), anyhow::Error> {
        let args = vec![
            "iter".to_string(),
            "install".to_string()];

        cli(args.into_iter()).await
    }
}
