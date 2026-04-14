import { useState, useEffect } from 'react'
import { KeepAwake } from '@capacitor-community/keep-awake'
import { agentPollingService } from './pollingService'

type Module = 'printer' | 'settings' // Adicionado Menu "Configurar Máquinas"
type ScalePage = 'setup' | 'status'

interface Config {
  agent_token: string | null
  supabase_key: string | null
  api_url: string
  polling_interval_secs: number
  auto_start: boolean
}

const LocalStorageAPI = {
  getConfig: async (): Promise<Config> => {
    const raw = localStorage.getItem('agent_config')
    if (raw) return JSON.parse(raw)
    return {
      agent_token: null,
      supabase_key: null,
      api_url: 'https://szlyzyflalerxuyxfxzh.supabase.co/functions/v1', // URL padrão do projeto atual
      polling_interval_secs: 5,
      auto_start: true
    }
  },
  saveConfig: async (cfg: Config) => {
    localStorage.setItem('agent_config', JSON.stringify(cfg))
  }
}

function App() {
  const [activeModule, setActiveModule] = useState<Module>('printer')
  const [config, setConfig] = useState<Config | null>(null)
  const [isOnline, setIsOnline] = useState(false)
  const [lastPoll, setLastPoll] = useState<string | null>(null)
  
  // Estado local para IPs e Nomes das Impressoras
  const [printersList, setPrintersList] = useState<{name: string, type: 'network'|'bluetooth', address: string}[]>([])

  useEffect(() => {
    KeepAwake.keepAwake().catch(console.error)
    loadConfig()

    const savedPrinters = localStorage.getItem('agent_printers')
    if (savedPrinters) {
      setPrintersList(JSON.parse(savedPrinters))
    } else {
      // Valor padrão se estiver vazio
      const defaultPrinters = [{ name: 'Caixa / Balcão Principal', type: 'network' as const, address: '192.168.1.100' }]
      setPrintersList(defaultPrinters)
      localStorage.setItem('agent_printers', JSON.stringify(defaultPrinters))
    }

    return () => agentPollingService.stop()
  }, [])

  useEffect(() => {
    if (config?.agent_token) {
      agentPollingService.setConfig(config.api_url, config.agent_token, printersList, config.supabase_key || '')
      agentPollingService.start(
        (time) => {
          setIsOnline(true)
          setLastPoll(time)
        },
        (errorMsg) => {
          setIsOnline(false)
          console.error("Poling Falhou:", errorMsg)
        }
      )
    } else {
      agentPollingService.stop()
      setIsOnline(false)
    }
  }, [config?.agent_token, config?.api_url])

  const loadConfig = async () => {
    try {
      const cfg = await LocalStorageAPI.getConfig()
      setConfig(cfg)
    } catch (e) {
      console.error('Failed to load config', e)
    }
  }

  const handlePrinterTokenSet = async (token: string, apiUrlParam?: string, supabaseKey?: string) => {
    if(!config) return;
    const cleanToken = token.trim();
    const cleanApiUrl = (apiUrlParam || config.api_url).trim();
    const cleanSupabaseKey = (supabaseKey || '').trim();
    const newCfg = { ...config, agent_token: cleanToken, api_url: cleanApiUrl, supabase_key: cleanSupabaseKey }
    await LocalStorageAPI.saveConfig(newCfg)
    await loadConfig()
  }

  const handleDisconnect = async () => {
    if(!config) return;
    const newCfg = { ...config, agent_token: null }
    await LocalStorageAPI.saveConfig(newCfg)
    await loadConfig()
    setActiveModule('printer')
  }

  const hasPrinterToken = !!config?.agent_token

  return (
    <div className="min-h-screen bg-slate-50 flex flex-col font-sans">
      <header className="bg-white border-b border-slate-200 px-6 py-4 rounded-b-2xl shadow-sm mb-4">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-3xl">🍽️</span>
            <div>
              <h1 className="font-bold text-slate-800 tracking-tight text-xl leading-tight">MiaCardapio</h1>
              <p className="text-xs font-semibold text-slate-400 text-primary-500 uppercase tracking-widest">Painel Agent (Tablet)</p>
            </div>
          </div>
          <div className="flex items-center gap-2 bg-slate-50 px-3 py-1.5 rounded-full border border-slate-100">
            <span className={`w-3 h-3 rounded-full ${isOnline ? 'bg-green-500 shadow-lg shadow-green-500/50 animate-pulse' : 'bg-red-500 shadow-lg shadow-red-500/50'}`} />
            <span className="text-sm font-bold text-slate-600">
              {isOnline ? 'Online' : 'Offline'}
            </span>
          </div>
        </div>
      </header>
      
      {hasPrinterToken && (
        <div className="px-4 mb-4">
           <div className="bg-white rounded-xl shadow-sm p-2 flex gap-2 border border-slate-200 w-full max-w-4xl mx-auto">
              <button 
                onClick={() => setActiveModule('printer')}
                className={`flex-1 py-3 text-sm font-bold rounded-lg transition-all ${activeModule === 'printer' ? 'bg-primary-50 text-blue-700' : 'text-slate-500 hover:bg-slate-50'}`}>
                Monitoramento
              </button>
              <button 
                onClick={() => setActiveModule('settings')}
                className={`flex-1 py-3 text-sm font-bold rounded-lg transition-all ${activeModule === 'settings' ? 'bg-primary-50 text-blue-700' : 'text-slate-500 hover:bg-slate-50'}`}>
                ⚙️ Configurar Máquinas
              </button>
           </div>
        </div>
      )}

      <main className="px-4 flex-1 overflow-y-auto w-full max-w-4xl mx-auto pb-8">
        
        {/* Painel Central */}
        <div className="bg-white rounded-2xl shadow-sm p-6 sm:p-8 flex flex-col gap-6 border border-slate-200">
             
           { activeModule === 'printer' ? ( 
             <>
               <div className="border-b border-slate-100 pb-4 flex justify-between items-end">
                 <div>
                   <h2 className="text-2xl font-bold text-slate-800">
                     {hasPrinterToken ? 'Conectado ao Servidor' : 'Conexão de Base'}
                   </h2>
                   <p className="text-slate-500 text-sm mt-1 font-medium">
                     {!hasPrinterToken ? 'Insira o token oficial do seu painel e a URL de API.' : 'Aguardando ping do recebimento de pedidos...'}
                     {lastPoll && <span className="block mt-1 font-mono text-xs bg-slate-100 p-1 px-2 rounded-md inline-block">Último pulso: {lastPoll}</span>}
                   </p>
                 </div>
                 
                 {hasPrinterToken && (
                   <button 
                    onClick={handleDisconnect}
                    className="px-4 py-2 bg-red-50 text-red-600 text-sm font-bold rounded-lg hover:bg-red-100 transition-colors">
                     Desconectar
                   </button>
                 )}
               </div>

               {!hasPrinterToken ? (
                 <form 
                   className="flex flex-col gap-4 max-w-lg"
                   onSubmit={(e) => {
                     e.preventDefault()
                     const f = new FormData(e.currentTarget)
                     handlePrinterTokenSet(
                       f.get('token') as string, 
                       f.get('api') as string,
                       f.get('supabase_key') as string
                     )
                   }}
                 >
                   <div className="flex flex-col gap-1.5">
                     <label className="text-xs font-bold text-slate-500 uppercase tracking-wide">URL do Supabase (Edge Functions)</label>
                     <input 
                       name="api"
                       type="url"
                       required
                       defaultValue={config?.api_url}
                       placeholder="https://xyz.supabase.co/functions/v1"
                       className="w-full border-2 border-slate-200 rounded-xl px-4 py-3 text-sm font-medium focus:ring-4 focus:ring-primary-100 focus:border-primary-500 outline-none transition-all"
                     />
                   </div>

                   <div className="flex flex-col gap-1.5">
                     <label className="text-xs font-bold text-slate-500 uppercase tracking-wide">Chave Pública Anon (apikey)</label>
                     <input 
                       name="supabase_key"
                       type="password"
                       required
                       defaultValue={config?.supabase_key || ''}
                       placeholder="Ache no painel do Supabase (Anon Key)"
                       className="w-full border-2 border-slate-200 rounded-xl px-4 py-3 text-sm font-medium focus:ring-4 focus:ring-primary-100 focus:border-primary-500 outline-none transition-all"
                     />
                   </div>
                   
                   <div className="flex flex-col gap-1.5">
                     <label className="text-xs font-bold text-slate-500 uppercase tracking-wide">Token do Agente Oculto (x-api-key)</label>
                     <input 
                       name="token"
                       type="password"
                       required
                       autoComplete="off"
                       placeholder="Token gerado no painel do restaurante..."
                       className="w-full border-2 border-slate-200 rounded-xl px-4 py-3 text-sm font-medium focus:ring-4 focus:ring-primary-100 focus:border-primary-500 outline-none transition-all"
                     />
                   </div>
                   
                   <button 
                     type="submit"
                     className="mt-2 bg-slate-900 hover:bg-slate-800 text-white rounded-xl px-6 py-4 font-bold shadow-lg shadow-slate-900/20 active:scale-[0.98] transition-all">
                     Vincular Tablet
                   </button>
                 </form>
               ) : (
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="bg-blue-50 border-2 border-blue-100 p-6 rounded-2xl flex items-center justify-between">
                      <div>
                        <p className="text-blue-500 text-xs font-bold uppercase tracking-widest mb-1">Status de Fila</p>
                        <p className="text-blue-900 font-bold text-lg">Pronto para imprimir</p>
                      </div>
                      <div className="bg-blue-200/50 p-3 rounded-full text-2xl">
                        📃
                      </div>
                    </div>
                  </div>
               )}
             </>
           ) : (
              // TELA DE CONFIGURAÇÃO DAS IMPRESSORAS (TABLET)
              <div className="flex flex-col gap-4">
                  <h2 className="text-2xl font-bold text-slate-800">Canais de Impressão (Físicas)</h2>
                  <p className="text-slate-500 text-sm font-medium mb-4">
                    Como tablets Android não possuem cabo USB/Portas COM como nos computadores, aqui nós configuramos suas térmicas via Conexão de Rede (IP) ou Pareamento Bluetooth Oficial. 
                  </p>
                  
                  <div className="flex flex-col gap-3">
                     {printersList.map((p, idx) => (
                        <div key={idx} className="flex justify-between items-center p-4 border-2 border-slate-100 rounded-xl bg-slate-50">
                           <div className="flex flex-col">
                              <span className="font-bold text-slate-700">{p.name}</span>
                              <span className="text-sm font-mono text-slate-500 mt-1">Conexão: <strong className="text-slate-700">{p.type.toUpperCase()} / {p.address}</strong></span>
                           </div>
                           <button 
                             onClick={() => {
                               if(window.confirm(`Deseja realmente remover a impressora "${p.name}"?`)) {
                                 const newList = printersList.filter((_, i) => i !== idx)
                                 setPrintersList(newList)
                                 localStorage.setItem('agent_printers', JSON.stringify(newList))
                               }
                             }}
                             className="text-slate-400 hover:text-red-500 p-2 transition-colors">
                             ✕
                           </button>
                        </div>
                     ))}
                  </div>

                  <hr className="my-2 border-slate-100" />
                  
                  <form 
                    onSubmit={(e) => {
                      e.preventDefault()
                      const f = new FormData(e.currentTarget)
                      const name = (f.get('printerName') as string).trim();
                      const address = (f.get('printerAddress') as string).trim();
                      const type = f.get('printerType') as 'network'|'bluetooth';

                      if (name.length < 3) {
                        alert("O nome da impressora deve ter pelo menos 3 caracteres.");
                        return;
                      }

                      if (address.length < 5) {
                        alert("O endereço (IP ou MAC) parece inválido.");
                        return;
                      }

                      const newPrinter = { name, type, address }
                      const newList = [...printersList, newPrinter]
                      setPrintersList(newList)
                      localStorage.setItem('agent_printers', JSON.stringify(newList))
                      e.currentTarget.reset()
                      alert(`Impressora "${name}" adicionada com sucesso!`);
                    }}
                    className="bg-white border-2 border-slate-200 rounded-2xl p-5 flex flex-col gap-4">
                     <h3 className="font-bold text-slate-700 mb-2">Adicionar Novo Setor</h3>
                     <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                        <select name="printerType" className="border-2 border-slate-200 rounded-lg px-3 py-2 text-sm font-bold text-slate-700 outline-none">
                           <option value="network">Rede (IP)</option>
                           <option value="bluetooth">Bluetooth</option>
                        </select>
                        <input name="printerName" required type="text" placeholder="Nome Ex: Cozinha" className="border-2 border-slate-200 py-2 px-3 rounded-lg text-sm" />
                        <input name="printerAddress" required type="text" placeholder="IP ou MAC" className="border-2 border-slate-200 py-2 px-3 rounded-lg font-mono text-sm" />
                     </div>
                     <button type="submit" className="bg-blue-600 text-white font-bold py-3 rounded-xl mt-2 hover:bg-blue-700 active:scale-95 transition-all">
                        Salvar e Vincular
                     </button>
                  </form>
              </div>
           )}

        </div>
      </main>
    </div>
  )
}

export default App
