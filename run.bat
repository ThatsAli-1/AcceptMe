@echo off
echo Starting AcceptMe...
echo.

REM Refresh PATH and run
powershell -Command "$env:PATH = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User'); npm run tauri dev"

pause

