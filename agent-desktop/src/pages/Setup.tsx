import { useState } from 'react'

interface SetupProps {
  onTokenSet: (token: string, supabaseKey?: string) => void
}

export default function Setup({ onTokenSet }: SetupProps) {
  const [token, setToken] = useState('')
  const [supabaseKey, setSupabaseKey] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!token.trim()) {
      setError('Insira o token do agente')
      return
    }
    
    setLoading(true)
    setError(null)
    
    try {
      await onTokenSet(token.trim(), supabaseKey.trim())
    } catch (err) {
      setError('Falha ao configurar token. Verifique e tente novamente.')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="max-w-md mx-auto mt-8">
      <div className="text-center mb-8">
        <span className="text-6xl mb-4 block">🖨️</span>
        <h2 className="text-2xl font-bold text-slate-800">Configurar Agente</h2>
        <p className="text-slate-600 mt-2">
          Cole o token gerado no painel do MiaCardapio
        </p>
      </div>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label htmlFor="supabaseKey" className="block text-sm font-medium text-slate-700 mb-1">
            Chave Pública Anon (apikey)
          </label>
          <input
            id="supabaseKey"
            type="password"
            value={supabaseKey}
            onChange={(e) => setSupabaseKey(e.target.value)}
            placeholder="Ache no painel do Supabase (Anon Key)..."
            className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <div>
          <label htmlFor="token" className="block text-sm font-medium text-slate-700 mb-1">
            Token do Agente Oculto (x-api-key)
          </label>
          <textarea
            id="token"
            value={token}
            onChange={(e) => setToken(e.target.value)}
            placeholder="Token gerado no painel do restaurante..."
            className="w-full px-3 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500 resize-none h-24"
          />
        </div>

        {error && (
          <p className="text-red-600 text-sm">{error}</p>
        )}

        <button
          type="submit"
          disabled={loading}
          className="w-full py-3 px-4 bg-primary-500 text-white font-medium rounded-lg hover:bg-primary-600 focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {loading ? 'Conectando...' : 'Conectar'}
        </button>
      </form>

      <div className="mt-8 p-4 bg-slate-100 rounded-lg">
        <h3 className="font-medium text-slate-800 mb-2">Como obter o token?</h3>
        <ol className="text-sm text-slate-600 space-y-1 list-decimal list-inside">
          <li>Acesse o painel do MiaCardapio</li>
          <li>Vá em Integrações → Impressoras</li>
          <li>Clique na aba "Agente"</li>
          <li>Clique em "Gerar Novo Token"</li>
          <li>Copie e cole aqui</li>
        </ol>
      </div>
    </div>
  )
}
