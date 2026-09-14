use aws_config::BehaviorVersion;
use prost_types::{Struct as PbStruct, Value as PbValue, value::Kind as PbKind};
use std::collections::BTreeMap;
use tonic::{Request, Response, Status};
use tracing::info;

pub mod tests;

pub mod aws;

use pb::provider_server::{Provider, ProviderServer};

use plugin_sdk::provider::provider as pb;
pub struct AwsProvider;

fn pb_struct_to_json(s: &PbStruct) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in &s.fields {
        map.insert(k.clone(), pb_value_to_json(v));
    }
    serde_json::Value::Object(map)
}

fn pb_value_to_json(v: &PbValue) -> serde_json::Value {
    match &v.kind {
        Some(PbKind::NullValue(_)) => serde_json::Value::Null,
        Some(PbKind::NumberValue(n)) => serde_json::Value::from(*n),
        Some(PbKind::StringValue(s)) => serde_json::Value::from(s.clone()),
        Some(PbKind::BoolValue(b)) => serde_json::Value::from(*b),
        Some(PbKind::StructValue(s)) => pb_struct_to_json(s),
        Some(PbKind::ListValue(list)) => {
            let arr = list.values.iter().map(pb_value_to_json).collect::<Vec<_>>();
            serde_json::Value::Array(arr)
        }
        None => serde_json::Value::Null,
    }
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
                kind: Some(PbKind::ListValue(prost_types::ListValue { values })),
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

#[tonic::async_trait]
impl Provider for AwsProvider {
    async fn get_capabilities(
        &self,
        _request: Request<pb::GetCapabilitiesRequest>,
    ) -> Result<Response<pb::GetCapabilitiesResponse>, Status> {
        Ok(Response::new(pb::GetCapabilitiesResponse {
            resource_types: vec!["EC2Instance".to_string()],
        }))
    }

    async fn plan(
        &self,
        request: Request<pb::PlanRequest>,
    ) -> Result<Response<pb::PlanResponse>, Status> {
        let req = request.into_inner();
        let supported = match req
            .component
            .as_ref()
            .and_then(|c| Some(c.component_type.as_str()))
        {
            Some("EC2Instance") => true,
            _ => false,
        };
        Ok(Response::new(pb::PlanResponse {
            supported,
            plan_summary: if supported {
                "Create EC2 instance".to_string()
            } else {
                "Unsupported component".to_string()
            },
            computed: Some(PbStruct {
                fields: Default::default(),
            }),
        }))
    }

    async fn apply(
        &self,
        request: Request<pb::ApplyRequest>,
    ) -> Result<Response<pb::ApplyResponse>, Status> {
        let req = request.into_inner();
        let ctx = req
            .context
            .ok_or_else(|| Status::invalid_argument("missing context"))?;
        let comp = req
            .component
            .ok_or_else(|| Status::invalid_argument("missing component"))?;
        match comp.component_type.as_str() {
            "EC2Instance" => apply_ec2(ctx.region, comp).await,
            other => Err(Status::unimplemented(format!(
                "Unsupported component_type: {other}"
            ))),
        }
    }

    async fn destroy(
        &self,
        _request: Request<pb::DestroyRequest>,
    ) -> Result<Response<pb::DestroyResponse>, Status> {
        Ok(Response::new(pb::DestroyResponse {
            success: false,
            error_message: "not implemented".to_string(),
        }))
    }
}

async fn apply_ec2(
    region: String,
    comp: pb::ComponentSpec,
) -> Result<Response<pb::ApplyResponse>, Status> {
    let props = comp
        .properties
        .ok_or_else(|| Status::invalid_argument("missing properties"))?;
    let json = pb_struct_to_json(&props);

    let image_id = json
        .get("image_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Status::invalid_argument("properties.image_id is required"))?
        .to_string();
    let instance_type_str = json
        .get("instance_type")
        .and_then(|v| v.as_str())
        .unwrap_or("t3.micro");

    // Optional fields
    let key_name = json
        .get("key_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let subnet_id = json
        .get("subnet_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let security_group_ids: Option<Vec<String>> = json
        .get("security_group_ids")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        });

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(aws_types::region::Region::new(region.clone()))
        .load()
        .await;
    let client = aws_sdk_ec2::Client::new(&config);

    let instance_type = match instance_type_str.parse() {
        Ok(it) => it,
        Err(_) => {
            return Err(Status::invalid_argument(format!(
                "invalid instance_type: {}",
                instance_type_str
            )));
        }
    };

    let mut req = client
        .run_instances()
        .image_id(image_id)
        .instance_type(instance_type)
        .min_count(1)
        .max_count(1);

    if let Some(kn) = key_name {
        req = req.key_name(kn);
    }
    if let Some(sn) = subnet_id {
        req = req.subnet_id(sn);
    }
    if let Some(sg_ids) = security_group_ids {
        req = req.set_security_group_ids(Some(sg_ids));
    }

    let resp = req.send().await.map_err(map_sdk_err)?;
    let instance_id = resp
        .instances()
        .first()
        .and_then(|i| i.instance_id())
        .ok_or_else(|| Status::internal("EC2 did not return instance id"))?
        .to_string();

    let mut outputs = BTreeMap::new();
    outputs.insert(
        "instance_id".to_string(),
        PbValue {
            kind: Some(PbKind::StringValue(instance_id.clone())),
        },
    );

    Ok(Response::new(pb::ApplyResponse {
        success: true,
        resource_id: instance_id,
        outputs: Some(PbStruct { fields: outputs }),
        error_message: String::new(),
    }))
}

fn map_sdk_err<E: std::fmt::Display>(e: E) -> Status {
    Status::internal(format!("AWS SDK error: {}", e))
}

pub async fn serve(addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let svc = AwsProvider;
    let addr = addr.parse()?;
    info!("aws-provider listening on {}", addr);
    tonic::transport::Server::builder()
        .add_service(ProviderServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
