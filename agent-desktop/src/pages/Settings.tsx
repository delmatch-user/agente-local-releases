import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

interface Config {
  agent_token: string | null
  api_url: string
  polling_interval_secs: number
  auto_start: boolean
}

interface SettingsProps {
  config: Config | null
  onConfigUpdate: () => void
}

interface PrinterInfo {
  name: string
  port: string
  connection_type: string
  status: string
}

export default function Settings({ config, onConfigUpdate }: SettingsProps) {
  const [printers, setPrinters] = useState<PrinterInfo[]>([])
  const [serialPorts, setSerialPorts] = useState<string[]>([])
  const [networkIp, setNetworkIp] = useState('')
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<string | null>(null)

  useEffect(() => {
    loadPrinters()
    loadSerialPorts()
  }, [])

  const loadPrinters = async () => {
    try {
      const p = await invoke<PrinterInfo[]>('get_printers')
      setPrinters(p)
    } catch (e) {
      console.error('Failed to load printers:', e)
    }
  }

  const loadSerialPorts = async () => {
    try {
      const ports = await invoke<string[]>('get_serial_ports')
      setSerialPorts(ports)
    } catch (e) {
      console.error('Failed to load serial ports:', e)
    }
  }

  const testPrinter = async (connectionType: string, address: string) => {
    setTesting(true)
    setTestResult(null)
    try {
      await invoke('test_printer', { connectionType, address })
      setTestResult('✅ Impressora OK!')
    } catch (e) {
      setTestResult(`❌ Erro: ${e}`)
    } finally {
      setTesting(false)
    }
  }

  const openDrawer = async () => {
    try {
      await invoke('open_cash_drawer', { 
        connectionType: 'network', 
        address: networkIp || '192.168.1.100'
      })
    } catch (e) {
      console.error('Failed to open drawer:', e)
    }
  }

  const handleDisconnect = async () => {
    if (confirm('Tem certeza que deseja desconectar o agente?')) {
      await invoke('stop_polling')
      await invoke('save_config', { 
        newConfig: { ...config, agent_token: null }
      })
      onConfigUpdate()
      window.location.reload()
    }
  }

  return (
    <div className="space-y-6">
      {/* Connection Info */}
      <div className="bg-white rounded-lg border border-slate-200 p-4">
        <h3 className="font-medium text-slate-800 mb-3">Conexão</h3>
        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-slate-600">Token:</span>
            <span className="font-mono text-slate-800">
              {config?.agent_token?.slice(0, 8)}...
            </span>
          </div>
          <div className="flex justify-between">
            <span className="text-slate-600">API:</span>
            <span className="text-slate-800 text-xs">{config?.api_url}</span>
          </div>
        </div>
        <button
          onClick={handleDisconnect}
          className="mt-4 w-full py-2 text-sm text-red-600 border border-red-300 rounded-lg hover:bg-red-50 transition-colors"
        >
          Desconectar
        </button>
      </div>

      {/* USB Printers */}
      <div className="bg-white rounded-lg border border-slate-200 p-4">
        <h3 className="font-medium text-slate-800 mb-3">🔌 Impressoras USB</h3>
        {serialPorts.length === 0 ? (
          <p className="text-sm text-slate-500">Nenhuma porta serial encontrada</p>
        ) : (
          <div className="space-y-2">
            {serialPorts.map((port) => (
              <div key={port} className="flex items-center justify-between p-2 bg-slate-50 rounded">
                <span className="text-sm font-mono">{port}</span>
                <button
                  onClick={() => testPrinter('usb', port)}
                  disabled={testing}
                  className="text-xs px-3 py-1 bg-primary-500 text-white rounded hover:bg-primary-600 disabled:opacity-50"
                >
                  Testar
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Network Printer */}
      <div className="bg-white rounded-lg border border-slate-200 p-4">
        <h3 className="font-medium text-slate-800 mb-3">🌐 Impressora de Rede</h3>
        <div className="flex gap-2">
          <input
            type="text"
            value={networkIp}
            onChange={(e) => setNetworkIp(e.target.value)}
            placeholder="192.168.1.100"
            className="flex-1 px-3 py-2 text-sm border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500"
          />
          <button
            onClick={() => testPrinter('network', networkIp || '192.168.1.100')}
            disabled={testing}
            className="px-4 py-2 text-sm bg-primary-500 text-white rounded-lg hover:bg-primary-600 disabled:opacity-50"
          >
            {testing ? '...' : 'Testar'}
          </button>
        </div>
        {testResult && (
          <p className={`mt-2 text-sm ${testResult.includes('OK') ? 'text-green-600' : 'text-red-600'}`}>
            {testResult}
          </p>
        )}
      </div>

      {/* Cash Drawer */}
      <div className="bg-white rounded-lg border border-slate-200 p-4">
        <h3 className="font-medium text-slate-800 mb-3">💰 Gaveta de Dinheiro</h3>
        <button
          onClick={openDrawer}
          className="w-full py-2 text-sm bg-slate-800 text-white rounded-lg hover:bg-slate-700 transition-colors"
        >
          Abrir Gaveta
        </button>
      </div>

      {/* Version */}
      <div className="text-center text-xs text-slate-400">
        MiaCardapio Agent v1.0.0
      </div>
    </div>
  )
}
