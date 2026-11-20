$ErrorActionPreference = 'Stop'
$root = (Get-Location).Path
$work = Join-Path $root 'Temp/T28'
$env:TEMP = $work
$env:TMP = $work
$env:RUSTUP_HOME = Join-Path $root 'Temp/toolchain/rustup'
$env:CARGO_HOME = Join-Path $root 'Temp/toolchain/cargo'
$env:PYTHONUTF8 = '1'
$env:PYTHONDONTWRITEBYTECODE = '1'
if (Test-Path -LiteralPath "$work/candidate.exe") { throw 'candidate already frozen' }
& ./Temp/toolchain/cargo/bin/cargo.exe build --release --offline *> "$work/build.log"
if ($LASTEXITCODE) { Get-Content "$work/build.log" -Tail 45; exit $LASTEXITCODE }
Copy-Item -LiteralPath target/release/lfz.exe -Destination "$work/candidate.exe"
$sha=(Get-FileHash -LiteralPath "$work/candidate.exe").Hash.ToLower()
Write-Output "Built candidate $sha"
& ./Temp/T28/regression.ps1
if ($LASTEXITCODE) { exit $LASTEXITCODE }
& ./Temp/T28/run-examples.ps1 -Sha $sha
exit $LASTEXITCODE
