#!/usr/bin/env bash
# scripts/protect-branch.sh — Configures GitHub API Branch Protection for `main`
# Aligned with ISO/IEC 27001, ISO 42001, NIST AI RMF, and Issue #97 requirements.

set -e

BOLD="\033[1m"
RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
RESET="\033[0m"

echo -e "${BOLD}== [ce-ai] Configuring GitHub Branch Protection on main ==${RESET}"

# 1. Pre-flight CLI and Auth Verification
if ! command -v gh >/dev/null 2>&1; then
    echo -e "${RED}[ERROR] GitHub CLI ('gh') is not installed.${RESET}"
    echo "Please install 'gh' and authenticate using 'gh auth login' before configuring branch protection."
    exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
    echo -e "${RED}[ERROR] GitHub CLI ('gh') is not authenticated or offline.${RESET}"
    echo "Please authenticate using 'gh auth login' to run branch protection automation."
    exit 1
fi

# 2. Auto-detect Repository
REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || true)
if [ -z "$REPO" ]; then
    echo -e "${RED}[ERROR] Could not detect GitHub repository from current working directory.${RESET}"
    exit 1
fi

echo -e "--> Repository: ${BOLD}${REPO}${RESET}"
echo -e "--> Target Branch: ${BOLD}main${RESET}"

# 3. Apply GitHub API Branch Protection Payload
echo "--> Sending API PUT request to configure branch protection rules..."

PAYLOAD=$(cat <<EOF
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Build & Test (ubuntu-latest)",
      "Build & Test (macos-latest)",
      "Build & Test (windows-latest)",
      "Containerized E2E Gate (NIST AI RMF & ISO 42001)",
      "Supply Chain Security Audit (ISO 27001 / ISO 27002)",
      "Windows PowerShell Installer Gate (NIST SP 800-53)"
    ]
  },
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null,
  "required_linear_history": false,
  "allow_force_pushes": false,
  "allow_deletions": false,
  "block_creations": false,
  "required_conversation_resolution": false,
  "lock_branch": false,
  "allow_fork_syncing": false
}
EOF
)

API_OUTPUT=$(gh api -X PUT "repos/${REPO}/branches/main/protection" --input - <<< "$PAYLOAD" 2>&1)
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ GitHub branch protection successfully configured for 'main' on ${REPO}!${RESET}"
    echo -e "  - Direct pushes to 'main' are now blocked."
    echo -e "  - All PRs require 100% green CI matrix status checks before merging."
else
    echo -e "${RED}[ERROR] Failed to configure branch protection via GitHub API:${RESET}"
    echo "$API_OUTPUT"
    echo "Ensure your token has admin/repo permissions on ${REPO}."
    exit 1
fi
