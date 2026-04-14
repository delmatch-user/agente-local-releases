@echo off
echo =======================================================
echo     GERADOR AUTOMATICO DE APK - MIACARDAPIO AGENT 
echo =======================================================
echo.
echo [1] Sincronizando arquivos Web e Javascript...
call npm run build
call npx cap sync android

echo.
echo [2] Compilando arquivo Android (.apk) silenciosamente...
cd android
call gradlew.bat assembleDebug
cd ..

echo.
echo [3] Copiando o APK Instalavel para sua area de trabalho (Agente Local)...
copy "android\app\build\outputs\apk\debug\app-debug.apk" "..\miacardapio-agent-tablet.apk"

echo.
echo =======================================================
echo PRONTO! O arquivo miacardapio-agent-tablet.apk foi gerado
echo e colocado ao lado desta pasta! 
echo.
echo Para instalar no seu cliente basta enviar esse arquivo
echo para o tablet via Google Drive, WhatsApp ou Cabo USB.
echo =======================================================
pause
