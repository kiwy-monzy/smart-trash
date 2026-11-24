# Starts the backend if it is not running, builds the UI, then opens the app.
#
#   powershell -ExecutionPolicy Bypass -File .\scripts\run-app.ps1
#
# Called by `npm run app` inside tauri-app/.

param(
    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$app = Join-Path $root 'apps\app'

function Test-Port([int]$Port) {
    try {
        $client = [System.Net.Sockets.TcpClient]::new()
        $client.Connect('127.0.0.1', $Port)
        $client.Close()
        return $true
    } catch {
        return $false
    }
}

# 1. gRPC server (the app talks to it directly, over gRPC-Web)
if (-not (Test-Port 50051)) {
    Write-Host 'starting smarttrash-server on :50051' -ForegroundColor Cyan
    $pg = Get-ChildItem (Join-Path $env:ProgramFiles 'PostgreSQL') -Directory -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match '^\d+$' } |
        Sort-Object { [int]$_.Name } -Descending |
        Select-Object -First 1
    if ($pg) { $env:PATH = "$($pg.FullName)\bin;$env:PATH" }

    Start-Process -FilePath (Join-Path $root 'target\debug\smarttrash-server.exe') `
        -WorkingDirectory $root -WindowStyle Minimized
    Start-Sleep -Seconds 2
}

# 2. UI bundle
Push-Location $app
try {
    if (-not $SkipBuild) {
        Write-Host 'building the UI (vite)' -ForegroundColor Cyan
        npm run build | Out-Null
    }

    # 3. native shell
    $exe = Join-Path $app 'src-tauri\target\release\smart-trash-app.exe'
    if (-not (Test-Path $exe)) {
        Write-Host 'building the Tauri app (first run takes a few minutes)' -ForegroundColor Cyan
        Push-Location (Join-Path $app 'src-tauri')
        try { cargo build --release } finally { Pop-Location }
    }

    Write-Host 'opening Smart Trash' -ForegroundColor Green
    & $exe
} finally {
    Pop-Location
}
