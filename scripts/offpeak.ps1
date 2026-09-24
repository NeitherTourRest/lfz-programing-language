# offpeak gate -- work only during DeepSeek's discounted window to save money.
#
# Usage:
#   powershell -File scripts/offpeak.ps1          # status; exit 0 = off-peak (go), 3 = peak (stop)
#   powershell -File scripts/offpeak.ps1 -Quiet    # exit code only
#
# Window (default): Beijing time (UTC+8) 00:30-08:30 = DeepSeek off-peak discount window.
# Override via env: LFZ_OFFPEAK_START / LFZ_OFFPEAK_END / LFZ_OFFPEAK_UTC_OFFSET
# The window may cross midnight (e.g. START=23:00, END=07:00).
# NOTE: keep this file pure ASCII -- Windows PowerShell 5.1 decodes BOM-less .ps1 as ANSI.
param([switch]$Quiet)
$ErrorActionPreference = 'Stop'

$startStr = if ($env:LFZ_OFFPEAK_START) { $env:LFZ_OFFPEAK_START } else { '00:30' }
$endStr   = if ($env:LFZ_OFFPEAK_END)   { $env:LFZ_OFFPEAK_END }   else { '08:30' }
$offsetH  = if ($env:LFZ_OFFPEAK_UTC_OFFSET) { [int]$env:LFZ_OFFPEAK_UTC_OFFSET } else { 8 }

function ConvertTo-Minutes([string]$hm) {
    $p = $hm.Split(':')
    if ($p.Count -lt 2) { throw ('bad time format: ' + $hm + ' (expect HH:mm)') }
    return ([int]$p[0]) * 60 + ([int]$p[1])
}

$start = ConvertTo-Minutes $startStr
$end   = ConvertTo-Minutes $endStr
$local = (Get-Date).ToUniversalTime().AddHours($offsetH)
$mins  = $local.Hour * 60 + $local.Minute

if ($start -lt $end) {
    $inWindow = ($mins -ge $start -and $mins -lt $end)
} else {
    $inWindow = ($mins -ge $start -or $mins -lt $end)
}

if ($inWindow) { $toChange = $end - $mins } else { $toChange = $start - $mins }
if ($toChange -le 0) { $toChange += 1440 }

if (-not $Quiet) {
    $sign = '+'
    if ($offsetH -lt 0) { $sign = '-' }
    $absH = [Math]::Abs($offsetH)
    $h = [int]($toChange / 60)
    $m = $toChange % 60
    Write-Output ('local time (UTC' + $sign + $absH + '): ' + $local.ToString('yyyy-MM-dd HH:mm'))
    Write-Output ('off-peak window: ' + $startStr + ' - ' + $endStr)
    if ($inWindow) {
        Write-Output ('status: OFF-PEAK (may work) - window closes in ' + $h + 'h ' + $m + 'm')
    } else {
        Write-Output ('status: PEAK (stop) - off-peak opens in ' + $h + 'h ' + $m + 'm')
    }
}

if ($inWindow) { exit 0 } else { exit 3 }
