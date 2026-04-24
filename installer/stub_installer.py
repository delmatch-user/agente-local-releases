"""
Instalador Hibrido - Agente Local MiaCardapio
Wizard que pergunta o modo de operacao antes de instalar.

Para compilar:
    python -m PyInstaller --onefile --windowed --name InstalarAgente stub_installer.py
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile
import tkinter as tk
import tkinter.ttk as ttk
import urllib.request
import winreg
from pathlib import Path
from threading import Thread

VERSION_URL = "https://raw.githubusercontent.com/delmatch-user/agente-local-releases/main/version.json"
INSTALL_DIR = Path(os.environ.get("LOCALAPPDATA", "C:/Users/Public")) / "AgenteLocal"
APP_NAME = "Agente Local MiaCardapio"
EXE_NAME = "AgenteLocal.exe"
REG_KEY = r"Software\Microsoft\Windows\CurrentVersion\Run"
REG_VALUE = "AgenteLocal"

BG = "#0f172a"
BG2 = "#1e293b"
ACCENT = "#6366f1"
ACCENT2 = "#4f46e5"
TEXT = "#f1f5f9"
TEXT2 = "#94a3b8"
GREEN = "#22c55e"
BTN_FG = "white"


class WizardApp:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title(f"Instalador - {APP_NAME}")
        self.root.resizable(False, False)
        self.root.configure(bg=BG)
        self.modo = None  # "rede_unica" ou "multi_rede"
        self._set_size(500, 420)
        self.show_welcome()

    def _set_size(self, w, h):
        self.root.update_idletasks()
        x = (self.root.winfo_screenwidth() - w) // 2
        y = (self.root.winfo_screenheight() - h) // 2
        self.root.geometry(f"{w}x{h}+{x}+{y}")

    def _clear(self):
        for widget in self.root.winfo_children():
            widget.destroy()

    def _header(self, emoji, titulo, subtitulo=""):
        tk.Label(self.root, text=emoji, font=("Segoe UI Emoji", 36),
                 bg=BG, fg=TEXT).pack(pady=(28, 4))
        tk.Label(self.root, text=titulo, font=("Segoe UI", 15, "bold"),
                 bg=BG, fg=TEXT).pack()
        if subtitulo:
            tk.Label(self.root, text=subtitulo, font=("Segoe UI", 10),
                     bg=BG, fg=TEXT2, wraplength=440).pack(pady=(4, 0))

    def _btn(self, parent, text, cmd, color=ACCENT, fg=BTN_FG, width=36):
        b = tk.Button(parent, text=text, command=cmd,
                      bg=color, fg=fg, font=("Segoe UI", 10, "bold"),
                      relief="flat", bd=0, cursor="hand2",
                      activebackground=ACCENT2, activeforeground=BTN_FG,
                      padx=14, pady=10, width=width)
        b.pack(pady=5, padx=30, fill="x")
        return b

    # ── Tela 1: escolha do modo ───────────────────────────────────────────────
    def show_welcome(self):
        self._clear()
        self._set_size(500, 430)
        self._header("🍽️", APP_NAME, "Bem-vindo ao instalador!\nComo este agente vai funcionar?")

        sep = tk.Frame(self.root, bg=BG2, height=1)
        sep.pack(fill="x", padx=30, pady=16)

        # Card rede única
        card1 = tk.Frame(self.root, bg=BG2, bd=0, relief="flat")
        card1.pack(fill="x", padx=30, pady=(0, 8))
        tk.Label(card1, text="🖥️  Rede Única", font=("Segoe UI", 11, "bold"),
                 bg=BG2, fg=TEXT, anchor="w").pack(fill="x", padx=16, pady=(12, 2))
        tk.Label(card1, text="Um local, uma rede. Impressoras conectadas\nna mesma rede local (ex: restaurante, loja).",
                 font=("Segoe UI", 9), bg=BG2, fg=TEXT2, anchor="w", justify="left").pack(fill="x", padx=16, pady=(0, 4))
        tk.Button(card1, text="Selecionar →", command=self.show_single_net,
                  bg=ACCENT, fg=BTN_FG, font=("Segoe UI", 9, "bold"),
                  relief="flat", cursor="hand2", padx=12, pady=6,
                  activebackground=ACCENT2, activeforeground=BTN_FG).pack(anchor="e", padx=16, pady=(0, 12))

        # Card multi-rede
        card2 = tk.Frame(self.root, bg=BG2, bd=0, relief="flat")
        card2.pack(fill="x", padx=30, pady=(0, 8))
        tk.Label(card2, text="🌐  Múltiplas Redes", font=("Segoe UI", 11, "bold"),
                 bg=BG2, fg=TEXT, anchor="w").pack(fill="x", padx=16, pady=(12, 2))
        tk.Label(card2, text="Várias filiais ou redes diferentes.\nCada rede com suas próprias impressoras.",
                 font=("Segoe UI", 9), bg=BG2, fg=TEXT2, anchor="w", justify="left").pack(fill="x", padx=16, pady=(0, 4))
        tk.Button(card2, text="Selecionar →", command=self.show_multi_net,
                  bg="#0ea5e9", fg=BTN_FG, font=("Segoe UI", 9, "bold"),
                  relief="flat", cursor="hand2", padx=12, pady=6,
                  activebackground="#0284c7", activeforeground=BTN_FG).pack(anchor="e", padx=16, pady=(0, 12))

    # ── Tela 2A: Rede única ───────────────────────────────────────────────────
    def show_single_net(self):
        self.modo = "rede_unica"
        self._clear()
        self._set_size(500, 420)
        self._header("✅", "Modo: Rede Única")

        frame = tk.Frame(self.root, bg=BG2)
        frame.pack(fill="x", padx=30, pady=16)

        tk.Label(frame, text="O agente vai:", font=("Segoe UI", 10, "bold"),
                 bg=BG2, fg=TEXT, anchor="w").pack(fill="x", padx=16, pady=(12, 4))
        for item in [
            "• Receber pedidos direto do servidor",
            "• Imprimir na impressora configurada",
            "• Funcionar automaticamente em segundo plano",
            "• Iniciar junto com o Windows",
        ]:
            tk.Label(frame, text=item, font=("Segoe UI", 9),
                     bg=BG2, fg=TEXT2, anchor="w").pack(fill="x", padx=24, pady=1)

        tk.Label(frame, text="Após instalar você precisará:",
                 font=("Segoe UI", 10, "bold"), bg=BG2, fg=TEXT, anchor="w").pack(fill="x", padx=16, pady=(12, 4))
        for item in [
            "1.  Inserir seu Token de acesso (fornecido pelo suporte)",
            "2.  Selecionar sua impressora na lista",
            "3.  Clicar em Iniciar — pronto!",
        ]:
            tk.Label(frame, text=item, font=("Segoe UI", 9),
                     bg=BG2, fg=GREEN, anchor="w").pack(fill="x", padx=24, pady=1)

        tk.Label(frame, text="", bg=BG2).pack(pady=4)

        btn_frame = tk.Frame(self.root, bg=BG)
        btn_frame.pack(fill="x", padx=30)
        tk.Button(btn_frame, text="← Voltar", command=self.show_welcome,
                  bg=BG2, fg=TEXT2, font=("Segoe UI", 9), relief="flat",
                  cursor="hand2", padx=10, pady=8).pack(side="left")
        tk.Button(btn_frame, text="Instalar agora →", command=self.show_installing,
                  bg=GREEN, fg="black", font=("Segoe UI", 10, "bold"),
                  relief="flat", cursor="hand2", padx=16, pady=8,
                  activebackground="#16a34a").pack(side="right")

    # ── Tela 2B: Multi-rede ───────────────────────────────────────────────────
    def show_multi_net(self):
        self.modo = "multi_rede"
        self._clear()
        self._set_size(500, 440)
        self._header("🌐", "Modo: Múltiplas Redes")

        frame = tk.Frame(self.root, bg=BG2)
        frame.pack(fill="x", padx=30, pady=16)

        tk.Label(frame, text="O agente vai:", font=("Segoe UI", 10, "bold"),
                 bg=BG2, fg=TEXT, anchor="w").pack(fill="x", padx=16, pady=(12, 4))
        for item in [
            "• Gerenciar impressoras de várias redes ao mesmo tempo",
            "• Cada rede com suas próprias configurações de impressoras",
            "• Enviar cada job para a impressora certa automaticamente",
            "• Funcionar mesmo se uma das redes ficar offline",
        ]:
            tk.Label(frame, text=item, font=("Segoe UI", 9),
                     bg=BG2, fg=TEXT2, anchor="w").pack(fill="x", padx=24, pady=1)

        tk.Label(frame, text="Após instalar você precisará:",
                 font=("Segoe UI", 10, "bold"), bg=BG2, fg=TEXT, anchor="w").pack(fill="x", padx=16, pady=(12, 4))
        for item in [
            "1.  Inserir seu Token de acesso (fornecido pelo suporte)",
            "2.  Adicionar cada rede (nome + faixa de IP, ex: 192.168.1.x)",
            "3.  Mapear as impressoras de cada rede",
            "4.  Clicar em Iniciar — o agente cuida do resto!",
        ]:
            tk.Label(frame, text=item, font=("Segoe UI", 9),
                     bg=BG2, fg="#38bdf8", anchor="w").pack(fill="x", padx=24, pady=1)

        tk.Label(frame, text="", bg=BG2).pack(pady=4)

        btn_frame = tk.Frame(self.root, bg=BG)
        btn_frame.pack(fill="x", padx=30)
        tk.Button(btn_frame, text="← Voltar", command=self.show_welcome,
                  bg=BG2, fg=TEXT2, font=("Segoe UI", 9), relief="flat",
                  cursor="hand2", padx=10, pady=8).pack(side="left")
        tk.Button(btn_frame, text="Instalar agora →", command=self.show_installing,
                  bg="#0ea5e9", fg="white", font=("Segoe UI", 10, "bold"),
                  relief="flat", cursor="hand2", padx=16, pady=8,
                  activebackground="#0284c7").pack(side="right")

    # ── Tela 3: Instalando ────────────────────────────────────────────────────
    def show_installing(self):
        self._clear()
        self._set_size(500, 300)
        self._header("⬇️", "Instalando...", "Aguarde, isso pode levar alguns instantes.")

        self.status_var = tk.StringVar(value="Verificando versão mais recente...")
        tk.Label(self.root, textvariable=self.status_var, font=("Segoe UI", 10),
                 bg=BG, fg=TEXT2, wraplength=440).pack(pady=(20, 6))

        self.progress = ttk.Style()
        self.progress.theme_use("default")
        self.progress.configure("green.Horizontal.TProgressbar",
                                troughcolor=BG2, background=ACCENT, thickness=14)
        self.pbar = ttk.Progressbar(self.root, length=440, mode="indeterminate",
                                    style="green.Horizontal.TProgressbar")
        self.pbar.pack(pady=4)
        self.pbar.start(10)

        self.detail_var = tk.StringVar(value="")
        tk.Label(self.root, textvariable=self.detail_var, font=("Segoe UI", 9),
                 bg=BG, fg="#475569").pack(pady=(4, 0))

        Thread(target=self._run_install, daemon=True).start()

    def _set_status(self, msg, detail=""):
        self.status_var.set(msg)
        self.detail_var.set(detail)
        self.root.update_idletasks()

    def _run_install(self):
        try:
            self.root.after(0, self._set_status, "Verificando versão mais recente...", VERSION_URL)
            req = urllib.request.Request(VERSION_URL, headers={"Cache-Control": "no-cache"})
            with urllib.request.urlopen(req, timeout=15) as r:
                info = json.loads(r.read())
            url = info["url"]
            versao = info["version"]

            self.root.after(0, self._set_status,
                            f"Baixando Agente Local v{versao}...",
                            "Isso pode levar alguns minutos")
            tmp_dir = Path(tempfile.mkdtemp())
            tmp_exe = tmp_dir / EXE_NAME

            def progress_hook(count, block_size, total_size):
                if total_size > 0:
                    pct = min(int(count * block_size * 100 / total_size), 100)
                    mb = total_size // 1024 // 1024
                    self.root.after(0, self.detail_var.set, f"{pct}% de {mb} MB")

            urllib.request.urlretrieve(url, tmp_exe, reporthook=progress_hook)

            self.root.after(0, self._set_status, "Instalando...", str(INSTALL_DIR))
            INSTALL_DIR.mkdir(parents=True, exist_ok=True)
            destino = INSTALL_DIR / EXE_NAME
            shutil.copy2(tmp_exe, destino)

            # Cria config.json pré-configurado conforme modo escolhido
            self._criar_config(INSTALL_DIR)

            self.root.after(0, self._set_status, "Criando atalho...", "")
            _criar_atalho(destino,
                          Path.home() / "OneDrive" / "Desktop" / f"{APP_NAME}.lnk")
            _criar_atalho(destino,
                          Path.home() / "Desktop" / f"{APP_NAME}.lnk")

            try:
                with winreg.OpenKey(winreg.HKEY_CURRENT_USER, REG_KEY, 0,
                                    winreg.KEY_SET_VALUE) as k:
                    winreg.SetValueEx(k, REG_VALUE, 0, winreg.REG_SZ, str(destino))
            except Exception:
                pass

            self.root.after(0, self._set_status, "Iniciando o agente...", "")
            subprocess.Popen([str(destino)], creationflags=subprocess.DETACHED_PROCESS)

            self.root.after(0, self.show_done, versao)

        except Exception as e:
            self.root.after(0, self.show_error, str(e))

    def _criar_config(self, install_dir: Path):
        config_path = install_dir / "config.json"
        if config_path.exists():
            return  # Nao sobrescreve config existente
        if self.modo == "multi_rede":
            config = {
                "token": "",
                "poll_interval": 3,
                "modo": "multi_rede",
                "redes": [
                    {
                        "id": "rede_1",
                        "nome": "Rede Principal",
                        "impressoras": []
                    }
                ],
                "impressoras": []
            }
        else:
            config = {
                "token": "",
                "poll_interval": 3,
                "modo": "rede_unica",
                "impressoras": []
            }
        with open(config_path, "w", encoding="utf-8") as f:
            json.dump(config, f, ensure_ascii=False, indent=2)

    # ── Tela 4: Concluído ─────────────────────────────────────────────────────
    def show_done(self, versao):
        self._clear()
        self._set_size(500, 300)
        self._header("🎉", f"Instalação concluída! v{versao}")

        modo_txt = "Rede Única" if self.modo == "rede_unica" else "Múltiplas Redes"
        msg = (f"Modo configurado: {modo_txt}\n\n"
               "O Agente Local está sendo iniciado.\n"
               "Configure seu Token de acesso na janela do agente.")
        tk.Label(self.root, text=msg, font=("Segoe UI", 10),
                 bg=BG, fg=TEXT2, justify="center", wraplength=440).pack(pady=20)

        tk.Button(self.root, text="Fechar", command=self.root.destroy,
                  bg=GREEN, fg="black", font=("Segoe UI", 10, "bold"),
                  relief="flat", cursor="hand2", padx=20, pady=10,
                  activebackground="#16a34a").pack()

    # ── Tela de erro ─────────────────────────────────────────────────────────
    def show_error(self, msg):
        self._clear()
        self._set_size(500, 280)
        self._header("❌", "Erro na instalação")
        tk.Label(self.root, text=msg, font=("Segoe UI", 9),
                 bg=BG, fg="#f87171", wraplength=440, justify="center").pack(pady=16)
        tk.Label(self.root, text="Verifique sua conexão com a internet e tente novamente.",
                 font=("Segoe UI", 9), bg=BG, fg=TEXT2).pack()
        btn_frame = tk.Frame(self.root, bg=BG)
        btn_frame.pack(pady=20)
        tk.Button(btn_frame, text="Tentar novamente", command=self.show_welcome,
                  bg=ACCENT, fg=BTN_FG, font=("Segoe UI", 10, "bold"),
                  relief="flat", cursor="hand2", padx=14, pady=8).pack(side="left", padx=8)
        tk.Button(btn_frame, text="Fechar", command=self.root.destroy,
                  bg=BG2, fg=TEXT2, font=("Segoe UI", 10),
                  relief="flat", cursor="hand2", padx=14, pady=8).pack(side="left", padx=8)

    def run(self):
        self.root.mainloop()


def _criar_atalho(target: Path, shortcut_path: Path):
    try:
        import win32com.client  # type: ignore
        shell = win32com.client.Dispatch("WScript.Shell")
        sc = shell.CreateShortCut(str(shortcut_path))
        sc.Targetpath = str(target)
        sc.WorkingDirectory = str(target.parent)
        sc.Description = APP_NAME
        sc.save()
    except Exception:
        pass


if __name__ == "__main__":
    WizardApp().run()
