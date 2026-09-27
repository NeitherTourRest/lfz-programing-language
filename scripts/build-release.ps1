#Requires -Version 5.1
<#
.SYNOPSIS
    Build the LFZ interpreter in release mode and package lfz.exe into an output folder.

.DESCRIPTION
    Steps:
      1. cargo build --release
      2. copy target\release\lfz.exe -> <OutDir>\lfz.exe
      3. print "<OutDir>\lfz.exe" --version
      4. smoke-run examples\hello.lfz with the packaged executable
    Exits with a non-zero code on any failure.

    NOTE: This file is intentionally pure ASCII. Windows PowerShell 5.1 decodes
    BOM-less .ps1 files as ANSI, so non-ASCII bytes (e.g. CJK) corrupt parsing.

.PARAMETER OutDir
    Output folder for lfz.exe. Default: <repo>\dist

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1
.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1 -OutDir C:\tmp\lfz
#>
[CmdletBinding()]
param(
    [string]$OutDir = ""
)

$ErrorActionPreference = "Stop"

function Fail([string]$Message) {
    Write-Host ("[build-release] ERROR: {0}" -f $Message) -ForegroundColor Red
    exit 1
}

# Resolve repository root = parent of this script's folder.
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir
if ([string]::IsNullOrWhiteSpace($OutDir)) {
    $OutDir = Join-Path $RepoRoot "dist"
}

Write-Host ("[build-release] repo root : {0}" -f $RepoRoot)
Write-Host ("[build-release] output dir: {0}" -f $OutDir)

# --- 1. Release build -------------------------------------------------------
Push-Location $RepoRoot
try {
    Write-Host "[build-release] cargo build --release"
    & cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Fail ("cargo build --release failed (exit {0})" -f $LASTEXITCODE)
    }
} finally {
    Pop-Location
}

$BuiltExe = Join-Path $RepoRoot "target\release\lfz.exe"
if (-not (Test-Path -LiteralPath $BuiltExe)) {
    Fail ("built executable not found: {0}" -f $BuiltExe)
}

# --- 2. Copy to output folder ----------------------------------------------
if (-not (Test-Path -LiteralPath $OutDir)) {
    New-Item -ItemType Directory -Path $OutDir | Out-Null
}
$DestExe = Join-Path $OutDir "lfz.exe"
Copy-Item -LiteralPath $BuiltExe -Destination $DestExe -Force
$Size = (Get-Item -LiteralPath $DestExe).Length
Write-Host ("[build-release] packaged: {0} ({1} bytes)" -f $DestExe, $Size)

# --- 3. Print version -------------------------------------------------------
Write-Host ("[build-release] version : & `"{0}`" --version" -f $DestExe)
& $DestExe --version
if ($LASTEXITCODE -ne 0) {
    Fail ("`"$DestExe`" --version failed (exit {0})" -f $LASTEXITCODE)
}

# --- 4. Smoke run examples\hello.lfz ---------------------------------------
$Hello = Join-Path $RepoRoot "examples\hello.lfz"
if (-not (Test-Path -LiteralPath $Hello)) {
    Fail ("smoke file not found: {0}" -f $Hello)
}
Write-Host ("[build-release] smoke   : & `"{0}`" `"{1}`"" -f $DestExe, $Hello)
& $DestExe $Hello
if ($LASTEXITCODE -ne 0) {
    Fail ("smoke run of examples\hello.lfz failed (exit {0})" -f $LASTEXITCODE)
}

Write-Host "[build-release] OK"
exit 0
