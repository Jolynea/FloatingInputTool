# FloatingInputTool

FloatingInputTool is a lightweight Windows tray utility for capturing fleeting notes into an Obsidian markdown file without requiring Obsidian to be open.

The first version is planned as an AutoHotkey v2 app with:

- background tray runtime
- global hotkey to open a floating input window
- no normal taskbar presence for the main window
- separate settings window
- direct markdown writing with auto-create support
- prepend-based note insertion using a custom Obsidian `fleeting` callout

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

Implementation files will be added as development proceeds.

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

Version 1 is expected to cover:

- system tray app lifecycle
- hidden floating input window
- configurable global hotkey
- settings window for destination file path and hotkey
- markdown file auto-creation
- prepend write behavior
- safe multi-line callout formatting

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

1. Scaffold the AutoHotkey v2 app structure.
2. Implement tray startup and hotkey registration.
3. Build the floating input window.
4. Add settings persistence.
5. Implement markdown formatting and prepend write logic.
6. Test the full capture flow on Windows.
7. Prepare compiled distribution.

## Future Direction

The first release is intentionally optimized for a small Windows-only utility. If the workflow proves useful, the app may later be rewritten in Tauri while preserving the same behavior contract:

- tray-based runtime
- global hotkey capture
- direct markdown writing
- `fleeting` callout output format
