export type ThemeMode = 'follow-system' | 'theme-white' | 'theme-dark' | 'custom'
export type ResolvedTheme = 'white' | 'dark' | 'custom'
export type FeedbackTone = 'normal' | 'error'
export type SaveShortcutMode = 'ctrl-enter-save' | 'enter-save'

export type CustomTheme = {
  windowColor: string
  windowOpacity: number
  textColor: string
  accentColor: string
}

export type MarkdownTarget = {
  id: string
  nickname: string
  path: string
}

export type AppConfig = {
  themeMode: ThemeMode
  targetFilePath: string
  targets: MarkdownTarget[]
  activeTargetId: string
  hotkey: string
  sideHideEnabled: number
  edgeSnapThresholdPx: number
  visibleHandleWidthPx: number
  hoverOpenDelayMs: number
  hoverCloseDelayMs: number
  hotzoneWidthPx: number
  debugShowHotzone: number
  saveShortcutMode: SaveShortcutMode
  emptyInputPlaceholderColor: string
  saveShortcutTextColor: string
  saveShortcutFontSizePx: number
  customTheme: CustomTheme
}

export type HotkeyUpdateResponse = {
  config: AppConfig
  warning?: string | null
}

export type ThemeModeChangedPayload = {
  themeMode: ThemeMode
}

export type AppConfigChangedPayload = AppConfig

export type CustomThemePreviewChangedPayload = {
  customTheme: CustomTheme
}

export type MainWindowMode = 'normal' | 'docked-left' | 'docked-right' | 'expanded-from-dock'

export type MainWindowModeChangedPayload = {
  mode: MainWindowMode
  dockSide?: 'left' | 'right' | null
}

export const themeOptions: Array<{ value: ThemeMode; label: string; note: string }> = [
  {
    value: 'follow-system',
    label: 'Follow System',
    note: 'Match the current Windows appearance.',
  },
  {
    value: 'theme-white',
    label: 'Theme White',
    note: 'Soft jade glass with a brighter surface.',
  },
  {
    value: 'theme-dark',
    label: 'Theme Dark',
    note: 'Dark frost glass for a more focused feel.',
  },
  {
    value: 'custom',
    label: 'Custom',
    note: 'Use your own color and opacity values.',
  },
]

export const saveShortcutOptions: Array<{ value: SaveShortcutMode; label: string; note: string }> = [
  {
    value: 'ctrl-enter-save',
    label: 'Ctrl+Enter saves',
    note: 'Enter inserts a new line.',
  },
  {
    value: 'enter-save',
    label: 'Enter saves',
    note: 'Ctrl+Enter inserts a new line.',
  },
]
