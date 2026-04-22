mod config;

use config::{default_app_config, load_app_config, save_app_config, AppConfig, ThemeMode, DEFAULT_HOTKEY};
use std::sync::Mutex;
use std::{fs, path::Path};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
  AppHandle, Emitter, Manager, Runtime, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
  WindowEvent, Wry,
};
#[cfg(desktop)]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

const MENU_ID_SHOW: &str = "show";
const MENU_ID_SETTINGS: &str = "settings";
const MENU_ID_THEME_FOLLOW_SYSTEM: &str = "theme-follow-system";
const MENU_ID_THEME_WHITE: &str = "theme-white";
const MENU_ID_THEME_DARK: &str = "theme-dark";
const MENU_ID_QUIT: &str = "quit";
const WINDOW_LABEL_MAIN: &str = "main";
const WINDOW_LABEL_SETTINGS: &str = "settings";
const EVENT_THEME_MODE_CHANGED: &str = "theme-mode-changed";

struct ThemeMenuItems {
  follow_system: CheckMenuItem<Wry>,
  theme_white: CheckMenuItem<Wry>,
  theme_dark: CheckMenuItem<Wry>,
}

struct AppState {
  config: Mutex<AppConfig>,
  theme_menu_items: ThemeMenuItems,
  active_hotkey: Mutex<Option<String>>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ThemeModeChangedPayload {
  theme_mode: ThemeMode,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct HotkeyUpdateResponse {
  config: AppConfig,
  warning: Option<String>,
}

#[tauri::command]
fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
  state
    .config
    .lock()
    .map(|config| config.clone())
    .map_err(|error| format!("failed to read app state: {error}"))
}

#[tauri::command]
fn set_theme_mode(
  app: AppHandle,
  state: State<'_, AppState>,
  theme_mode: ThemeMode,
) -> Result<AppConfig, String> {
  let mut config = state
    .config
    .lock()
    .map_err(|error| format!("failed to lock app state: {error}"))?;

  config.theme_mode = theme_mode;
  save_app_config(&app, &config)?;
  sync_theme_menu_items(&state.theme_menu_items, theme_mode)?;

  let next_config = config.clone();
  drop(config);

  app
    .emit(
      EVENT_THEME_MODE_CHANGED,
      ThemeModeChangedPayload { theme_mode },
    )
    .map_err(|error| format!("failed to emit theme mode change: {error}"))?;

  Ok(next_config)
}

#[tauri::command]
fn set_target_file_path(
  app: AppHandle,
  state: State<'_, AppState>,
  target_file_path: String,
) -> Result<AppConfig, String> {
  let normalized_target_file_path = normalize_target_file_path(&target_file_path)?;
  let mut config = state
    .config
    .lock()
    .map_err(|error| format!("failed to lock app state: {error}"))?;

  config.target_file_path = normalized_target_file_path;
  save_app_config(&app, &config)?;

  Ok(config.clone())
}

#[tauri::command]
fn set_hotkey(
  app: AppHandle,
  state: State<'_, AppState>,
  hotkey: String,
) -> Result<HotkeyUpdateResponse, String> {
  let normalized_hotkey = hotkey.trim().to_string();
  let mut config = state
    .config
    .lock()
    .map_err(|error| format!("failed to lock app state: {error}"))?;

  config.hotkey = normalized_hotkey.clone();
  save_app_config(&app, &config)?;
  let next_config = config.clone();
  drop(config);

  let warning = match activate_hotkey(&app, &state, &normalized_hotkey) {
    Ok(()) => None,
    Err(error) => Some(error),
  };

  Ok(HotkeyUpdateResponse {
    config: next_config,
    warning,
  })
}

#[tauri::command]
fn save_note(state: State<'_, AppState>, note_text: String) -> Result<(), String> {
  if note_text.trim().is_empty() {
    return Err(String::from("note content cannot be empty"));
  }

  let config = state
    .config
    .lock()
    .map_err(|error| format!("failed to lock app state: {error}"))?
    .clone();

  let note_block = format_note_block(&note_text);
  prepend_to_file(&config.target_file_path, &note_block)
}

fn sync_theme_menu_items(items: &ThemeMenuItems, active_mode: ThemeMode) -> Result<(), String> {
  items
    .follow_system
    .set_checked(active_mode == ThemeMode::FollowSystem)
    .map_err(|error| format!("failed to update tray theme menu: {error}"))?;
  items
    .theme_white
    .set_checked(active_mode == ThemeMode::ThemeWhite)
    .map_err(|error| format!("failed to update tray theme menu: {error}"))?;
  items
    .theme_dark
    .set_checked(active_mode == ThemeMode::ThemeDark)
    .map_err(|error| format!("failed to update tray theme menu: {error}"))?;

  Ok(())
}

fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
  if let Some(window) = app.get_webview_window(WINDOW_LABEL_MAIN) {
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
  }
}

fn open_settings<R: Runtime>(app: &AppHandle<R>) {
  if let Err(error) = open_or_focus_settings_window(app) {
    log::error!("{error}");
  }
}

fn toggle_main_window<R: Runtime>(app: &AppHandle<R>) {
  if let Some(settings_window) = app.get_webview_window(WINDOW_LABEL_SETTINGS) {
    let _ = settings_window.show();
    let _ = settings_window.unminimize();
    let _ = settings_window.set_focus();
    return;
  }

  if let Some(window) = app.get_webview_window(WINDOW_LABEL_MAIN) {
    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
      let _ = window.hide();
    } else {
      let _ = window.show();
      let _ = window.unminimize();
      let _ = window.set_focus();
    }
  }
}

fn enable_main_window<R: Runtime>(app: &AppHandle<R>) {
  if let Some(main_window) = app.get_webview_window(WINDOW_LABEL_MAIN) {
    let _ = main_window.set_enabled(true);
    let _ = main_window.set_focus();
  }
}

fn attach_settings_window_events<R: Runtime>(app: &AppHandle<R>, settings_window: &WebviewWindow<R>) {
  let app_handle = app.clone();
  settings_window.on_window_event(move |event| {
    if matches!(event, WindowEvent::Destroyed) {
      enable_main_window(&app_handle);
    }
  });
}

fn build_settings_window<R: Runtime>(
  app: &AppHandle<R>,
  _main_window: &WebviewWindow<R>,
) -> Result<WebviewWindow<R>, String> {
  WebviewWindowBuilder::new(
    app,
    WINDOW_LABEL_SETTINGS,
    WebviewUrl::App("index.html".into()),
  )
  .title("FloatingInputTool Settings")
  .inner_size(560.0, 620.0)
  .min_inner_size(500.0, 560.0)
  .resizable(true)
  .fullscreen(false)
  .decorations(false)
  .transparent(true)
  .shadow(false)
  .always_on_top(true)
  .focused(true)
  .center()
  .build()
  .map_err(|error| format!("failed to create Settings window: {error}"))
}

fn open_or_focus_settings_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
  show_main_window(app);

  let main_window = app
    .get_webview_window(WINDOW_LABEL_MAIN)
    .ok_or_else(|| String::from("main window is missing"))?;

  if let Some(settings_window) = app.get_webview_window(WINDOW_LABEL_SETTINGS) {
    let _ = settings_window.show();
    let _ = settings_window.unminimize();
    let _ = settings_window.set_focus();
    main_window
      .set_enabled(false)
      .map_err(|error| format!("failed to disable main window while focusing Settings: {error}"))?;
    return Ok(());
  }

  match build_settings_window(app, &main_window) {
    Ok(settings_window) => {
      attach_settings_window_events(app, &settings_window);
      let _ = settings_window.set_focus();
      main_window
        .set_enabled(false)
        .map_err(|error| format!("failed to disable main window after opening Settings: {error}"))?;
      Ok(())
    }
    Err(error) => {
      let _ = main_window.set_enabled(true);
      Err(error)
    }
  }
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
  match event.id().as_ref() {
    MENU_ID_SHOW => show_main_window(app),
    MENU_ID_SETTINGS => open_settings(app),
    MENU_ID_THEME_FOLLOW_SYSTEM => {
      if let Err(error) = set_theme_mode(app.clone(), app.state::<AppState>(), ThemeMode::FollowSystem) {
        log::error!("{error}");
      }
    }
    MENU_ID_THEME_WHITE => {
      if let Err(error) = set_theme_mode(app.clone(), app.state::<AppState>(), ThemeMode::ThemeWhite) {
        log::error!("{error}");
      }
    }
    MENU_ID_THEME_DARK => {
      if let Err(error) = set_theme_mode(app.clone(), app.state::<AppState>(), ThemeMode::ThemeDark) {
        log::error!("{error}");
      }
    }
    MENU_ID_QUIT => app.exit(0),
    _ => {}
  }
}

fn create_tray(app: &AppHandle, theme_mode: ThemeMode) -> Result<ThemeMenuItems, String> {
  let show_item = MenuItem::with_id(app, MENU_ID_SHOW, "Show", true, None::<&str>)
    .map_err(|error| format!("failed to create Show menu item: {error}"))?;
  let settings_item = MenuItem::with_id(app, MENU_ID_SETTINGS, "Settings", true, None::<&str>)
    .map_err(|error| format!("failed to create Settings menu item: {error}"))?;
  let follow_system_item = CheckMenuItem::with_id(
    app,
    MENU_ID_THEME_FOLLOW_SYSTEM,
    "Follow System",
    true,
    theme_mode == ThemeMode::FollowSystem,
    None::<&str>,
  )
  .map_err(|error| format!("failed to create Follow System menu item: {error}"))?;
  let theme_white_item = CheckMenuItem::with_id(
    app,
    MENU_ID_THEME_WHITE,
    "Theme White",
    true,
    theme_mode == ThemeMode::ThemeWhite,
    None::<&str>,
  )
  .map_err(|error| format!("failed to create Theme White menu item: {error}"))?;
  let theme_dark_item = CheckMenuItem::with_id(
    app,
    MENU_ID_THEME_DARK,
    "Theme Dark",
    true,
    theme_mode == ThemeMode::ThemeDark,
    None::<&str>,
  )
  .map_err(|error| format!("failed to create Theme Dark menu item: {error}"))?;
  let separator = PredefinedMenuItem::separator(app)
    .map_err(|error| format!("failed to create separator menu item: {error}"))?;
  let quit_item = MenuItem::with_id(app, MENU_ID_QUIT, "Quit", true, None::<&str>)
    .map_err(|error| format!("failed to create Quit menu item: {error}"))?;

  let theme_submenu = Submenu::with_items(
    app,
    "Theme",
    true,
    &[&follow_system_item, &theme_white_item, &theme_dark_item],
  )
  .map_err(|error| format!("failed to create tray theme submenu: {error}"))?;

  let tray_menu = Menu::with_items(
    app,
    &[&show_item, &settings_item, &theme_submenu, &separator, &quit_item],
  )
  .map_err(|error| format!("failed to create tray menu: {error}"))?;

  let icon = app
    .default_window_icon()
    .cloned()
    .ok_or_else(|| String::from("default window icon is missing"))?;

  TrayIconBuilder::with_id("main-tray")
    .icon(icon)
    .tooltip("Floating Input Tool")
    .menu(&tray_menu)
    .show_menu_on_left_click(false)
    .on_tray_icon_event(|tray, event| {
      if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
      } = event
      {
        show_main_window(tray.app_handle());
      }
    })
    .build(app)
    .map_err(|error| format!("failed to create tray icon: {error}"))?;

  Ok(ThemeMenuItems {
    follow_system: follow_system_item,
    theme_white: theme_white_item,
    theme_dark: theme_dark_item,
  })
}

fn normalize_target_file_path(target_file_path: &str) -> Result<String, String> {
  let normalized = target_file_path.trim();
  if normalized.is_empty() {
    return Err(String::from("target file path is required"));
  }

  if normalized.to_ascii_lowercase().ends_with(".md") {
    return Ok(normalized.to_string());
  }

  Ok(format!("{normalized}.md"))
}

fn activate_hotkey<R: Runtime>(
  app: &AppHandle<R>,
  state: &State<'_, AppState>,
  requested_hotkey: &str,
) -> Result<(), String> {
  let normalized_hotkey = requested_hotkey.trim();
  if normalized_hotkey.is_empty() {
    return Err(String::from(
      "Hotkey was saved, but it is empty. The previous working hotkey will stay active.",
    ));
  }

  let mut active_hotkey = state
    .active_hotkey
    .lock()
    .map_err(|error| format!("failed to lock active hotkey state: {error}"))?;

  if active_hotkey.as_deref() == Some(normalized_hotkey) {
    return Ok(());
  }

  let previous_hotkey = active_hotkey.clone();
  if let Some(previous_hotkey_value) = previous_hotkey.as_deref() {
    let _ = app.global_shortcut().unregister(previous_hotkey_value);
  }

  match app.global_shortcut().register(normalized_hotkey) {
    Ok(()) => {
      *active_hotkey = Some(normalized_hotkey.to_string());
      Ok(())
    }
    Err(error) => {
      if let Some(previous_hotkey_value) = previous_hotkey.as_deref() {
        let _ = app.global_shortcut().register(previous_hotkey_value);
      }
      *active_hotkey = previous_hotkey;
      Err(format!(
        "Hotkey was saved, but it could not be activated. It may be invalid or already in use. {error}"
      ))
    }
  }
}

fn register_startup_hotkey<R: Runtime>(app: &AppHandle<R>, configured_hotkey: &str) -> Option<String> {
  let normalized_hotkey = configured_hotkey.trim();
  if !normalized_hotkey.is_empty() {
    match app.global_shortcut().register(normalized_hotkey) {
      Ok(()) => return Some(normalized_hotkey.to_string()),
      Err(error) => {
        log::warn!(
          "failed to register configured hotkey {normalized_hotkey}: {error}; falling back to {DEFAULT_HOTKEY}"
        );
      }
    }
  }

  match app.global_shortcut().register(DEFAULT_HOTKEY) {
    Ok(()) => Some(DEFAULT_HOTKEY.to_string()),
    Err(error) => {
      log::error!("failed to register fallback hotkey {DEFAULT_HOTKEY}: {error}");
      None
    }
  }
}

fn format_note_block(note_text: &str) -> String {
  let normalized_text = note_text.replace("\r\n", "\n").replace('\r', "\n");
  let trimmed_text = normalized_text.trim_end_matches('\n');
  let timestamp = current_timestamp_string();

  let mut note_block = format!("> [!fleeting] {timestamp}\r\n");
  for line in trimmed_text.split('\n') {
    if line.is_empty() {
      note_block.push_str(">\r\n");
    } else {
      note_block.push_str("> ");
      note_block.push_str(line);
      note_block.push_str("\r\n");
    }
  }
  note_block.push_str("\r\n");

  note_block
}

fn prepend_to_file(target_file_path: &str, note_block: &str) -> Result<(), String> {
  ensure_destination_exists(target_file_path)?;

  let existing_content = if Path::new(target_file_path).exists() {
    fs::read_to_string(target_file_path)
      .map_err(|error| format!("failed to read target file {target_file_path}: {error}"))?
  } else {
    String::new()
  };

  let mut output = String::from(note_block);
  if !existing_content.is_empty() {
    output.push_str(&existing_content);
  }

  fs::write(target_file_path, output)
    .map_err(|error| format!("failed to write target file {target_file_path}: {error}"))
}

fn ensure_destination_exists(target_file_path: &str) -> Result<(), String> {
  let target_path = Path::new(target_file_path);
  if let Some(parent) = target_path.parent() {
    if !parent.as_os_str().is_empty() {
      fs::create_dir_all(parent)
        .map_err(|error| format!("failed to create directory {}: {error}", parent.display()))?;
    }
  }

  if !target_path.exists() {
    fs::write(target_path, "")
      .map_err(|error| format!("failed to create target file {}: {error}", target_path.display()))?;
  }

  Ok(())
}

fn current_timestamp_string() -> String {
  #[allow(deprecated)]
  let now = std::time::SystemTime::now();
  let datetime: chrono::DateTime<chrono::Local> = now.into();
  datetime.format("%Y-%m-%d %H:%M").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(
      tauri_plugin_global_shortcut::Builder::new()
        .with_handler(|app, _shortcut, event| {
          if event.state() == ShortcutState::Pressed {
            toggle_main_window(app);
          }
        })
        .build(),
    )
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      let config = match load_app_config(app.handle()) {
        Ok(config) => config,
        Err(error) => {
          log::warn!("{error}");
          default_app_config(app.handle())
        }
      };

      let active_hotkey = register_startup_hotkey(app.handle(), &config.hotkey);
      let theme_menu_items = create_tray(app.handle(), config.theme_mode)?;

      app.manage(AppState {
        config: Mutex::new(config),
        theme_menu_items,
        active_hotkey: Mutex::new(active_hotkey),
      });

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_app_config,
      set_theme_mode,
      set_target_file_path,
      set_hotkey,
      save_note
    ])
    .on_menu_event(handle_menu_event)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
