# Runs cargo inside the Visual Studio MSVC environment, with PostgreSQL
# libraries available for Diesel (pq-sys links against libpq).
#
# Why this script exists:
#  - C:\msys64\usr\bin\link.exe (GNU coreutils) otherwise shadows the MSVC
#    linker and every Rust build fails with a confusing "extra operand" error.
#  - Diesel needs libpq.lib at build time and libpq.dll at run time.
#
# Usage:
#   .\build.ps1 cargo build
#   .\build.ps1 cargo test
#   .\build.ps1 cargo run -p smarttrash-services --bin smarttrash-server

param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArgs = @('build')
)

# --- Visual Studio C++ toolchain --------------------------------------------
$vcvars = Join-Path ${env:ProgramFiles} 'Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat'
if (-not (Test-Path $vcvars)) {
    $vcvars = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
}
if (-not (Test-Path $vcvars)) {
    Write-Error 'vcvars64.bat not found — install the Visual Studio C++ build tools.'
    exit 1
}

# --- PostgreSQL (for Diesel) -------------------------------------------------
$pgRoot = Get-ChildItem (Join-Path ${env:ProgramFiles} 'PostgreSQL') -Directory -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match '^\d+$' } |
    Sort-Object { [int]$_.Name } -Descending |
    Select-Object -First 1

if ($pgRoot) {
    $env:PQ_LIB_DIR = Join-Path $pgRoot.FullName 'lib'          # build time: libpq.lib
    $env:PATH = (Join-Path $pgRoot.FullName 'bin') + ';' + $env:PATH  # run time: libpq.dll
} else {
    Write-Warning 'PostgreSQL not found under Program Files — Diesel builds will fail.'
}

$command = $CargoArgs -join ' '
cmd /c "`"$vcvars`" >nul 2>&1 && $command"
exit $LASTEXITCODE
