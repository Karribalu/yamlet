use crate::{
    commands::validate::validate_file, models::InfraConfig,
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

pub async fn execute(config: &Config) {
    println!("Executing plan command with config: {:?}", config);

    let file_path = &config.options.file_path;
    println!("File path is: {}", file_path);
    let is_valid = validate_file(file_path);

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
            create_components(&config.metadata.name, &config.region, &config.components).await;
        }
        Err(err) => {
            eprintln!("Failed to parse YAML into InfraConfig: {}", err);
        }
    }
}

async fn create_components(_name: &str, region: &str, components: &[crate::models::Component]) {
    for component in components {}
}
