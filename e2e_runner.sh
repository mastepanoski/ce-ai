#!/usr/bin/env bash
set -euo pipefail

echo "== [E2E] Starting ce-ai E2E validation in isolated environment =="

export HOME=/tmp/ce-ai-home
rm -rf "$HOME"
mkdir -p "$HOME/.config/opencode"

# 1. Setup pre-existing user config
cat <<'EOF' > "$HOME/.config/opencode/opencode.json"
{
  "plugin": ["pre-existing-plugin"],
  "skills": {
    "paths": ["/usr/share/skills"]
  }
}
EOF

echo "== [E2E 1] Running ce-ai install =="
ce-ai install --harness opencode --source /tmp/ce-source

echo "== [E2E 2] Asserting install outcome =="
grep -q "compound-engineering/plugins/compound-engineering.js" "$HOME/.config/opencode/opencode.json" || {
  echo "FAIL: plugin entry not added to opencode.json"
  exit 1
}
grep -q "compound-engineering/skills" "$HOME/.config/opencode/opencode.json" || {
  echo "FAIL: skills path not added to opencode.json"
  exit 1
}
test -f "$HOME/.config/opencode/compound-engineering/install-manifest.json" || {
  echo "FAIL: install-manifest.json missing"
  exit 1
}

echo "== [E2E 3] Running ce-ai sync --dry-run =="
SYNC_OUT=$(ce-ai sync --dry-run)
echo "$SYNC_OUT" | grep -q -E "(plan: no changes|up-to-date)" || {
  echo "FAIL: sync --dry-run output did not report no changes: $SYNC_OUT"
  exit 1
}

echo "== [E2E 4] Running ce-ai models set =="
ce-ai models set ce-brainstorm opencode-go/kimi-k2.6

echo "== [E2E 5] Asserting model assignment =="
grep -q "ce-brainstorm" "$HOME/.config/opencode/opencode.json" || {
  echo "FAIL: ce-brainstorm not found in opencode.json"
  exit 1
}
grep -q "opencode-go/kimi-k2.6" "$HOME/.config/opencode/opencode.json" || {
  echo "FAIL: model opencode-go/kimi-k2.6 not found in opencode.json"
  exit 1
}

echo "== [E2E 6] Checking ce-ai status =="
STATUS_OUT=$(ce-ai status)
echo "$STATUS_OUT" | grep -q "opencode" || {
  echo "FAIL: status output did not report opencode: $STATUS_OUT"
  exit 1
}

echo "== [E2E 7] Running ce-ai uninstall =="
ce-ai uninstall --harness opencode

echo "== [E2E 8] Asserting uninstall restoration =="
grep -q "pre-existing-plugin" "$HOME/.config/opencode/opencode.json" || {
  echo "FAIL: pre-existing-plugin lost after uninstall"
  exit 1
}
if grep -q "compound-engineering/plugins/compound-engineering.js" "$HOME/.config/opencode/opencode.json"; then
  echo "FAIL: compound-engineering plugin entry still present after uninstall"
  exit 1
fi
if [ -d "$HOME/.config/opencode/compound-engineering" ]; then
  echo "FAIL: managed directory still present after uninstall"
  exit 1
fi

echo "== [E2E] ALL GATES PASSED SUCCESSFULLY! =="
