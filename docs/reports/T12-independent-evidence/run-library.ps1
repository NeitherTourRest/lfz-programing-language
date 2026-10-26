$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
Push-Location $repo
try {
    $work = Join-Path $repo 'Temp/T12/verifier-library'
    New-Item -ItemType Directory -Force $work | Out-Null
    $env:TEMP = $work
    $env:TMP = $work
    $bundled = Join-Path $repo 'Temp/toolchain/cargo/bin'
    if (Test-Path (Join-Path $bundled 'cargo.exe')) {
        $env:RUSTUP_HOME = Join-Path $repo 'Temp/toolchain/rustup'
        $env:CARGO_HOME = Join-Path $repo 'Temp/toolchain/cargo'
        $cargo = Join-Path $bundled 'cargo.exe'
        $rustc = Join-Path $bundled 'rustc.exe'
    } else {
        $cargo = 'cargo'
        $rustc = 'rustc'
    }
    & $cargo build --lib
    if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
    $binary = Join-Path $work 'library-check.exe'
    & $rustc --edition=2021 docs/reports/T12-independent-evidence/library.rs --extern lfz=target/debug/liblfz.rlib -L dependency=target/debug/deps -o $binary
    if ($LASTEXITCODE -ne 0) { throw 'rustc probe build failed' }
    & $binary
    if ($LASTEXITCODE -ne 0) { throw 'library probe failed' }
} finally {
    Pop-Location
}