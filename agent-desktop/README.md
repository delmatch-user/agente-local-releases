# MiaCardapio Agent - Agente Local de Impressão

Aplicação desktop para gerenciar impressoras térmicas, balanças e gavetas de dinheiro conectadas ao MiaCardapio.

## 🚀 Tecnologias

- **Tauri** - Framework para apps desktop (Rust + Web)
- **Rust** - Backend nativo para acesso a hardware
- **React + TypeScript** - Interface do usuário
- **ESC/POS** - Protocolo de impressoras térmicas

## 📋 Pré-requisitos

### Windows
```bash
# Instalar Rust
winget install Rustlang.Rust

# Instalar Node.js
winget install OpenJS.NodeJS.LTS

# Instalar Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools
```

### macOS
```bash
# Instalar Xcode Command Line Tools
xcode-select --install

# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Instalar Node.js
brew install node
```

### Linux (Ubuntu/Debian)
```bash
# Dependências do sistema
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libudev-dev

# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Instalar Node.js
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt install -y nodejs
```

## 🛠️ Instalação

```bash
# Clonar ou extrair o projeto
cd miacardapio-agent

# Instalar dependências Node.js
npm install

# Instalar Tauri CLI
npm install -g @tauri-apps/cli
```

## 🏃 Desenvolvimento

```bash
# Rodar em modo desenvolvimento
npm run tauri dev
```

## 📦 Build para Produção

```bash
# Windows (.exe + .msi)
npm run tauri build

# macOS (.dmg + .app)
npm run tauri build

# Linux (.deb + .AppImage)
npm run tauri build
```

Os instaladores serão gerados em `src-tauri/target/release/bundle/`

## ⚙️ Configuração

1. Abra o agente
2. Na tela inicial, cole o **Token do Agente** gerado no painel MiaCardapio
3. Configure as impressoras conectadas (USB ou Rede)
4. O agente começará a receber jobs automaticamente

## 🖨️ Impressoras Suportadas

- Impressoras térmicas ESC/POS (58mm e 80mm)
- Conexão USB (COM port)
- Conexão de Rede (TCP/IP)
- Modelos testados:
  - Epson TM-T20
  - Elgin i9
  - Bematech MP-4200 TH
  - Tanca TP-650

## ⚖️ Balanças Suportadas

- Protocolo Toledo
- Conexão Serial (RS-232)
- Modelos testados:
  - Toledo Prix 3
  - Filizola Platina

## 💰 Gavetas de Dinheiro

- Acionamento via impressora (pulso RJ11)
- Comando ESC/POS padrão

## 📡 API Endpoints

O agente se comunica com os seguintes endpoints:

| Endpoint | Método | Descrição |
|----------|--------|-----------|
| `/functions/v1/agent-unified-poll` | GET | Busca jobs e comandos pendentes |
| `/functions/v1/agent-heartbeat` | POST | Envia status de saúde |
| `/functions/v1/print-job-status` | POST | Reporta status de impressão |

## 🔐 Segurança

- Token único por restaurante
- Comunicação via HTTPS
- Dados armazenados localmente em SQLite criptografado
- Sem acesso externo ao hardware local

## 📝 Logs

Logs são salvos em:
- **Windows**: `%APPDATA%\miacardapio-agent\logs\`
- **macOS**: `~/Library/Application Support/miacardapio-agent/logs/`
- **Linux**: `~/.config/miacardapio-agent/logs/`

## 🆘 Suporte

Em caso de problemas:
1. Verifique os logs do agente
2. Confirme que o token está correto
3. Teste a conexão da impressora manualmente
4. Entre em contato pelo painel MiaCardapio

## 📄 Licença

Proprietário - MiaCardapio © 2024
