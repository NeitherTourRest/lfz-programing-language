#Requires -Version 5.1
<#
.SYNOPSIS
    Install / uninstall the LFZ interpreter as a user-level `lfz` command.

.DESCRIPTION
    Installs lfz.exe into %LOCALAPPDATA%\Programs\lfz and appends that folder to the
    USER PATH using [Environment]::SetEnvironmentVariable(..., 'User') -- never setx,
    which truncates / merges PATH. The previous user PATH is backed up to
    %LOCALAPPDATA%\Programs\lfz\path-backup.txt before any change.

    SAFE BY DEFAULT: without -Apply the script only PRINTS what it would do (dry-run)
    and changes nothing. Pass -Apply to actually install or uninstall.

    NOTE: this file is intentionally pure ASCII. Windows PowerShell 5.1 decodes
    BOM-less .ps1 files as ANSI, so non-ASCII bytes (e.g. CJK) corrupt parsing.

.PARAMETER Apply
    Actually perform the planned changes. Without it: dry-run only.
.PARAMETER Uninstall
    Remove the install folder and drop it from the user PATH (backup is kept).
.PARAMETER SourceExe
    Path to the lfz.exe to install. Default: <repo>\dist\lfz.exe

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\install-lfz.ps1
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\install-lfz.ps1 -Apply
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\install-lfz.ps1 -Uninstall -Apply
#>
[CmdletBinding()]
param(
    [switch]$Apply,
    [switch]$Uninstall,
    [string]$SourceExe = ""
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir
if ([string]::IsNullOrWhiteSpace($SourceExe)) {
    $SourceExe = Join-Path $RepoRoot "dist\lfz.exe"
}

$InstallDir  = Join-Path $env:LOCALAPPDATA "Programs\lfz"
$DestExe     = Join-Path $InstallDir "lfz.exe"
$BackupFile  = Join-Path $InstallDir "path-backup.txt"
$MovedBackup = Join-Path $env:LOCALAPPDATA "Programs\lfz-path-backup.txt"

function Fail([string]$Message) {
    Write-Host ("[install-lfz] ERROR: {0}" -f $Message) -ForegroundColor Red
    exit 1
}

function Get-UserPath {
    $p = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($null -eq $p) { return "" }
    return $p
}

function Get-PathSegments([string]$PathText) {
    if ([string]::IsNullOrEmpty($PathText)) { return @() }
    return @($PathText -split ';' | Where-Object { $_ -ne "" })
}

function Test-PathHasDir([string]$PathText, [string]$Dir) {
    $needle = $Dir.TrimEnd('\')
    foreach ($seg in (Get-PathSegments $PathText)) {
        if ($seg.TrimEnd('\') -ieq $needle) { return $true }
    }
    return $false
}

$UserPath = Get-UserPath

Write-Host "=============================================="
Write-Host " LFZ user installer"
Write-Host "=============================================="
Write-Host (" mode        : {0}" -f $(if ($Uninstall) { "UNINSTALL" } else { "INSTALL" }))
Write-Host (" apply       : {0}" -f $(if ($Apply) { "yes (changes WILL be made)" } else { "no (dry-run)" }))
Write-Host (" install dir : {0}" -f $InstallDir)
Write-Host (" dest exe    : {0}" -f $DestExe)
Write-Host (" backup file : {0}" -f $BackupFile)
if (-not $Uninstall) {
    Write-Host (" source exe  : {0}" -f $SourceExe)
}
Write-Host ""

if ($Uninstall) {
    # ---------------------------------------------------------------- UNINSTALL
    $dirExists    = Test-Path -LiteralPath $InstallDir
    $pathHasDir   = Test-PathHasDir $UserPath $InstallDir
    $backupExists = Test-Path -LiteralPath $BackupFile

    Write-Host "[uninstall] planned actions:"
    if ($pathHasDir) {
        Write-Host ("  - remove '{0}' from the USER PATH" -f $InstallDir)
    } else {
        Write-Host "  - user PATH does not contain the install dir (no PATH change)"
    }
    if ($backupExists) {
        Write-Host ("  - move '{0}' -> '{1}' (keep the backup)" -f $BackupFile, $MovedBackup)
    }
    if ($dirExists) {
        Write-Host ("  - delete directory '{0}'" -f $InstallDir)
    } else {
        Write-Host ("  - directory '{0}' does not exist (nothing to delete)" -f $InstallDir)
    }

    if (-not $Apply) {
        Write-Host ""
        Write-Host "[dry-run] no changes made. Re-run with -Apply to actually uninstall."
        exit 0
    }

    if ($pathHasDir) {
        $kept = @(Get-PathSegments $UserPath | Where-Object { $_.TrimEnd('\') -ine $InstallDir.TrimEnd('\') })
        [Environment]::SetEnvironmentVariable("Path", ($kept -join ';'), "User")
        Write-Host "[uninstall] removed install dir from the USER PATH."
    }
    if ($backupExists) {
        if (Test-Path -LiteralPath $MovedBackup) { Remove-Item -LiteralPath $MovedBackup -Force }
        Move-Item -LiteralPath $BackupFile -Destination $MovedBackup -Force
        Write-Host ("[uninstall] backup kept at '{0}'." -f $MovedBackup)
    }
    if ($dirExists) {
        Remove-Item -LiteralPath $InstallDir -Recurse -Force
        Write-Host ("[uninstall] deleted '{0}'." -f $InstallDir)
    }
    Write-Host "[uninstall] OK"
    exit 0
}

# -------------------------------------------------------------------- INSTALL
$dirExists    = Test-Path -LiteralPath $InstallDir
$pathHasDir   = Test-PathHasDir $UserPath $InstallDir
$sourceExists = Test-Path -LiteralPath $SourceExe

Write-Host "[install] planned actions:"
if ($sourceExists) {
    Write-Host ("  - copy '{0}' -> '{1}'" -f $SourceExe, $DestExe)
} else {
    Write-Host ("  - source executable NOT FOUND: '{0}'" -f $SourceExe)
    Write-Host "    build it first, e.g.: powershell -File scripts\build-release.ps1"
}
if (-not $dirExists) {
    Write-Host ("  - create directory '{0}'" -f $InstallDir)
}
Write-Host ("  - back up current USER PATH to '{0}'" -f $BackupFile)
if ($pathHasDir) {
    Write-Host ("  - USER PATH already contains '{0}' (no PATH change)" -f $InstallDir)
} else {
    Write-Host ("  - append '{0}' to the USER PATH" -f $InstallDir)
}
Write-Host ""
Write-Host "[install] current USER PATH:"
Write-Host ("  {0}" -f $(if ([string]::IsNullOrEmpty($UserPath)) { "<empty>" } else { $UserPath }))
if (-not $pathHasDir) {
    $preview = if ([string]::IsNullOrEmpty($UserPath)) { $InstallDir } else { $UserPath.TrimEnd(';') + ';' + $InstallDir }
    Write-Host "[install] planned new USER PATH:"
    Write-Host ("  {0}" -f $preview)
}

if (-not $Apply) {
    Write-Host ""
    Write-Host "[dry-run] no changes made. Re-run with -Apply to actually install."
    exit 0
}

if (-not $sourceExists) {
    Fail ("source executable not found: {0}" -f $SourceExe)
}

# Apply: create dir -> back up PATH -> copy exe -> update PATH
if (-not $dirExists) {
    New-Item -ItemType Directory -Path $InstallDir | Out-Null
    Write-Host ("[install] created '{0}'." -f $InstallDir)
}
Set-Content -LiteralPath $BackupFile -Value $UserPath -Encoding UTF8
Write-Host ("[install] backed up USER PATH to '{0}'." -f $BackupFile)

Copy-Item -LiteralPath $SourceExe -Destination $DestExe -Force
Write-Host ("[install] copied executable to '{0}'." -f $DestExe)

if (-not $pathHasDir) {
    $newPath = if ([string]::IsNullOrEmpty($UserPath)) { $InstallDir } else { $UserPath.TrimEnd(';') + ';' + $InstallDir }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "[install] appended install dir to the USER PATH."
} else {
    Write-Host "[install] USER PATH already contains install dir (unchanged)."
}

Write-Host ""
Write-Host "[install] OK. Open a NEW terminal, then run:  lfz --version"
exit 0
