#!/usr/bin/env bash
# Release integrity automation:
#  1) Downloads every release asset for TAG_NAME and emits SHA256SUMS.txt (ISO/IEC 27002
#     integrity control: published digests let users verify downloads independently).
#  2) Regenerates Formula/ce-ai.rb against that tag with the computed checksums, so the
#     Homebrew tap can never drift behind the latest release again.
#
# Usage (CI sets these automatically):
#   TAG_NAME=v1.21.3 GH_REPO=mastepanoski/ce-ai ./scripts/release-integrity.sh
#
# Fails closed: any missing asset, empty digest, or partial download aborts with exit 1.

set -euo pipefail

TAG_NAME="${TAG_NAME:?TAG_NAME is required (e.g. v1.21.3)}"
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

declare -A DIGESTS=()

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

  DIGESTS["$asset"]="$digest"
  echo "${digest}  ${asset}" >> "$SUMS_FILE"
  echo "    ${asset} -> ${digest}"
done

cp "$SUMS_FILE" ./SHA256SUMS.txt
echo "==> Wrote SHA256SUMS.txt ($(wc -l < ./SHA256SUMS.txt | tr -d ' ') entries)"

echo "==> Regenerating Formula/ce-ai.rb for ${TAG_NAME}"
cat > Formula/ce-ai.rb <<EOF
class CeAi < Formula
  desc "Compound Engineering AI plugin installer and manager"
  homepage "https://github.com/${GH_REPO}"
  version "${TAG_NAME#v}"
  license "MIT"

  # Managed by scripts/release-integrity.sh — regenerate with:
  #   TAG_NAME=${TAG_NAME} GH_REPO=${GH_REPO} ./scripts/release-integrity.sh
  # Do not edit URLs or checksums by hand.
  on_macos do
    if Hardware::CPU.intel?
      url "${BASE_URL}/ce-ai-x86_64-apple-darwin.tar.gz"
      sha256 "${DIGESTS[ce-ai-x86_64-apple-darwin.tar.gz]}"
    else
      url "${BASE_URL}/ce-ai-aarch64-apple-darwin.tar.gz"
      sha256 "${DIGESTS[ce-ai-aarch64-apple-darwin.tar.gz]}"
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "${BASE_URL}/ce-ai-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "${DIGESTS[ce-ai-x86_64-unknown-linux-gnu.tar.gz]}"
    else
      url "${BASE_URL}/ce-ai-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "${DIGESTS[ce-ai-aarch64-unknown-linux-gnu.tar.gz]}"
    end
  end

  def install
    bin.install "ce-ai"
  end

  test do
    assert_match "ce-ai", shell_output("#{bin}/ce-ai --version")
  end
end
EOF

if ! grep -q "$TAG_NAME" Formula/ce-ai.rb; then
  echo "ERROR: formula regeneration failed to embed $TAG_NAME" >&2
  exit 1
fi

formula_hashes=$(grep -c 'sha256 "' Formula/ce-ai.rb || true)
if [[ "$formula_hashes" -ne 4 ]]; then
  echo "ERROR: expected 4 checksums in formula, found $formula_hashes" >&2
  exit 1
fi

ruby -c Formula/ce-ai.rb >/dev/null || { echo "ERROR: formula syntax invalid" >&2; exit 1; }
echo "==> Formula regenerated and validated (4 checksums, ruby syntax OK)"
