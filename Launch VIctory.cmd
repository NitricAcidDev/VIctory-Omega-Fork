@echo off
setlocal
title VIctory Primary Launcher
powershell.exe -NoLogo -NoProfile -ExecutionPolicy RemoteSigned -File "%~dp0victory-normal-launch.ps1" %*
if errorlevel 1 pause
exit /b %ERRORLEVEL%
