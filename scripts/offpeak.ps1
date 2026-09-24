# offpeak gate (DeepSeek) -- save money by working only OFF-PEAK.
#
# OFFICIAL (api-docs.deepseek.com/quick_start/pricing):
#   Peak = Mon-Fri 01:00-04:00 and 06:00-10:00 UTC, excluding Chinese public holidays.
#   ALL other hours are off-peak (off-peak price = 50% of peak).
#   Beijing (UTC+8): peak = Mon-Fri 09:00-12:00 and 14:00-18:00; off-peak otherwise
#   (incl. 12:00-14:00 lunch, nights, weekends, Chinese public holidays).
#
# Usage:
#   powershell -File scripts/offpeak.ps1          # status; exit 0 = OFF-PEAK (go), 3 = PEAK (stop)
#   powershell -File scripts/offpeak.ps1 -Quiet    # exit code only
# Override:
#   LFZ_PEAK_UTC   default "01:00-04:00,06:00-10:00"  (UTC ranges)
#   LFZ_PEAK_DAYS  default "1-5"                      (0=Sun .. 6=Sat)
#   LFZ_HOLIDAYS   default ""                         e.g. "2026-10-01,2026-10-02"
#
# NOTE: keep this file pure ASCII -- Windows PowerShell 5.1 decodes a BOM-less .ps1 as ANSI.
param([switch]$Quiet)
$ErrorActionPreference = 'Stop'

$peakSpec = if ($env:LFZ_PEAK_UTC) { $env:LFZ_PEAK_UTC } else { '01:00-04:00,06:00-10:00' }
$daySpec  = if ($env:LFZ_PEAK_DAYS) { $env:LFZ_PEAK_DAYS } else { '1-5' }
$holidays = @()
if ($env:LFZ_HOLIDAYS) { $holidays = @($env:LFZ_HOLIDAYS.Split(',') | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne '' }) }

function ConvertTo-Minutes([string]$hm) {
    $p = $hm.Split(':')
    if ($p.Count -lt 2) { throw ('bad time: ' + $hm) }
    return ([int]$p[0]) * 60 + ([int]$p[1])
}

$ranges = @()
foreach ($r in $peakSpec.Split(',')) {
    $r = $r.Trim()
    if ($r -eq '') { continue }
    $ab = $r.Split('-')
    if ($ab.Count -ne 2) { throw ('bad range: ' + $r) }
    $ranges += , @((ConvertTo-Minutes $ab[0]), (ConvertTo-Minutes $ab[1]))
}

$peakDays = New-Object System.Collections.Generic.HashSet[int]
foreach ($d in $daySpec.Split(',')) {
    $d = $d.Trim()
    if ($d -eq '') { continue }
    if ($d.Contains('-')) {
        $ab = $d.Split('-')
        for ($i = [int]$ab[0]; $i -le [int]$ab[1]; $i++) { [void]$peakDays.Add($i) }
    } else { [void]$peakDays.Add([int]$d) }
}

function Test-IsPeak([datetime]$t) {
    $d = [int]$t.DayOfWeek
    if (-not $peakDays.Contains($d)) { return $false }
    if ($holidays -contains $t.ToString('yyyy-MM-dd')) { return $false }
    $m = $t.Hour * 60 + $t.Minute
    foreach ($rg in $ranges) { if ($m -ge $rg[0] -and $m -lt $rg[1]) { return $true } }
    return $false
}

$utc = (Get-Date).ToUniversalTime()
$inPeak = Test-IsPeak $utc

# minutes until the next state flip (scan forward up to 7 days)
$toChange = 0
for ($k = 0; $k -lt 10080; $k++) {
    if ((Test-IsPeak ($utc.AddMinutes($k + 1))) -ne $inPeak) { $toChange = $k + 1; break }
}

if (-not $Quiet) {
    $bj = $utc.AddHours(8)
    Write-Output ('now  UTC ' + $utc.ToString('yyyy-MM-dd HH:mm') + ' (' + $utc.DayOfWeek + ')   Beijing ' + $bj.ToString('yyyy-MM-dd HH:mm'))
    Write-Output ('peak = Mon-Fri ' + $peakSpec + ' UTC  (Beijing 09:00-12:00 & 14:00-18:00), holidays excluded')
    if ($inPeak) {
        Write-Output ('status: PEAK (stop) - off-peak starts in ' + [int]($toChange / 60) + 'h ' + ($toChange % 60) + 'm')
    } else {
        Write-Output ('status: OFF-PEAK (may work, 50% price) - peak starts in ' + [int]($toChange / 60) + 'h ' + ($toChange % 60) + 'm')
    }
}

if ($inPeak) { exit 3 } else { exit 0 }
