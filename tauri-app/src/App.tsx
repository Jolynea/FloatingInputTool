import { useEffect, useMemo, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import './App.css'
import type { AppConfig, ResolvedTheme, ThemeModeChangedPayload } from './appModel'

function App() {
  const [prefersDark, setPrefersDark] = useState(() =>
    window.matchMedia('(prefers-color-scheme: dark)').matches,
  )
  const [themeMode, setThemeMode] = useState<AppConfig['themeMode']>('follow-system')
  const [draft, setDraft] = useState('')
  const [createdAt, setCreatedAt] = useState(() => formatTimestamp(new Date()))
  const [captureFeedback, setCaptureFeedback] = useState('')
  const [isSavingNote, setIsSavingNote] = useState(false)

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

    const loadConfig = async () => {
      try {
        const config = await invoke<AppConfig>('get_app_config')
        if (!ignore) {
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

    loadConfig()
    attachThemeListener().catch((error) => {
      console.error('Failed to attach theme listener', error)
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

  const canSaveNote = draft.trim().length > 0 && !isSavingNote

  const hideWindow = async () => {
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
      await getCurrentWindow().startDragging()
    } catch (error) {
      console.error('Failed to start window drag', error)
    }
  }

  const handleHide = async () => {
    setCaptureFeedback('')
    await hideWindow()
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
      setCaptureFeedback('Saved to markdown.')
      await hideWindow()
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      setCaptureFeedback(message)
    } finally {
      setIsSavingNote(false)
    }
  }

  return (
    <main className={`app-shell theme-${resolvedTheme}`}>
      <section className="floating-window" role="dialog" aria-label="Fleeting note">
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
            aria-label="Fleeting note content"
            spellCheck={false}
            value={draft}
            onChange={(event) => {
              setDraft(event.target.value)
              if (captureFeedback) {
                setCaptureFeedback('')
              }
            }}
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
              {isSavingNote ? 'Saving' : 'Save'}
            </button>
          </div>
        </footer>
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
