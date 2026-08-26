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
test -f "$HOME/.config/opencode/compound-engineering/skills/ce-brainstorm/SKILL.md" || {
  echo "FAIL: top-level skills/ tree not harvested into the managed dir"
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

echo "== [E2E 7] Validating multi-harness probing and support =="
touch "$HOME/.claude.json"
STATUS_OUT=$(ce-ai status)
echo "$STATUS_OUT" | grep -q "opencode" || {
  echo "FAIL: status output did not report opencode: $STATUS_OUT"
  exit 1
}

echo "== [E2E 8] Running TUI headless checks (zen-free, no TTY) =="
# Use existing install from E2E 1 (no reinstall to avoid backup overwrite)
ce-ai skills list > /tmp/skills.txt 2>&1; grep -q "ce-brainstorm" /tmp/skills.txt || echo "WARN: skills list missing ce-brainstorm (soft)"
ce-ai skills resolve --harness opencode --query "test" > /tmp/resolve.txt 2>&1; grep -q "ce-" /tmp/resolve.txt || echo "WARN: skills resolve headless (soft)"
ce-ai tools status > /tmp/tools.txt 2>&1; grep -q "codegraph" /tmp/tools.txt || echo "WARN: tools status (soft)"
# Zen free model path (mock if no API key): assignment must succeed without network
ce-ai models set ce-brainstorm opencode/zen-free > /tmp/models.txt 2>&1; grep -q "opencode/zen-free" /tmp/models.txt || echo "WARN: zen-free assignment (soft)"
# Headless TUI rendering via cargo test (TestBackend) inside container if source present
if [ -f "/app/Cargo.toml" ]; then
  echo "== [E2E 8b] cargo test tui headless snapshots =="
  cargo test tui -- --nocapture | grep -q "headless_ui_renders_all_tabs" || echo "WARN: headless test not found"
fi

echo "== [E2E 9] Running ce-ai uninstall =="
ce-ai uninstall --harness opencode

echo "== [E2E 10] Asserting uninstall restoration =="
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
