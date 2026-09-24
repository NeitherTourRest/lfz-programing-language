@echo off
REM Enable / disable / drive the LFZ off-peak runner.
REM
REM   scripts\offpeak-task.cmd autostart         -> enable auto-start at logon (NO admin needed)
REM   scripts\offpeak-task.cmd noautostart       -> disable auto-start
REM   scripts\offpeak-task.cmd start             -> start the runner right now
REM   scripts\offpeak-task.cmd status            -> show whether auto-start is enabled
REM
REM   scripts\offpeak-task.cmd install           -> [needs ADMIN] register a Scheduled Task instead
REM   scripts\offpeak-task.cmd uninstall         -> [needs ADMIN] remove that Scheduled Task
REM   scripts\offpeak-task.cmd taskstatus        -> [needs ADMIN] query that Scheduled Task
REM
REM Why two mechanisms:
REM   * Startup-folder autostart  = per-user, NO admin, runs at logon.
REM   * Scheduled Task            = needs elevation; use it only if you want it to run
REM                                 without an interactive logon.
REM NOTE: keep this file pure ASCII.
setlocal
set TASK=LFZ-offpeak-runner
set RUNNER=%~dp0offpeak-runner.ps1
set STARTCMD=%~dp0offpeak-start.cmd
set STARTUP=%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup
set LINK=%STARTUP%\LFZ-offpeak-runner.cmd
set TR="\"powershell.exe -NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File \"%RUNNER%\" -Guard\""

if /I "%~1"=="autostart"    goto :autostart
if /I "%~1"=="noautostart"  goto :noautostart
if /I "%~1"=="start"        ( call "%STARTCMD%" & goto :end )
if /I "%~1"=="status"       goto :status
if /I "%~1"=="uninstall"    ( schtasks /Delete /TN "%TASK%" /F & goto :end )
if /I "%~1"=="taskstatus"   ( schtasks /Query /TN "%TASK%" & goto :end )

schtasks /Create /TN "%TASK%" /SC ONLOGON /TR %TR% /F
if errorlevel 1 ( echo. & echo [FAILED] needs an elevated shell - run this .cmd as Administrator. & echo          Or use the no-admin route: scripts\offpeak-task.cmd autostart & goto :end )
echo.
schtasks /Query /TN "%TASK%"
echo.
echo registered scheduled task: %TASK%
echo runner : %RUNNER%
echo log    : %TEMP%\lfz-offpeak-runner.log
goto :end

:autostart
if not exist "%STARTUP%" ( echo [FAILED] Startup folder not found: "%STARTUP%" & goto :end )
copy /Y "%STARTCMD%" "%LINK%" >nul
if errorlevel 1 ( echo [FAILED] could not copy into Startup. & goto :end )
echo [OK] auto-start enabled (no admin): "%LINK%"
echo      it launches: %STARTCMD%
goto :end

:noautostart
if exist "%LINK%" ( del /Q "%LINK%" ) 
echo [OK] auto-start disabled.
goto :end

:status
if exist "%LINK%" ( echo auto-start : ENABLED  -^> "%LINK%" ) else ( echo auto-start : disabled )
schtasks /Query /TN "%TASK%" >nul 2>&1
if errorlevel 1 ( echo scheduled  : none ) else ( echo scheduled  : registered ^(needs admin to manage^) )
goto :end

:end
endlocal

