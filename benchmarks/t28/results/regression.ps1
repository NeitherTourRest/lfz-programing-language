$ErrorActionPreference='Stop'
$root=(Get-Location).Path
$env:RUSTUP_HOME="$root/Temp/toolchain/rustup"
$env:CARGO_HOME="$root/Temp/toolchain/cargo"
$env:TEMP="$root/Temp/T28"
$env:TMP=$env:TEMP
$env:PYTHONUTF8='1'
$env:PYTHONDONTWRITEBYTECODE='1'
$python='D:/app/ANACONDA/python.exe'
$cargo='./Temp/toolchain/cargo/bin/cargo.exe'
$bin='Temp/T28/candidate.exe'
$sha=(Get-FileHash -LiteralPath $bin).Hash.ToLower()
if ((Get-FileHash -LiteralPath target/release/lfz.exe).Hash.ToLower() -ne $sha) { throw 'target differs from candidate' }
function Run-Checked {
    param($Name, $File, [object[]] $Arguments)
    & $File @Arguments *> "Temp/T28/$Name.log"
    $code=$LASTEXITCODE
    Write-Output "$Name exit=$code"
    if ($code -ne 0) {
        Get-Content -LiteralPath "Temp/T28/$Name.log" -Tail 30
        throw "$Name failed"
    }
}
Run-Checked 'rust-release' $cargo @('test','--release','--offline','--','--test-threads=1')
Run-Checked 'regression' $python @('-B','Temp/T28/regression.py',$sha)
Run-Checked 'official' "./$bin" @('test','--json')
Run-Checked 'repl' $python @('-B','tests/t20_cli_repl.py','--binary',$bin,'--expected-sha',$sha,'--output','Temp/T28/repl.json')
Run-Checked 'formatter' $python @('-B','tests/t21_cli_fmt.py','--binary',$bin,'--expected-sha',$sha,'--baseline','Temp/T21-release/lfz-T20-a545d0.exe','--baseline-sha','a545d0c294eb85276abc5556f297434806ea1def598192ee664593550c579a3c','--output','Temp/T28/formatter.json')
$budget=& "./$bin" run --json --max-steps 60 tests/fixtures/t27_find_index_budget.lfz 2> Temp/T28/budget.stderr
$code=$LASTEXITCODE
$budget | Set-Content -LiteralPath Temp/T28/budget.json -Encoding utf8NoBOM
$record=$budget | ConvertFrom-Json
if ($code -ne 2 -or $record.error -ne 'ExecutionLimitError' -or $record.message -ne '执行步数预算已耗尽' -or (Get-Item Temp/T28/budget.stderr).Length -ne 0) { throw "unexpected budget result: $code $budget" }
Write-Output 'Budget execution limit PASS'
if ((Get-FileHash -LiteralPath $bin).Hash.ToLower() -ne $sha -or (Get-FileHash -LiteralPath target/release/lfz.exe).Hash.ToLower() -ne $sha) { throw 'candidate changed' }
exit 0
