export type ThemeMode = 'follow-system' | 'theme-white' | 'theme-dark'
export type ResolvedTheme = 'white' | 'dark'
export type FeedbackTone = 'normal' | 'error'

export type AppConfig = {
  themeMode: ThemeMode
  targetFilePath: string
  hotkey: string
}

export type HotkeyUpdateResponse = {
  config: AppConfig
  warning?: string | null
}

export type ThemeModeChangedPayload = {
  themeMode: ThemeMode
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
]
