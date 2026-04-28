# Task Plan

## Goal

Build the first working version of FloatingInputTool as a lightweight AutoHotkey v2 Windows tray utility that captures multi-line fleeting notes into a configurable Obsidian markdown file using prepend-based `fleeting` callout blocks.

## Current Phase

Phase 13 - Implement single-window side hide in Tauri

## Phases

| Phase | Status | Description |
|-------|--------|-------------|
| 1 | complete | Create design spec, initialize git repo, connect remote, and set up planning files |
| 2 | complete | Scaffold AutoHotkey v2 project structure and shared config helpers |
| 3 | complete | Implement tray lifecycle and global hotkey registration |
| 4 | complete | Implement floating input window behavior |
| 5 | complete | Implement settings window and config persistence |
| 6 | complete | Implement markdown formatting and prepend file writing |
| 7 | in_progress | Manual verification of the end-to-end capture workflow |
| 8 | pending | Prepare packaging and usage notes for compiled distribution |
| 9 | in_progress | Add theme switching, tray theme controls, and visual packaging refinements |
| 10 | in_progress | Preserve the AutoHotkey prototype and move the main UI implementation to Tauri |
| 11 | in_progress | Scaffold Tauri app, restore the approved visual shell, and wire persistent theme switching |
| 12 | in_progress | Split Settings into a dedicated modal window and add shortcut recording |
| 13 | in_progress | Implement single-window side hide with hover-based expand and re-hide |
| 14 | in_progress | Polish packaging behavior, tray-only presence, side-hide stability, and dock animation |

## Decisions

- First implementation target is Windows only.
- AutoHotkey v2 is the chosen technology for version 1.
- Notes are written directly to markdown files without calling Obsidian APIs.
- The main capture UI must stay out of the taskbar and run from the system tray.
- Destination file path is configured in a separate settings window.
- Output format is a `fleeting` callout block with timestamp and multi-line-safe quoting.
- If the destination file or parent directories do not exist, the tool creates them.
- Side docking is deferred; first version hides back to the background instead.
- Initial config storage will use `%AppData%\FloatingInputTool\config.ini` to avoid write-permission issues for compiled builds.
- Theme mode should be configurable from both the Settings window and the tray menu, with options for dark, white, and follow-system.
- The AutoHotkey version is now a behavior prototype, not the final UI implementation path.
- Final visual fidelity will be implemented in Tauri/WebView so the app can match the approved mockup.
- The future side-hide behavior should be implemented directly in the Tauri window layer.
- The initial Tauri scaffold lives in `tauri-app/` to keep the migration isolated from the preserved AHK prototype.
- Side hide will use a single-window docking model rather than a second functional capture window.
- Main and settings windows should stay out of the Windows taskbar; the tray icon is the app's persistent presence.

## Risks

- AutoHotkey window style tuning may require iteration to fully hide the main window from the taskbar while keeping it usable.
- Hotkey registration conflicts may need clear validation and recovery.
- UTF-8 write behavior must be handled carefully so markdown content stays clean.
- Multi-line callout formatting must preserve blank lines without breaking structure.
- The current environment does not have AutoHotkey installed, so runtime validation is temporarily blocked.
- Tauri migration adds setup and packaging complexity compared with the AHK prototype.
- Tray, global hotkey, and filesystem integrations still need to be fully ported from the AHK prototype into Tauri.
- The new multi-window flow must re-enable the main window reliably if the Settings window closes through any path.
- Side-hide monitor detection and dock geometry must fail safely so the main window never becomes stranded off-screen.

## Errors Encountered

| Error | Attempt | Resolution |
|-------|---------|------------|
| Git push rejected because remote already had an initial commit | 1 | Fetched origin/main, rebased local work onto it, then pushed successfully |
| GitHub SSL error during pull/rebase flow | 1 | Used already-fetched origin/main locally and continued with a local rebase |

## Next Actions

1. Manually verify that taskbar buttons are hidden while the tray icon remains usable.
2. Manually verify first-click typing after side-handle expansion.
3. Tune side-hide animation duration if the current `160ms` movement feels too slow or too fast.
