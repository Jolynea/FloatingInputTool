# Floating Input Tool Settings Window Rework

Date: 2026-04-22

## Goal

Replace the current in-window Settings overlay with a dedicated native modal settings window in the Tauri app, while preserving the existing theme, target-file-path, and hotkey configuration features.

This rework must solve three current issues:

1. The overlay backdrop escapes the rounded main-window frame and shows sharp corners.
2. The Settings UI is constrained by the main window size.
3. The hotkey field requires manual text entry instead of capturing a pressed shortcut.

## Recommended Approach

Use a separate Tauri window for Settings and treat it as the only place where app preferences are edited.

This window will be opened from the tray and from the main capture window. When it opens, the main capture window remains visible but is blocked from interaction until Settings closes. If Settings is already open, opening it again should focus the existing Settings window instead of creating a duplicate.

This approach is preferred over keeping an overlay inside the main window because it cleanly separates responsibilities, removes layout coupling, and matches the user's expectation of a true second-level settings window.

## Window Behavior

### Main Window

- Keeps the existing fleeting-note capture UI.
- Cannot be interacted with while the Settings window is open.
- Regains focus eligibility after Settings closes.

### Settings Window

- Opens as a dedicated undecorated Tauri window.
- Uses the same overall visual language as the main window, but with its own size and layout.
- Starts at a size that fits all current settings without requiring the user to resize the main window.
- May be lightly resizable.
- Closes without affecting the main draft state.

### Open Rules

- Tray `Settings` opens the Settings window.
- Main-window `Settings` opens the same Settings window.
- If Settings is already open, the app brings that window to the front and focuses it.

## Settings Content

The Settings window contains three groups:

1. Theme
   `Follow System`, `Theme White`, `Theme Dark`
2. Target markdown file
   Editable path field with existing save behavior
3. Global hotkey
   Shortcut recorder plus save behavior

The Settings window remains the only place where path and hotkey are edited directly.

## Hotkey Recording

The existing plain text hotkey field is replaced with recorder behavior.

### Capture Rules

- When the hotkey field is focused, the app listens for pressed keys and builds the shortcut string automatically.
- Modifier keys (`Ctrl`, `Alt`, `Shift`, `Meta`) should combine with the main key in the app's stored accelerator format.
- The field displays the recorded result immediately.
- A plain modifier-only shortcut should not be accepted as a complete shortcut.

### Save Rules

- Saving still follows the current behavior already approved for the project:
  - save the requested hotkey to config even if registration fails
  - show a red warning (`#f54a45`) for invalid or conflicting shortcuts
  - keep the last successfully registered shortcut active for the current session
  - fall back to the default hotkey on startup if the saved one cannot be registered

## Data Flow

1. Main window or tray requests Settings.
2. Rust ensures a single Settings window exists.
3. Frontend Settings UI loads current persisted config from Rust.
4. Theme, path, and hotkey saves continue to invoke the existing Rust commands.
5. Rust emits any needed UI updates back to the main window as it already does for theme and runtime state.

## Error Handling

- If Settings creation fails, log the failure and keep the main window usable.
- If hotkey registration fails, keep the current warning behavior and do not leave the app without a working shortcut for the active session.
- If path saving fails, show the existing inline error feedback in the Settings window.

## Testing Scope

Manual verification should cover:

- opening Settings from both tray and main window
- confirming the main window is not interactable while Settings is open
- confirming Settings is not clipped by the main window size
- verifying duplicate Settings windows are not created
- recording a valid hotkey by pressing keys instead of typing text
- recording an invalid or conflicting hotkey and seeing the red warning
- ensuring theme and target-path saves still behave exactly as before
