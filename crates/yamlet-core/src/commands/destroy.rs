use crate::{
    commands::validate::validate_file, models::InfraConfig, utils::constants::TEMPLATES_DIR,
};

#[derive(clap::Args, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    #[clap(flatten)]
    pub options: Options,
}

#[derive(clap::Args, Debug)]
pub struct Options {
    #[clap(short = 'f', long = "filepath")]
    pub file_path: String,
}

pub fn execute(config: &Config) {
    println!("Executing plan command with config: {:?}", config);

    let file_path = &config.options.file_path;
    println!("File path is: {}", file_path);
    let is_valid = validate_file(file_path);
    // if is_valid {
    //     println!("YAML file is valid.");
    // } else {
    //     println!("YAML file is invalid.");
    // }

    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Failed to read file: {}", err);
            return;
        }
    };

    // Try to parse using the structured model
    match InfraConfig::from_yaml(&content) {
        Ok(config) => {
            println!("Successfully parsed YAML using InfraConfig model");
            delete_components(&config.region, &config.components);
        }
        Err(err) => {
            eprintln!("Failed to parse YAML into InfraConfig: {}", err);
        }
    }
}

fn delete_components(region: &str, components: &[crate::models::Component]) {
    for component in components {
        match component.component_type.as_str() {
            "EC2Instance" => {
                // Create EC2 instance Terraform code
                delete_ec2_instance(region, component);
            }
            _ => {
                eprintln!("Unsupported component type: {}", component.component_type);
            }
        }
    }
}

fn delete_ec2_instance(region: &str, component: &crate::models::Component) {
    let name = &component.name;
    let instance_type = component
        .get_property_as_string("instanceType")
        .unwrap_or_else(|| "t2.micro".to_string());
    let ami_id = component
        .get_property_as_string("ami")
        .unwrap_or_else(|| "ami-0c55b159cbfafe1f0".to_string()); // Default to Ubuntu 20.04 LTS AMI

    // let temp_dir = std::env::temp_dir();
    let temp_dir = std::env::current_dir()
        .expect("Unable to get current directory")
        .join("workspaces")
        .join(name);

    // Copy the template files to a new directory for this component from TEMPLATES_DIR
    let component_dir = temp_dir.join(format!("{}_{}", name, "EC2Instance"));
    std::fs::create_dir_all(&component_dir).expect("Unable to create component directory");

    let templates_dir = std::path::Path::new(TEMPLATES_DIR).join("EC2Instance");
    println!("Copying templates from {:?}", templates_dir);
    for entry in std::fs::read_dir(templates_dir).expect("Unable to read templates directory") {
        let entry = entry.expect("Unable to read entry");
        let src = entry.path();
        let dst = component_dir.join(src.file_name().expect("Unable to get file name"));
        std::fs::copy(&src, &dst).expect("Unable to copy template file");
    }

    let tfvars_content = format!(
        "aws_region = \"{}\"\ninstance_type = \"{}\"\nami_id = \"{}\"",
        region, instance_type, ami_id
    );
    std::fs::write(component_dir.join("vars.tfvars"), tfvars_content)
        .expect("Unable to write tfvars file");

    // Run terraform init and apply in the component directory
    let mut init_status = std::process::Command::new("terraform");
    let init_status = init_status
        .current_dir(&component_dir)
        .arg("init")
        .status()
        .expect("Failed to execute terraform init");
    if !init_status.success() {
        eprintln!("terraform init failed");
        return;
    }
    let mut apply_status = std::process::Command::new("terraform");
    let apply_status = apply_status
        .arg("destroy")
        .current_dir(&component_dir)
        .arg("-var-file=vars.tfvars")
        .arg("-auto-approve")
        .status()
        .expect("Failed to execute terraform apply");
    if !apply_status.success() {
        eprintln!("terraform apply failed");
        return;
    }
}
