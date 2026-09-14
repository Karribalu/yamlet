use crate::models::InfraConfig;

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

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Failed to read file: {0}")]
    FileReadError(String),
    #[error("Failed to parse YAML: {0}")]
    YamlParseError(String),
    #[error("InfraConfig validation error: {0}")]
    InfraConfigValidationError(String),
}

pub fn execute(config: &Config) -> Result<(), ValidationError> {
    println!("Executing validate command with config: {:?}", config);

    let file_path = &config.options.file_path;
    println!("File path is: {}", file_path);
    validate_file(file_path)?;
    Ok(())
}
pub fn validate_file(file_path: &str) -> Result<(), ValidationError> {
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Failed to read file: {}", err);
            return Err(ValidationError::FileReadError(err.to_string()));
        }
    };

    // Try to parse using the structured model
    match InfraConfig::from_yaml(&content) {
        Ok(config) => {
            println!("Successfully parsed YAML using InfraConfig model");
            return validate_infra_config(&config);
        }
        Err(err) => {
            eprintln!("Failed to parse YAML into InfraConfig: {}", err);
            return Err(ValidationError::YamlParseError(err.to_string()));
        }
    }
}

pub fn validate_infra_config(config: &InfraConfig) -> Result<(), ValidationError> {
    // Validate kind
    // if !constants::SupportKind::is_valid(&config.kind) {
    //     eprintln!("Invalid kind: {}", config.kind);
    //     return Err(ValidationError::InfraConfigValidationError(format!(
    //         "Invalid kind: {}",
    //         config.kind
    //     )));
    // }

    // if !constants::SupportCloud::is_valid(&config.cloud) {
    //     eprintln!("Invalid cloud: {}", config.cloud);
    //     return false;
    // }

    // Validate metadata
    if config.metadata.name.is_empty() {
        eprintln!("Metadata name cannot be empty");
        return Err(ValidationError::InfraConfigValidationError(
            "Metadata cannot be empty".to_string(),
        ));
    }

    // Validate components
    if config.components.is_empty() {
        eprintln!("At least one component is required");
        return Err(ValidationError::InfraConfigValidationError(
            "At least one component is required".to_string(),
        ));
    }

    for (idx, component) in config.components.iter().enumerate() {
        if component.component_type.is_empty() {
            eprintln!("Component {} has empty type", idx);
            return Err(ValidationError::InfraConfigValidationError(
                "Component type cannot be empty".to_string(),
            ));
        }
        if component.name.is_empty() {
            eprintln!("Component {} has empty name", idx);
            return Err(ValidationError::InfraConfigValidationError(
                "Component name cannot be empty".to_string(),
            ));
        }
    }

    println!("InfraConfig validation passed");
    Ok(())
}
