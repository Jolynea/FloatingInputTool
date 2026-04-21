# Progress

## 2026-04-21

### Session 1

- Reviewed the existing Obsidian vault and plugin state.
- Evaluated QuickAdd and Advanced URI as possible paths.
- Compared direct Obsidian integration against a dedicated local tool.
- Chose a standalone Windows utility as the better long-term fit.
- Chose AutoHotkey v2 for version 1, with a future migration path to Tauri if needed.
- Wrote and reviewed the design spec.
- Initialized the local git repository.
- Connected the GitHub remote and pushed the initial spec commit.

### Session 2

- Replaced the placeholder README with a project overview and roadmap.
- Created `task_plan.md`, `findings.md`, and `progress.md` to support step-by-step implementation.
- Recorded the current decisions, known risks, and next implementation actions.
- Switched the active plan to Phase 2 so implementation can begin from project scaffolding.

### Session 3

- Added the initial AutoHotkey v2 project scaffold.
- Created the main entry script plus `src` modules for app bootstrapping, constants, config loading, and tray lifecycle.
- Added a `.gitignore` for compiled artifacts and temporary output.
- Chose `%AppData%\FloatingInputTool\config.ini` as the first config storage location.
- Checked the local environment and confirmed AutoHotkey is not installed here, so runtime verification is still pending.

### Session 4

- Replaced the tray placeholders with real application wiring.
- Added a hidden floating input window controller and a separate settings window controller.
- Added note formatting and prepend-based markdown writing modules.
- Implemented dynamic hotkey updating so future settings changes can rebind the global shortcut.
- Advanced the task plan to the manual verification phase, with runtime testing currently blocked by the missing AutoHotkey installation in this environment.

### Session 5

- Installed AutoHotkey v2 locally with `winget`.
- Fixed `#Warn` variable-name collisions in the file writer and settings window code.
- Switched both GUIs to lazy creation so the app no longer flashes a window on startup.
- Confirmed startup now behaves correctly and the global hotkey opens the input window successfully.

### Session 6

- Added theme mode plumbing to config, application state, settings UI, and tray menu.
- Added a first implementation of `theme-dark`, `theme-white`, and `follow-system`.
- Refactored the input window toward the approved custom header layout with separate close, hide, timestamp, and Save regions.

### Session 7

- Compared the approved browser mockup against the AutoHotkey implementation and confirmed the AHK GUI cannot fully reproduce the required visual fidelity.
- Decided to preserve the current AutoHotkey code as a functional prototype and move the main UI implementation to Tauri.
- Recorded that the future side-hide behavior should be implemented in Tauri rather than forced into the native AHK window layer.
