// Serviço de Polling e Impressão para Tablet via Supabase
// Reproduz o fluxo original do Rust (agent-unified-poll / print-job-status)
import { ThermalPrinter, PrinterType } from 'capacitor-thermal-printer';
import { EscPosFormatter } from './escPosFormatter';

export interface PrintJob {
  id: string;
  order_id?: string;
  printer_type?: string;
  printer_id?: string;
  content: any;
  copies?: number;
  job_type?: string;
  pickup_code?: string;
  created_at: string;
}

export interface PrinterConfig {
  name: string;
  type: 'network' | 'bluetooth';
  address: string;
}

export interface PollResponse {
  print_jobs: PrintJob[];
  scale_requests: any[];
  commands: any[];
  config: any;
  server_time: string;
}

class PollingService {
  private isRunning: boolean = false;
  private intervalId: any = null;
  private api_url: string = '';
  private token: string = '';
  private supabase_key: string = '';
  private printers: PrinterConfig[] = [];
  private formatter = new EscPosFormatter(32); // Largura padrão de 58mm para teste

  setConfig(api_url: string, token: string, printers: PrinterConfig[], supabase_key?: string) {
    this.api_url = api_url;
    this.token = token;
    this.printers = printers;
    this.supabase_key = supabase_key || '';
  }

  start(onPollSuccess: (time: string, config: any) => void, onError: (err: string) => void) {
    if (this.isRunning) return;
    this.isRunning = true;

    // Loop a cada 5 segundos
    this.intervalId = setInterval(async () => {
      if (!this.token) return;

      try {
        const url = `${this.api_url}/agent-unified-poll`;
        
        const headers: any = {
          'x-api-key': this.token,
          'Content-Type': 'application/json'
        };

        // Se tivermos uma chave do Supabase (Anon Key), adicionamos os headers necessários
        if (this.supabase_key) {
          headers['apikey'] = this.supabase_key;
          headers['Authorization'] = `Bearer ${this.supabase_key}`;
        }

        const response = await fetch(url, {
          method: 'GET',
          headers: headers
        });

        if (!response.ok) {
          throw new Error(`HTTP Error: ${response.status}`);
        }

        const data: PollResponse = await response.json();
        
        onPollSuccess(data.server_time || new Date().toLocaleTimeString(), data.config);

        // Processa Jobs de Impressão
        for (const job of data.print_jobs || []) {
          this.processPrintJob(job);
        }

        // Processa Comandos (ex: Abrir Gaveta, Balança)
        for (const cmd of data.commands || []) {
           console.log(`[Comando Solicitado]: ${cmd.command_type}`, cmd);
        }

        // Processa Balança Requests
        for (const scale of data.scale_requests || []) {
           if(scale.status === 'pending' || scale.status === 'reading') {
              this.processScaleRequest(scale);
           }
        }

      } catch (err: any) {
        onError(err.message || 'Erro desconhecido');
      }
    }, 5000);
  }

  stop() {
    this.isRunning = false;
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  private async processPrintJob(job: PrintJob) {
    console.log(`Iniciando trabalho de impressão: ${job.id}`);
    
    try {
      const targetPrinter = this.printers.find(p => p.address === job.printer_id) || this.printers[0];

      if (!targetPrinter) {
        throw new Error('Nenhuma impressora configurada no tablet.');
      }

      console.log(`Enviando para: ${targetPrinter.name} (${targetPrinter.address})`);

      // 1. Conecta Bluetooth/Network via Capacitor
      try {
        await ThermalPrinter.connect({
          type: targetPrinter.type === 'network' ? PrinterType.NETWORK : PrinterType.BLUETOOTH,
          address: targetPrinter.address,
          port: targetPrinter.type === 'network' ? 9100 : undefined
        });
        console.log("Conectado à impressora!");
      } catch (connErr) {
        console.error("Erro ao conectar na impressora:", connErr);
        throw new Error(`Falha na conexão com ${targetPrinter.name}: ${targetPrinter.address}`);
      }

      // 2. Formata o cupom
      const formattedText = this.formatter.formatReceipt(job.content, job.job_type || "RECIBO");

      // 3. Dispara a escrita real
      await ThermalPrinter.print({
        content: formattedText
      });

      // 4. Desconecta para liberar a fila para outros dispositivos
      await ThermalPrinter.disconnect();
      
      console.log(`Job ${job.id} impresso com sucesso real em: ${targetPrinter.name}`);
      await this.reportJobStatus(job.id, 'printed');

    } catch (e: any) {
      console.error(`Falha no Job ${job.id}`, e);
      await this.reportJobStatus(job.id, 'failed', e.message);
      
      // Tenta desconectar por segurança
      try { await ThermalPrinter.disconnect(); } catch {}
    }
  }

  private async reportJobStatus(job_id: string, status: string, error_message?: string) {
    try {
      const url = `${this.api_url}/print-job-status`;
      
      const headers: any = {
        'x-api-key': this.token,
        'Content-Type': 'application/json'
      };

      if (this.supabase_key) {
        headers['apikey'] = this.supabase_key;
        headers['Authorization'] = `Bearer ${this.supabase_key}`;
      }

      await fetch(url, {
        method: 'POST',
        headers: headers,
        body: JSON.stringify({
          job_id,
          status,
          error_message: error_message || null
        })
      });
    } catch (e) {
      console.error('Falha ao reportar status ao Supabase:', e);
    }
  }

  private async processScaleRequest(scaleReq: any) {
    console.log(`Processando solicitação de balança: ${scaleReq.id}`);
    
    try {
      // No Tablet Android, a leitura física de balança Serial/USB exige plugins específicos.
      // Por enquanto, simulamos uma leitura ou preparamos o gancho para o plugin.
      // O importante é que o FLUXO de volta para o Supabase esteja correto.
      
      const simulatedWeight = 1250; // 1.25kg para teste
      
      console.log(`Peso identificado: ${simulatedWeight}g. Enviando para o sistema principal...`);
      await this.reportScaleWeight(scaleReq.id, simulatedWeight);

    } catch (e: any) {
      console.error(`Falha ao ler balança ${scaleReq.id}`, e);
      await this.reportScaleWeight(scaleReq.id, 0, e.message);
    }
  }

  private async reportScaleWeight(request_id: string, weight: number, error_message?: string) {
    try {
      const url = `${this.api_url}/scale-weight-receive`;
      
      const headers: any = {
        'x-api-key': this.token,
        'Content-Type': 'application/json'
      };

      if (this.supabase_key) {
        headers['apikey'] = this.supabase_key;
        headers['Authorization'] = `Bearer ${this.supabase_key}`;
      }

      const body = error_message ? {
        request_id,
        peso: 0,
        unidade: 'g',
        status: 'error',
        error_message
      } : {
        request_id,
        peso: weight / 1000.0, // Converte gramas para kg conforme padrão do sistema
        unidade: 'kg'
      };

      await fetch(url, {
        method: 'POST',
        headers: headers,
        body: JSON.stringify(body)
      });
      
      console.log(`Sucesso: Peso enviado para o sistema principal.`);
    } catch (e) {
      console.error('Erro ao enviar peso para o Supabase:', e);
    }
  }
}

export const agentPollingService = new PollingService();
