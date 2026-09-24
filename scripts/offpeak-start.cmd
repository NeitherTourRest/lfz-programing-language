@echo off
REM Launch the LFZ off-peak runner in the background (no admin needed).
REM It works only during DeepSeek off-peak hours and parks on peak. Log: %TEMP%\lfz-offpeak-runner.log
setlocal
set RUNNER=%~dp0offpeak-runner.ps1
start "" powershell.exe -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File "%RUNNER%" -Guard
echo off-peak runner started in the background.
echo log: %TEMP%\lfz-offpeak-runner.log
endlocal
