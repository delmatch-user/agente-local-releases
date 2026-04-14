import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

interface StatusProps {
  isOnline: boolean
  lastPoll: string | null
}

interface Stats {
  total_jobs: number
  successful_jobs: number
  failed_jobs: number
  today_jobs: number
}

interface JobRecord {
  id: string
  order_id: string | null
  job_type: string
  status: string
  error_message: string | null
  created_at: string
  printed_at: string | null
}

export default function Status({ isOnline, lastPoll }: StatusProps) {
  const [stats, setStats] = useState<Stats | null>(null)
  const [recentJobs, setRecentJobs] = useState<JobRecord[]>([])
  const [startTime] = useState(Date.now())

  useEffect(() => {
    loadStats()
    loadRecentJobs()
    
    const interval = setInterval(() => {
      loadStats()
      loadRecentJobs()
    }, 5000)
    
    const unlistenPrinted = listen<string>('job_printed', () => {
      loadStats()
      loadRecentJobs()
    })
    
    return () => {
      clearInterval(interval)
      unlistenPrinted.then(fn => fn())
    }
  }, [])

  const loadStats = async () => {
    try {
      const s = await invoke<Stats>('get_stats')
      setStats(s)
    } catch (e) {
      console.error('Failed to load stats:', e)
    }
  }

  const loadRecentJobs = async () => {
    try {
      const jobs = await invoke<JobRecord[]>('get_recent_jobs', { limit: 10 })
      setRecentJobs(jobs)
    } catch (e) {
      console.error('Failed to load jobs:', e)
    }
  }

  const formatUptime = () => {
    const secs = Math.floor((Date.now() - startTime) / 1000)
    const hours = Math.floor(secs / 3600)
    const mins = Math.floor((secs % 3600) / 60)
    return `${hours}h ${mins}m`
  }

  const formatTime = (isoString: string) => {
    const date = new Date(isoString)
    return date.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' })
  }

  return (
    <div className="space-y-6">
      {/* Connection Status */}
      <div className={`p-4 rounded-lg ${isOnline ? 'bg-green-50 border border-green-200' : 'bg-red-50 border border-red-200'}`}>
        <div className="flex items-center gap-3">
          <span className={`w-4 h-4 rounded-full ${isOnline ? 'bg-green-500' : 'bg-red-500'}`} />
          <div>
            <p className={`font-medium ${isOnline ? 'text-green-800' : 'text-red-800'}`}>
              {isOnline ? 'Agente Conectado' : 'Agente Desconectado'}
            </p>
            {lastPoll && (
              <p className="text-sm text-green-600">
                Última sync: {formatTime(lastPoll)}
              </p>
            )}
          </div>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-white p-4 rounded-lg border border-slate-200">
          <p className="text-3xl font-bold text-slate-800">{stats?.today_jobs ?? 0}</p>
          <p className="text-sm text-slate-600">Impressões hoje</p>
        </div>
        <div className="bg-white p-4 rounded-lg border border-slate-200">
          <p className="text-3xl font-bold text-slate-800">{formatUptime()}</p>
          <p className="text-sm text-slate-600">Tempo online</p>
        </div>
        <div className="bg-white p-4 rounded-lg border border-slate-200">
          <p className="text-3xl font-bold text-green-600">{stats?.successful_jobs ?? 0}</p>
          <p className="text-sm text-slate-600">Sucesso</p>
        </div>
        <div className="bg-white p-4 rounded-lg border border-slate-200">
          <p className="text-3xl font-bold text-red-600">{stats?.failed_jobs ?? 0}</p>
          <p className="text-sm text-slate-600">Falhas</p>
        </div>
      </div>

      {/* Recent Jobs */}
      <div className="bg-white rounded-lg border border-slate-200">
        <div className="p-4 border-b border-slate-200">
          <h3 className="font-medium text-slate-800">Impressões Recentes</h3>
        </div>
        <div className="divide-y divide-slate-100">
          {recentJobs.length === 0 ? (
            <p className="p-4 text-slate-500 text-center text-sm">
              Nenhuma impressão ainda
            </p>
          ) : (
            recentJobs.map((job) => (
              <div key={job.id} className="p-3 flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <span className={`w-2 h-2 rounded-full ${
                    job.status === 'printed' ? 'bg-green-500' : 
                    job.status === 'failed' ? 'bg-red-500' : 'bg-yellow-500'
                  }`} />
                  <div>
                    <p className="text-sm font-medium text-slate-800">
                      {job.job_type === 'kitchen' ? '🍳 Cozinha' : '🧾 Recibo'}
                      {job.order_id && ` #${job.order_id.slice(0, 8)}`}
                    </p>
                    {job.error_message && (
                      <p className="text-xs text-red-600">{job.error_message}</p>
                    )}
                  </div>
                </div>
                <span className="text-xs text-slate-500">
                  {formatTime(job.created_at)}
                </span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  )
}
