use serde::Serialize;

use crate::models::{InfraConfig, Plan, PlanError};

pub mod constants;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum OperationType {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComponentPreview {
    pub component_type: String,
    pub name: String,
    pub operation_type: OperationType,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanPreviewDeployment {
    pub deployment_type: String,
    pub deployment_name: String,
    pub components: Vec<ComponentPreview>,
}

pub fn plan_components(
    config: &InfraConfig,
) -> Result<(Plan, PlanPreviewDeployment), crate::models::PlanError> {
    // let dependency_tree = plan_components_sequence(&config.components);
    let mut preview_plan = PlanPreviewDeployment {
        deployment_type: config.kind.as_str().to_string(),
        deployment_name: config.metadata.name.clone(),
        components: Vec::new(),
    };
    tracing::info!("Planning components: {:?}", config.components);
    for component in &config.components {
        preview_plan.components.push(ComponentPreview {
            component_type: component.component_type.clone(),
            name: component.name.clone(),
            operation_type: OperationType::Create, // For now, assume all are Create operations
        });
        match component.component_type.as_str() {
            "EC2Instance" => {
                // Create EC2 instance Terraform code
                plan_ec2_instance(&config.region, component)?;
            }
            _ => {
                eprintln!("Unsupported component type: {}", component.component_type);
            }
        }
    }

    Ok((Plan {}, preview_plan))
}

// fn plan_components_sequence(
//     components: &[crate::models::Component],
// ) -> Vec<&crate::models::Component> {
//     let mut sequence = Vec::new();
//     let mut visited = HashSet::new();

//     for component in components {
//         if !visited.contains(component) {
//             plan_component_sequence(component, &mut sequence, &mut visited);
//         }
//     }

//     sequence
// }

fn plan_ec2_instance(region: &str, component: &crate::models::Component) -> Result<(), PlanError> {
    let name = &component.name;
    let instance_type = component
        .get_property_as_string("instance_type")
        .expect("Missing mandatory property 'instance_type' in component 'EC2Instance'");
    let ami_id = component
        .get_property_as_string("ami")
        .expect("Missing mandatory property 'ami' in component 'EC2Instance'");

    tracing::debug!(
        "Planning EC2 Instance: name={}, region={}, instance_type={}, ami={}",
        name,
        region,
        instance_type,
        ami_id
    );
    // Here you would generate the custom plan for the EC2 instance

    Ok(())
}
