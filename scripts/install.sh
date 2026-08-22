#!/usr/bin/env bash
# Universal One-Line Installer Script for ce-ai
# Resolves operating system, architecture, fetches latest release asset, and installs to ~/.ce-ai/bin/

set -e

COLOR_BLUE="\033[0;34m"
COLOR_GREEN="\033[0;32m"
COLOR_RED="\033[0;31m"
COLOR_RESET="\033[0m"

echo -e "${COLOR_BLUE}🚀 Installing ce-ai (Compound Engineering AI CLI)...${COLOR_RESET}"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
  x86_64|amd64)
    ARCH_NAME="x86_64"
    ;;
  aarch64|arm64)
    ARCH_NAME="aarch64"
    ;;
  *)
    echo -e "${COLOR_RED}❌ Unsupported architecture: ${ARCH}${COLOR_RESET}"
    exit 1
    ;;
esac

case "$OS" in
  linux)
    TARGET="${ARCH_NAME}-unknown-linux-gnu"
    ASSET_NAME="ce-ai-${TARGET}.tar.gz"
    ;;
  darwin)
    TARGET="${ARCH_NAME}-apple-darwin"
    ASSET_NAME="ce-ai-${TARGET}.tar.gz"
    ;;
  *)
    echo -e "${COLOR_RED}❌ Unsupported OS: ${OS}. For Windows, use install.ps1${COLOR_RESET}"
    exit 1
    ;;
esac

INSTALL_DIR="$HOME/.ce-ai/bin"
mkdir -p "$INSTALL_DIR"

STATIC_URL="https://github.com/mastepanoski/ce-ai/releases/latest/download/${ASSET_NAME}"
TMP_DIR=$(mktemp -d)
TMP_FILE="${TMP_DIR}/${ASSET_NAME}"

attempt_download() {
  _url="$1"
  rm -f "$TMP_FILE"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$_url" -o "$TMP_FILE" 2>/dev/null || true
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$TMP_FILE" "$_url" 2>/dev/null || true
  else
    echo -e "${COLOR_RED}❌ Neither curl nor wget is available.${COLOR_RESET}"
    exit 1
  fi
  # Valid asset: exists, non-trivial size, and (for zips/tars) not an error page.
  [ -s "$TMP_FILE" ] && [ "$(wc -c < "$TMP_FILE")" -gt 1000 ]
}

resolve_latest_url() {
  # Prefer the API-resolved asset URL; fall back to the static redirect URL.
  if command -v curl >/dev/null 2>&1; then
    _api_url=$(curl -fsSL "https://api.github.com/repos/mastepanoski/ce-ai/releases/latest" 2>/dev/null | /usr/bin/grep -o "browser_download_url.*${ASSET_NAME}\"" | /usr/bin/head -1 | /usr/bin/cut -d'"' -f3)
  else
    _api_url=$(wget -qO- "https://api.github.com/repos/mastepanoski/ce-ai/releases/latest" 2>/dev/null | /usr/bin/grep -o "browser_download_url.*${ASSET_NAME}\"" | /usr/bin/head -1 | /usr/bin/cut -d'"' -f3)
  fi
  if [ -n "$_api_url" ]; then
    echo "$_api_url"
  else
    echo "$STATIC_URL"
  fi
}

echo "📦 Fetching latest release asset: ${ASSET_NAME}..."
DOWNLOADED=false
ATTEMPTED=""

# Retry loop: covers transient 404s during release publication windows.
i=1
while [ $i -le 3 ] && [ "$DOWNLOADED" = false ]; do
  if [ $i -gt 1 ]; then
    BACKOFF=$((10 * (i - 1)))
    echo "⏳ Asset not ready yet; retrying in ${BACKOFF}s (attempt ${i}/3)..."
    sleep "$BACKOFF"
  fi
  URL=$(resolve_latest_url)
  ATTEMPTED="${ATTEMPTED}${URL}\n"
  echo "📥 Downloading from ${URL}..."
  if attempt_download "$URL"; then DOWNLOADED=true; fi
  i=$((i + 1))
done

# Fallback: scan recent releases when latest lacks the platform asset.
if [ "$DOWNLOADED" = false ]; then
  echo "🔍 Scanning recent releases for ${ASSET_NAME}..."
  if command -v curl >/dev/null 2>&1; then
    FALLBACK_URLS=$(curl -fsSL "https://api.github.com/repos/mastepanoski/ce-ai/releases?per_page=5" 2>/dev/null | /usr/bin/grep -o "browser_download_url[^\"]*\"[^\"]*${ASSET_NAME}\"" | /usr/bin/cut -d'"' -f3)
  else
    FALLBACK_URLS=$(wget -qO- "https://api.github.com/repos/mastepanoski/ce-ai/releases?per_page=5" 2>/dev/null | /usr/bin/grep -o "browser_download_url[^\"]*\"[^\"]*${ASSET_NAME}\"" | /usr/bin/cut -d'"' -f3)
  fi
  for URL in $FALLBACK_URLS; do
    case "$ATTEMPTED" in *"$URL"*) continue ;; esac
    echo "📥 Falling back to recent release: ${URL}..."
    ATTEMPTED="${ATTEMPTED}${URL}\n"
    if attempt_download "$URL"; then DOWNLOADED=true; break; fi
  done
fi

if [ "$DOWNLOADED" != true ]; then
  echo -e "${COLOR_RED}❌ Failed to download a valid release asset after retries and fallback. Attempted URLs:${COLOR_RESET}"
  printf "$ATTEMPTED"
  exit 1
fi

echo "📂 Extracting to ${INSTALL_DIR}..."
tar -xzf "$TMP_FILE" -C "$INSTALL_DIR"
chmod +x "${INSTALL_DIR}/ce-ai"
rm -rf "$TMP_DIR"

echo -e "${COLOR_GREEN}✅ ce-ai successfully installed to ${INSTALL_DIR}/ce-ai${COLOR_RESET}"
echo ""
echo "To add ce-ai to your PATH, add this line to your ~/.zshrc or ~/.bashrc:"
echo -e "  ${COLOR_BLUE}export PATH=\"\$HOME/.ce-ai/bin:\$PATH\"${COLOR_RESET}"
