@echo off
rem Starts 500: double-click this file, or run `play` here. See scripts\play.ps1.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\play.ps1" %*
if errorlevel 1 pause
