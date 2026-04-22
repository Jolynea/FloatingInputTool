import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { getCurrentWindow } from '@tauri-apps/api/window'
import './index.css'
import App from './App.tsx'
import SettingsApp from './SettingsApp.tsx'

const activeView = getCurrentWindow().label === 'settings' ? 'settings' : 'main'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    {activeView === 'settings' ? <SettingsApp /> : <App />}
  </StrictMode>,
)
