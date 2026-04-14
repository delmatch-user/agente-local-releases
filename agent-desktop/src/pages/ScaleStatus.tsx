import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

interface ScaleStatusProps {
  isOnline: boolean
}

interface WeightReading {
  weight: number
  request_id?: string
  stable?: boolean
  timestamp: string
}

export default function ScaleStatus({ isOnline }: ScaleStatusProps) {
  const [currentWeight, setCurrentWeight] = useState<number | null>(null)
  const [readings, setReadings] = useState<WeightReading[]>([])
  const [manualReading, setManualReading] = useState(false)
  const [scaleStatus, setScaleStatus] = useState<any>(null)

  useEffect(() => {
    loadScaleStatus()

    const unlistenWeight = listen<any>('scale_weight_read', (event) => {
      const { weight, request_id, stable } = event.payload
      setCurrentWeight(weight)
      setReadings(prev => [{
        weight,
        request_id,
        stable,
        timestamp: new Date().toISOString(),
      }, ...prev.slice(0, 19)])
    })

    const unlistenError = listen<any>('scale_weight_error', (event) => {
      console.error('Scale error:', event.payload.error)
    })

    return () => {
      unlistenWeight.then(fn => fn())
      unlistenError.then(fn => fn())
    }
  }, [])

  const loadScaleStatus = async () => {
    try {
      const status = await invoke('get_scale_status')
      setScaleStatus(status)
    } catch (e) {
      console.error('Failed to load scale status:', e)
    }
  }

  const readManually = async () => {
    setManualReading(true)
    try {
      const weight = await invoke<number>('read_scale_weight', { port: 'COM3' })
      setCurrentWeight(weight)
      setReadings(prev => [{
        weight,
        stable: true,
        timestamp: new Date().toISOString(),
      }, ...prev.slice(0, 19)])
    } catch (e) {
      console.error('Manual read failed:', e)
    } finally {
      setManualReading(false)
    }
  }

  const formatTime = (isoString: string) => {
    const date = new Date(isoString)
    return date.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit', second: '2-digit' })
  }

  const formatWeight = (grams: number) => {
    if (grams >= 1000) {
      return `${(grams / 1000).toFixed(3)} kg`
    }
    return `${grams.toFixed(1)} g`
  }

  return (
    <div className="space-y-6">
      {/* Connection Status */}
      <div className={`p-4 rounded-lg ${isOnline ? 'bg-green-50 border border-green-200' : 'bg-red-50 border border-red-200'}`}>
        <div className="flex items-center gap-3">
          <span className={`w-4 h-4 rounded-full ${isOnline ? 'bg-green-500' : 'bg-red-500'}`} />
          <div>
            <p className={`font-medium ${isOnline ? 'text-green-800' : 'text-red-800'}`}>
              {isOnline ? 'Balança Conectada' : 'Balança Desconectada'}
            </p>
            {scaleStatus?.scale_config && (
              <p className="text-sm text-green-600">
                Protocolo: {scaleStatus.scale_config.protocol}
              </p>
            )}
          </div>
        </div>
      </div>

      {/* Current Weight */}
      <div className="bg-white p-6 rounded-lg border border-slate-200 text-center">
        <p className="text-sm text-slate-600 mb-2">Peso Atual</p>
        <p className="text-5xl font-bold text-slate-800 font-mono">
          {currentWeight !== null ? formatWeight(currentWeight) : '---'}
        </p>
        <button
          onClick={readManually}
          disabled={manualReading}
          className="mt-4 px-6 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 disabled:opacity-50 transition-colors text-sm"
        >
          {manualReading ? 'Lendo...' : '⚖️ Ler Peso Agora'}
        </button>
      </div>

      {/* Reading History */}
      <div className="bg-white rounded-lg border border-slate-200">
        <div className="p-4 border-b border-slate-200">
          <h3 className="font-medium text-slate-800">Leituras Recentes</h3>
        </div>
        <div className="divide-y divide-slate-100">
          {readings.length === 0 ? (
            <p className="p-4 text-slate-500 text-center text-sm">
              Nenhuma leitura ainda
            </p>
          ) : (
            readings.map((reading, idx) => (
              <div key={idx} className="p-3 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <span className={`w-2 h-2 rounded-full ${reading.stable !== false ? 'bg-green-500' : 'bg-yellow-500'}`} />
                  <div>
                    <p className="text-sm font-medium text-slate-800 font-mono">
                      {formatWeight(reading.weight)}
                    </p>
                    {reading.request_id && (
                      <p className="text-xs text-slate-500">#{reading.request_id.slice(0, 8)}</p>
                    )}
                  </div>
                </div>
                <span className="text-xs text-slate-500">
                  {formatTime(reading.timestamp)}
                </span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  )
}
