import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow, Window } from '@tauri-apps/api/window'
import './App.css'
import type {
  AppConfig,
  FeedbackTone,
  HotkeyUpdateResponse,
  ResolvedTheme,
  SaveShortcutMode,
  ThemeMode,
  ThemeModeChangedPayload,
} from './appModel'
import { saveShortcutOptions, themeOptions } from './appModel'

const MODIFIER_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta'])

function SettingsApp() {
  const [prefersDark, setPrefersDark] = useState(() =>
    window.matchMedia('(prefers-color-scheme: dark)').matches,
  )
  const [themeMode, setThemeMode] = useState<ThemeMode>('follow-system')
  const [targetFilePath, setTargetFilePath] = useState('')
  const [targetFileInput, setTargetFileInput] = useState('')
  const [hotkey, setHotkey] = useState('')
  const [hotkeyInput, setHotkeyInput] = useState('')
  const [saveShortcutMode, setSaveShortcutMode] = useState<SaveShortcutMode>('ctrl-enter-save')
  const [isSavingTheme, setIsSavingTheme] = useState(false)
  const [isSavingPath, setIsSavingPath] = useState(false)
  const [isSavingHotkey, setIsSavingHotkey] = useState(false)
  const [isSavingSaveShortcutMode, setIsSavingSaveShortcutMode] = useState(false)
  const [settingsFeedback, setSettingsFeedback] = useState('')
  const [settingsFeedbackTone, setSettingsFeedbackTone] = useState<FeedbackTone>('normal')
  const [isRecordingHotkey, setIsRecordingHotkey] = useState(false)

  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const onChange = (event: MediaQueryListEvent) => {
      setPrefersDark(event.matches)
    }

    media.addEventListener('change', onChange)
    return () => media.removeEventListener('change', onChange)
  }, [])

  useEffect(() => {
    let ignore = false
    let unlistenThemeChange: (() => void) | undefined

    const loadConfig = async () => {
      try {
        const config = await invoke<AppConfig>('get_app_config')
        if (!ignore) {
          setThemeMode(config.themeMode)
          setTargetFilePath(config.targetFilePath)
          setTargetFileInput(config.targetFilePath)
          setHotkey(config.hotkey)
          setHotkeyInput(config.hotkey)
          setSaveShortcutMode(config.saveShortcutMode)
        }
      } catch (error) {
        console.error('Failed to load settings config', error)
      }
    }

    const attachThemeListener = async () => {
      unlistenThemeChange = await listen<ThemeModeChangedPayload>('theme-mode-changed', (event) => {
        if (!ignore) {
          setThemeMode(event.payload.themeMode)
          setIsSavingTheme(false)
        }
      })
    }

    loadConfig()
    attachThemeListener().catch((error) => {
      console.error('Failed to attach settings listeners', error)
    })

    return () => {
      ignore = true
      unlistenThemeChange?.()
    }
  }, [])

  const resolvedTheme = useMemo<ResolvedTheme>(() => {
    if (themeMode === 'theme-dark') {
      return 'dark'
    }

    if (themeMode === 'theme-white') {
      return 'white'
    }

    return prefersDark ? 'dark' : 'white'
  }, [prefersDark, themeMode])

  const themeModeLabel = useMemo(() => {
    return themeOptions.find((option) => option.value === themeMode)?.label ?? 'Follow System'
  }, [themeMode])

  const hotkeyRecorderNote = useMemo(() => {
    if (isRecordingHotkey) {
      return 'Press your shortcut now. Modifier-only keys are ignored.'
    }

    if (hotkey) {
      return `Current active: ${hotkey}`
    }

    return 'No active shortcut yet.'
  }, [hotkey, isRecordingHotkey])

  const handleWindowDragStart = async (event: React.MouseEvent<HTMLElement>) => {
    if (event.button !== 0) {
      return
    }

    const target = event.target as HTMLElement | null
    if (target?.closest('button, textarea, input, select, [data-no-drag]')) {
      return
    }

    try {
      await getCurrentWindow().startDragging()
    } catch (error) {
      console.error('Failed to start settings window drag', error)
    }
  }

  const handleThemeChange = async (nextThemeMode: ThemeMode) => {
    if (themeMode === nextThemeMode || isSavingTheme) {
      return
    }

    setIsSavingTheme(true)
    setSettingsFeedback('')
    setSettingsFeedbackTone('normal')

    try {
      const config = await invoke<AppConfig>('set_theme_mode', { themeMode: nextThemeMode })
      setThemeMode(config.themeMode)
      setSettingsFeedback('Theme updated.')
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setSettingsFeedback(message)
      setSettingsFeedbackTone('error')
      setIsSavingTheme(false)
    }
  }

  const handleSavePath = async () => {
    setIsSavingPath(true)
    setSettingsFeedback('')
    setSettingsFeedbackTone('normal')

    try {
      const config = await invoke<AppConfig>('set_target_file_path', {
        targetFilePath: targetFileInput,
      })
      setTargetFilePath(config.targetFilePath)
      setTargetFileInput(config.targetFilePath)
      setSettingsFeedback('Target markdown file updated.')
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setSettingsFeedback(message)
      setSettingsFeedbackTone('error')
    } finally {
      setIsSavingPath(false)
    }
  }

  const handleSaveHotkey = async () => {
    setIsSavingHotkey(true)
    setSettingsFeedback('')
    setSettingsFeedbackTone('normal')

    try {
      const response = await invoke<HotkeyUpdateResponse>('set_hotkey', {
        hotkey: hotkeyInput,
      })
      setHotkey(response.config.hotkey)
      setHotkeyInput(response.config.hotkey)

      if (response.warning) {
        setSettingsFeedback(response.warning)
        setSettingsFeedbackTone('error')
      } else {
        setSettingsFeedback('Hotkey updated.')
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setSettingsFeedback(message)
      setSettingsFeedbackTone('error')
    } finally {
      setIsSavingHotkey(false)
    }
  }

  const handleSaveShortcutModeChange = async (nextMode: SaveShortcutMode) => {
    if (saveShortcutMode === nextMode || isSavingSaveShortcutMode) {
      return
    }

    setIsSavingSaveShortcutMode(true)
    setSettingsFeedback('')
    setSettingsFeedbackTone('normal')

    try {
      const config = await invoke<AppConfig>('set_save_shortcut_mode', {
        saveShortcutMode: nextMode,
      })
      setSaveShortcutMode(config.saveShortcutMode)
      setSettingsFeedback('Save shortcut updated.')
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setSettingsFeedback(message)
      setSettingsFeedbackTone('error')
    } finally {
      setIsSavingSaveShortcutMode(false)
    }
  }

  const handleHotkeyKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    event.preventDefault()
    event.stopPropagation()

    const nextHotkey = buildAcceleratorFromKeyboardEvent(event)
    if (!nextHotkey) {
      setSettingsFeedback('')
      setSettingsFeedbackTone('normal')
      return
    }

    setHotkeyInput(nextHotkey)
    setSettingsFeedback('')
    setSettingsFeedbackTone('normal')
  }

  const handleClose = async () => {
    try {
      const mainWindow = await Window.getByLabel('main')
      await mainWindow?.setEnabled(true)
      await getCurrentWindow().hide()
      await mainWindow?.setFocus()
    } catch (error) {
      console.error('Failed to close settings window', error)
    }
  }

  return (
    <main className={`settings-app-shell theme-${resolvedTheme}`}>
      <section className="settings-panel settings-window" role="dialog" aria-label="Settings">
        <div className="settings-header">
          <div
            className="settings-window-drag-area settings-drag-area"
            data-tauri-drag-region
            onMouseDown={handleWindowDragStart}
          >
            <h2 data-tauri-drag-region>Settings</h2>
            <p data-tauri-drag-region>
              Theme, destination markdown path, and shortcut recording live here.
            </p>
          </div>
          <button
            className="icon-button settings-close-button"
            type="button"
            aria-label="Close settings"
            data-no-drag
            onClick={handleClose}
          >
            &times;
          </button>
        </div>

        <div className="settings-content">
          <div className="settings-section">
            <div className="settings-label-row">
              <span className="settings-label">Target File</span>
            </div>
            <input
              className="settings-input"
              type="text"
              value={targetFileInput}
              onChange={(event) => setTargetFileInput(event.target.value)}
              placeholder="D:\\OneDrive\\Obsidian\\Fleeting Note.md"
            />
            <div className="settings-actions">
              <button
                className="settings-save-button"
                type="button"
                onClick={handleSavePath}
                disabled={isSavingPath}
              >
                {isSavingPath ? 'Saving Path' : 'Save Path'}
              </button>
              <span className="settings-inline-value">{targetFilePath}</span>
            </div>
          </div>

          <div className="settings-section">
            <div className="settings-label-row">
              <span className="settings-label">Hotkey</span>
              <span className="settings-value">{hotkey || 'Not active'}</span>
            </div>
            <input
              className={`settings-input hotkey-recorder ${isRecordingHotkey ? 'is-recording' : ''}`}
              type="text"
              value={hotkeyInput}
              readOnly
              onFocus={() => setIsRecordingHotkey(true)}
              onBlur={() => setIsRecordingHotkey(false)}
              onKeyDown={handleHotkeyKeyDown}
              placeholder="Click here, then press your shortcut"
            />
            <div className="settings-actions">
              <button
                className="settings-save-button"
                type="button"
                onClick={handleSaveHotkey}
                disabled={isSavingHotkey}
              >
                {isSavingHotkey ? 'Saving Hotkey' : 'Save Hotkey'}
              </button>
              <span className="settings-inline-value">{hotkeyRecorderNote}</span>
              <span className="settings-inline-value">
                Save keeps the entered hotkey in config. If it conflicts, the previous working hotkey stays active.
              </span>
            </div>
          </div>

          <div className="settings-section">
            <div className="settings-label-row">
              <span className="settings-label">Theme</span>
              <span className="settings-value">{themeModeLabel}</span>
            </div>

            <div className="theme-option-list">
              {themeOptions.map((option) => {
                const selected = option.value === themeMode
                return (
                  <button
                    key={option.value}
                    className={`theme-option ${selected ? 'is-selected' : ''}`}
                    type="button"
                    onClick={() => handleThemeChange(option.value)}
                    disabled={isSavingTheme}
                  >
                    <span className="theme-option-label">{option.label}</span>
                    <span className="theme-option-note">{option.note}</span>
                  </button>
                )
              })}
            </div>
          </div>

          <div className="settings-section">
            <div className="settings-label-row">
              <span className="settings-label">Save Shortcut</span>
              <span className="settings-value">
                {saveShortcutOptions.find((option) => option.value === saveShortcutMode)?.label}
              </span>
            </div>

            <div className="theme-option-list">
              {saveShortcutOptions.map((option) => {
                const selected = option.value === saveShortcutMode
                return (
                  <button
                    key={option.value}
                    className={`theme-option ${selected ? 'is-selected' : ''}`}
                    type="button"
                    onClick={() => handleSaveShortcutModeChange(option.value)}
                    disabled={isSavingSaveShortcutMode}
                  >
                    <span className="theme-option-label">{option.label}</span>
                    <span className="theme-option-note">{option.note}</span>
                  </button>
                )
              })}
            </div>
          </div>
        </div>

        {settingsFeedback ? (
          <div className={`settings-feedback ${settingsFeedbackTone === 'error' ? 'is-error' : ''}`}>
            {settingsFeedback}
          </div>
        ) : null}
      </section>
    </main>
  )
}

function buildAcceleratorFromKeyboardEvent(event: React.KeyboardEvent<HTMLInputElement>) {
  const mainKey = normalizeAcceleratorKey(event.key)
  if (!mainKey) {
    return null
  }

  const parts: string[] = []
  if (event.ctrlKey) {
    parts.push('Ctrl')
  }
  if (event.altKey) {
    parts.push('Alt')
  }
  if (event.shiftKey) {
    parts.push('Shift')
  }
  if (event.metaKey) {
    parts.push('Meta')
  }

  parts.push(mainKey)
  return parts.join('+')
}

function normalizeAcceleratorKey(key: string) {
  if (MODIFIER_KEYS.has(key)) {
    return null
  }

  if (key === ' ') {
    return 'Space'
  }

  if (/^[a-z]$/i.test(key)) {
    return key.toUpperCase()
  }

  if (/^[0-9]$/.test(key)) {
    return key
  }

  if (/^F([1-9]|1[0-9]|2[0-4])$/i.test(key)) {
    return key.toUpperCase()
  }

  const specialKeyMap: Record<string, string> = {
    ArrowUp: 'Up',
    ArrowDown: 'Down',
    ArrowLeft: 'Left',
    ArrowRight: 'Right',
    Escape: 'Esc',
    Enter: 'Enter',
    Tab: 'Tab',
    Backspace: 'Backspace',
    Delete: 'Delete',
    Insert: 'Insert',
    Home: 'Home',
    End: 'End',
    PageUp: 'PageUp',
    PageDown: 'PageDown',
  }

  return specialKeyMap[key] ?? null
}

export default SettingsApp
