# Paint DX - PowerShell Uninstaller Script
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$InstallScript = Join-Path $ScriptDir "install.ps1"
& "$InstallScript" -Uninstall
