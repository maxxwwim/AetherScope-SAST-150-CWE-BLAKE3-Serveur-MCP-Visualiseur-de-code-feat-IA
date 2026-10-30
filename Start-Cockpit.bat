@echo off
setlocal enabledelayedexpansion
title AetherScope V2 - Lancement du Cockpit
chcp 65001 >nul

set "SCRIPT_DIR=%~dp0"
set "ENGINE_EXE="

if exist "%SCRIPT_DIR%aether-engine.exe" (
    set "ENGINE_EXE=%SCRIPT_DIR%aether-engine.exe"
) else if exist "%SCRIPT_DIR%target\release\aether-engine.exe" (
    set "ENGINE_EXE=%SCRIPT_DIR%target\release\aether-engine.exe"
) else if exist "%SCRIPT_DIR%target-build\release\aether-engine.exe" (
    set "ENGINE_EXE=%SCRIPT_DIR%target-build\release\aether-engine.exe"
) else if exist "%LOCALAPPDATA%\AetherScope\bin\aether-engine.exe" (
    set "ENGINE_EXE=%LOCALAPPDATA%\AetherScope\bin\aether-engine.exe"
)

if not defined ENGINE_EXE (
    echo [ERREUR] aether-engine.exe est introuvable.
    pause
    exit /b 1
)

:: Nettoyage automatique des anciens processus fantomes sur le port 3030
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3030 ^| findstr LISTENING') do (
    set "PORT_PID=%%a"
    if defined PORT_PID (
        taskkill /PID !PORT_PID! /F >nul 2>&1
    )
)

:: Demarrage immediat
start "" "%ENGINE_EXE%"
exit /b 0
