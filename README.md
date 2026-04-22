# FloatingInputTool

FloatingInputTool is a lightweight Windows tray utility for capturing fleeting notes into an Obsidian markdown file without requiring Obsidian to be open.

The project currently has a working AutoHotkey v2 prototype for core behavior, but the main UI direction is now shifting to Tauri so the final product can match the approved high-fidelity visual design.

The current prototype includes:

- background tray runtime
- global hotkey to open a floating input window
- no normal taskbar presence for the main window
- separate settings window
- direct markdown writing with auto-create support
- prepend-based note insertion using a custom Obsidian `fleeting` callout
- theme switching with `theme-dark`, `theme-white`, and `follow-system`

The next implementation stage is planned in Tauri/WebView so the app can fully reproduce the approved frosted-glass visual style.

The first Tauri scaffold now lives in [tauri-app](D:/Claude/Project/FloatingInputTool/tauri-app).

## Why This Exists

The goal is to make quick note capture feel closer to a dedicated system tool than a full note-taking app:

- capture from anywhere
- keep the UI minimal
- avoid interrupting the current workflow
- preserve a markdown structure that is easy to style later in Obsidian

## Planned Note Format

Each capture is written to the top of the configured markdown file in this form:

```md
> [!fleeting] 2026-04-21 11:40
> First line
> Second line
>
> Third line
```

This keeps every entry as a self-contained block and makes later CSS customization much easier.

## Project Structure

```text
FloatingInputTool/
|- docs/
|  \- superpowers/
|     \- specs/
|        \- 2026-04-21-floating-input-tool-design.md
|- README.md
|- task_plan.md
|- findings.md
\- progress.md
```

Implementation files are being added incrementally. The AutoHotkey prototype remains in the repo as a behavior reference while the main UI direction moves toward Tauri.

The current scaffold already includes:

```text
FloatingInputTool/
|- FloatingInputTool.ahk
|- src/
|  |- App.ahk
|  |- Config.ahk
|  |- Constants.ahk
|  |- FileWriter.ahk
|  |- InputWindow.ahk
|  |- NoteFormatter.ahk
|  |- SettingsWindow.ahk
|  \- TrayController.ahk
\- .gitignore
```

## Current Scope

The validated product scope still covers:

- system tray app lifecycle
- hidden floating input window
- configurable global hotkey
- settings window for destination file path and hotkey
- markdown file auto-creation
- prepend write behavior
- safe multi-line callout formatting
- theme selection from both Settings and the tray menu
- a future side-hide interaction for preserving draft text without writing to markdown

Version 1 intentionally does not include:

- rich text editing
- advanced edge docking animations
- Obsidian plugin integration
- cross-platform support

## Development Workflow

This project uses file-based planning in the repo root:

- [task_plan.md](D:/Claude/Project/FloatingInputTool/task_plan.md) for phases and decisions
- [findings.md](D:/Claude/Project/FloatingInputTool/findings.md) for discoveries and technical notes
- [progress.md](D:/Claude/Project/FloatingInputTool/progress.md) for session-by-session execution logs

The approved design spec lives at [2026-04-21-floating-input-tool-design.md](D:/Claude/Project/FloatingInputTool/docs/superpowers/specs/2026-04-21-floating-input-tool-design.md).

## Roadmap

1. Preserve the AutoHotkey prototype as a functional reference.
2. Scaffold the Tauri app shell.
3. Port tray, hotkey, settings, and markdown writing behavior into Tauri.
4. Rebuild the input window to match the approved visual mockup.
5. Add theme switching with dark, white, and follow-system modes.
6. Implement side-hide behavior in the Tauri window layer.
7. Prepare compiled distribution.

## Future Direction

The first release is intentionally optimized for a small Windows-only utility. The project is now formally moving toward Tauri while preserving the same behavior contract:

- tray-based runtime
- global hotkey capture
- direct markdown writing
- `fleeting` callout output format
