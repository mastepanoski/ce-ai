# PowerShell Universal Installer Script for ce-ai (Windows)
$ErrorActionPreference = "Stop"

Write-Host "[ce-ai] Installing ce-ai (Compound Engineering AI CLI)..." -ForegroundColor Cyan

$Arch = (Get-CimInstance Win32_Processor).Architecture
switch ($Arch) {
    9 { $ArchName = "x86_64" }
    12 { $ArchName = "aarch64" }
    default { $ArchName = "x86_64" }
}

$Target = "$ArchName-pc-windows-msvc"
$AssetName = "ce-ai-$Target.zip"
$StaticUrl = "https://github.com/mastepanoski/ce-ai/releases/latest/download/$AssetName"

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

function Resolve-LatestDownloadUrl {
    # Prefer the API-resolved asset URL; fall back to the static redirect URL.
    try {
        $ReleaseApi = "https://api.github.com/repos/mastepanoski/ce-ai/releases/latest"
        $ReleaseInfo = Invoke-RestMethod -Uri $ReleaseApi -Headers @{"User-Agent"="ce-ai-installer/1.0"} -UseBasicParsing
        $MatchedAsset = $ReleaseInfo.assets | Where-Object { $_.name -eq $AssetName }
        if ($MatchedAsset -and $MatchedAsset.browser_download_url) {
            return $MatchedAsset.browser_download_url
        }
    } catch {
        # Publication window or API hiccup: fall through to static URL.
    }
    return $StaticUrl
}

function Attempt-Download($Url) {
    $ProgressPreference = 'SilentlyContinue'
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12

    # Method 1: curl.exe
    if (Get-Command curl.exe -ErrorAction SilentlyContinue) {
        & curl.exe -fsSL -A "ce-ai-installer/1.0" -o "$TempZip" "$Url" 2>$null
    }

    # Method 2: .NET HttpWebRequest with AutoRedirect
    if (-not (Test-ValidZipFile $TempZip)) {
        if (Test-Path $TempZip) { Remove-Item $TempZip -Force }
        try {
            $req = [System.Net.HttpWebRequest]::Create($Url)
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
            Write-Host "[ce-ai] Download attempt failed: $_" -ForegroundColor DarkYellow
        }
    }
    return (Test-ValidZipFile $TempZip)
}

Write-Host "[ce-ai] Downloading $AssetName..." -ForegroundColor Yellow

$AttemptedUrls = @()
$Downloaded = $false

# Retry loop: covers transient 404s during release publication windows.
for ($Attempt = 1; $Attempt -le 3 -and -not $Downloaded; $Attempt++) {
    if ($Attempt -gt 1) {
        $Backoff = 10 * ($Attempt - 1)
        Write-Host "[ce-ai] Asset not ready yet; retrying in $Backoff seconds (attempt $($Attempt)/3)..." -ForegroundColor Yellow
        Start-Sleep -Seconds $Backoff
    }
    $Url = Resolve-LatestDownloadUrl
    $AttemptedUrls += $Url
    Write-Host "[ce-ai] Downloading from $Url..." -ForegroundColor Yellow
    $Downloaded = Attempt-Download $Url
}

# Fallback: scan recent releases when latest lacks the platform asset.
if (-not $Downloaded) {
    try {
        $Recent = Invoke-RestMethod -Uri "https://api.github.com/repos/mastepanoski/ce-ai/releases?per_page=5" -Headers @{"User-Agent"="ce-ai-installer/1.0"} -UseBasicParsing
        foreach ($Rel in $Recent) {
            $Matched = $Rel.assets | Where-Object { $_.name -eq $AssetName }
            if ($Matched -and $Matched.browser_download_url) {
                $Url = $Matched.browser_download_url
                if ($AttemptedUrls -notcontains $Url) {
                    $AttemptedUrls += $Url
                    Write-Host "[ce-ai] Falling back to recent release $($Rel.tag_name)..." -ForegroundColor Yellow
                    $Downloaded = Attempt-Download $Url
                    if ($Downloaded) { break }
                }
            }
        }
    } catch {
        # Exhausted: final error below lists attempted URLs.
    }
}

if (-not $Downloaded) {
    Write-Error "[ce-ai] Failed to download a valid release asset after retries and fallback. Attempted URLs: $($AttemptedUrls -join ', ')"
    exit 1
}

Write-Host "[ce-ai] Extracting to $InstallDir..." -ForegroundColor Yellow
Expand-Archive -Path $TempZip -DestinationPath $InstallDir -Force
Remove-Item -Path $TempZip -Force

Write-Host "[ce-ai] ce-ai successfully installed to $InstallDir\ce-ai.exe" -ForegroundColor Green
Write-Host "[ce-ai] Add $InstallDir to your System PATH to run ce-ai from anywhere." -ForegroundColor Cyan

# End of script
