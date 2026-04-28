import { useEffect, useMemo, useState } from 'react'
import type { CSSProperties, PointerEvent } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { emit, listen } from '@tauri-apps/api/event'
import { getCurrentWindow, Window } from '@tauri-apps/api/window'
import './App.css'
import type {
  AppConfig,
  CustomTheme,
  FeedbackTone,
  HotkeyUpdateResponse,
  MarkdownTarget,
  ResolvedTheme,
  SaveShortcutMode,
  ThemeMode,
  ThemeModeChangedPayload,
} from './appModel'
import { saveShortcutOptions, themeOptions } from './appModel'

const MODIFIER_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta'])

const defaultCustomTheme: CustomTheme = {
  windowColor: '#F8F8FF',
  windowOpacity: 0.86,
  textColor: '#333333',
  accentColor: '#3EB4BF',
}

const defaultNoteTemplate = `> [!fleeting]+ {{timestamp}}
>
{{text.callout}}`

function SettingsApp() {
  const [prefersDark, setPrefersDark] = useState(() =>
    window.matchMedia('(prefers-color-scheme: dark)').matches,
  )
  const [themeMode, setThemeMode] = useState<ThemeMode>('follow-system')
  const [targets, setTargets] = useState<MarkdownTarget[]>([])
  const [activeTargetId, setActiveTargetId] = useState('')
  const [isTargetListExpanded, setIsTargetListExpanded] = useState(true)
  const [isEditingTargets, setIsEditingTargets] = useState(false)
  const [hotkey, setHotkey] = useState('')
  const [hotkeyInput, setHotkeyInput] = useState('')
  const [saveShortcutMode, setSaveShortcutMode] = useState<SaveShortcutMode>('ctrl-enter-save')
  const [noteTemplate, setNoteTemplate] = useState(defaultNoteTemplate)
  const [noteTemplateInput, setNoteTemplateInput] = useState(defaultNoteTemplate)
  const [customThemeDraft, setCustomThemeDraft] = useState<CustomTheme>(defaultCustomTheme)
  const [savedCustomTheme, setSavedCustomTheme] = useState<CustomTheme>(defaultCustomTheme)
  const [draggedTargetId, setDraggedTargetId] = useState('')
  const [isSavingTheme, setIsSavingTheme] = useState(false)
  const [isSavingTargets, setIsSavingTargets] = useState(false)
  const [isSavingHotkey, setIsSavingHotkey] = useState(false)
  const [isSavingSaveShortcutMode, setIsSavingSaveShortcutMode] = useState(false)
  const [isSavingNoteTemplate, setIsSavingNoteTemplate] = useState(false)
  const [isSavingCustomTheme, setIsSavingCustomTheme] = useState(false)
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
          setTargets(config.targets)
          setActiveTargetId(config.activeTargetId)
          setHotkey(config.hotkey)
          setHotkeyInput(config.hotkey)
          setSaveShortcutMode(config.saveShortcutMode)
          setNoteTemplate(config.noteTemplate)
          setNoteTemplateInput(config.noteTemplate)
          setCustomThemeDraft(config.customTheme)
          setSavedCustomTheme(config.customTheme)
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

    if (themeMode === 'custom') {
      return 'custom'
    }

    return prefersDark ? 'dark' : 'white'
  }, [prefersDark, themeMode])

  const themeModeLabel = useMemo(() => {
    return themeOptions.find((option) => option.value === themeMode)?.label ?? 'Follow System'
  }, [themeMode])

  const settingsShellStyle = {
    '--custom-window-color': customThemeDraft.windowColor,
    '--custom-text-color': customThemeDraft.textColor,
    '--custom-accent-color': customThemeDraft.accentColor,
  } as CSSProperties

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
      if (config.themeMode === 'custom') {
        void emit('custom-theme-preview-changed', { customTheme: customThemeDraft })
      }
      setSettingsFeedback('Theme updated.')
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setSettingsFeedback(message)
      setSettingsFeedbackTone('error')
      setIsSavingTheme(false)
    }
  }

  const handleAddTarget = () => {
    const nextTarget: MarkdownTarget = {
      id: createTargetId(),
      nickname: 'New Target',
      path: '',
    }
    setTargets((currentTargets) => [...currentTargets, nextTarget])
    setActiveTargetId(nextTarget.id)
  }

  const handleUpdateTarget = (targetId: string, patch: Partial<MarkdownTarget>) => {
    setTargets((currentTargets) =>
      currentTargets.map((target) => (target.id === targetId ? { ...target, ...patch } : target)),
    )
  }

  const handleRemoveTarget = (targetId: string) => {
    setTargets((currentTargets) => {
      if (currentTargets.length <= 1) {
        return currentTargets
      }

      const nextTargets = currentTargets.filter((target) => target.id !== targetId)
      if (activeTargetId === targetId) {
        setActiveTargetId(nextTargets[0]?.id ?? '')
      }
      return nextTargets
    })
  }

  const moveTarget = (draggedId: string, targetId: string, placement: 'before' | 'after') => {
    if (!draggedId || draggedId === targetId) {
      return
    }

    setTargets((currentTargets) => {
      const draggedTarget = currentTargets.find((target) => target.id === draggedId)
      if (!draggedTarget) {
        return currentTargets
      }

      const nextTargets = currentTargets.filter((target) => target.id !== draggedId)
      const targetIndex = nextTargets.findIndex((target) => target.id === targetId)
      if (targetIndex === -1) {
        return currentTargets
      }

      nextTargets.splice(targetIndex + (placement === 'after' ? 1 : 0), 0, draggedTarget)
      return nextTargets
    })
  }

  const handleTargetDragStart = (targetId: string, event: PointerEvent<HTMLButtonElement>) => {
    if (!isEditingTargets) {
      return
    }

    if (event.button !== 0) {
      return
    }

    event.preventDefault()
    event.currentTarget.setPointerCapture(event.pointerId)
    setDraggedTargetId(targetId)
  }

  useEffect(() => {
    if (!draggedTargetId) {
      return
    }

    const handlePointerMove = (event: globalThis.PointerEvent) => {
      const targetRow = document
        .elementsFromPoint(event.clientX, event.clientY)
        .find(
          (element): element is HTMLElement =>
            element instanceof HTMLElement && typeof element.dataset.targetId === 'string',
        )

      const targetId = targetRow?.dataset.targetId
      if (!targetId || targetId === draggedTargetId) {
        return
      }

      const rect = targetRow.getBoundingClientRect()
      const placement = event.clientY > rect.top + rect.height / 2 ? 'after' : 'before'
      moveTarget(draggedTargetId, targetId, placement)
    }

    const handlePointerEnd = () => {
      setDraggedTargetId('')
    }

    window.addEventListener('pointermove', handlePointerMove)
    window.addEventListener('pointerup', handlePointerEnd)
    window.addEventListener('blur', handlePointerEnd)

    return () => {
      window.removeEventListener('pointermove', handlePointerMove)
      window.removeEventListener('pointerup', handlePointerEnd)
      window.removeEventListener('blur', handlePointerEnd)
    }
  }, [draggedTargetId])

  const handleSaveTargets = async () => {
    setIsSavingTargets(true)
    setSettingsFeedback('')
    setSettingsFeedbackTone('normal')

    try {
      const config = await invoke<AppConfig>('set_markdown_targets', {
        targets,
        activeTargetId,
      })
      setTargets(config.targets)
      setActiveTargetId(config.activeTargetId)
      setIsEditingTargets(false)
      setSettingsFeedback('Markdown targets updated.')
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setSettingsFeedback(message)
      setSettingsFeedbackTone('error')
    } finally {
      setIsSavingTargets(false)
    }
  }

  const handleCancelTargets = async () => {
    try {
      const config = await invoke<AppConfig>('get_app_config')
      setTargets(config.targets)
      setActiveTargetId(config.activeTargetId)
      setIsEditingTargets(false)
      setSettingsFeedback('Markdown target changes discarded.')
      setSettingsFeedbackTone('normal')
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setSettingsFeedback(message)
      setSettingsFeedbackTone('error')
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

  const handleSaveNoteTemplate = async () => {
    setIsSavingNoteTemplate(true)
    setSettingsFeedback('')
    setSettingsFeedbackTone('normal')

    try {
      const config = await invoke<AppConfig>('set_note_template', {
        noteTemplate: noteTemplateInput,
      })
      setNoteTemplate(config.noteTemplate)
      setNoteTemplateInput(config.noteTemplate)
      setSettingsFeedback('Note template updated.')
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setSettingsFeedback(message)
      setSettingsFeedbackTone('error')
    } finally {
      setIsSavingNoteTemplate(false)
    }
  }

  const handleCancelNoteTemplate = () => {
    setNoteTemplateInput(noteTemplate)
    setSettingsFeedback('')
    setSettingsFeedbackTone('normal')
  }

  const handleResetNoteTemplate = () => {
    setNoteTemplateInput(defaultNoteTemplate)
    setSettingsFeedback('')
    setSettingsFeedbackTone('normal')
  }

  const handleCustomThemeDraftChange = (nextCustomTheme: CustomTheme) => {
    setCustomThemeDraft(nextCustomTheme)
    if (themeMode === 'custom') {
      void emit('custom-theme-preview-changed', { customTheme: nextCustomTheme })
    }
  }

  const handleSaveCustomTheme = async () => {
    setIsSavingCustomTheme(true)
    setSettingsFeedback('')
    setSettingsFeedbackTone('normal')

    try {
      const config = await invoke<AppConfig>('set_custom_theme', {
        customTheme: customThemeDraft,
      })
      setCustomThemeDraft(config.customTheme)
      setSavedCustomTheme(config.customTheme)
      setSettingsFeedback('Custom theme saved.')
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setSettingsFeedback(message)
      setSettingsFeedbackTone('error')
    } finally {
      setIsSavingCustomTheme(false)
    }
  }

  const handleCancelCustomTheme = () => {
    setCustomThemeDraft(savedCustomTheme)
    if (themeMode === 'custom') {
      void emit('custom-theme-preview-changed', { customTheme: savedCustomTheme })
    }
    setSettingsFeedback('')
    setSettingsFeedbackTone('normal')
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
    <main className={`settings-app-shell theme-${resolvedTheme}`} style={settingsShellStyle}>
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
              <span className="settings-label">Markdown Targets</span>
              <button
                className="settings-text-button"
                type="button"
                onClick={() => {
                  setIsTargetListExpanded((isExpanded) => !isExpanded)
                  setIsEditingTargets(false)
                  setDraggedTargetId('')
                }}
              >
                {isTargetListExpanded ? 'Collapse' : 'Expand'}
              </button>
            </div>
            {isTargetListExpanded ? (
              <div className={`target-list-editor ${isEditingTargets ? 'is-editing' : ''}`}>
                <div className="target-editor-header" aria-hidden="true">
                  <span />
                  <span>Nickname</span>
                  <span>Path</span>
                  <span>Options</span>
                </div>
                {targets.map((target) => (
                  <div
                    className={`target-editor-row ${target.id === draggedTargetId ? 'is-dragging' : ''}`}
                    key={target.id}
                    data-target-id={target.id}
                  >
                    <button
                      className="target-drag-handle"
                      type="button"
                      aria-label={`Drag ${target.nickname || target.path || 'target'}`}
                      onPointerDown={(event) => handleTargetDragStart(target.id, event)}
                    >
                      ⋮⋮
                    </button>
                    <span className="target-drag-placeholder" aria-hidden="true">
                      ||
                    </span>
                    <label className="target-editor-field">
                      <input
                        className="settings-input"
                        type="text"
                        value={target.nickname}
                        readOnly={!isEditingTargets}
                        onChange={(event) => handleUpdateTarget(target.id, { nickname: event.target.value })}
                      />
                    </label>
                    <label className="target-editor-field target-editor-path">
                      <input
                        className="settings-input"
                        type="text"
                        value={target.path}
                        readOnly={!isEditingTargets}
                        onChange={(event) => handleUpdateTarget(target.id, { path: event.target.value })}
                        placeholder="D:\\OneDrive\\Obsidian\\Fleeting Note.md"
                      />
                    </label>
                    <div className="target-editor-actions">
                      <label className="target-active-option">
                        <input
                          type="radio"
                          checked={target.id === activeTargetId}
                          disabled={!isEditingTargets}
                          onChange={() => setActiveTargetId(target.id)}
                        />
                        Active
                      </label>
                      <button
                        className="settings-text-button target-remove-button"
                        type="button"
                        onClick={() => handleRemoveTarget(target.id)}
                        disabled={!isEditingTargets || targets.length <= 1}
                      >
                        Remove
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            ) : null}
            {isTargetListExpanded ? (
              <div className="settings-actions">
                {isEditingTargets ? (
                  <>
                    <button className="settings-save-button" type="button" onClick={handleAddTarget}>
                      Add Target
                    </button>
                    <button
                      className="settings-save-button"
                      type="button"
                      onClick={handleSaveTargets}
                      disabled={isSavingTargets}
                    >
                      {isSavingTargets ? 'Saving Targets' : 'Save Targets'}
                    </button>
                    <button className="settings-save-button is-danger" type="button" onClick={handleCancelTargets}>
                      Cancel
                    </button>
                  </>
                ) : (
                  <button className="settings-save-button" type="button" onClick={() => setIsEditingTargets(true)}>
                    Edit
                  </button>
                )}
              </div>
            ) : null}
          </div>

          <div className="settings-section">
            <div className="settings-label-row">
              <span className="settings-label">Note Template</span>
              <span className="settings-value">Markdown</span>
            </div>
            <textarea
              className="settings-input note-template-input"
              value={noteTemplateInput}
              onChange={(event) => setNoteTemplateInput(event.target.value)}
              spellCheck={false}
            />
            <p className="settings-inline-value">
              Placeholders: {'{{timestamp}}'}, {'{{text}}'}, {'{{text.callout}}'}.
            </p>
            <div className="settings-actions">
              <button
                className="settings-save-button"
                type="button"
                onClick={handleSaveNoteTemplate}
                disabled={isSavingNoteTemplate}
              >
                {isSavingNoteTemplate ? 'Saving Template' : 'Save Template'}
              </button>
              <button className="settings-save-button is-secondary" type="button" onClick={handleResetNoteTemplate}>
                Default
              </button>
              <button className="settings-save-button is-danger" type="button" onClick={handleCancelNoteTemplate}>
                Cancel
              </button>
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

            {themeMode === 'custom' ? (
              <div className="custom-theme-editor">
                <label className="custom-theme-field">
                  <span className="settings-label">Window Color</span>
                  <input
                    type="color"
                    value={customThemeDraft.windowColor}
                    onChange={(event) =>
                      handleCustomThemeDraftChange({
                        ...customThemeDraft,
                        windowColor: event.target.value,
                      })
                    }
                  />
                </label>

                <label className="custom-theme-field">
                  <span className="settings-label">Window Opacity</span>
                  <input
                    className="settings-input"
                    type="number"
                    min="0.35"
                    max="1"
                    step="0.01"
                    value={customThemeDraft.windowOpacity}
                    onChange={(event) =>
                      handleCustomThemeDraftChange({
                        ...customThemeDraft,
                        windowOpacity: parseOpacityInput(event.target.value),
                      })
                    }
                  />
                </label>

                <label className="custom-theme-field">
                  <span className="settings-label">Text Color</span>
                  <input
                    type="color"
                    value={customThemeDraft.textColor}
                    onChange={(event) =>
                      handleCustomThemeDraftChange({
                        ...customThemeDraft,
                        textColor: event.target.value,
                      })
                    }
                  />
                </label>

                <label className="custom-theme-field">
                  <span className="settings-label">Accent Color</span>
                  <input
                    type="color"
                    value={customThemeDraft.accentColor}
                    onChange={(event) =>
                      handleCustomThemeDraftChange({
                        ...customThemeDraft,
                        accentColor: event.target.value,
                      })
                    }
                  />
                </label>

                <div className="settings-actions">
                  <button
                    className="settings-save-button"
                    type="button"
                    onClick={handleSaveCustomTheme}
                    disabled={isSavingCustomTheme}
                  >
                    {isSavingCustomTheme ? 'Saving Custom' : 'Save Custom'}
                  </button>
                  <button className="settings-save-button is-secondary" type="button" onClick={handleCancelCustomTheme}>
                    Cancel
                  </button>
                </div>
              </div>
            ) : null}
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

function parseOpacityInput(value: string) {
  const nextOpacity = Number(value)
  if (!Number.isFinite(nextOpacity)) {
    return defaultCustomTheme.windowOpacity
  }

  return Math.min(Math.max(nextOpacity, 0.35), 1)
}

function createTargetId() {
  return `target-${Date.now().toString(36)}`
}

export default SettingsApp
