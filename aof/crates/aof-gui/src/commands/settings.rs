// Settings Command Handlers for Tauri

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub default_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub default_provider: String,
    pub default_temperature: f32,
    pub default_max_tokens: u32,
    pub auto_save: bool,
    pub log_level: String,
    pub providers: Vec<ProviderConfig>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            default_provider: "anthropic".to_string(),
            default_temperature: 0.7,
            default_max_tokens: 4096,
            auto_save: true,
            log_level: "info".to_string(),
            providers: vec![
                ProviderConfig {
                    provider: "anthropic".to_string(),
                    api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
                    base_url: None,
                    default_model: "claude-3-5-sonnet-20241022".to_string(),
                },
                ProviderConfig {
                    provider: "openai".to_string(),
                    api_key: std::env::var("OPENAI_API_KEY").ok(),
                    base_url: None,
                    default_model: "gpt-4o".to_string(),
                },
                ProviderConfig {
                    provider: "ollama".to_string(),
                    api_key: None,
                    base_url: Some("http://localhost:11434".to_string()),
                    default_model: "llama2".to_string(),
                },
                ProviderConfig {
                    provider: "groq".to_string(),
                    api_key: std::env::var("GROQ_API_KEY").ok(),
                    base_url: None,
                    default_model: "llama-3.1-70b-versatile".to_string(),
                },
            ],
        }
    }
}

/// Get current settings
#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.read().await;

    // Convert internal AppSettings to command AppSettings
    let app_settings = AppSettings {
        theme: settings.theme.clone(),
        default_provider: "anthropic".to_string(), // TODO: Add to state
        default_temperature: settings.default_temperature,
        default_max_tokens: 4096, // TODO: Add to state
        auto_save: settings.auto_save,
        log_level: settings.log_level.clone(),
        providers: AppSettings::default().providers, // Load from config
    };

    Ok(app_settings)
}

/// Update settings
#[tauri::command]
pub async fn settings_update(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut app_settings = state.settings.write().await;

    app_settings.theme = settings.theme;
    app_settings.default_temperature = settings.default_temperature;
    app_settings.auto_save = settings.auto_save;
    app_settings.log_level = settings.log_level;

    tracing::info!("Settings updated successfully");
    Ok(())
}

/// Test provider connection
#[tauri::command]
pub async fn provider_test_connection(
    provider: String,
    api_key: String,
    base_url: Option<String>,
) -> Result<String, String> {
    tracing::info!("Testing connection for provider: {}", provider);

    match provider.as_str() {
        "anthropic" => {
            // TODO: Actually test the connection once provider is refactored
            if api_key.is_empty() {
                return Err("API key is required for Anthropic".to_string());
            }
            Ok("Successfully configured Anthropic API".to_string())
        }
        "openai" => {
            // TODO: Actually test the connection
            if api_key.is_empty() {
                return Err("API key is required for OpenAI".to_string());
            }
            Ok("Successfully configured OpenAI API".to_string())
        }
        "ollama" => {
            // Ollama runs locally
            let base = base_url.unwrap_or_else(|| "http://localhost:11434".to_string());
            Ok(format!("Ollama configured at {}", base))
        }
        "groq" => {
            // Groq uses OpenAI-compatible API
            if api_key.is_empty() {
                return Err("API key is required for Groq".to_string());
            }
            Ok("Successfully configured Groq API".to_string())
        }
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

/// Get available models for a provider
#[tauri::command]
pub async fn provider_list_models(provider: String) -> Result<Vec<String>, String> {
    match provider.as_str() {
        "anthropic" => Ok(vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ]),
        "openai" => Ok(vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
        ]),
        "ollama" => Ok(vec![
            "llama2".to_string(),
            "llama3".to_string(),
            "mistral".to_string(),
            "codellama".to_string(),
            "phi".to_string(),
        ]),
        "groq" => Ok(vec![
            "llama-3.1-70b-versatile".to_string(),
            "llama-3.1-8b-instant".to_string(),
            "mixtral-8x7b-32768".to_string(),
            "gemma-7b-it".to_string(),
        ]),
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}

/// Reset settings to defaults
#[tauri::command]
pub async fn settings_reset(state: State<'_, AppState>) -> Result<AppSettings, String> {
    {
        let mut settings = state.settings.write().await;
        let default_settings = crate::state::AppSettings::default();
        *settings = default_settings;
        tracing::info!("Settings reset to defaults");
    }

    settings_get(state).await
}

/// Export settings to JSON
#[tauri::command]
pub async fn settings_export(state: State<'_, AppState>) -> Result<String, String> {
    let settings = settings_get(state).await?;
    serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))
}

/// Import settings from JSON
#[tauri::command]
pub async fn settings_import(
    json_content: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let settings: AppSettings = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse settings JSON: {}", e))?;

    settings_update(settings, state).await
}
