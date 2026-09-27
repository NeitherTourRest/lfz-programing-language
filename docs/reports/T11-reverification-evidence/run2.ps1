$ErrorActionPreference = 'Stop'
$root = (Get-Location).Path
$exe  = Join-Path $root 'target\release\lfz.exe'
$tmp  = Join-Path $root 'Temp\T11'
$outf = Join-Path $tmp 'b2.out'
$errf = Join-Path $tmp 'b2.err'
$sb   = New-Object System.Text.StringBuilder
function AddLine($s) { [void]$sb.AppendLine($s) }
function RunOne($f) {
    if (Test-Path $outf) { Remove-Item $outf -Force }
    if (Test-Path $errf) { Remove-Item $errf -Force }
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $p = Start-Process -FilePath $exe -ArgumentList @('run', $f) -WorkingDirectory $tmp `
         -NoNewWindow -Wait -PassThru -RedirectStandardOutput $outf -RedirectStandardError $errf
    $sw.Stop()
    $code = $p.ExitCode
    $o = if (Test-Path $outf) { [System.IO.File]::ReadAllText($outf,[Text.Encoding]::UTF8) } else { '' }
    $e = if (Test-Path $errf) { [System.IO.File]::ReadAllText($errf,[Text.Encoding]::UTF8) } else { '' }
    AddLine "### $f"
    AddLine "EXIT: $code (0x$('{0:X8}' -f [uint32]$code))  TIME: $($sw.ElapsedMilliseconds) ms"
    AddLine "STDOUT: $($o.TrimEnd())"
    $eShort = $e
    if ($eShort.Length -gt 400) { $eShort = $eShort.Substring(0,400) + ' ...[trunc]' }
    AddLine "STDERR: $($eShort.TrimEnd())"
    AddLine ""
}
foreach ($f in @(
    't_fmt_width.lfz','t_fmt_width2.lfz','t_fmt_width_ok.lfz',
    't_fmt_prec.lfz','t_fmt_prec2.lfz','t_fmt_prec_ok.lfz',
    't_strrepeat.lfz','t_repeat_empty.lfz',
    't_unary_1001.lfz','t_not_1001.lfz',
    't_floor_huge.lfz','t_int_huge.lfz',
    't_deep_display.lfz','t_deep_eq.lfz'
)) { RunOne $f }
[System.IO.File]::WriteAllText((Join-Path $tmp 'out2.txt'), $sb.ToString(), (New-Object System.Text.UTF8Encoding($false)))
Write-Output "DONE"
