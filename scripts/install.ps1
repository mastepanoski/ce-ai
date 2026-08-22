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
try {
    $ReleaseApi = "https://api.github.com/repos/mastepanoski/ce-ai/releases/latest"
    $ReleaseInfo = Invoke-RestMethod -Uri $ReleaseApi -Headers @{"User-Agent"="ce-ai-installer"} -UseBasicParsing
    $MatchedAsset = $ReleaseInfo.assets | Where-Object { $_.name -eq $AssetName }
    if ($MatchedAsset -and $MatchedAsset.browser_download_url) {
        $DownloadUrl = $MatchedAsset.browser_download_url
    }
} catch {
    # Fallback to default redirect URL
}

$InstallDir = Join-Path $env:USERPROFILE ".ce-ai\bin"
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

$TempZip = Join-Path $env:TEMP $AssetName

function Test-ValidZipFile($Path) {
    if (-not (Test-Path $Path)) { return $false }
    $item = Get-Item $Path -ErrorAction SilentlyContinue
    if ($null -eq $item -or $item.Length -lt 1000) { return $false }
    return $true
}

Write-Host "📦 Downloading $AssetName from $DownloadUrl..." -ForegroundColor Yellow
$ProgressPreference = 'SilentlyContinue'
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12

if (Get-Command curl.exe -ErrorAction SilentlyContinue) {
    & curl.exe -L -A "ce-ai-installer/1.0" -o "$TempZip" "$DownloadUrl"
}

if (-not (Test-ValidZipFile $TempZip)) {
    Write-Host "  Trying PowerShell System.Net.WebClient fallback..." -ForegroundColor Yellow
    try {
        $WebClient = New-Object System.Net.WebClient
        $WebClient.Headers.Add("User-Agent", "ce-ai-installer/1.0")
        $WebClient.DownloadFile($DownloadUrl, $TempZip)
    } catch {
        Write-Host "  WebClient Download Error: $_" -ForegroundColor Red
    }
}

if (-not (Test-ValidZipFile $TempZip)) {
    throw "❌ Failed to download valid release asset from $DownloadUrl"
}

Write-Host "📂 Extracting to $InstallDir..." -ForegroundColor Yellow
Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
Remove-Item -Path $TempZip -Force

Write-Host "✅ ce-ai successfully installed to $InstallDir\ce-ai.exe" -ForegroundColor Green
Write-Host "Add '$InstallDir' to your System PATH to run 'ce-ai' from anywhere." -ForegroundColor Cyan
