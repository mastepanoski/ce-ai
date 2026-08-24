# OpenSpec Design: Release v0.8.0 Release Pipeline & Installer Architecture

## GitHub Actions Release Workflow (`.github/workflows/release.yml`)

```yaml
name: Release Pipeline
on:
  push:
    tags:
      - 'v*'

jobs:
  publish-binaries:
    name: Build & Publish ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest
          - target: aarch64-pc-windows-msvc
            os: windows-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build Binary
        run: cargo build --release --target ${{ matrix.target }}
      - name: Archive Assets & Compute SHA256
        run: |
          tar -czvf ce-ai-${{ matrix.target }}.tar.gz -C target/${{ matrix.target }}/release ce-ai
          shasum -a 256 ce-ai-${{ matrix.target }}.tar.gz > ce-ai-${{ matrix.target }}.tar.gz.sha256
      - name: Upload Release Asset
        uses: softprops/action-gh-release@v2
        with:
          files: |
            ce-ai-${{ matrix.target }}.tar.gz
            ce-ai-${{ matrix.target }}.tar.gz.sha256
```

## Universal Installer Script (`scripts/install.sh`)

```bash
#!/usr/bin/env bash
set -e

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${ARCH}-unknown-linux-gnu"
if [ "$OS" = "darwin" ]; then
  TARGET="${ARCH}-apple-darwin"
fi

URL="https://github.com/mastepanoski/ce-ai/releases/latest/download/ce-ai-${TARGET}.tar.gz"
INSTALL_DIR="$HOME/.ce-ai/bin"

mkdir -p "$INSTALL_DIR"
curl -fsSL "$URL" | tar -xz -C "$INSTALL_DIR"
echo "✅ ce-ai successfully installed to $INSTALL_DIR/ce-ai"
```
