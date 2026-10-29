# ==========================================================
# AetherScope V2 - Windows Installer (PowerShell)
# ==========================================================
param(
    [string]$InstallDir = (Join-Path $env:LOCALAPPDATA "AetherScope"),
    [switch]$SkipMcp,
    [switch]$SkipShortcut
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " INSTALLATEUR AETHERSCOPE V2 (WINDOWS X64) " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$binDir = Join-Path $InstallDir "bin"

# 1. Recherche des executables
$engineExe = $null
$mcpExe = $null

$candidates = @(
    $scriptDir,
    (Join-Path $scriptDir "target\release")
)

foreach ($c in $candidates) {
    $e = Join-Path $c "aether-engine.exe"
    $m = Join-Path $c "aetherscope-mcp.exe"
    if (Test-Path $e) { $engineExe = (Resolve-Path $e).Path }
    if (Test-Path $m) { $mcpExe = (Resolve-Path $m).Path }
    if ($engineExe -and $mcpExe) { break }
}

if (-not $engineExe) {
    Write-Host "[ERREUR] aether-engine.exe est introuvable." -ForegroundColor Red
    Write-Host "Veuillez compiler le projet via: cargo build --release" -ForegroundColor Yellow
    exit 1
}

Write-Host "[INFO] Dossier cible : $InstallDir" -ForegroundColor White
if (-not (Test-Path $binDir)) {
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
}

# 2. Copie des fichiers
Write-Host "[INFO] Copie des binaires vers $binDir..." -ForegroundColor DarkGray
Copy-Item -Path $engineExe -Destination (Join-Path $binDir "aether-engine.exe") -Force
if ($mcpExe) {
    Copy-Item -Path $mcpExe -Destination (Join-Path $binDir "aetherscope-mcp.exe") -Force
}

$launcherSrc = Join-Path $scriptDir "AetherScope-Launcher.bat"
if (Test-Path $launcherSrc) {
    Copy-Item -Path $launcherSrc -Destination (Join-Path $InstallDir "AetherScope-Launcher.bat") -Force
}
$startCockpitSrc = Join-Path $scriptDir "Start-Cockpit.bat"
if (Test-Path $startCockpitSrc) {
    Copy-Item -Path $startCockpitSrc -Destination (Join-Path $InstallDir "Start-Cockpit.bat") -Force
}
$setupMcpSrc = Join-Path $scriptDir "setup-mcp.ps1"
if (Test-Path $setupMcpSrc) {
    Copy-Item -Path $setupMcpSrc -Destination (Join-Path $InstallDir "setup-mcp.ps1") -Force
}

# 3. Ajout au PATH Utilisateur Windows
Write-Host "[INFO] Configuration de la variable PATH..." -ForegroundColor White
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$paths = $userPath -split ";" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
if ($paths -notcontains $binDir) {
    $newUserPath = ($paths + $binDir) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
    $env:Path = "$env:Path;$binDir"
    Write-Host "  [OK] $binDir ajoute au PATH utilisateur !" -ForegroundColor Green
} else {
    Write-Host "  [OK] Le dossier est deja present dans le PATH." -ForegroundColor DarkGray
}

# 4. Creation du raccourci Bureau
if (-not $SkipShortcut) {
    try {
        $desktopPath = [Environment]::GetFolderPath("Desktop")
        $shortcutPath = Join-Path $desktopPath "AetherScope V2.lnk"
        $targetExe = Join-Path $binDir "aether-engine.exe"
        $launcherTarget = Join-Path $InstallDir "Start-Cockpit.bat"
        if (Test-Path $launcherTarget) {
            $targetExe = $launcherTarget
        }

        $wscript = New-Object -ComObject WScript.Shell
        $shortcut = $wscript.CreateShortcut($shortcutPath)
        $shortcut.TargetPath = $targetExe
        $shortcut.WorkingDirectory = $InstallDir
        $shortcut.Description = "AetherScope V2 - Cockpit Spatial 3D et Linter SAST"
        $shortcut.Save()
        Write-Host "  [OK] Raccourci cree sur le Bureau : $shortcutPath" -ForegroundColor Green
    }
    catch {
        Write-Host "  [!] Echec creation raccourci Bureau : $_" -ForegroundColor Yellow
    }
}

# 5. Configuration automatique du MCP
if (-not $SkipMcp) {
    Write-Host ""
    $mcpScript = Join-Path $InstallDir "setup-mcp.ps1"
    if (Test-Path $mcpScript) {
        & $mcpScript
    } elseif (Test-Path $setupMcpSrc) {
        & $setupMcpSrc
    }
}

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " AETHERSCOPE V2 EST INSTALLE ET OPERATIONNEL !" -ForegroundColor Green
Write-Host " Commandes disponibles :" -ForegroundColor White
Write-Host "   - Double-clic sur le raccourci Bureau AetherScope V2" -ForegroundColor White
Write-Host "   - Tapez 'aether-engine' dans votre terminal" -ForegroundColor White
Write-Host "==========================================================" -ForegroundColor Cyan
