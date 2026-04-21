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
- Approved design spec path:
  [2026-04-21-floating-input-tool-design.md](D:/Claude/Project/FloatingInputTool/docs/superpowers/specs/2026-04-21-floating-input-tool-design.md)
