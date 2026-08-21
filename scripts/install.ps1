# PowerShell Universal Installer Script for ce-ai (Windows)
$ErrorActionPreference = "Stop"

Write-Host "🚀 Installing ce-ai (Compound Engineering AI CLI)..." -ForegroundColor Cyan

$Arch = (Get-CimInstance Win32_Processor).Architecture
switch ($Arch) {
    9 { $ArchName = "x86_64" }
    12 { $ArchName = "aarch64" }
    default { $ArchName = "x86_64" }
}

$Target = "$ArchName-pc-windows-msvc"
$AssetName = "ce-ai-$Target.zip"
$DownloadUrl = "https://github.com/mastepanoski/ce-ai/releases/latest/download/$AssetName"

$InstallDir = Join-Path $env:USERPROFILE ".ce-ai\bin"
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

$TempZip = Join-Path $env:TEMP $AssetName

Write-Host "📦 Downloading $AssetName..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip

Write-Host "📂 Extracting to $InstallDir..." -ForegroundColor Yellow
Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
Remove-Item -Path $TempZip -Force

Write-Host "✅ ce-ai successfully installed to $InstallDir\ce-ai.exe" -ForegroundColor Green
Write-Host "Add '$InstallDir' to your System PATH to run 'ce-ai' from anywhere." -ForegroundColor Cyan
