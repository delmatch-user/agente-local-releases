@echo off
echo ============================================
echo  Build: Stub Installer - Agente Local
echo ============================================
echo.

:: Instala dependencias necessarias
echo [1/3] Instalando dependencias...
pip install pyinstaller pywin32 pillow --quiet
if errorlevel 1 (
    echo ERRO: Falha ao instalar dependencias.
    pause
    exit /b 1
)

:: Muda para a pasta do installer
cd /d "%~dp0"

:: Compila o stub installer
echo [2/3] Compilando InstalarAgente.exe...
pyinstaller ^
    --onefile ^
    --windowed ^
    --name InstalarAgente ^
    --add-data "icone.ico;." ^
    --icon=icone.ico ^
    stub_installer.py
if errorlevel 1 (
    echo ERRO: Falha na compilacao. Tentando sem icone...
    pyinstaller --onefile --windowed --name InstalarAgente stub_installer.py
    if errorlevel 1 (
        echo ERRO: Compilacao falhou.
        pause
        exit /b 1
    )
)

echo [3/3] Limpando arquivos temporarios...
if exist build rmdir /s /q build
if exist InstalarAgente.spec del InstalarAgente.spec

echo.
echo ============================================
echo  SUCESSO!
echo  Installer gerado em: dist\InstalarAgente.exe
echo  Distribua esse arquivo para os clientes.
echo ============================================
echo.
pause
