use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager, Runtime};

const CONFIG_FILE_NAME: &str = "config.json";
const DEFAULT_TARGET_FILE_NAME: &str = "闂康娓呭崟.md";
pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+Space";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeMode {
  #[default]
  FollowSystem,
  ThemeWhite,
  ThemeDark,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
  #[serde(default)]
  pub target_file_path: String,
  #[serde(default = "default_hotkey")]
  pub hotkey: String,
  pub theme_mode: ThemeMode,
}

pub fn load_app_config<R: Runtime>(app: &AppHandle<R>) -> Result<AppConfig, String> {
  let config_path = config_file_path(app)?;
  if !config_path.exists() {
    return Ok(default_app_config(app));
  }

  let raw = fs::read_to_string(&config_path)
    .map_err(|error| format!("failed to read config from {}: {error}", config_path.display()))?;

  let mut config: AppConfig = serde_json::from_str(&raw)
    .map_err(|error| format!("failed to parse config from {}: {error}", config_path.display()))?;

  if config.target_file_path.trim().is_empty() {
    config.target_file_path = default_target_file_path(app)?;
  }
  if config.hotkey.trim().is_empty() {
    config.hotkey = DEFAULT_HOTKEY.into();
  }

  Ok(config)
}

pub fn save_app_config<R: Runtime>(app: &AppHandle<R>, config: &AppConfig) -> Result<(), String> {
  let config_path = config_file_path(app)?;
  let parent = config_path
    .parent()
    .ok_or_else(|| format!("failed to resolve config directory for {}", config_path.display()))?;

  fs::create_dir_all(parent)
    .map_err(|error| format!("failed to create config directory {}: {error}", parent.display()))?;

  let raw = serde_json::to_string_pretty(config)
    .map_err(|error| format!("failed to serialize config: {error}"))?;

  fs::write(&config_path, raw)
    .map_err(|error| format!("failed to write config to {}: {error}", config_path.display()))
}

fn config_file_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
  app
    .path()
    .app_config_dir()
    .map(|path| path.join(CONFIG_FILE_NAME))
    .map_err(|error| format!("failed to resolve app config directory: {error}"))
}

pub fn default_app_config<R: Runtime>(app: &AppHandle<R>) -> AppConfig {
  AppConfig {
    target_file_path: default_target_file_path(app).unwrap_or_else(|_| DEFAULT_TARGET_FILE_NAME.into()),
    hotkey: DEFAULT_HOTKEY.into(),
    theme_mode: ThemeMode::default(),
  }
}

fn default_target_file_path<R: Runtime>(app: &AppHandle<R>) -> Result<String, String> {
  app
    .path()
    .document_dir()
    .map(|path| path.join(DEFAULT_TARGET_FILE_NAME).to_string_lossy().into_owned())
    .map_err(|error| format!("failed to resolve default target file path: {error}"))
}

fn default_hotkey() -> String {
  DEFAULT_HOTKEY.into()
}
