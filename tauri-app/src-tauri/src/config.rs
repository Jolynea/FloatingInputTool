use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager, Runtime};

const CONFIG_FILE_NAME: &str = "config.json";
const DEFAULT_TARGET_FILE_NAME: &str = "Fleeting Note.md";
pub const DEFAULT_HOTKEY: &str = "Ctrl+Alt+Space";
pub const DEFAULT_SIDE_HIDE_ENABLED: u8 = 1;
pub const DEFAULT_EDGE_SNAP_THRESHOLD_PX: u32 = 36;
pub const DEFAULT_VISIBLE_HANDLE_WIDTH_PX: u32 = 22;
pub const DEFAULT_HOVER_OPEN_DELAY_MS: u64 = 180;
pub const DEFAULT_HOVER_CLOSE_DELAY_MS: u64 = 320;
pub const DEFAULT_HOTZONE_WIDTH_PX: u32 = 36;
pub const DEFAULT_DEBUG_SHOW_HOTZONE: u8 = 0;
pub const DEFAULT_SAVE_SHORTCUT_MODE: SaveShortcutMode = SaveShortcutMode::CtrlEnterSave;
pub const DEFAULT_EMPTY_INPUT_PLACEHOLDER_COLOR: &str = "rgba(51, 51, 51, 0.42)";
pub const DEFAULT_SAVE_SHORTCUT_TEXT_COLOR: &str = "currentColor";
pub const DEFAULT_SAVE_SHORTCUT_FONT_SIZE_PX: u32 = 9;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeMode {
  #[default]
  FollowSystem,
  ThemeWhite,
  ThemeDark,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SaveShortcutMode {
  #[default]
  CtrlEnterSave,
  EnterSave,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
  #[serde(default)]
  pub target_file_path: String,
  #[serde(default = "default_hotkey")]
  pub hotkey: String,
  #[serde(default = "default_side_hide_enabled")]
  pub side_hide_enabled: u8,
  #[serde(default = "default_edge_snap_threshold_px")]
  pub edge_snap_threshold_px: u32,
  #[serde(default = "default_visible_handle_width_px")]
  pub visible_handle_width_px: u32,
  #[serde(default = "default_hover_open_delay_ms")]
  pub hover_open_delay_ms: u64,
  #[serde(default = "default_hover_close_delay_ms")]
  pub hover_close_delay_ms: u64,
  #[serde(default = "default_hotzone_width_px")]
  pub hotzone_width_px: u32,
  #[serde(default = "default_debug_show_hotzone")]
  pub debug_show_hotzone: u8,
  #[serde(default = "default_save_shortcut_mode")]
  pub save_shortcut_mode: SaveShortcutMode,
  #[serde(default = "default_empty_input_placeholder_color")]
  pub empty_input_placeholder_color: String,
  #[serde(default = "default_save_shortcut_text_color")]
  pub save_shortcut_text_color: String,
  #[serde(default = "default_save_shortcut_font_size_px")]
  pub save_shortcut_font_size_px: u32,
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
  if config.edge_snap_threshold_px == 0 {
    config.edge_snap_threshold_px = DEFAULT_EDGE_SNAP_THRESHOLD_PX;
  }
  if config.visible_handle_width_px == 0 {
    config.visible_handle_width_px = DEFAULT_VISIBLE_HANDLE_WIDTH_PX;
  }
  if config.hover_open_delay_ms == 0 {
    config.hover_open_delay_ms = DEFAULT_HOVER_OPEN_DELAY_MS;
  }
  if config.hover_close_delay_ms == 0 {
    config.hover_close_delay_ms = DEFAULT_HOVER_CLOSE_DELAY_MS;
  }
  if config.hotzone_width_px == 0 {
    config.hotzone_width_px = DEFAULT_HOTZONE_WIDTH_PX;
  }
  if config.empty_input_placeholder_color.trim().is_empty() {
    config.empty_input_placeholder_color = DEFAULT_EMPTY_INPUT_PLACEHOLDER_COLOR.into();
  }
  if config.save_shortcut_text_color.trim().is_empty() {
    config.save_shortcut_text_color = DEFAULT_SAVE_SHORTCUT_TEXT_COLOR.into();
  }
  if config.save_shortcut_font_size_px == 0 {
    config.save_shortcut_font_size_px = DEFAULT_SAVE_SHORTCUT_FONT_SIZE_PX;
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
    side_hide_enabled: DEFAULT_SIDE_HIDE_ENABLED,
    edge_snap_threshold_px: DEFAULT_EDGE_SNAP_THRESHOLD_PX,
    visible_handle_width_px: DEFAULT_VISIBLE_HANDLE_WIDTH_PX,
    hover_open_delay_ms: DEFAULT_HOVER_OPEN_DELAY_MS,
    hover_close_delay_ms: DEFAULT_HOVER_CLOSE_DELAY_MS,
    hotzone_width_px: DEFAULT_HOTZONE_WIDTH_PX,
    debug_show_hotzone: DEFAULT_DEBUG_SHOW_HOTZONE,
    save_shortcut_mode: DEFAULT_SAVE_SHORTCUT_MODE,
    empty_input_placeholder_color: DEFAULT_EMPTY_INPUT_PLACEHOLDER_COLOR.into(),
    save_shortcut_text_color: DEFAULT_SAVE_SHORTCUT_TEXT_COLOR.into(),
    save_shortcut_font_size_px: DEFAULT_SAVE_SHORTCUT_FONT_SIZE_PX,
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

fn default_side_hide_enabled() -> u8 {
  DEFAULT_SIDE_HIDE_ENABLED
}

fn default_edge_snap_threshold_px() -> u32 {
  DEFAULT_EDGE_SNAP_THRESHOLD_PX
}

fn default_visible_handle_width_px() -> u32 {
  DEFAULT_VISIBLE_HANDLE_WIDTH_PX
}

fn default_hover_open_delay_ms() -> u64 {
  DEFAULT_HOVER_OPEN_DELAY_MS
}

fn default_hover_close_delay_ms() -> u64 {
  DEFAULT_HOVER_CLOSE_DELAY_MS
}

fn default_hotzone_width_px() -> u32 {
  DEFAULT_HOTZONE_WIDTH_PX
}

fn default_debug_show_hotzone() -> u8 {
  DEFAULT_DEBUG_SHOW_HOTZONE
}

fn default_save_shortcut_mode() -> SaveShortcutMode {
  DEFAULT_SAVE_SHORTCUT_MODE
}

fn default_empty_input_placeholder_color() -> String {
  DEFAULT_EMPTY_INPUT_PLACEHOLDER_COLOR.into()
}

fn default_save_shortcut_text_color() -> String {
  DEFAULT_SAVE_SHORTCUT_TEXT_COLOR.into()
}

fn default_save_shortcut_font_size_px() -> u32 {
  DEFAULT_SAVE_SHORTCUT_FONT_SIZE_PX
}
