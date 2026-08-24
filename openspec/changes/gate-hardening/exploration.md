# Exploration
Issue offered two enforcement options (hard-fail vs greppable-SKIPPED-that-
CI-fails). Hard-fail chosen: simpler, no CI parsing coupling, and the E2E
job runs on ubuntu-latest where Docker is guaranteed — only local Windows
hosts lose the target, with explicit guidance to execute from WSL/Linux/macOS.
The resolve_latest_release graceful-fallback pattern is untouched: this PR
only touches the gate, not product network resilience.
