use crate::{
    commands::validate::validate_file,
    models::{InfraConfig, PlanError},
    utils::{OperationType, PlanPreviewDeployment, plan_components},
};
use comfy_table::{Attribute, Cell, Color, Table};

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

fn format_plan_preview(preview: &PlanPreviewDeployment) {
    let mut table = Table::new();
    table.load_preset(comfy_table::presets::NOTHING);
    table.set_header(vec!["", "Type", "Name", "Plan", "Info"]);

    // Add deployment row
    table.add_row(vec![
        Cell::new(""),
        Cell::new(&preview.deployment_type),
        Cell::new(&preview.deployment_name),
        Cell::new(""),
        Cell::new(""),
    ]);

    // Add component rows
    for (index, component) in preview.components.iter().enumerate() {
        let is_last = index == preview.components.len() - 1;
        let prefix = if is_last { "└─" } else { "├─" };

        let (operation_symbol, operation_text, operation_color) = match component.operation_type {
            OperationType::Create => ("+", "create", Color::Green),
            OperationType::Update => ("~", "update", Color::Yellow),
            OperationType::Delete => ("-", "delete", Color::Red),
        };

        table.add_row(vec![
            Cell::new(operation_symbol).fg(operation_color),
            Cell::new(format!(" {} {}", prefix, component.component_type)),
            Cell::new(&component.name),
            Cell::new(operation_text).fg(operation_color),
            Cell::new(""),
        ]);
    }

    println!("\n{}", table);
}

pub fn execute(config: &Config) {
    println!("Executing plan command with config: {:?}", config);

    let file_path = &config.options.file_path;
    println!("File path is: {}", file_path);

    match validate_file(file_path) {
        Ok(()) => {
            println!("Validation is passed");
            tracing::debug!("Plan Validation is passed");
        }
        Err(err) => {
            println!("Plan Failed while validating file: {}", err.to_string());
            tracing::error!(message = "Plan Failed", error = %err);
        }
    }

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
            tracing::info!("Starting the planning stage with the config: {}", config);
            match plan_components(&config) {
                Ok((_plan, preview)) => {
                    println!("Plan generated successfully:");
                    format_plan_preview(&preview);
                }
                Err(err) => {
                    eprintln!("Failed to generate plan: {}", err);
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to parse YAML into InfraConfig: {}", err);
        }
    }
}
