// Config Commands - Tauri handlers for configuration management

use aof_core::AgentConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

use crate::state::AppState;

/// Config validation result
#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub config: Option<ConfigSummary>,
}

/// Validation error details
#[derive(Debug, Serialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub line: Option<usize>,
}

/// Config summary for display
#[derive(Debug, Serialize)]
pub struct ConfigSummary {
    pub name: String,
    pub model: String,
    pub tools_count: usize,
    pub max_iterations: usize,
    pub temperature: f32,
    pub has_system_prompt: bool,
    pub has_memory: bool,
}

/// Saved config metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub created_at: String,
    pub modified_at: String,
    pub description: Option<String>,
}

/// Validate YAML configuration
#[tauri::command]
pub async fn config_validate(yaml_content: String) -> Result<ValidationResult, String> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Try to parse YAML
    let config: Result<AgentConfig, _> = serde_yaml::from_str(&yaml_content);

    match config {
        Ok(cfg) => {
            // Validate required fields
            if cfg.name.is_empty() {
                errors.push(ValidationError {
                    field: "name".to_string(),
                    message: "Agent name is required".to_string(),
                    line: None,
                });
            }

            if cfg.model.is_empty() {
                errors.push(ValidationError {
                    field: "model".to_string(),
                    message: "Model is required".to_string(),
                    line: None,
                });
            }

            // Validate temperature range
            if cfg.temperature < 0.0 || cfg.temperature > 2.0 {
                errors.push(ValidationError {
                    field: "temperature".to_string(),
                    message: "Temperature must be between 0.0 and 2.0".to_string(),
                    line: None,
                });
            }

            // Warnings for best practices
            if cfg.system_prompt.is_none() {
                warnings.push("No system prompt defined - agent will use default behavior. Consider adding a system prompt to guide agent behavior.".to_string());
            }

            if cfg.max_iterations > 50 {
                warnings.push(format!(
                    "High max_iterations ({}) may lead to long execution times and increased costs",
                    cfg.max_iterations
                ));
            }

            if cfg.max_iterations < 5 {
                warnings.push(format!(
                    "Low max_iterations ({}) might prevent complex tasks from completing",
                    cfg.max_iterations
                ));
            }

            if cfg.tools.is_empty() {
                warnings.push("No tools configured - agent will only use LLM responses without external tool capabilities.".to_string());
            }

            // Check temperature bounds
            if cfg.temperature > 1.0 {
                warnings.push(format!(
                    "High temperature ({}) may produce more creative but less focused responses",
                    cfg.temperature
                ));
            } else if cfg.temperature < 0.3 {
                warnings.push(format!(
                    "Low temperature ({}) may produce very deterministic but less creative responses",
                    cfg.temperature
                ));
            }

            let summary = ConfigSummary {
                name: cfg.name.clone(),
                model: cfg.model.clone(),
                tools_count: cfg.tools.len(),
                max_iterations: cfg.max_iterations,
                temperature: cfg.temperature,
                has_system_prompt: cfg.system_prompt.is_some(),
                has_memory: cfg.memory.is_some(),
            };

            Ok(ValidationResult {
                valid: errors.is_empty(),
                errors,
                warnings,
                config: Some(summary),
            })
        }
        Err(e) => {
            // Parse YAML error for line number
            let error_msg = e.to_string();
            let line = extract_line_from_yaml_error(&error_msg);

            errors.push(ValidationError {
                field: "yaml".to_string(),
                message: format!("YAML parse error: {}", error_msg),
                line,
            });

            Ok(ValidationResult {
                valid: false,
                errors,
                warnings,
                config: None,
            })
        }
    }
}

/// Extract line number from YAML error message
fn extract_line_from_yaml_error(error: &str) -> Option<usize> {
    // serde_yaml errors often contain "at line X"
    if let Some(idx) = error.find("at line ") {
        let start = idx + 8;
        let end = error[start..]
            .find(|c: char| !c.is_ascii_digit())
            .map(|i| start + i)
            .unwrap_or(error.len());
        error[start..end].parse().ok()
    } else {
        None
    }
}

/// Save configuration
#[tauri::command]
pub async fn config_save(
    name: String,
    yaml_content: String,
    state: State<'_, AppState>,
) -> Result<ConfigMetadata, String> {
    // Validate first
    let validation = config_validate(yaml_content.clone()).await?;
    if !validation.valid {
        return Err("Configuration is invalid".to_string());
    }

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let metadata = ConfigMetadata {
        id: id.clone(),
        name: name.clone(),
        path: None,
        created_at: now.clone(),
        modified_at: now,
        description: validation.config.map(|c| format!("Model: {}", c.model)),
    };

    // Store in state
    {
        let mut configs = state.configs.write().await;
        configs.insert(id.clone(), (metadata.clone(), yaml_content));
    }

    Ok(metadata)
}

/// Load configuration by ID
#[tauri::command]
pub async fn config_load(
    id: String,
    state: State<'_, AppState>,
) -> Result<(ConfigMetadata, String), String> {
    let configs = state.configs.read().await;

    configs
        .get(&id)
        .cloned()
        .ok_or_else(|| format!("Configuration {} not found", id))
}

/// List all saved configurations
#[tauri::command]
pub async fn config_list(
    state: State<'_, AppState>,
) -> Result<Vec<ConfigMetadata>, String> {
    let configs = state.configs.read().await;

    let list: Vec<ConfigMetadata> = configs
        .values()
        .map(|(meta, _)| meta.clone())
        .collect();

    Ok(list)
}

/// Delete configuration
#[tauri::command]
pub async fn config_delete(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut configs = state.configs.write().await;

    if configs.remove(&id).is_some() {
        Ok(())
    } else {
        Err(format!("Configuration {} not found", id))
    }
}

/// Generate example configuration
#[tauri::command]
pub async fn config_generate_example() -> Result<String, String> {
    let example = r#"# AOF Agent Configuration
name: example-agent
model: gemini-2.0-flash

# System prompt defines agent behavior
system_prompt: |
  You are a helpful assistant that can answer questions
  and help with various tasks.

# Available tools for the agent
tools:
  - read_file
  - write_file
  - execute_command

# Execution settings
max_iterations: 10
temperature: 0.7
max_tokens: 4096

# Optional memory backend
# memory: vector
"#;

    Ok(example.to_string())
}
