# Custom Theme And Target Lists Design

## Context

FloatingInputTool currently writes fleeting notes to a single markdown target and supports built-in white, dark, and system-following themes. The next iteration adds two independent capabilities:

- A single custom theme for the main capture window.
- Multiple markdown targets with a quick switcher in the capture window.

These capabilities should be implemented in separate passes so visual configuration changes do not get mixed with write-target model changes.

## Goals

- Keep `config.json` as valid JSON for app reads and writes.
- Add a human-readable `config.help.md` beside the config so manual edits are understandable.
- Add one custom theme option, not multiple custom presets.
- Use hex colors for user-facing color config and numeric fields for opacity.
- Support multiple markdown targets with nicknames.
- Remember the last selected markdown target when the user switches targets.
- Preserve compatibility with the existing `targetFilePath` field.

## Non-Goals

- Do not convert the app config to JSONC.
- Do not support multiple named custom skins in this pass.
- Do not make the Settings window transparent.
- Do not remove existing built-in themes.

## Config Reference File

The app will keep using pure JSON at:

```text
%AppData%\FloatingInputTool\config.json
```

A generated or maintained companion file will explain fields at:

```text
%AppData%\FloatingInputTool\config.help.md
```

The help file should explain each field, where it appears in the UI, valid values, and examples. It is documentation only; the app does not parse it.

## Custom Theme

Add `custom` to `themeMode`:

```json
{
  "themeMode": "custom"
}
```

Add a single custom theme object:

```json
{
  "customTheme": {
    "windowColor": "#F8F8FF",
    "windowOpacity": 0.86,
    "textColor": "#333333",
    "accentColor": "#3EB4BF"
  }
}
```

Field behavior:

- `windowColor`: main capture window surface color.
- `windowOpacity`: main capture window surface opacity, clamped to a safe range.
- `textColor`: primary text color in the main capture window.
- `accentColor`: accent color for caret, selection, selected states, and related emphasis.

The Settings window does not follow `windowOpacity`. After saving the custom theme, it may sync stable color accents from the custom theme while preserving readability.

Settings behavior:

- Theme options include `Custom`.
- Selecting `Custom` expands custom theme controls below the theme option list.
- Editing custom theme fields previews the main capture window immediately.
- `Save` persists the custom theme.
- `Cancel` resets unsaved custom theme edits back to the last saved config.

## Multiple Markdown Targets

Keep `targetFilePath` for compatibility. Introduce:

```json
{
  "targets": [
    {
      "id": "default",
      "nickname": "Fleeting",
      "path": "D:\\OneDrive\\Obsidian\\Fleeting Note.md"
    }
  ],
  "activeTargetId": "default"
}
```

Migration behavior:

- If `targets` is missing or empty, create one target from `targetFilePath`.
- If `activeTargetId` is missing or invalid, select the first target.
- Continue writing `targetFilePath` as the active target path for compatibility.

Main capture window behavior:

- Show a horizontal target switcher below the window title.
- Each target button uses `nickname`.
- If `nickname` is empty, use the markdown file name.
- Clicking a target immediately updates `activeTargetId` and persists it.
- The next app launch selects the last clicked target.
- Save writes to the current active target.

Settings behavior:

- Target list supports expand/collapse.
- Each target row supports nickname and markdown path editing.
- Users can add, update, and remove targets.
- At least one target must remain.
- If the active target is removed, select the first remaining target.

## Implementation Order

1. Add `config.help.md` generation/maintenance and custom theme support.
2. Add multiple markdown targets and the main-window target switcher.

Each step should be build-verified with:

```text
npm run build
cargo check
```
