# offpeak-runner -- unattended worker for the LFZ project.
#
# Whenever we are OFF-PEAK it runs ONE unit of work ('opencode run' as the team-lead
# agent); while PEAK it parks (checks every 5 min) and resumes automatically.
# Register it once with:  powershell -File scripts/offpeak-task.ps1 install
#
# Usage:
#   powershell -File scripts/offpeak-runner.ps1 -Once                 # one unit if off-peak
#   powershell -File scripts/offpeak-runner.ps1 -Guard                # loop (work / park) forever
#   powershell -File scripts/offpeak-runner.ps1 -Guard -MaxRuns 5
#   powershell -File scripts/offpeak-runner.ps1 -DryRun -Once         # print command only
#
# Log: %TEMP%\lfz-offpeak-runner.log
# NOTE: keep this file pure ASCII (PowerShell 5.1 decodes BOM-less .ps1 as ANSI).
param(
  [string]$ProjectDir = (Split-Path -Parent $PSScriptRoot),
  [string]$Agent = 'team-lead',
  [string]$Message = 'Continue the LFZ project: read .opencode/team/PROJECT_STATE.md, TEAM_BOARD.md and your own STATUS.md, then advance the next unfinished sub-phase (dispatch -> independent check -> atomic commit). If a MAJOR decision needs the user (scope change, tech choice, spending), STOP and record it in PROJECT_STATE instead of deciding.',
  [int]$MaxRuns = 3,
  [switch]$Guard,
  [switch]$Once,
  [switch]$DryRun,
  [switch]$NoAuto
)
$ErrorActionPreference = 'Stop'
$gate = Join-Path $PSScriptRoot 'offpeak.ps1'
$log = Join-Path $env:TEMP 'lfz-offpeak-runner.log'

function Write-Log([string]$m) {
  $line = '[' + (Get-Date).ToString('yyyy-MM-dd HH:mm:ss') + '] ' + $m
  Write-Output $line
  Add-Content -LiteralPath $log -Value $line -ErrorAction SilentlyContinue
}
function Test-OffPeak {
  & powershell -NoProfile -File $gate -Quiet | Out-Null
  return ($LASTEXITCODE -eq 0)
}
function Wait-OffPeak {
  while (-not (Test-OffPeak)) { Start-Sleep -Seconds 300 }
}

if (-not (Test-OffPeak)) {
  Write-Log 'PEAK now.'
  if (-not $Guard) { exit 3 }
  Write-Log 'parking until off-peak (checks every 5 min)...'
  Wait-OffPeak
  Write-Log 'off-peak reached.'
}

$max = if ($Once) { 1 } else { $MaxRuns }
$n = 0
while (($max -le 0) -or ($n -lt $max)) {
  if (-not (Test-OffPeak)) {
    Write-Log 'peak started - stopping.'
    if (-not $Guard) { break }
    Write-Log 'parking until off-peak...'
    Wait-OffPeak
    Write-Log 'off-peak reached - resuming.'
  }
  $n++
  $oc = @('opencode', 'run', '--agent', $Agent, '--dir', $ProjectDir)
  if (-not $NoAuto) { $oc += '--auto' }
  $oc += $Message
  Write-Log ('run #' + $n + ' (agent=' + $Agent + ', dir=' + $ProjectDir + ')')
  if ($DryRun) {
    Write-Log ('DRY-RUN: ' + ($oc -join ' '))
    continue
  }
  & $oc[0] $oc[1..($oc.Count - 1)]
  Write-Log ('run #' + $n + ' exit=' + $LASTEXITCODE)
  Start-Sleep -Seconds 20
}
Write-Log ('done. runs=' + $n)
