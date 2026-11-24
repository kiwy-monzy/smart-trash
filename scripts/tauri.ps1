# Starts the Tauri CLI inside the Visual Studio MSVC environment, with the
# PostgreSQL client library on PATH (same setup as build.ps1).
#
#   powershell -File ..\..\scripts\tauri.ps1 dev
#   powershell -File ..\..\scripts\tauri.ps1 build
#
# Called by `npm run desktop:dev` / `npm run desktop:build`.

param(
    [Parameter(Position = 0)]
    [ValidateSet('dev', 'build', 'android:dev', 'android:build', 'ios:dev', 'ios:build')]
    [string]$Mode = 'dev'
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$app = Join-Path $root 'apps\app'

# --- Visual Studio C++ toolchain -------------------------------------------
$vcvars = Join-Path ${env:ProgramFiles} 'Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat'
if (-not (Test-Path $vcvars)) {
    $vcvars = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
}
if (-not (Test-Path $vcvars)) {
    Write-Error 'vcvars64.bat not found — install the Visual Studio C++ build tools.'
    exit 1
}

# --- PostgreSQL (Diesel / libpq) -------------------------------------------
$pg = Get-ChildItem (Join-Path ${env:ProgramFiles} 'PostgreSQL') -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match '^\d+$' } |
    Sort-Object { [int]$_.Name } -Descending |
    Select-Object -First 1
if ($pg) {
    $env:PQ_LIB_DIR = Join-Path $pg.FullName 'lib'
    $env:PATH = (Join-Path $pg.FullName 'bin') + ';' + $env:PATH
}

$config = if ($Mode -eq 'dev') { ' --config src-tauri/tauri.dev.conf.json' } else { '' }
$command = "npx tauri $Mode$config"

Push-Location $app
try {
    cmd /c "`"$vcvars`" >nul 2>&1 && $command"
    exit $LASTEXITCODE
} finally {
    Pop-Location
}
