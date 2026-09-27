$ErrorActionPreference = 'Stop'
$root = (Get-Location).Path
$exe  = Join-Path $root 'target\release\lfz.exe'
$tmp  = Join-Path $root 'Temp\T11'
$outf = Join-Path $tmp 'raw.out'
$errf = Join-Path $tmp 'raw.err'
$sb   = New-Object System.Text.StringBuilder

function ReadU8($p) {
    if (-not (Test-Path $p)) { return '' }
    return [System.IO.File]::ReadAllText($p, [System.Text.Encoding]::UTF8)
}
function AddLine($s) { [void]$sb.AppendLine($s) }

function RunCase($label, $arglist) {
    if (Test-Path $outf) { Remove-Item $outf -Force }
    if (Test-Path $errf) { Remove-Item $errf -Force }
    $p = Start-Process -FilePath $exe -ArgumentList $arglist -WorkingDirectory $tmp `
         -NoNewWindow -Wait -PassThru -RedirectStandardOutput $outf -RedirectStandardError $errf
    $code = $p.ExitCode
    $o = ReadU8 $outf
    $e = ReadU8 $errf
    $ob = if (Test-Path $outf) { (Get-Item $outf).Length } else { 0 }
    $eb = if (Test-Path $errf) { (Get-Item $errf).Length } else { 0 }
    AddLine "### $label"
    AddLine "CMD: lfz $($arglist -join ' ')"
    AddLine "EXIT: $code  (0x$('{0:X8}' -f [uint32]$code))"
    AddLine "STDOUT_BYTES: $ob"
    AddLine "STDOUT:"
    AddLine $o.TrimEnd("`r","`n")
    AddLine "STDERR_BYTES: $eb"
    AddLine "STDERR:"
    AddLine $e.TrimEnd("`r","`n")
    AddLine ""
    return @{ code = $code; o = $o; e = $e; ob = $ob; eb = $eb }
}

AddLine "=== T11 REVERIFICATION RAW EVIDENCE ==="
AddLine "EXE: $exe"
AddLine "EXE_BYTES: $((Get-Item $exe).Length)"
AddLine "EXE_MTIME: $((Get-Item $exe).LastWriteTime.ToString('yyyy-MM-dd HH:mm:ss'))"
AddLine "EXE_SHA256: $((Get-FileHash $exe -Algorithm SHA256).Hash)"
AddLine ""

# ---- Check 1/2: capacity overflow ----
RunCase "C1 repeat capacity" @('run','t_repeat_big.lfz') | Out-Null
RunCase "C2 range capacity"  @('run','t_range_big.lfz')  | Out-Null

# ---- Check 3: parse nesting ----
RunCase "C3a par x1000 balanced" @('run','t_par_1000_bal.lfz') | Out-Null
RunCase "C3b par x1001 balanced" @('run','t_par_1001_bal.lfz') | Out-Null
RunCase "C3c par x1001 prefix"   @('run','t_par_1001_pre.lfz') | Out-Null
RunCase "C3d brk x1001 prefix"   @('run','t_brk_1001_pre.lfz') | Out-Null
RunCase "C3e brk x1001 balanced" @('run','t_brk_1001_bal.lfz') | Out-Null
RunCase "C3f struct x1001 prefix" @('run','t_struct_1001_pre.lfz') | Out-Null
RunCase "C3g fn x1001 prefix"    @('run','t_fn_1001_pre.lfz') | Out-Null
RunCase "C3h par x1001 json"     @('--json','run','t_par_1001_bal.lfz') | Out-Null

# ---- Check 4: AST depth ----
RunCase "C4a add 9999"    @('run','t_add_9999.lfz') | Out-Null
RunCase "C4b add 10000"   @('run','t_add_10000.lfz') | Out-Null
RunCase "C4c add 10001"   @('run','t_add_10001.lfz') | Out-Null
RunCase "C4d add 100000"  @('run','t_add_100000.lfz') | Out-Null
RunCase "C4e idx 10001"   @('run','t_idx_10001.lfz') | Out-Null
RunCase "C4f idx 100000"  @('run','t_idx_100000.lfz') | Out-Null
RunCase "C4g add 10000 json" @('--json','run','t_add_10000.lfz') | Out-Null

# ---- Check 5: dump x --json ----
RunCase "C5a json run dump" @('--json','run','t_dumpjson.lfz') | Out-Null
RunCase "C5b non-json run dump" @('run','t_dumpjson.lfz') | Out-Null
RunCase "C5c json run dump interleave" @('--json','run','t_dumpjson_interleave.lfz') | Out-Null

# ---- Check 6: oob span ----
RunCase "C6a write oob" @('run','t_oob_write.lfz') | Out-Null
RunCase "C6b write oob json" @('--json','run','t_oob_write.lfz') | Out-Null
RunCase "C6c read oob" @('run','t_oob_read.lfz') | Out-Null
RunCase "C6d read oob json" @('--json','run','t_oob_read.lfz') | Out-Null

# ---- Check 7: spec 9.2 sample B ----
$r7 = RunCase "C7 spec92 sample B" @('run','t_s92_b.lfz')

# ---- Check 9: test --json single JSON? ----
RunCase "C9 json test" @('--json','test') | Out-Null

[System.IO.File]::WriteAllText((Join-Path $tmp 'out.txt'), $sb.ToString(), (New-Object System.Text.UTF8Encoding($false)))
Write-Output "DONE"
