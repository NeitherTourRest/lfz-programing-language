param([Parameter(Mandatory=$true)][string]$Sha)
$ErrorActionPreference = 'Stop'
$work = Join-Path (Get-Location) 'Temp/T29'
$env:TEMP = $work
$env:TMP = $work
$env:PYTHONUTF8 = '1'
$python = 'D:/app/ANACONDA/python.exe'
$binary = 'Temp/T29/candidate.exe'
& $python -B app/verify_jsonroute.py --binary $binary --expected-sha $Sha --output Temp/T29/jsonroute.json
if ($LASTEXITCODE) { exit $LASTEXITCODE }
& $python -B docs/reports/T27-guide-evidence/verify.py --binary $binary --sha256 $Sha --output Temp/T29/guide.json
if ($LASTEXITCODE) { exit $LASTEXITCODE }
& $python -B app/verify_repl_jsondesk.py --binary $binary --expected-sha $Sha --output Temp/T29/jsondesk.json
if ($LASTEXITCODE) { exit $LASTEXITCODE }
$src=(Get-Content -LiteralPath Temp/T27-ai/verify.py -Raw).Replace('Temp/T27/candidate.exe',$binary).Replace('4816058d402a5d25c0455a0f9c5329febb0dd60629fc27d18654213a947ac636',$Sha).Replace("with_name('results.json')","with_name('ai-guide.json')")
Set-Content -LiteralPath Temp/T29/verify-ai.py -Encoding utf8NoBOM -NoNewline -Value $src
& $python -B Temp/T29/verify-ai.py
exit $LASTEXITCODE
