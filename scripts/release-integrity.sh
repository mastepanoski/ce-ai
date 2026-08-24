#!/usr/bin/env bash
# Release integrity metadata:
# Downloads every release asset for TAG_NAME and emits SHA256SUMS.txt so users can
# verify downloads independently (ISO/IEC 27002 integrity control).
#
# Homebrew distribution is owned by the mastepanoski/homebrew-ce-ai tap, whose
# self-update workflow tracks ce-ai releases autonomously — do NOT keep a formula
# copy in this repository.
#
# Usage (CI sets these automatically):
#   TAG_NAME=v1.21.6 GH_REPO=mastepanoski/ce-ai ./scripts/release-integrity.sh
#
# Fails closed: any missing asset or malformed digest aborts with exit 1.

set -euo pipefail

TAG_NAME="${TAG_NAME:?TAG_NAME is required (e.g. v1.21.6)}"
GH_REPO="${GH_REPO:-mastepanoski/ce-ai}"

ASSETS=(
  "ce-ai-x86_64-apple-darwin.tar.gz"
  "ce-ai-aarch64-apple-darwin.tar.gz"
  "ce-ai-x86_64-unknown-linux-gnu.tar.gz"
  "ce-ai-aarch64-unknown-linux-gnu.tar.gz"
  "ce-ai-x86_64-pc-windows-msvc.zip"
  "ce-ai-aarch64-pc-windows-msvc.zip"
)

BASE_URL="https://github.com/${GH_REPO}/releases/download/${TAG_NAME}"
WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT
SUMS_FILE="$WORK_DIR/SHA256SUMS.txt"
: > "$SUMS_FILE"

echo "==> Computing SHA256 digests for ${TAG_NAME} (${GH_REPO})"
for asset in "${ASSETS[@]}"; do
  tmp_file="$WORK_DIR/$asset"
  curl --fail --silent --show-error --location --output "$tmp_file" "$BASE_URL/$asset"

  if [[ ! -s "$tmp_file" ]]; then
    echo "ERROR: downloaded asset is empty or missing: $asset" >&2
    exit 1
  fi

  digest="$(shasum -a 256 "$tmp_file" | awk '{print $1}')"
  if [[ ! "$digest" =~ ^[0-9a-f]{64}$ ]]; then
    echo "ERROR: invalid digest for $asset: $digest" >&2
    exit 1
  fi

  echo "${digest}  ${asset}" >> "$SUMS_FILE"
  echo "    ${asset} -> ${digest}"
done

cp "$SUMS_FILE" ./SHA256SUMS.txt
entries="$(wc -l < ./SHA256SUMS.txt | tr -d ' ')"
if [[ "$entries" -ne ${#ASSETS[@]} ]]; then
  echo "ERROR: expected ${#ASSETS[@]} digests, wrote $entries" >&2
  exit 1
fi
echo "==> Wrote SHA256SUMS.txt (${entries} entries)"
