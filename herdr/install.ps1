# ASCII and PowerShell 5.1 only: 5.1 reads a BOM-less script in the ANSI code page.
$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot

function Fail([string]$Why) {
    [Console]::Error.WriteLine("reviewr: $Why")
    exit 1
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Fail 'installing this fork requires Rust (https://rustup.rs)'
}

Push-Location $Root
try {
    Write-Output 'reviewr: building the checked-out fork'
    & cargo build --release --locked
    if ($LASTEXITCODE -ne 0) {
        Fail 'cargo build failed'
    }
    $bin = Join-Path $Root 'bin'
    New-Item -ItemType Directory -Force -Path $bin | Out-Null
    $staged = Join-Path $bin 'herdr-reviewr.exe.staging'
    $final = Join-Path $bin 'herdr-reviewr.exe'
    Copy-Item -LiteralPath (Join-Path $Root 'target/release/herdr-reviewr.exe') -Destination $staged -Force
    Move-Item -LiteralPath $staged -Destination $final -Force
    Write-Output "reviewr: installed $final"
} catch {
    Fail $_.Exception.Message
} finally {
    Pop-Location
}
