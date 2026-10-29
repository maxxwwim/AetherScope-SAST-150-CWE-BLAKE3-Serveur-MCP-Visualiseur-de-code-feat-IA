# ==========================================================
# AetherScope V2 - MCP Auto Configurator (PowerShell)
# ==========================================================
param(
    [string]$Port = "3030"
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host " AETHERSCOPE V2 - CONFIGURATEUR MCP AUTOMATIQUE " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

$mcpExeCandidates = @(
    (Join-Path $scriptDir "aetherscope-mcp.exe"),
    (Join-Path $scriptDir "target\release\aetherscope-mcp.exe"),
    (Join-Path $env:LOCALAPPDATA "AetherScope\bin\aetherscope-mcp.exe")
)

$mcpExe = $null
foreach ($cand in $mcpExeCandidates) {
    if (Test-Path $cand) {
        $mcpExe = (Resolve-Path $cand).Path
        break
    }
}

if (-not $mcpExe) {
    Write-Host "[ERREUR] aetherscope-mcp.exe est introuvable." -ForegroundColor Red
    Write-Host "Veuillez compiler le projet via: cargo build --release" -ForegroundColor Yellow
    exit 1
}

Write-Host "[OK] Serveur MCP detecte : $mcpExe" -ForegroundColor Green
Write-Host "[INFO] Port cible : $Port" -ForegroundColor DarkGray
Write-Host ""

$mcpEntry = @{
    command = $mcpExe
    args = @("--port", $Port)
}

function Update-McpConfig {
    param(
        [string]$Path,
        [string]$Client
    )

    try {
        $parentDir = Split-Path -Parent $Path
        if (-not (Test-Path $parentDir)) {
            New-Item -ItemType Directory -Force -Path $parentDir | Out-Null
        }

        $cfg = $null
        if (Test-Path $Path) {
            $raw = Get-Content -Path $Path -Raw -Encoding UTF8
            if (-not [string]::IsNullOrWhiteSpace($raw)) {
                try { $cfg = $raw | ConvertFrom-Json } catch { $cfg = $null }
            }
        }

        if (-not $cfg) {
            $cfg = [PSCustomObject]@{
                mcpServers = [PSCustomObject]@{}
            }
        }

        if (-not $cfg.PSObject.Properties['mcpServers']) {
            $cfg | Add-Member -MemberType NoteProperty -Name "mcpServers" -Value ([PSCustomObject]@{})
        }

        $serverObj = [PSCustomObject]@{
            command = $mcpExe
            args = @("--port", $Port)
        }

        if ($cfg.mcpServers.PSObject.Properties['aetherscope']) {
            $cfg.mcpServers.aetherscope = $serverObj
        } else {
            $cfg.mcpServers | Add-Member -MemberType NoteProperty -Name "aetherscope" -Value $serverObj
        }

        $jsonOutput = $cfg | ConvertTo-Json -Depth 10
        [System.IO.File]::WriteAllText($Path, $jsonOutput, [System.Text.Encoding]::UTF8)

        Write-Host "  [OK] $Client configure avec succes :" -ForegroundColor Green
        Write-Host "       $Path" -ForegroundColor DarkGray
    }
    catch {
        Write-Host "  [!] Echec pour $Client : $_" -ForegroundColor Yellow
    }
}

# 1. Claude Desktop
$claudeFile = Join-Path $env:APPDATA "Claude\claude_desktop_config.json"
Write-Host "Configuration de Claude Desktop..." -ForegroundColor White
Update-McpConfig -Path $claudeFile -Client "Claude Desktop"

# 2. Antigravity / Gemini CLI
$geminiFile = Join-Path $env:USERPROFILE ".gemini\config\mcp_config.json"
Write-Host "Configuration de Antigravity / Gemini..." -ForegroundColor White
Update-McpConfig -Path $geminiFile -Client "Antigravity / Gemini"

# 3. Cursor
$cursorFile = Join-Path $env:USERPROFILE ".cursor\mcp.json"
Write-Host "Configuration de Cursor..." -ForegroundColor White
Update-McpConfig -Path $cursorFile -Client "Cursor"

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Green
Write-Host " Configuration MCP terminee avec succes !" -ForegroundColor Green
Write-Host " Redemarrez votre client IA pour activer AetherScope." -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
