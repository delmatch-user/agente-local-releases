"""
Stub Installer - Agente Local MiaCardapio
Baixa e instala a versao mais recente do Agente Local automaticamente.

Para compilar:
    pyinstaller --onefile --windowed --name InstalarAgente --icon=icone.ico stub_installer.py

O .exe gerado em dist/InstalarAgente.exe pode ser distribuido para clientes.
Ao executar, ele baixa a versao mais recente e instala sem intervencao manual.
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


class InstallerUI:
    def __init__(self):
        self.root = tk.Tk()
        self.root.title(f"Instalador - {APP_NAME}")
        self.root.geometry("480x260")
        self.root.resizable(False, False)
        self.root.configure(bg="#1a1a2e")
        self._center_window()

        tk.Label(self.root, text="🍽️", font=("Segoe UI Emoji", 32), bg="#1a1a2e", fg="white").pack(pady=(20, 4))
        tk.Label(self.root, text=APP_NAME, font=("Segoe UI", 14, "bold"), bg="#1a1a2e", fg="white").pack()

        self.status_var = tk.StringVar(value="Preparando instalação...")
        tk.Label(self.root, textvariable=self.status_var, font=("Segoe UI", 10),
                 bg="#1a1a2e", fg="#94a3b8", wraplength=420).pack(pady=(16, 8))

        self.progress = ttk.Progressbar(self.root, length=400, mode="indeterminate")
        self.progress.pack(pady=4)
        self.progress.start(10)

        self.detail_var = tk.StringVar(value="")
        tk.Label(self.root, textvariable=self.detail_var, font=("Segoe UI", 9),
                 bg="#1a1a2e", fg="#64748b").pack(pady=(4, 0))

        self.root.after(300, self._start_install)

    def _center_window(self):
        self.root.update_idletasks()
        w, h = 480, 260
        x = (self.root.winfo_screenwidth() - w) // 2
        y = (self.root.winfo_screenheight() - h) // 2
        self.root.geometry(f"{w}x{h}+{x}+{y}")

    def set_status(self, msg, detail=""):
        self.status_var.set(msg)
        self.detail_var.set(detail)
        self.root.update_idletasks()

    def set_done(self, msg):
        self.progress.stop()
        self.progress.configure(mode="determinate", value=100)
        self.status_var.set(msg)
        self.root.after(2500, self.root.destroy)

    def set_error(self, msg):
        self.progress.stop()
        self.status_var.set("Erro na instalação")
        self.detail_var.set(msg)
        tk.Button(self.root, text="Fechar", command=self.root.destroy,
                  bg="#ef4444", fg="white", font=("Segoe UI", 10),
                  relief="flat", padx=16, pady=6).pack(pady=12)

    def _start_install(self):
        Thread(target=self._run_install, daemon=True).start()

    def _run_install(self):
        try:
            # 1. Buscar versao mais recente
            self.root.after(0, self.set_status, "Verificando versão mais recente...", VERSION_URL)
            req = urllib.request.Request(VERSION_URL, headers={"Cache-Control": "no-cache"})
            with urllib.request.urlopen(req, timeout=15) as r:
                info = json.loads(r.read())
            url = info["url"]
            versao = info["version"]

            # 2. Baixar exe
            self.root.after(0, self.set_status,
                            f"Baixando Agente Local v{versao}...",
                            "Isso pode levar alguns minutos")
            tmp_dir = Path(tempfile.mkdtemp())
            tmp_exe = tmp_dir / EXE_NAME

            def progress_hook(count, block_size, total_size):
                if total_size > 0:
                    pct = int(count * block_size * 100 / total_size)
                    self.root.after(0, self.detail_var.set,
                                    f"{min(pct,100)}% de {total_size // 1024 // 1024} MB")

            urllib.request.urlretrieve(url, tmp_exe, reporthook=progress_hook)

            # 3. Instalar
            self.root.after(0, self.set_status, "Instalando...", str(INSTALL_DIR))
            INSTALL_DIR.mkdir(parents=True, exist_ok=True)
            destino = INSTALL_DIR / EXE_NAME
            shutil.copy2(tmp_exe, destino)

            # 4. Atalho na area de trabalho
            self.root.after(0, self.set_status, "Criando atalho...", "")
            _criar_atalho(destino, Path.home() / "Desktop" / f"{APP_NAME}.lnk")

            # 5. Registro de startup
            try:
                with winreg.OpenKey(winreg.HKEY_CURRENT_USER, REG_KEY, 0,
                                    winreg.KEY_SET_VALUE) as k:
                    winreg.SetValueEx(k, REG_VALUE, 0, winreg.REG_SZ, str(destino))
            except Exception:
                pass  # Startup e opcional

            # 6. Lanca o agente
            self.root.after(0, self.set_status, "Iniciando o agente...", "")
            subprocess.Popen([str(destino)], creationflags=subprocess.DETACHED_PROCESS)

            self.root.after(0, self.set_done,
                            f"Instalacao concluida! v{versao}\nO Agente Local esta sendo iniciado.")

        except Exception as e:
            self.root.after(0, self.set_error, str(e))

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
        # Atalho e opcional, falha silenciosa
        pass


if __name__ == "__main__":
    InstallerUI().run()
