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
    try {
        $item = Get-Item $Path -ErrorAction SilentlyContinue
        if ($null -eq $item -or $item.Length -lt 1000) { return $false }
        $fs = [System.IO.File]::OpenRead($Path)
        $buffer = New-Object byte[] 2
        $bytesRead = $fs.Read($buffer, 0, 2)
        $fs.Close()
        if ($bytesRead -eq 2 -and $buffer[0] -eq 80 -and $buffer[1] -eq 75) {
            return $true
        }
        return $false
    } catch {
        return $false
    }
}

Write-Host "📦 Downloading $AssetName from $DownloadUrl..." -ForegroundColor Yellow
$ProgressPreference = 'SilentlyContinue'
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12

# Method 1: curl.exe
if (Get-Command curl.exe -ErrorAction SilentlyContinue) {
    & curl.exe -fsSL -A "ce-ai-installer/1.0" -o "$TempZip" "$DownloadUrl"
}

# Method 2: .NET HttpWebRequest with AutoRedirect
if (-not (Test-ValidZipFile $TempZip)) {
    Write-Host "[ce-ai] Trying HttpWebRequest auto-redirect fallback..." -ForegroundColor Yellow
    try {
        if (Test-Path $TempZip) { Remove-Item $TempZip -Force }
        $req = [System.Net.HttpWebRequest]::Create($DownloadUrl)
        $req.UserAgent = "ce-ai-installer/1.0"
        $req.AllowAutoRedirect = $true
        $res = $req.GetResponse()
        $inStream = $res.GetResponseStream()
        $outStream = [System.IO.File]::Create($TempZip)
        $inStream.CopyTo($outStream)
        $outStream.Close()
        $inStream.Close()
        $res.Close()
    } catch {
        Write-Host "[ce-ai] Download Error: $_" -ForegroundColor Red
    }
}

if (-not (Test-ValidZipFile $TempZip)) {
    Write-Error "[ce-ai] Failed to download valid release asset from $DownloadUrl"
    exit 1
}

Write-Host "[ce-ai] Extracting to $InstallDir..." -ForegroundColor Yellow
Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
Remove-Item -Path $TempZip -Force

Write-Host "[ce-ai] ce-ai successfully installed to $InstallDir\ce-ai.exe" -ForegroundColor Green
Write-Host "[ce-ai] Add $InstallDir to your System PATH to run ce-ai from anywhere." -ForegroundColor Cyan
