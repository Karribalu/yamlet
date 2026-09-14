use std::collections::{BTreeMap, HashMap};

use prost_types::{value::Kind as PbKind, ListValue, Struct as PbStruct, Value as PbValue};
use tonic::transport::{Channel, Endpoint};

use crate::models::{CloudProvider, Component, Dependency, InfraConfig};
use plugin_sdk::provider::provider::provider_client::ProviderClient;
use plugin_sdk::provider::provider as pb;

pub struct ProviderClientRegistry {
    // map of cloud name (e.g., "AWS") to gRPC endpoint URL
    endpoints: HashMap<String, String>,
}

impl ProviderClientRegistry {
    pub fn from_env() -> Self {
        let mut endpoints = HashMap::new();
        if let Ok(url) = std::env::var("LETUS_PROVIDER_AWS_ENDPOINT") {
            endpoints.insert("AWS".to_string(), url);
        }
        // Future: GCP, Azure, etc.
        Self { endpoints }
    }

    pub fn has_endpoint(&self, cloud: &CloudProvider) -> bool {
        self.endpoints.contains_key(cloud.as_str())
    }

    pub async fn get_client(
        &self,
        cloud: &CloudProvider,
    ) -> Result<ProviderClient<Channel>, String> {
        let url = self
            .endpoints
            .get(cloud.as_str())
            .ok_or_else(|| format!("no endpoint configured for {}", cloud.as_str()))?
            .clone();
        let endpoint = Endpoint::from_shared(url.clone()).map_err(|e| e.to_string())?;
        let channel = endpoint.connect().await.map_err(|e| e.to_string())?;
        Ok(ProviderClient::new(channel))
    }
}

pub async fn grpc_apply_component(
    registry: &ProviderClientRegistry,
    config: &InfraConfig,
    component: &Component,
) -> Result<pb::ApplyResponse, tonic::Status> {
    let mut client = registry
        .get_client(&config.cloud)
        .await
        .map_err(|e| tonic::Status::unavailable(format!("failed to connect to provider: {e}")))?;

    let ctx = pb::InfraContext {
        deployment_name: config.metadata.name.clone(),
        workspace: std::env::var("LETUS_WORKSPACE").unwrap_or_else(|_| "default".to_string()),
        cloud: config.cloud.as_str().to_string(),
        region: config.region.clone(),
        variables: Default::default(),
    };

    let req = pb::ApplyRequest {
        context: Some(ctx),
        component: Some(component_to_pb(component)),
    };

    client.apply(req).await.map(|r| r.into_inner())
}

fn serde_yaml_to_json(value: &serde_yaml::Value) -> serde_json::Value {
    // Convert YAML typing to JSON typing via string roundtrip
    // Safer: direct mapping when possible; here we leverage serde conversion
    serde_json::to_value(value).unwrap_or(serde_json::Value::Null)
}

fn json_to_pb_value(v: serde_json::Value) -> PbValue {
    match v {
        serde_json::Value::Null => PbValue {
            kind: Some(PbKind::NullValue(0)),
        },
        serde_json::Value::Bool(b) => PbValue {
            kind: Some(PbKind::BoolValue(b)),
        },
        serde_json::Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            PbValue {
                kind: Some(PbKind::NumberValue(f)),
            }
        }
        serde_json::Value::String(s) => PbValue {
            kind: Some(PbKind::StringValue(s)),
        },
        serde_json::Value::Array(arr) => {
            let values = arr.into_iter().map(json_to_pb_value).collect::<Vec<_>>();
            PbValue {
                kind: Some(PbKind::ListValue(ListValue { values })),
            }
        }
        serde_json::Value::Object(map) => {
            let fields = map
                .into_iter()
                .map(|(k, v)| (k, json_to_pb_value(v)))
                .collect::<BTreeMap<_, _>>();
            PbValue {
                kind: Some(PbKind::StructValue(PbStruct { fields })),
            }
        }
    }
}

fn yaml_to_pb_struct(yaml: &serde_yaml::Value) -> PbStruct {
    let json = serde_yaml_to_json(yaml);
    match json {
        serde_json::Value::Object(map) => {
            let fields = map
                .into_iter()
                .map(|(k, v)| (k, json_to_pb_value(v)))
                .collect::<BTreeMap<_, _>>();
            PbStruct { fields }
        }
        _ => PbStruct {
            fields: Default::default(),
        },
    }
}

fn dependency_to_pb(dep: &Dependency) -> pb::Dependency {
    pb::Dependency {
        r#type: dep.dep_type.clone(),
        name: dep.name.clone(),
    }
}

fn component_to_pb(component: &Component) -> pb::ComponentSpec {
    pb::ComponentSpec {
        component_type: component.component_type.clone(),
        name: component.name.clone(),
        properties: Some(yaml_to_pb_struct(&component.properties)),
        depends_on: component
            .depends_on
            .as_ref()
            .map(|v| v.iter().map(dependency_to_pb).collect())
            .unwrap_or_default(),
        connects_to: component
            .connects_to
            .as_ref()
            .map(|v| v.iter().map(dependency_to_pb).collect())
            .unwrap_or_default(),
    }
}

// Added Plan RPC client function
pub async fn grpc_plan_component(
    registry: &ProviderClientRegistry,
    config: &InfraConfig,
    component: &Component,
) -> Result<pb::PlanResponse, tonic::Status> {
    let mut client = registry
        .get_client(&config.cloud)
        .await
        .map_err(|e| tonic::Status::unavailable(format!("failed to connect to provider: {e}")))?;

    let ctx = pb::InfraContext {
        deployment_name: config.metadata.name.clone(),
        workspace: std::env::var("LETUS_WORKSPACE").unwrap_or_else(|_| "default".to_string()),
        cloud: config.cloud.as_str().to_string(),
        region: config.region.clone(),
        variables: Default::default(),
    };

    let req = pb::PlanRequest {
        context: Some(ctx),
        component: Some(component_to_pb(component)),
    };

    client.plan(req).await.map(|r| r.into_inner())
}
