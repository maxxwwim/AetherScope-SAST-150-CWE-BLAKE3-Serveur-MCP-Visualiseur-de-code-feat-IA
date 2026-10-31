@echo off
setlocal enabledelayedexpansion
title AetherScope V2 - Control Center
chcp 65001 >nul

set "SCRIPT_DIR=%~dp0"
set "ENGINE_EXE="
set "MCP_EXE="
set "MC_RUNNER_EXE="

if exist "%LOCALAPPDATA%\AetherScope\bin\aether-engine.exe" set "ENGINE_EXE=%LOCALAPPDATA%\AetherScope\bin\aether-engine.exe"
if exist "%SCRIPT_DIR%target-build\release\aether-engine.exe" set "ENGINE_EXE=%SCRIPT_DIR%target-build\release\aether-engine.exe"
if exist "%SCRIPT_DIR%target\release\aether-engine.exe" set "ENGINE_EXE=%SCRIPT_DIR%target\release\aether-engine.exe"
if exist "%SCRIPT_DIR%aether-engine.exe" set "ENGINE_EXE=%SCRIPT_DIR%aether-engine.exe"

if exist "%LOCALAPPDATA%\AetherScope\bin\aetherscope-mcp.exe" set "MCP_EXE=%LOCALAPPDATA%\AetherScope\bin\aetherscope-mcp.exe"
if exist "%SCRIPT_DIR%target-build\release\aetherscope-mcp.exe" set "MCP_EXE=%SCRIPT_DIR%target-build\release\aetherscope-mcp.exe"
if exist "%SCRIPT_DIR%target\release\aetherscope-mcp.exe" set "MCP_EXE=%SCRIPT_DIR%target\release\aetherscope-mcp.exe"
if exist "%SCRIPT_DIR%aetherscope-mcp.exe" set "MCP_EXE=%SCRIPT_DIR%aetherscope-mcp.exe"

if exist "%LOCALAPPDATA%\AetherScope\bin\aether-mc-runner.exe" set "MC_RUNNER_EXE=%LOCALAPPDATA%\AetherScope\bin\aether-mc-runner.exe"
if exist "%SCRIPT_DIR%target\release\aether-mc-runner.exe" set "MC_RUNNER_EXE=%SCRIPT_DIR%target\release\aether-mc-runner.exe"
if exist "%SCRIPT_DIR%aether-mc-runner.exe" set "MC_RUNNER_EXE=%SCRIPT_DIR%aether-mc-runner.exe"

if not defined ENGINE_EXE (
    echo [ERREUR] Impossible de trouver aether-engine.exe !
    echo Veuillez compiler le projet via cargo build --release ou executer install.ps1.
    pause
    exit /b 1
)

:MENU
cls
echo ==============================================================
echo       🪐 AETHERSCOPE V2 — COCKPIT SPATIAL ET MCP SUITE
echo ==============================================================
echo.
echo   [1] 🚀 Lancer le Cockpit Spatial Complet (GUI 3D + Serveur Axum + MCP)
echo   [2] 🤖 Lancer le Serveur MCP Stdio autonome (aetherscope-mcp.exe)
echo   [3] 🌐 Lancer le Serveur en Arriere-Plan (Mode Headless Axum)
echo   [4] 🔌 Configurer les Clients IA (Claude / Antigravity / Cursor)
echo   [5] 🎯 Scanner et Synchroniser un Projet (SAST Live Synchro GUI)
echo   [6] ⛏️ Lancer Aether Minecraft Cockpit (Web Flask + Runner)
echo   [7] 🧹 Liberer le port 3030 (Fermer tous les processus Aether)
echo   [8] 📦 Installer AetherScope dans Windows (PATH + Bureau)
echo   [9] ❌ Quitter
echo.
echo ==============================================================
set /p "CHOICE=Choisissez une option (1-9) [defaut: 1] : "

if "%CHOICE%"=="" set "CHOICE=1"
if "%CHOICE%"=="1" goto LAUNCH_GUI
if "%CHOICE%"=="2" goto LAUNCH_MCP
if "%CHOICE%"=="3" goto LAUNCH_HEADLESS
if "%CHOICE%"=="4" goto CONFIG_MCP
if "%CHOICE%"=="5" goto SCAN_SAST
if "%CHOICE%"=="6" goto LAUNCH_MC
if "%CHOICE%"=="7" goto KILL_PORT
if "%CHOICE%"=="8" goto RUN_INSTALL
if "%CHOICE%"=="9" goto QUIT

echo Option invalide.
timeout /t 2 >nul
goto MENU

:LAUNCH_GUI
cls
echo [INFO] Verification du port 3030...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3030 ^| findstr LISTENING') do (
    set "PORT_PID=%%a"
    if defined PORT_PID (
        echo [ATTENTION] Le port 3030 est deja occupe par le processus PID !PORT_PID!.
        echo Fermeture du processus zombie pour eviter les erreurs de connexion...
        taskkill /PID !PORT_PID! /F >nul 2>&1
    )
)
echo [INFO] Demarrage du Cockpit AetherScope V2...
start "" "%ENGINE_EXE%"

if defined MCP_EXE (
    echo [INFO] Demarrage de la passerelle MCP AetherScope...
    start "AetherScope MCP Server" /min "%MCP_EXE%" --port 3030
)

echo [OK] Cockpit et passerelle MCP actives et synchronises !
timeout /t 2 >nul
exit /b 0

:LAUNCH_MCP
cls
echo ==============================================================
echo       🤖 AETHERSCOPE V2 — SERVEUR MCP STDIO DEDIE
echo ==============================================================
echo.
echo Le serveur MCP communique en JSON-RPC 2.0 sur stdio.
echo Port de synchronisation HTTP Cockpit : 3030
echo.
if not defined MCP_EXE (
    echo [ERREUR] Impossible de trouver aetherscope-mcp.exe !
    pause
    goto MENU
)
echo Demarrage du serveur MCP (Ctrl+C pour arreter)...
"%MCP_EXE%" --port 3030
pause
goto MENU

:LAUNCH_HEADLESS
cls
echo [INFO] Lancement du serveur Axum + FileWatcher sans interface graphique...
echo Le serveur restera actif sur http://127.0.0.1:3030
"%ENGINE_EXE%" --no-gui
pause
goto MENU

:CONFIG_MCP
cls
echo [INFO] Execution du configurateur MCP...
powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%setup-mcp.ps1"
echo.
pause
goto MENU

:SCAN_SAST
cls
set /p "SCAN_PATH=Chemin du projet a scanner [defaut: .] : "
if "%SCAN_PATH%"=="" set "SCAN_PATH=."
echo [INFO] Audit SAST et Synchronisation en direct en cours sur %SCAN_PATH%...
"%ENGINE_EXE%" --path "%SCAN_PATH%" --audit-only
echo.
echo [OK] Rapport disponible dans : %SCAN_PATH%\.aether\LIVE_AUDIT.md
pause
goto MENU

:LAUNCH_MC
cls
echo [INFO] Lancement du Control-Plane Minecraft Cockpit (http://127.0.0.1:5000)...
start http://127.0.0.1:5000
python "%SCRIPT_DIR%services\mc-control-plane\app.py"
pause
goto MENU

:KILL_PORT
cls
echo [INFO] Arret des processus aether-engine et aetherscope-mcp...
taskkill /F /IM aether-engine.exe >nul 2>&1
taskkill /F /IM aetherscope-mcp.exe >nul 2>&1
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3030 ^| findstr LISTENING') do (
    set "PORT_PID=%%a"
    if defined PORT_PID (
        taskkill /PID !PORT_PID! /F >nul 2>&1
    )
)
echo [OK] Tous les processus AetherScope ont ete arretes et le port 3030 est libere !
timeout /t 2 >nul
goto MENU

:RUN_INSTALL
cls
echo [INFO] Lancement de l'installateur Windows...
powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%install.ps1"
echo.
pause
goto MENU

:QUIT
exit /b 0
