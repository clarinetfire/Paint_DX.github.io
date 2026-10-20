# Paint DX - Authenticode Signing Helper Script
# Signs paint-app.exe or installer with a code signing certificate using signtool.exe.

param(
    [string]$TargetFile = "target\release\paint-app.exe",
    [string]$CertPath,
    [string]$CertPassword,
    [string]$TimestampServer = "http://timestamp.digicert.com"
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path $TargetFile)) {
    Write-Error "Target file '$TargetFile' does not exist."
}

# Locate signtool.exe from Windows SDKs
$SignTool = Get-ChildItem -Path "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Filter "signtool.exe" -Recurse -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match "x64" } |
    Select-Object -First 1 -ExpandProperty FullName

if (-not $SignTool) {
    # Check PATH
    $SignTool = (Get-Command signtool.exe -ErrorAction SilentlyContinue).Source
}

if (-not $SignTool) {
    Write-Warning "signtool.exe not found in Windows Kit or PATH. Skipping Authenticode signing."
    exit 0
}

Write-Host "Signing '$TargetFile' using $SignTool..." -ForegroundColor Cyan

if ($CertPath -and (Test-Path $CertPath)) {
    $argsList = @("sign", "/fd", "SHA256", "/tr", $TimestampServer, "/td", "SHA256", "/f", $CertPath)
    if ($CertPassword) {
        $argsList += @("/p", $CertPassword)
    }
    $argsList += $TargetFile
    & "$SignTool" $argsList
} else {
    Write-Host "No certificate specified. Attempting auto-signing using current user cert store (if available)..." -ForegroundColor Yellow
    $argsList = @("sign", "/a", "/fd", "SHA256", "/tr", $TimestampServer, "/td", "SHA256", $TargetFile)
    & "$SignTool" $argsList
}

Write-Host "Signing process completed." -ForegroundColor Green
