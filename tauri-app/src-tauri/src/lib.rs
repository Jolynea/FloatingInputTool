mod config;

use config::{default_app_config, load_app_config, save_app_config, AppConfig, ThemeMode, DEFAULT_HOTKEY};
use std::sync::Mutex;
use std::{fs, path::Path};
use std::{thread, time::Duration};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
  AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Runtime, State, WebviewUrl,
  WebviewWindow, WebviewWindowBuilder, WindowEvent, Wry,
};
#[cfg(desktop)]
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
#[cfg(windows)]
use windows_sys::Win32::Foundation::POINT;
#[cfg(windows)]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

const MENU_ID_SHOW: &str = "show";
const MENU_ID_SETTINGS: &str = "settings";
const MENU_ID_THEME_FOLLOW_SYSTEM: &str = "theme-follow-system";
const MENU_ID_THEME_WHITE: &str = "theme-white";
const MENU_ID_THEME_DARK: &str = "theme-dark";
const MENU_ID_QUIT: &str = "quit";
const WINDOW_LABEL_MAIN: &str = "main";
const WINDOW_LABEL_SETTINGS: &str = "settings";
const EVENT_THEME_MODE_CHANGED: &str = "theme-mode-changed";
const EVENT_MAIN_WINDOW_MODE_CHANGED: &str = "main-window-mode-changed";
const AUTO_DOCK_SETTLE_MS: u64 = 180;

struct ThemeMenuItems {
  follow_system: CheckMenuItem<Wry>,
  theme_white: CheckMenuItem<Wry>,
  theme_dark: CheckMenuItem<Wry>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum DockSide {
  Left,
  Right,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "kebab-case")]
enum MainWindowMode {
  #[default]
  Normal,
  DockedLeft,
  DockedRight,
  ExpandedFromDock,
}

#[derive(Debug, Clone, Copy, Default)]
struct StoredWindowBounds {
  x: i32,
  y: i32,
  width: u32,
  height: u32,
}

#[derive(Debug, Clone, Default)]
struct MainWindowRuntimeState {
  mode: MainWindowMode,
  dock_side: Option<DockSide>,
  normal_bounds: Option<StoredWindowBounds>,
  docked_bounds: Option<StoredWindowBounds>,
}

struct AppState {
  config: Mutex<AppConfig>,
  theme_menu_items: ThemeMenuItems,
  active_hotkey: Mutex<Option<String>>,
  main_window_state: Mutex<MainWindowRuntimeState>,
  auto_dock_generation: Mutex<u64>,
  manual_drag_in_progress: Mutex<bool>,
  auto_dock_suppressed_until_ms: Mutex<u128>,
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

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MainWindowModeChangedPayload {
  mode: MainWindowMode,
  dock_side: Option<DockSide>,
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

#[tauri::command]
fn begin_manual_window_drag(state: State<'_, AppState>) -> Result<(), String> {
  let mut is_dragging = state
    .manual_drag_in_progress
    .lock()
    .map_err(|error| format!("failed to lock manual drag state: {error}"))?;
  *is_dragging = true;
  Ok(())
}

#[tauri::command]
fn end_manual_window_drag(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
  {
    let mut is_dragging = state
      .manual_drag_in_progress
      .lock()
      .map_err(|error| format!("failed to lock manual drag state: {error}"))?;
    *is_dragging = false;
  }

  sync_main_window_mode_after_move(&app)
}

#[tauri::command]
fn is_cursor_inside_main_window(app: AppHandle, padding_px: Option<i32>) -> Result<bool, String> {
  let window = app
    .get_webview_window(WINDOW_LABEL_MAIN)
    .ok_or_else(|| String::from("main window is missing"))?;
  let bounds = capture_window_bounds(&window)?;
  let padding = padding_px.unwrap_or(0).max(0);

  let Some((cursor_x, cursor_y)) = current_cursor_position() else {
    return Ok(false);
  };

  let left = bounds.x - padding;
  let top = bounds.y - padding;
  let right = bounds.x + bounds.width as i32 + padding;
  let bottom = bounds.y + bounds.height as i32 + padding;
  let is_inside = cursor_x >= left && cursor_x < right && cursor_y >= top && cursor_y < bottom;
  log::info!(
    "[side-hide] is_cursor_inside_main_window cursor=({}, {}) bounds=({}, {}, {}, {}) padding={} inside={}",
    cursor_x,
    cursor_y,
    left,
    top,
    right,
    bottom,
    padding,
    is_inside
  );
  Ok(is_inside)
}

#[tauri::command]
fn hide_or_dock_main_window(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
  let config = state
    .config
    .lock()
    .map_err(|error| format!("failed to lock app config: {error}"))?
    .clone();

  let window = app
    .get_webview_window(WINDOW_LABEL_MAIN)
    .ok_or_else(|| String::from("main window is missing"))?;

  if config.side_hide_enabled == 0 {
    window
      .hide()
      .map_err(|error| format!("failed to hide main window: {error}"))?;
    update_main_window_runtime_state(
      &app,
      &state,
      MainWindowRuntimeState {
        mode: MainWindowMode::Normal,
        dock_side: None,
        normal_bounds: Some(capture_window_bounds(&window)?),
        docked_bounds: None,
      },
    )?;
    return Ok(());
  }

  let current_bounds = capture_window_bounds(&window)?;
  if let Some((dock_side, docked_bounds)) = compute_docked_bounds(&window, &current_bounds, &config)? {
    apply_window_bounds(&window, docked_bounds)?;
    set_window_resizable(&window, false)?;
    update_main_window_runtime_state(
      &app,
      &state,
      MainWindowRuntimeState {
        mode: dock_side.to_docked_mode(),
        dock_side: Some(dock_side),
        normal_bounds: Some(current_bounds),
        docked_bounds: Some(docked_bounds),
      },
    )?;
  } else {
    window
      .hide()
      .map_err(|error| format!("failed to hide main window: {error}"))?;
    update_main_window_runtime_state(
      &app,
      &state,
      MainWindowRuntimeState {
        mode: MainWindowMode::Normal,
        dock_side: None,
        normal_bounds: Some(current_bounds),
        docked_bounds: None,
      },
    )?;
  }

  Ok(())
}

#[tauri::command]
fn restore_docked_main_window(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
  let window = app
    .get_webview_window(WINDOW_LABEL_MAIN)
    .ok_or_else(|| String::from("main window is missing"))?;

  let runtime_state = state
    .main_window_state
    .lock()
    .map_err(|error| format!("failed to lock main window state: {error}"))?
    .clone();

  let Some(normal_bounds) = runtime_state.normal_bounds else {
    return Ok(());
  };

  if runtime_state.mode == MainWindowMode::Normal {
    return Ok(());
  }

  apply_window_bounds(&window, normal_bounds)?;
  set_window_resizable(&window, true)?;
  {
    let mut suppressed_until = state
      .auto_dock_suppressed_until_ms
      .lock()
      .map_err(|error| format!("failed to lock auto dock suppression state: {error}"))?;
    *suppressed_until = current_time_millis() + (AUTO_DOCK_SETTLE_MS as u128 * 3);
  }
  log::info!(
    "[side-hide] restore_docked_main_window normal_bounds=({}, {}, {}, {}) mode_before={:?}",
    normal_bounds.x,
    normal_bounds.y,
    normal_bounds.width,
    normal_bounds.height,
    runtime_state.mode
  );
  window
    .show()
    .map_err(|error| format!("failed to show restored main window: {error}"))?;
  window
    .set_focus()
    .map_err(|error| format!("failed to focus restored main window: {error}"))?;

  update_main_window_runtime_state(
    &app,
    &state,
    MainWindowRuntimeState {
      mode: MainWindowMode::ExpandedFromDock,
      dock_side: runtime_state.dock_side,
      normal_bounds: Some(normal_bounds),
      docked_bounds: runtime_state.docked_bounds,
    },
  )
}

#[tauri::command]
fn redock_main_window(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
  let window = app
    .get_webview_window(WINDOW_LABEL_MAIN)
    .ok_or_else(|| String::from("main window is missing"))?;
  let config = state
    .config
    .lock()
    .map_err(|error| format!("failed to lock app config: {error}"))?
    .clone();

  let runtime_state = state
    .main_window_state
    .lock()
    .map_err(|error| format!("failed to lock main window state: {error}"))?
    .clone();
  let current_bounds = capture_window_bounds(&window)?;
  log::info!(
    "[side-hide] redock_main_window current_bounds=({}, {}, {}, {}) mode_before={:?}",
    current_bounds.x,
    current_bounds.y,
    current_bounds.width,
    current_bounds.height,
    runtime_state.mode
  );

  if let Some((dock_side, docked_bounds)) = compute_docked_bounds(&window, &current_bounds, &config)? {
    log::info!(
      "[side-hide] redock_main_window docking side={:?} docked_bounds=({}, {}, {}, {})",
      dock_side,
      docked_bounds.x,
      docked_bounds.y,
      docked_bounds.width,
      docked_bounds.height
    );
    apply_window_bounds(&window, docked_bounds)?;
    set_window_resizable(&window, false)?;
    update_main_window_runtime_state(
      &app,
      &state,
      MainWindowRuntimeState {
        mode: dock_side.to_docked_mode(),
        dock_side: Some(dock_side),
        normal_bounds: Some(current_bounds),
        docked_bounds: Some(docked_bounds),
      },
    )
  } else {
    log::info!("[side-hide] redock_main_window canceled because current position is not dockable");
    update_main_window_runtime_state(
      &app,
      &state,
      MainWindowRuntimeState {
        mode: MainWindowMode::Normal,
        dock_side: None,
        normal_bounds: Some(current_bounds),
        docked_bounds: None,
      },
    )?;

    if runtime_state.mode != MainWindowMode::Normal {
      set_window_resizable(&window, true)?;
      window
        .show()
        .map_err(|error| format!("failed to keep main window visible after canceling dock: {error}"))?;
      window
        .set_focus()
        .map_err(|error| format!("failed to focus main window after canceling dock: {error}"))?;
    }

    Ok(())
  }
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
  let state = app.state::<AppState>();
  if let Err(error) = restore_main_window_to_normal(app, &state) {
    log::warn!("{error}");
  }

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
    let state = app.state::<AppState>();
    let mode = state
      .main_window_state
      .lock()
      .map(|runtime_state| runtime_state.mode)
      .unwrap_or(MainWindowMode::Normal);

    if mode != MainWindowMode::Normal {
      show_main_window(app);
      return;
    }

    let is_visible = window.is_visible().unwrap_or(false);
    if is_visible {
      let _ = window.hide();
    } else {
      show_main_window(app);
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

fn attach_main_window_events<R: Runtime>(app: &AppHandle<R>, main_window: &WebviewWindow<R>) {
  let app_handle = app.clone();
  main_window.on_window_event(move |event| {
    if matches!(event, WindowEvent::Moved(_)) {
      schedule_main_window_auto_dock(&app_handle);
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

fn sync_main_window_mode_after_move<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
  let state = app.state::<AppState>();
  let suppressed_until = *state
    .auto_dock_suppressed_until_ms
    .lock()
    .map_err(|error| format!("failed to lock auto dock suppression state: {error}"))?;
  let now_ms = current_time_millis();
  if suppressed_until > now_ms {
    log::info!(
      "[side-hide] sync_main_window_mode_after_move skipped due to suppression now_ms={} suppressed_until={}",
      now_ms,
      suppressed_until
    );
    return Ok(());
  }

  let is_dragging = *state
    .manual_drag_in_progress
    .lock()
    .map_err(|error| format!("failed to lock manual drag state: {error}"))?;
  if is_dragging {
    return Ok(());
  }

  let config = state
    .config
    .lock()
    .map_err(|error| format!("failed to lock app config: {error}"))?
    .clone();

  if config.side_hide_enabled == 0 {
    return Ok(());
  }

  let window = app
    .get_webview_window(WINDOW_LABEL_MAIN)
    .ok_or_else(|| String::from("main window is missing"))?;
  let current_bounds = capture_window_bounds(&window)?;
  let current_mode = state
    .main_window_state
    .lock()
    .map_err(|error| format!("failed to lock main window state: {error}"))?
    .mode;

  match current_mode {
    MainWindowMode::DockedLeft | MainWindowMode::DockedRight => Ok(()),
    MainWindowMode::Normal => {
      if let Some((dock_side, docked_bounds)) = compute_docked_bounds(&window, &current_bounds, &config)? {
        apply_window_bounds(&window, docked_bounds)?;
        set_window_resizable(&window, false)?;
        update_main_window_runtime_state(
          app,
          &state,
          MainWindowRuntimeState {
            mode: dock_side.to_docked_mode(),
            dock_side: Some(dock_side),
            normal_bounds: Some(current_bounds),
            docked_bounds: Some(docked_bounds),
          },
        )?;
      }

      Ok(())
    }
    MainWindowMode::ExpandedFromDock => {
      if let Some((dock_side, docked_bounds)) = compute_docked_bounds(&window, &current_bounds, &config)? {
        apply_window_bounds(&window, docked_bounds)?;
        set_window_resizable(&window, false)?;
        update_main_window_runtime_state(
          app,
          &state,
          MainWindowRuntimeState {
            mode: dock_side.to_docked_mode(),
            dock_side: Some(dock_side),
            normal_bounds: Some(current_bounds),
            docked_bounds: Some(docked_bounds),
          },
        )?;
      } else {
        set_window_resizable(&window, true)?;
        update_main_window_runtime_state(
          app,
          &state,
          MainWindowRuntimeState {
            mode: MainWindowMode::Normal,
            dock_side: None,
            normal_bounds: Some(current_bounds),
            docked_bounds: None,
          },
        )?;
      }

      Ok(())
    }
  }
}

fn schedule_main_window_auto_dock<R: Runtime>(app: &AppHandle<R>) {
  let next_generation = {
    let state = app.state::<AppState>();
    let mut generation = match state.auto_dock_generation.lock() {
      Ok(generation) => generation,
      Err(error) => {
        log::warn!("failed to lock auto dock generation: {error}");
        return;
      }
    };
    *generation += 1;
    *generation
  };

  let app_handle = app.clone();
  thread::spawn(move || {
    thread::sleep(Duration::from_millis(AUTO_DOCK_SETTLE_MS));

    let state = app_handle.state::<AppState>();
    let current_generation = match state.auto_dock_generation.lock() {
      Ok(generation) => *generation,
      Err(error) => {
        log::warn!("failed to lock auto dock generation after delay: {error}");
        return;
      }
    };

    if current_generation != next_generation {
      return;
    }

    let is_manual_drag_active = match state.manual_drag_in_progress.lock() {
      Ok(is_dragging) => *is_dragging,
      Err(error) => {
        log::warn!("failed to lock manual drag state after delay: {error}");
        return;
      }
    };

    if is_manual_drag_active {
      while is_left_mouse_button_down() {
        thread::sleep(Duration::from_millis(16));
      }

      match state.manual_drag_in_progress.lock() {
        Ok(mut is_dragging) => {
          *is_dragging = false;
        }
        Err(error) => {
          log::warn!("failed to clear manual drag state after release: {error}");
          return;
        }
      }
    }

    if let Err(error) = sync_main_window_mode_after_move(&app_handle) {
      log::warn!("{error}");
    }
  });
}

#[cfg(windows)]
fn is_left_mouse_button_down() -> bool {
  unsafe { (GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000) != 0 }
}

#[cfg(not(windows))]
fn is_left_mouse_button_down() -> bool {
  false
}

#[cfg(windows)]
fn current_cursor_position() -> Option<(i32, i32)> {
  let mut point = POINT { x: 0, y: 0 };
  let ok = unsafe { GetCursorPos(&mut point) };
  if ok == 0 {
    None
  } else {
    Some((point.x, point.y))
  }
}

#[cfg(not(windows))]
fn current_cursor_position() -> Option<(i32, i32)> {
  None
}

fn capture_window_bounds<R: Runtime>(window: &WebviewWindow<R>) -> Result<StoredWindowBounds, String> {
  let position = window
    .outer_position()
    .map_err(|error| format!("failed to read window position: {error}"))?;
  let size = window
    .outer_size()
    .map_err(|error| format!("failed to read window size: {error}"))?;

  Ok(StoredWindowBounds {
    x: position.x,
    y: position.y,
    width: size.width,
    height: size.height,
  })
}

fn apply_window_bounds<R: Runtime>(
  window: &WebviewWindow<R>,
  bounds: StoredWindowBounds,
) -> Result<(), String> {
  window
    .set_size(PhysicalSize::new(bounds.width, bounds.height))
    .map_err(|error| format!("failed to set window size: {error}"))?;
  window
    .set_position(PhysicalPosition::new(bounds.x, bounds.y))
    .map_err(|error| format!("failed to set window position: {error}"))
}

fn set_window_resizable<R: Runtime>(window: &WebviewWindow<R>, is_resizable: bool) -> Result<(), String> {
  window
    .set_resizable(is_resizable)
    .map_err(|error| format!("failed to set window resizable state: {error}"))
}

fn compute_docked_bounds<R: Runtime>(
  window: &WebviewWindow<R>,
  current_bounds: &StoredWindowBounds,
  config: &AppConfig,
) -> Result<Option<(DockSide, StoredWindowBounds)>, String> {
  let Some(monitor) = window
    .current_monitor()
    .map_err(|error| format!("failed to resolve current monitor: {error}"))?
  else {
    return Ok(None);
  };

  let work_area = monitor.work_area();
  let work_left = work_area.position.x;
  let work_top = work_area.position.y;
  let work_right = work_left + work_area.size.width as i32;
  let work_bottom = work_top + work_area.size.height as i32;
  let threshold = config.edge_snap_threshold_px as i32;

  let left_gap = current_bounds.x - work_left;
  let right_gap = work_right - (current_bounds.x + current_bounds.width as i32);
  let near_left = left_gap <= threshold;
  let near_right = right_gap <= threshold;

  if !near_left && !near_right {
    return Ok(None);
  }

  let dock_side = match (near_left, near_right) {
    (true, true) => {
      if left_gap.abs() <= right_gap.abs() {
        DockSide::Left
      } else {
        DockSide::Right
      }
    }
    (true, false) => DockSide::Left,
    (false, true) => DockSide::Right,
    (false, false) => return Ok(None),
  };

  let exposed_width = effective_exposed_width(config) as i32;
  let max_y = (work_bottom - current_bounds.height as i32).max(work_top);
  let clamped_y = current_bounds.y.clamp(work_top, max_y);

  let dock_x = match dock_side {
    DockSide::Left => work_left - (current_bounds.width as i32 - exposed_width),
    DockSide::Right => work_right - exposed_width,
  };

  Ok(Some((
    dock_side,
    StoredWindowBounds {
      x: dock_x,
      y: clamped_y,
      width: current_bounds.width,
      height: current_bounds.height,
    },
  )))
}

fn effective_exposed_width(config: &AppConfig) -> u32 {
  config.visible_handle_width_px.max(config.hotzone_width_px).max(1)
}

fn restore_main_window_to_normal<R: Runtime>(
  app: &AppHandle<R>,
  state: &State<'_, AppState>,
) -> Result<(), String> {
  let runtime_state = state
    .main_window_state
    .lock()
    .map_err(|error| format!("failed to lock main window state: {error}"))?
    .clone();

  if runtime_state.mode == MainWindowMode::Normal {
    return Ok(());
  }

  let window = app
    .get_webview_window(WINDOW_LABEL_MAIN)
    .ok_or_else(|| String::from("main window is missing"))?;

  if let Some(normal_bounds) = runtime_state.normal_bounds {
    apply_window_bounds(&window, normal_bounds)?;
    update_main_window_runtime_state(
      app,
      state,
      MainWindowRuntimeState {
        mode: MainWindowMode::Normal,
        dock_side: None,
        normal_bounds: Some(normal_bounds),
        docked_bounds: None,
      },
    )?;
  }

  Ok(())
}

fn update_main_window_runtime_state<R: Runtime>(
  app: &AppHandle<R>,
  state: &State<'_, AppState>,
  next_state: MainWindowRuntimeState,
) -> Result<(), String> {
  {
    let mut runtime_state = state
      .main_window_state
      .lock()
      .map_err(|error| format!("failed to lock main window state: {error}"))?;
    *runtime_state = next_state.clone();
  }

  app
    .emit(
      EVENT_MAIN_WINDOW_MODE_CHANGED,
      MainWindowModeChangedPayload {
        mode: next_state.mode,
        dock_side: next_state.dock_side,
      },
    )
    .map_err(|error| format!("failed to emit main window mode change: {error}"))
}

impl DockSide {
  fn to_docked_mode(self) -> MainWindowMode {
    match self {
      Self::Left => MainWindowMode::DockedLeft,
      Self::Right => MainWindowMode::DockedRight,
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

fn current_time_millis() -> u128 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or(0)
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
        main_window_state: Mutex::new(MainWindowRuntimeState::default()),
        auto_dock_generation: Mutex::new(0),
        manual_drag_in_progress: Mutex::new(false),
        auto_dock_suppressed_until_ms: Mutex::new(0),
      });

      if let Some(main_window) = app.get_webview_window(WINDOW_LABEL_MAIN) {
        attach_main_window_events(app.handle(), &main_window);
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_app_config,
      set_theme_mode,
      set_target_file_path,
      set_hotkey,
      save_note,
      begin_manual_window_drag,
      end_manual_window_drag,
      is_cursor_inside_main_window,
      hide_or_dock_main_window,
      restore_docked_main_window,
      redock_main_window
    ])
    .on_menu_event(handle_menu_event)
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
