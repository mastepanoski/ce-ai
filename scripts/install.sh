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

DOWNLOAD_URL="https://github.com/mastepanoski/ce-ai/releases/latest/download/${ASSET_NAME}"
TMP_DIR=$(mktemp -d)
TMP_FILE="${TMP_DIR}/${ASSET_NAME}"

echo "📦 Fetching latest release asset: ${ASSET_NAME}..."
if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$DOWNLOAD_URL" -o "$TMP_FILE"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TMP_FILE" "$DOWNLOAD_URL"
else
  echo -e "${COLOR_RED}❌ Neither curl nor wget is available.${COLOR_RESET}"
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
