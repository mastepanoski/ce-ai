# Design: Installer Download Resilience

## install.ps1

Restructure resolve+download into a function `Attempt-Download($Url)` returning
bool (valid zip written to `$TempZip`). Flow:

```
for attempt in 1..3:
    resolve latest via API (keep existing fallback to static URL)
    if Attempt-Download(latest_url): break
    if attempt < 3: Start-Sleep (10s, 20s)
if still invalid:
    releases = Invoke-RestMethod ".../releases?per_page=5"
    foreach release with matching asset name (newest first):
        if Attempt-Download(browser_download_url): break
if still invalid: Write-Error listing attempted URLs; exit 1
```

`Test-ValidZipFile` reused unchanged. TLS12 + User-Agent headers preserved.

## install.sh

Mirror semantics in POSIX-ish bash:

```
attempt_download() { curl/wget -> TMP_FILE; verify size > 1000 bytes; }
for i in 1 2 3: resolve latest (API w/ grep for browser_download_url of ASSET_NAME) ; attempt ; sleep 10*i between
fallback: curl -s ".../releases?per_page=5" | grep browser_download_url | grep ASSET_NAME | head -1
fail loud after exhaustion
```

Size verification (`[ -s file ]` + byte floor) replaces blind success.

## Out of design scope

No changes to release.yml ordering, no checksum handling.
