---
module: scripts/install.ps1
date: 2026-08-21
problem_type: build_error
category: build_error
component: tooling
severity: high
symptoms:
  - "PowerShell 5.1 fails with TerminatorExpectedAtEndOfString at line 91 due to multi-byte UTF-8 emoji tokenizer corruption"
  - "Expand-Archive fails with ArchiveCmdletPathNotFound because WebClient downloaded HTML 302 redirect response instead of release zip"
root_cause: logic_error
resolution_type: code_fix
tags:
  - powershell
  - utf8-encoding
  - http-redirect
  - windows-ci
  - installer-script
---

# Fix PowerShell 5.1 Tokenizer Corruption and Cross-Domain HTTP 302 Download Failures in Windows Installer

## Problem
The Windows PowerShell installer script (`scripts/install.ps1`) failed on the `windows-latest` GitHub Actions runner during automated deployment testing. Multi-byte UTF-8 emoji characters caused string parsing corruption in PowerShell 5.1, while cross-domain HTTP 302 redirects caused binary asset download failures when using standard download APIs.

## Symptoms
- PowerShell runtime syntax errors during script initialization:
  `TerminatorExpectedAtEndOfString: The string is missing the terminator: ".`
- Asset extraction failures when downloading release zip archives:
  `ArchiveCmdletPathNotFound: Path 'ce-ai-x86_64-pc-windows-msvc.zip' not found.`
- `System.Net.WebClient` or `Invoke-WebRequest` returning truncated response streams or failing silently on cross-domain 302 redirects from GitHub Releases to Amazon S3 storage buckets.

## What Didn't Work
- Using UTF-8 emojis in terminal output or error messages (e.g., `throw "❌ Failed to download"`): PowerShell 5.1 on Windows legacy OEM codepages misinterprets multi-byte UTF-8 sequences as single-byte characters, shifting byte offsets and causing string termination syntax errors.
- Standard `(New-Object System.Net.WebClient).DownloadFile($url, $output)`: Failed to reliably follow cross-domain HTTP 302 redirects from `github.com` release assets to S3 storage without customized headers or redirect handling settings.
- Relying on `curl.exe -L` without explicitly setting custom `User-Agent` headers or verifying returned content, which produced empty files or HTML error pages disguised as ZIP downloads.

## Solution
1. **100% ASCII-Compatible PowerShell Script**: Removed all multi-byte UTF-8 emojis and special characters from `scripts/install.ps1`, replacing them with standard ASCII indicators (e.g., `[ce-ai]`).
2. **Robust `.NET HttpWebRequest` Stream Copy with Auto-Redirect**: Implemented custom download logic using `System.Net.HttpWebRequest` with `AllowAutoRedirect = $true`, explicit `User-Agent` headers, and stream copying.
3. **ZIP Magic Byte Validation**: Added verification checking for the `PK` header (`80, 75` in decimal / `$buffer[0] -eq 80 -and $buffer[1] -eq 75`) before passing the file to `Expand-Archive`.

```powershell
# Robust HTTP Download with Auto-Redirect and Header Handling
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

# Verify ZIP Magic Bytes (PK header: 0x50 0x4B -> 80, 75)
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
```

4. **Version & Asset Manifest Updates**: Bumped patch version to `1.0.8` across `Cargo.toml`, `Formula/ce-ai.rb`, and `CHANGELOG.md`.
5. **CI Status Check Gate**: Enforced mandatory PR status check gates preventing direct unverified merges to `main`.

## Why This Works
- **Tokenizer Stability**: PowerShell 5.1 (the default on Windows Server 2019/2022 runners) calculates character offsets using the active OEM codepage (e.g., CP437 or CP1252) when script files lack BOM or when parsed in legacy environments. Multi-byte UTF-8 emoji sequences (3 to 4 bytes each) confuse the lexer, leading it to miscalculate quotation mark positions and raise `TerminatorExpectedAtEndOfString`. Removing non-ASCII bytes guarantees deterministic tokenization across all Windows codepages.
- **Cross-Domain HTTP 302 Support**: `[System.Net.HttpWebRequest]` with `AllowAutoRedirect = $true` automatically preserves HTTP request context and handles header transitions across 302 redirects from `github.com/releases/download/...` to S3 storage servers.
- **Early Integrity Detection**: Validating the ZIP header magic bytes (`$buffer[0] -eq 80 -and $buffer[1] -eq 75`) catches failed redirects, HTML error pages (e.g., `<!DOCTYPE...`), or 0-byte corrupt files before invoking `Expand-Archive`, providing clean actionable diagnostics.

## Prevention
- **PowerShell 5.1 ASCII Constraint**: Enforce strict 100% ASCII encoding in all `.ps1` installer scripts to remain compatible with legacy PowerShell 5.1 environments.
- **Magic Byte Validation Pattern**: Always validate binary headers (magic bytes) after downloading artifacts before extraction or execution.
- **Explicit Stream Redirection**: Use `.NET HttpWebRequest` with explicit `AllowAutoRedirect` and `User-Agent` headers rather than high-level cmdlets that hide redirect behavior.
- **Strict CI Status Check Policy**: Never merge code or installer script updates directly into `main` without verified passing CI status checks across all matrix runners (Linux, macOS, Windows).
