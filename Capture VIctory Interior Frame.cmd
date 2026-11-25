@echo off
powershell.exe -NoLogo -NoProfile -ExecutionPolicy RemoteSigned -File "%~dp0capture-victory-interior.ps1"
if errorlevel 1 pause
