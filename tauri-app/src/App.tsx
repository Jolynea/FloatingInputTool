import { useEffect, useMemo, useRef, useState } from 'react'
import type { CSSProperties } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import './App.css'
import type {
  AppConfig,
  MainWindowMode,
  MainWindowModeChangedPayload,
  ResolvedTheme,
  SaveShortcutMode,
  ThemeModeChangedPayload,
} from './appModel'

function sideHideDebugLog(label: string, details?: Record<string, unknown>) {
  const time = new Date().toISOString()
  if (details) {
    console.log(`[side-hide][${time}] ${label}`, details)
  } else {
    console.log(`[side-hide][${time}] ${label}`)
  }
}

function App() {
  const [appConfig, setAppConfig] = useState<AppConfig | null>(null)
  const [prefersDark, setPrefersDark] = useState(() =>
    window.matchMedia('(prefers-color-scheme: dark)').matches,
  )
  const [themeMode, setThemeMode] = useState<AppConfig['themeMode']>('follow-system')
  const [mainWindowMode, setMainWindowMode] = useState<MainWindowMode>('normal')
  const [dockSide, setDockSide] = useState<'left' | 'right' | null>(null)
  const [draft, setDraft] = useState('')
  const [createdAt, setCreatedAt] = useState(() => formatTimestamp(new Date()))
  const [captureFeedback, setCaptureFeedback] = useState('')
  const [isSavingNote, setIsSavingNote] = useState(false)
  const [isEditorFocused, setIsEditorFocused] = useState(false)
  const openTimerRef = useRef<number | null>(null)
  const closeTimerRef = useRef<number | null>(null)
  const manualDragActiveRef = useRef(false)
  const previousMainWindowModeRef = useRef<MainWindowMode>('normal')
  const isEditorFocusedRef = useRef(false)
  const cursorInsideWindowRef = useRef(false)
  const dockReopenBlockedUntilRef = useRef(0)
  const textareaRef = useRef<HTMLTextAreaElement | null>(null)

  const cursorPaddingPx = 8
  const dockReopenCooldownMs = 260

  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const onChange = (event: MediaQueryListEvent) => {
      setPrefersDark(event.matches)
    }

    media.addEventListener('change', onChange)
    return () => media.removeEventListener('change', onChange)
  }, [])

  useEffect(() => {
    const timer = window.setInterval(() => {
      setCreatedAt(formatTimestamp(new Date()))
    }, 60_000)

    return () => window.clearInterval(timer)
  }, [])

  useEffect(() => {
    let ignore = false
    let unlistenThemeChange: (() => void) | undefined
    let unlistenMainWindowModeChange: (() => void) | undefined

    const loadConfig = async () => {
      try {
        const config = await invoke<AppConfig>('get_app_config')
        if (!ignore) {
          setAppConfig(config)
          setThemeMode(config.themeMode)
        }
      } catch (error) {
        console.error('Failed to load app config', error)
      }
    }

    const attachThemeListener = async () => {
      unlistenThemeChange = await listen<ThemeModeChangedPayload>('theme-mode-changed', (event) => {
        if (!ignore) {
          setThemeMode(event.payload.themeMode)
        }
      })
    }

    const attachMainWindowModeListener = async () => {
      unlistenMainWindowModeChange = await listen<MainWindowModeChangedPayload>(
        'main-window-mode-changed',
        (event) => {
          if (!ignore) {
            setMainWindowMode(event.payload.mode)
            setDockSide(event.payload.dockSide ?? null)
          }
        },
      )
    }

    loadConfig()
    attachThemeListener().catch((error) => {
      console.error('Failed to attach theme listener', error)
    })
    attachMainWindowModeListener().catch((error) => {
      console.error('Failed to attach main window mode listener', error)
    })

    return () => {
      ignore = true
      unlistenThemeChange?.()
      unlistenMainWindowModeChange?.()
    }
  }, [])

  useEffect(() => {
    const previousMode = previousMainWindowModeRef.current

    if (mainWindowMode !== 'docked-left' && mainWindowMode !== 'docked-right' && openTimerRef.current) {
      window.clearTimeout(openTimerRef.current)
      openTimerRef.current = null
    }

    if (mainWindowMode !== 'expanded-from-dock' && closeTimerRef.current) {
      window.clearTimeout(closeTimerRef.current)
      closeTimerRef.current = null
    }

    if (mainWindowMode === 'expanded-from-dock') {
      cursorInsideWindowRef.current = true
      clearCloseTimer()

      if (previousMode === 'docked-left' || previousMode === 'docked-right') {
        window.setTimeout(() => {
          textareaRef.current?.focus()
        }, 0)
      }
    }

    if (
      (mainWindowMode === 'docked-left' || mainWindowMode === 'docked-right') &&
      previousMode === 'expanded-from-dock'
    ) {
      dockReopenBlockedUntilRef.current = Date.now() + dockReopenCooldownMs
    }

    sideHideDebugLog('mainWindowMode changed', {
      previousMode,
      nextMode: mainWindowMode,
      dockSide,
    })

    previousMainWindowModeRef.current = mainWindowMode
  }, [dockSide, mainWindowMode])

  useEffect(() => {
    isEditorFocusedRef.current = isEditorFocused
    if (isEditorFocused) {
      clearCloseTimer()
    }
  }, [isEditorFocused])

  useEffect(
    () => () => {
      if (openTimerRef.current) {
        window.clearTimeout(openTimerRef.current)
      }

      if (closeTimerRef.current) {
        window.clearTimeout(closeTimerRef.current)
      }
    },
    [],
  )

  useEffect(() => {
    if (mainWindowMode !== 'expanded-from-dock') {
      cursorInsideWindowRef.current = false
      return
    }

    let cancelled = false
    let timeoutId: number | null = null

    const pollCursorInsideWindow = async () => {
      if (cancelled) {
        return
      }

      try {
        const isInside = await invoke<boolean>('is_cursor_inside_main_window', {
          paddingPx: cursorPaddingPx,
        })

        if (cancelled) {
          return
        }

        cursorInsideWindowRef.current = isInside
        sideHideDebugLog('expanded poll', {
          isInside,
          isEditorFocused: isEditorFocusedRef.current,
          hasCloseTimer: Boolean(closeTimerRef.current),
        })

        if (isEditorFocusedRef.current || isInside) {
          clearCloseTimer()
        } else if (!closeTimerRef.current) {
          scheduleRedock('poll-outside')
        }
      } catch (error) {
        console.error('Failed to check cursor position against main window', error)
      } finally {
        if (!cancelled) {
          timeoutId = window.setTimeout(() => {
            void pollCursorInsideWindow()
          }, 120)
        }
      }
    }

    void pollCursorInsideWindow()

    return () => {
      cancelled = true
      if (timeoutId) {
        window.clearTimeout(timeoutId)
      }
    }
  }, [mainWindowMode])

  useEffect(() => {
    const finalizeManualWindowDrag = () => {
      if (!manualDragActiveRef.current) {
        return
      }

      manualDragActiveRef.current = false
      sideHideDebugLog('manual drag ended from mouseup')
      void invoke('end_manual_window_drag').catch((error) => {
        console.error('Failed to finalize manual window drag state', error)
      })
    }

    window.addEventListener('mouseup', finalizeManualWindowDrag)
    return () => {
      window.removeEventListener('mouseup', finalizeManualWindowDrag)
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

  const canSaveNote = draft.trim().length > 0 && !isSavingNote
  const isDocked = mainWindowMode === 'docked-left' || mainWindowMode === 'docked-right'
  const isExpandedFromDock = mainWindowMode === 'expanded-from-dock'
  const shouldShowDebugHotzone = appConfig?.debugShowHotzone === 1 && isDocked
  const saveShortcutMode: SaveShortcutMode = appConfig?.saveShortcutMode ?? 'ctrl-enter-save'
  const saveShortcutLabel = saveShortcutMode === 'enter-save' ? 'Enter' : 'Ctrl+Enter'

  const hideWindow = async () => {
    clearOpenTimer()
    clearCloseTimer()
    try {
      await getCurrentWindow().hide()
    } catch (error) {
      console.error('Failed to hide window', error)
    }
  }

  const handleWindowDragStart = async (event: React.MouseEvent<HTMLElement>) => {
    if (event.button !== 0) {
      return
    }

    const target = event.target as HTMLElement | null
    if (target?.closest('button, textarea, input, select, [data-no-drag]')) {
      return
    }

    try {
      manualDragActiveRef.current = true
      sideHideDebugLog('manual drag started')
      void invoke('begin_manual_window_drag').catch((error) => {
        console.error('Failed to mark manual window drag as started', error)
        manualDragActiveRef.current = false
      })
      await getCurrentWindow().startDragging()
    } catch (error) {
      console.error('Failed to start window drag', error)
      manualDragActiveRef.current = false
      void invoke('end_manual_window_drag').catch((cleanupError) => {
        console.error('Failed to finalize manual window drag state', cleanupError)
      })
    }
  }

  const handleHide = async () => {
    setCaptureFeedback('')
    try {
      await invoke('hide_or_dock_main_window')
    } catch (error) {
      console.error('Failed to hide or dock window', error)
      await hideWindow()
    }
  }

  const handleClose = async () => {
    setDraft('')
    setCaptureFeedback('')
    await hideWindow()
  }

  const handleSaveNote = async () => {
    if (!canSaveNote) {
      return
    }

    setIsSavingNote(true)
    setCaptureFeedback('')

    try {
      await invoke('save_note', { noteText: draft })
      setDraft('')
      setCreatedAt(formatTimestamp(new Date()))
      setCaptureFeedback('Saved to markdown.')
      if (isExpandedFromDock) {
        clearCloseTimer()
        window.setTimeout(() => {
          textareaRef.current?.focus()
        }, 0)
      } else {
        await hideWindow()
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setCaptureFeedback(message)
    } finally {
      setIsSavingNote(false)
    }
  }

  const insertEditorNewline = () => {
    const editor = textareaRef.current
    if (!editor) {
      setDraft((currentDraft) => `${currentDraft}\n`)
      return
    }

    const selectionStart = editor.selectionStart
    const selectionEnd = editor.selectionEnd
    setDraft((currentDraft) => {
      const nextDraft = `${currentDraft.slice(0, selectionStart)}\n${currentDraft.slice(selectionEnd)}`
      window.setTimeout(() => {
        editor.selectionStart = selectionStart + 1
        editor.selectionEnd = selectionStart + 1
      }, 0)
      return nextDraft
    })
  }

  const handleEditorKeyDown = (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key !== 'Enter' || event.shiftKey || event.altKey || event.metaKey) {
      return
    }

    if (saveShortcutMode === 'enter-save' && event.ctrlKey) {
      event.preventDefault()
      insertEditorNewline()
      return
    }

    const shouldSave = saveShortcutMode === 'enter-save' ? !event.ctrlKey : event.ctrlKey
    if (!shouldSave) {
      return
    }

    event.preventDefault()
    void handleSaveNote()
  }

  const clearOpenTimer = () => {
    if (openTimerRef.current) {
      window.clearTimeout(openTimerRef.current)
      openTimerRef.current = null
    }
  }

  const clearCloseTimer = () => {
    if (closeTimerRef.current) {
      window.clearTimeout(closeTimerRef.current)
      closeTimerRef.current = null
    }
  }

  const handleDockHotzoneEnter = () => {
    if (!isDocked) {
      return
    }

    if (Date.now() < dockReopenBlockedUntilRef.current) {
      sideHideDebugLog('dock hotzone enter suppressed by cooldown', {
        blockedUntil: dockReopenBlockedUntilRef.current,
      })
      return
    }

    clearOpenTimer()
    const delay = appConfig?.hoverOpenDelayMs ?? 180
    sideHideDebugLog('dock hotzone enter', { delay })
    openTimerRef.current = window.setTimeout(async () => {
      openTimerRef.current = null
      try {
        sideHideDebugLog('restore_docked_main_window invoke')
        await invoke('restore_docked_main_window')
      } catch (error) {
        console.error('Failed to restore docked window', error)
      }
    }, delay)
  }

  const handleDockHotzoneLeave = () => {
    sideHideDebugLog('dock hotzone leave')
    clearOpenTimer()
  }

  const scheduleRedock = (reason: string, delayOverride?: number) => {
    if (mainWindowMode !== 'expanded-from-dock') {
      sideHideDebugLog('scheduleRedock skipped: not expanded', { reason, mainWindowMode })
      return
    }

    if (isEditorFocusedRef.current || cursorInsideWindowRef.current) {
      sideHideDebugLog('scheduleRedock skipped: blocked by focus/inside', {
        reason,
        isEditorFocused: isEditorFocusedRef.current,
        cursorInside: cursorInsideWindowRef.current,
      })
      clearCloseTimer()
      return
    }

    clearCloseTimer()
    const delay = delayOverride ?? appConfig?.hoverCloseDelayMs ?? 320
    sideHideDebugLog('scheduleRedock armed', { reason, delay })
    closeTimerRef.current = window.setTimeout(async () => {
      closeTimerRef.current = null

      if (isEditorFocusedRef.current || cursorInsideWindowRef.current) {
        sideHideDebugLog('scheduleRedock fire canceled by focus/inside', {
          reason,
          isEditorFocused: isEditorFocusedRef.current,
          cursorInside: cursorInsideWindowRef.current,
        })
        return
      }

      try {
        const isInside = await invoke<boolean>('is_cursor_inside_main_window', {
          paddingPx: cursorPaddingPx,
        })
        sideHideDebugLog('scheduleRedock fire recheck', { reason, isInside })
        if (isInside) {
          return
        }

        sideHideDebugLog('redock_main_window invoke', { reason })
        await invoke('redock_main_window')
      } catch (error) {
        console.error('Failed to redock window', error)
      }
    }, delay)
  }

  const appShellStyle = {
    '--dock-visible-width': `${appConfig?.visibleHandleWidthPx ?? 22}px`,
    '--dock-hotzone-width': `${appConfig?.hotzoneWidthPx ?? 36}px`,
    '--empty-input-placeholder-color': appConfig?.emptyInputPlaceholderColor ?? 'rgba(51, 51, 51, 0.42)',
    '--save-shortcut-text-color': appConfig?.saveShortcutTextColor ?? 'currentColor',
    '--save-shortcut-font-size': `${appConfig?.saveShortcutFontSizePx ?? 9}px`,
  } as CSSProperties

  return (
    <main
      className={`app-shell theme-${resolvedTheme} ${isDocked ? `is-docked dock-${dockSide}` : ''} ${
        mainWindowMode === 'expanded-from-dock' ? 'is-expanded-from-dock' : ''
      }`}
      style={appShellStyle}
    >
      <section className={`floating-window ${isDocked ? 'is-docked' : ''}`} role="dialog" aria-label="Fleeting note">
        {isDocked ? (
          <div
            className={`dock-shell dock-shell-${dockSide ?? 'left'}`}
            onMouseEnter={handleDockHotzoneEnter}
            onMouseLeave={handleDockHotzoneLeave}
          >
            {shouldShowDebugHotzone ? <div className="dock-hotzone-debug" aria-hidden="true" /> : null}
            <div className="dock-handle" aria-hidden="true">
              <span className="dock-handle-label">Fleeting Note</span>
            </div>
          </div>
        ) : (
          <>
            <header className="window-header">
              <div
                className="window-drag-area"
                data-tauri-drag-region
                onMouseDown={handleWindowDragStart}
              >
                <h1 data-tauri-drag-region>Fleeting Note</h1>
              </div>
              <button
                className="icon-button close-button"
                type="button"
                aria-label="Close"
                onClick={handleClose}
              >
                &times;
              </button>
            </header>

            <div className="editor-shell">
              <textarea
                ref={textareaRef}
                aria-label="Fleeting note content"
                spellCheck={false}
                value={draft}
                placeholder="Capture your fleeting thoughts"
                onFocus={() => {
                  setIsEditorFocused(true)
                  sideHideDebugLog('editor focus')
                  clearCloseTimer()
                }}
                onBlur={() => {
                  setIsEditorFocused(false)
                  sideHideDebugLog('editor blur', {
                    mainWindowMode,
                    cursorInside: cursorInsideWindowRef.current,
                  })
                  if (mainWindowMode === 'expanded-from-dock' && !cursorInsideWindowRef.current) {
                    scheduleRedock('editor-blur')
                  }
                }}
                onChange={(event) => {
                  setDraft(event.target.value)
                  if (captureFeedback) {
                    setCaptureFeedback('')
                  }
                }}
                onKeyDown={handleEditorKeyDown}
              />
            </div>

            <footer className="window-footer">
              <div className="window-meta">
                <time>{createdAt}</time>
                {captureFeedback ? <span className="feedback-text">{captureFeedback}</span> : null}
              </div>

              <div className="window-actions">
                <button
                  className="icon-button hide-button"
                  type="button"
                  aria-label="Hide"
                  onClick={handleHide}
                >
                  <svg className="ghost-icon" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                    <path d="M16 10.5C16 11.3284 15.5523 12 15 12C14.4477 12 14 11.3284 14 10.5C14 9.67157 14.4477 9 15 9C15.5523 9 16 9.67157 16 10.5Z" />
                    <ellipse cx="9" cy="10.5" rx="1" ry="1.5" />
                    <path d="M22 12.3006C22 6.61173 17.5228 2 12 2C6.47715 2 2 6.61173 2 12.3006V19.723C2 21.0453 3.35098 21.9054 4.4992 21.314C5.42726 20.836 6.5328 20.9069 7.39614 21.4998C8.36736 22.1667 9.63264 22.1667 10.6039 21.4998L10.9565 21.2576C11.5884 20.8237 12.4116 20.8237 13.0435 21.2576L13.3961 21.4998C14.3674 22.1667 15.6326 22.1667 16.6039 21.4998C17.4672 20.9069 18.5727 20.836 19.5008 21.314C20.649 21.9054 22 21.0453 22 19.723V16.0118" />
                  </svg>
                </button>
                <button className="save-button" type="button" disabled={!canSaveNote} onClick={handleSaveNote}>
                  <span className="save-button-label">{isSavingNote ? 'Saving' : 'Save'}</span>
                  <span className="save-button-shortcut">{saveShortcutLabel}</span>
                </button>
              </div>
            </footer>
          </>
        )}
      </section>
    </main>
  )
}

function formatTimestamp(date: Date) {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  const hours = String(date.getHours()).padStart(2, '0')
  const minutes = String(date.getMinutes()).padStart(2, '0')

  return `${year}-${month}-${day} ${hours}:${minutes}`
}

export default App
