import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import Setup from './pages/Setup'
import Status from './pages/Status'
import Settings from './pages/Settings'
import ScaleSetup from './pages/ScaleSetup'
import ScaleStatus from './pages/ScaleStatus'

type Module = 'printer' | 'scale'
type PrinterPage = 'setup' | 'status' | 'settings'
type ScalePage = 'setup' | 'status'

interface Config {
  agent_token: string | null
  scale_token: string | null
  api_url: string
  polling_interval_secs: number
  auto_start: boolean
}

function App() {
  const [activeModule, setActiveModule] = useState<Module>('printer')
  const [printerPage, setPrinterPage] = useState<PrinterPage>('setup')
  const [scalePage, setScalePage] = useState<ScalePage>('setup')
  const [config, setConfig] = useState<Config | null>(null)
  const [isOnline, setIsOnline] = useState(false)
  const [lastPoll, setLastPoll] = useState<string | null>(null)

  useEffect(() => {
    loadConfig()
    
    const unlistenSuccess = listen<string>('poll_success', (event) => {
      setIsOnline(true)
      setLastPoll(event.payload)
    })
    
    const unlistenError = listen<string>('poll_error', () => {
      setIsOnline(false)
    })
    
    return () => {
      unlistenSuccess.then(fn => fn())
      unlistenError.then(fn => fn())
    }
  }, [])

  const loadConfig = async () => {
    try {
      const cfg = await invoke<Config>('get_config')
      setConfig(cfg)
      if (cfg.agent_token) {
        setPrinterPage('status')
        await invoke('start_polling')
      }
      if (cfg.scale_token) {
        setScalePage('status')
      }
    } catch (e) {
      console.error('Failed to load config:', e)
    }
  }

  const handlePrinterTokenSet = async (token: string, supabaseKey?: string) => {
    await invoke('set_agent_token', { token, supabaseKey: supabaseKey || null })
    await invoke('start_polling')
    await loadConfig()
    setPrinterPage('status')
  }

  const handleScaleTokenSet = async (token: string) => {
    await invoke('set_scale_token', { token })
    await loadConfig()
    setScalePage('status')
  }

  const hasPrinterToken = !!config?.agent_token
  const hasScaleToken = !!config?.scale_token

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Header */}
      <header className="drag-region bg-white border-b border-slate-200 px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="text-2xl">🍽️</span>
            <h1 className="font-semibold text-slate-800">MiaCardapio Agent</h1>
          </div>
          <div className="flex items-center gap-2 no-drag">
            <span className={`w-2 h-2 rounded-full ${isOnline ? 'bg-green-500' : 'bg-red-500'}`} />
            <span className="text-sm text-slate-600">
              {isOnline ? 'Conectado' : 'Desconectado'}
            </span>
          </div>
        </div>
      </header>

      {/* Module Tabs */}
      <div className="bg-white border-b border-slate-200 px-4">
        <div className="flex gap-1">
          <button
            onClick={() => setActiveModule('printer')}
            className={`py-3 px-4 text-sm font-medium rounded-t-lg transition-colors flex items-center gap-2 ${
              activeModule === 'printer'
                ? 'bg-primary-50 text-primary-700 border-b-2 border-primary-500'
                : 'text-slate-600 hover:text-slate-800 hover:bg-slate-50'
            }`}
          >
            🖨️ Impressora
            {hasPrinterToken && (
              <span className={`w-2 h-2 rounded-full ${isOnline ? 'bg-green-500' : 'bg-red-500'}`} />
            )}
          </button>
          <button
            onClick={() => setActiveModule('scale')}
            className={`py-3 px-4 text-sm font-medium rounded-t-lg transition-colors flex items-center gap-2 ${
              activeModule === 'scale'
                ? 'bg-primary-50 text-primary-700 border-b-2 border-primary-500'
                : 'text-slate-600 hover:text-slate-800 hover:bg-slate-50'
            }`}
          >
            ⚖️ Balança
            {hasScaleToken && (
              <span className={`w-2 h-2 rounded-full ${isOnline ? 'bg-green-500' : 'bg-yellow-500'}`} />
            )}
          </button>
        </div>
      </div>

      {/* Sub-navigation */}
      {activeModule === 'printer' && hasPrinterToken && (
        <nav className="bg-white border-b border-slate-200 px-4">
          <div className="flex gap-4">
            <button
              onClick={() => setPrinterPage('status')}
              className={`py-2 px-1 text-sm font-medium border-b-2 transition-colors ${
                printerPage === 'status'
                  ? 'border-primary-500 text-primary-600'
                  : 'border-transparent text-slate-600 hover:text-slate-800'
              }`}
            >
              Status
            </button>
            <button
              onClick={() => setPrinterPage('settings')}
              className={`py-2 px-1 text-sm font-medium border-b-2 transition-colors ${
                printerPage === 'settings'
                  ? 'border-primary-500 text-primary-600'
                  : 'border-transparent text-slate-600 hover:text-slate-800'
              }`}
            >
              Configurações
            </button>
          </div>
        </nav>
      )}

      {activeModule === 'scale' && hasScaleToken && (
        <nav className="bg-white border-b border-slate-200 px-4">
          <div className="flex gap-4">
            <button
              onClick={() => setScalePage('status')}
              className={`py-2 px-1 text-sm font-medium border-b-2 transition-colors ${
                scalePage === 'status'
                  ? 'border-primary-500 text-primary-600'
                  : 'border-transparent text-slate-600 hover:text-slate-800'
              }`}
            >
              Status
            </button>
          </div>
        </nav>
      )}

      {/* Content */}
      <main className="p-4">
        {/* Printer Module */}
        {activeModule === 'printer' && (
          <>
            {printerPage === 'setup' && !hasPrinterToken && (
              <Setup onTokenSet={handlePrinterTokenSet} />
            )}
            {printerPage === 'status' && hasPrinterToken && (
              <Status isOnline={isOnline} lastPoll={lastPoll} />
            )}
            {printerPage === 'settings' && (
              <Settings config={config} onConfigUpdate={loadConfig} />
            )}
          </>
        )}

        {/* Scale Module */}
        {activeModule === 'scale' && (
          <>
            {scalePage === 'setup' && !hasScaleToken && (
              <ScaleSetup onTokenSet={handleScaleTokenSet} />
            )}
            {scalePage === 'status' && hasScaleToken && (
              <ScaleStatus isOnline={isOnline} />
            )}
          </>
        )}
      </main>
    </div>
  )
}

export default App
