# Floating Input Tool Design

## Summary

Build a lightweight Windows desktop utility using AutoHotkey v2 that runs in the system tray, stays out of the taskbar, and lets the user quickly capture fleeting notes into an Obsidian markdown file even when Obsidian is not running.

The first release is optimized for reliability and small size rather than visual polish. It will support a hidden background process, a global hotkey to open a floating input window, a separate settings window for configuring the target markdown file path, and prepend-based note insertion using a fixed Obsidian callout format.

## Goals

- Run in the background with a tray icon and no taskbar presence for the main input window.
- Open a floating multi-line input window via a global hotkey.
- Keep the input window focused on capture only; settings such as file path must live in a separate settings window.
- Write notes directly to a configurable markdown file without requiring Obsidian to be open.
- Automatically create the target markdown file and parent directories if they do not exist.
- Prepend new notes to the top of the file.
- Store each note in a stable custom callout structure suitable for later CSS styling in Obsidian.
- Keep the app lightweight enough for easy use on two Windows machines and future single-file distribution.

## Non-Goals

- Rich text editing or toolbar-based formatting.
- A full window docking system with advanced animations in the first version.
- Automatic list-style conversion for second- and third-level ordered lists.
- Obsidian plugin integration or any dependency on Obsidian APIs.
- Cross-platform support in the first version.

## User Experience

### Startup and runtime

- The tool launches as a background tray utility.
- No main application window appears on startup.
- The input window does not create a standard taskbar button.
- The tray menu exposes actions such as Open Input, Settings, Reload, and Exit.

### Input flow

- The user presses a configurable global hotkey.
- A compact floating input window appears above other windows.
- The user types one or more lines of text.
- The user submits the note with a keyboard shortcut or a submit button.
- The tool writes the note, clears the input box, and hides the window.

### Settings flow

- The user opens a separate settings window from the tray menu.
- The settings window allows editing the target markdown file path and the capture hotkey.
- Settings are saved outside the main input window so the capture UI remains minimal.

## Output Format

Each captured note is written as a `fleeting` callout block so later CSS can target only this note type.

Example output:

```md
> [!fleeting] 2026-04-21 11:40
> First line
> Second line
>
> Third line
```

Formatting rules:

- The callout title is the local timestamp in `YYYY-MM-DD HH:mm` format.
- Every line of note content is prefixed with `> `.
- Blank input lines become `>` to preserve paragraph breaks inside the callout.
- A blank line is inserted after each note block.
- New note blocks are always prepended to the top of the target file.

## Architecture

The application is a single AutoHotkey v2 program organized into a small number of focused modules or sections:

### Tray controller

- Creates the tray icon and tray menu.
- Handles startup behavior, app lifetime, Reload, and Exit.
- Acts as the entry point for opening the input window and settings window.

### Input window controller

- Owns the floating multi-line capture window.
- Keeps the window hidden until invoked by the hotkey or tray menu.
- Supports topmost display and drag movement.
- Hides itself after submit or cancel.

### Settings controller

- Owns the secondary configuration window.
- Edits and persists user settings.
- Validates the target file path and hotkey values before saving.

### Config store

- Persists settings in a local configuration file stored beside the app or in a predictable app-specific location.
- Initial version should prefer a simple `.ini` file for low complexity and easy portability.
- Provides defaults when the config file does not exist.

### Note formatter

- Converts raw input text into the final `fleeting` callout markdown block.
- Normalizes line endings.
- Handles blank lines safely so multi-paragraph notes do not break the callout structure.

### File writer

- Ensures the parent directory exists.
- Creates the target markdown file if missing.
- Reads the existing file content.
- Prepends the newly formatted block.
- Writes the combined content back to disk using UTF-8.

## Data Flow

1. The app starts and loads config.
2. The tray controller registers the global hotkey.
3. The user presses the hotkey.
4. The input window opens and receives text input.
5. On submit, the raw text is sent to the note formatter.
6. The formatter produces a `fleeting` callout block with timestamp.
7. The file writer ensures the destination exists and prepends the note block.
8. The input window clears its content and hides.

## Window Behavior

### Main input window

- Hidden by default.
- Marked topmost when shown.
- Excluded from the taskbar.
- Small and movable by dragging its title area or designated drag zone.
- Multi-line text box with submit and cancel actions.

### Hide behavior

The first version should use hide-to-background behavior instead of a true side-docked collapsed tab. The user asked for a side-hide concept, but implementing a robust docked edge state in the first version adds complexity without affecting the core capture loop. The window will therefore hide completely after submit or cancel, while preserving a future path to add edge docking later.

### Settings window

- Separate from the input window.
- Opened from the tray menu.
- Not topmost by default.
- Allows updating:
  - target markdown file path
  - global hotkey

## Error Handling

- If the configured file path is invalid, the tool should show a clear message and refuse to save until corrected.
- If directory creation fails, the tool should notify the user and keep the input text in place.
- If file writing fails, the tool should not clear the input box.
- If the hotkey registration fails, the app should show an actionable error at startup or when saving settings.
- Empty submissions should be ignored and should not create a note block.

## Distribution Strategy

The first release should prioritize AutoHotkey v2 source plus a compiled executable path. The architecture should avoid tightly coupling UI logic and formatting logic so a future rewrite in Tauri can reuse the same behavior contract:

- configurable destination file
- global hotkey
- background tray runtime
- prepend write behavior
- `fleeting` callout output format

This keeps AutoHotkey as a fast validation path while preserving a clean migration route later.

## Testing Strategy

Manual test coverage for the first version should include:

- app starts into tray without showing a main window
- input window does not appear in the taskbar
- hotkey opens the input window reliably
- submit writes a new note to the top of the file
- multi-line input remains inside one callout block
- blank lines remain valid inside the callout
- target file is auto-created when missing
- parent directories are auto-created when missing
- write failure preserves input text
- settings changes persist across restart

## Open Decisions Already Resolved

- Technology choice: AutoHotkey v2 for first release.
- Destination strategy: direct markdown write, independent of Obsidian runtime.
- Note organization: single markdown file, prepend insertion.
- Visual structure: custom `fleeting` callout blocks for CSS-friendly styling.
- Configuration visibility: destination path belongs in a secondary settings window, not the main input window.

## Follow-up Implementation Scope

The implementation plan should cover:

- project scaffolding for AutoHotkey v2
- tray lifecycle and hotkey registration
- hidden floating input window
- settings window and config persistence
- markdown formatting and prepend write logic
- packaging notes for future compiled distribution
