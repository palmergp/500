@echo off
rem Publishes 500 online for an iPhone's home screen: double-click this file, or run `publish` here. See scripts\publish.ps1.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\publish.ps1" %*
if errorlevel 1 pause
