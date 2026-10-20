# Paint DX - PowerShell Portable Installer & File Association Script
# Installs Paint DX to the user's LocalAppData and configures file associations and shortcuts.

param(
    [switch]$Uninstall
)

$ErrorActionPreference = "Stop"

$AppName = "Paint DX"
$InstallDir = Join-Path $env:LOCALAPPDATA "Programs\PaintDX"
$ExeTarget = Join-Path $InstallDir "paint-app.exe"
$IcoTarget = Join-Path $InstallDir "assets\app.ico"

if ($Uninstall) {
    Write-Host "Uninstalling $AppName..." -ForegroundColor Cyan
    
    # Remove file associations
    & "$ExeTarget" --unregister-associations 2>$null
    
    # Remove Start Menu shortcut
    $ShortcutDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
    $ShortcutPath = Join-Path $ShortcutDir "$AppName.lnk"
    if (Test-Path $ShortcutPath) {
        Remove-Item -Force $ShortcutPath
    }

    # Remove Desktop shortcut if present
    $DesktopShortcut = Join-Path ([Environment]::GetFolderPath("Desktop")) "$AppName.lnk"
    if (Test-Path $DesktopShortcut) {
        Remove-Item -Force $DesktopShortcut
    }

    # Remove installation directory
    if (Test-Path $InstallDir) {
        Remove-Item -Recurse -Force $InstallDir
    }

    Write-Host "$AppName has been uninstalled successfully." -ForegroundColor Green
    exit 0
}

Write-Host "Installing $AppName to $InstallDir..." -ForegroundColor Cyan

# Ensure target directories exist
New-Item -ItemType Directory -Force -Path (Join-Path $InstallDir "assets") | Out-Null

# Source paths
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$ProjectRoot = Split-Path -Parent $ScriptDir
$SourceExe = Join-Path $ProjectRoot "target\release\paint-app.exe"
if (-not (Test-Path $SourceExe)) {
    $SourceExe = Join-Path $ProjectRoot "target\debug\paint-app.exe"
}
$SourceIco = Join-Path $ProjectRoot "crates\paint-app\assets\app.ico"
$SourcePng = Join-Path $ProjectRoot "crates\paint-app\assets\app.png"

if (-not (Test-Path $SourceExe)) {
    Write-Error "Could not find paint-app.exe. Please build the project first with 'cargo build --release'."
}

# Copy files
Copy-Item -Force $SourceExe $ExeTarget
if (Test-Path $SourceIco) { Copy-Item -Force $SourceIco $IcoTarget }
if (Test-Path $SourcePng) { Copy-Item -Force $SourcePng (Join-Path $InstallDir "assets\app.png") }

# Register file associations
Write-Host "Configuring .paintproj file associations..." -ForegroundColor Cyan
& "$ExeTarget" --register-associations

# Create Start Menu Shortcut
$WScriptShell = New-Object -ComObject WScript.Shell
$StartMenuDir = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
$Shortcut = $WScriptShell.CreateShortcut((Join-Path $StartMenuDir "$AppName.lnk"))
$Shortcut.TargetPath = $ExeTarget
$Shortcut.IconLocation = "$IcoTarget,0"
$Shortcut.Description = "Paint DX - Modernized Windows XP Paint for Windows 11"
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Save()

Write-Host "$AppName installed and registered successfully!" -ForegroundColor Green
Write-Host "You can launch it from the Start Menu or run: & `"$ExeTarget`"" -ForegroundColor Yellow
