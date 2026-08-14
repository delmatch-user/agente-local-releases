# -*- mode: python ; coding: utf-8 -*-


a = Analysis(
    ['agente_local.py'],
    pathex=[],
    binaries=[],
    datas=[],
    hiddenimports=['win32api', 'win32print', 'serial.tools.list_ports', 'websockets', 'pynput', 'usb', 'pystray', 'PIL', 'PIL.Image', 'PIL.ImageDraw'],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name='AgenteLocal',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    upx_exclude=[],
    # Pasta persistente em vez de %TEMP%/_MEI* — evita erro
    # "base_library.zip not found" quando antivirus/limpador apaga o Temp em runtime.
    # FOI A PERDA DESTA LINHA (spec regenerado pelo build.py na v5.76) que causou o
    # "abre e fecha sozinho" em cliente real. NAO deixe o PyInstaller CLI sobrescrever
    # este spec: builde com `python -m PyInstaller AgenteLocal.spec`.
    runtime_tmpdir='%LOCALAPPDATA%/AgenteLocalMIA/runtime',
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
