@echo off
rem Builds app\500.exe: the whole game in one file, to copy to another PC. See scripts\make-app.ps1.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\make-app.ps1"
pause
