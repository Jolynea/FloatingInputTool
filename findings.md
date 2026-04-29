# Findings

## Product Decisions

- A dedicated local tool is a better fit than continuing with Quicker because the main value is in the capture experience, not automation breadth.
- The app should work even when Obsidian is not running.
- The destination markdown file must be configurable because the user wants to reuse the tool across two Windows machines.

## UX Decisions

- The main capture surface should be a floating multi-line window opened by a global hotkey.
- The app should run in the tray and avoid a normal taskbar presence for the main capture window.
- The destination path must not appear in the main input window; it belongs in a secondary settings window.
- First version should hide back to the background instead of implementing more complex side-docked behavior.

## Data Format Decisions

- A single-file note stream is preferred over per-note files or daily-note embedding.
- Every note should use a custom Obsidian callout type: `fleeting`.
- The callout structure is chosen because it is safer for later CSS styling than heading-plus-paragraph layouts.
- Multi-line input must be converted so each line remains inside the callout block.

## Technical Notes

- The project repo is initialized at `D:\Claude\Project\FloatingInputTool`.
- GitHub remote is connected to [Jolynea/FloatingInputTool](https://github.com/Jolynea/FloatingInputTool).
- The first scaffold will use a single AutoHotkey entry script plus small `src` modules for constants, config loading, and tray lifecycle.
- Config storage is set to `%AppData%\FloatingInputTool\config.ini` for reliability in both source-run and compiled app scenarios.
- AutoHotkey is not installed in the current environment, so the scaffold could be reviewed statically but not executed locally yet.
- The next code pass replaced tray placeholders with actual controller wiring for the tray menu, dynamic hotkey registration, input window, settings window, note formatting, and prepend-based file writing.
- At the source level, the first round of core functionality is now present; the remaining uncertainty is runtime behavior under a real AutoHotkey v2 installation.
- Runtime validation now confirms the app starts into the tray correctly and the default hotkey can open the input window.
- The next packaging step is to keep both approved visual themes and expose them through Settings plus tray menu controls, with a follow-system mode tied to the Windows app theme registry value.
- The approved visual design cannot be fully reproduced with native AutoHotkey GUI controls. AutoHotkey can approximate the workflow, but not the HTML-level frosted-glass fidelity.
- Because of that constraint, the project is pivoting to Tauri/WebView for the main UI while keeping the AHK build as a working behavior prototype.
- The future side-hide interaction should be implemented directly in Tauri, where custom window behavior and animation are a much better fit.
- `tauri-app/` is now scaffolded with React + Vite on the frontend and a compiling `src-tauri/` Rust shell.
- The first Tauri frontend pass already recreates the floating-window visual shell with white/dark theme preview modes and an undecorated transparent window config.
- Theme switching is now best owned by Tauri itself: Rust persists the selected mode, updates tray menu checks, and emits frontend events so the React shell stays in sync.
- The first Tauri settings surface can stay lightweight for now; it only needs to expose the decisions that matter for continued visual review, starting with theme mode.
- Rust was installed via the official `rustup-init.exe` installer because `winget` package routes were unreliable in this environment.
- Tauri's `skipTaskbar` window option hides a window from the Windows taskbar while keeping the tray icon available.
- The redock guard depends on refs rather than React state; focus handlers must update those refs synchronously or first-click timing can still collapse the expanded window.
- Side-hide movement was previously a direct `set_size`/`set_position` jump. Native bounds interpolation gives a smoother transition without changing the React visual shell.
- Restore-from-dock should derive its horizontal target from the active dock side and monitor work area instead of trusting stale saved `normal_bounds.x`.
- Approved design spec path:
  [2026-04-21-floating-input-tool-design.md](D:/Claude/Project/FloatingInputTool/docs/superpowers/specs/2026-04-21-floating-input-tool-design.md)
