# Task Plan

## Goal

Build the first working version of FloatingInputTool as a lightweight AutoHotkey v2 Windows tray utility that captures multi-line fleeting notes into a configurable Obsidian markdown file using prepend-based `fleeting` callout blocks.

## Current Phase

Phase 7 - Manual verification of the end-to-end capture workflow

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

## Risks

- AutoHotkey window style tuning may require iteration to fully hide the main window from the taskbar while keeping it usable.
- Hotkey registration conflicts may need clear validation and recovery.
- UTF-8 write behavior must be handled carefully so markdown content stays clean.
- Multi-line callout formatting must preserve blank lines without breaking structure.
- The current environment does not have AutoHotkey installed, so runtime validation is temporarily blocked.

## Errors Encountered

| Error | Attempt | Resolution |
|-------|---------|------------|
| Git push rejected because remote already had an initial commit | 1 | Fetched origin/main, rebased local work onto it, then pushed successfully |
| GitHub SSL error during pull/rebase flow | 1 | Used already-fetched origin/main locally and continued with a local rebase |

## Next Actions

1. Run the script on a Windows machine with AutoHotkey v2 installed.
2. Verify taskbar behavior, tray behavior, hotkey invocation, settings save, and file writing.
3. Fix any AHK runtime issues discovered during manual verification.
