# Proposal: Local Decision Engine Providers (Kev & Laya-MLX)

## Problem Statement

The `ce-ai` Pluggable Decision Engine (System 1) currently depends predominantly on two provider modalities:
1. **Jev (`jev`)**: Remote cloud provider hosted by TypeSafe AI. While accurate, it introduces network round-trips (~500–1200ms latency), incurs financial cost against user spending budgets, and requires transmitting task context over the public internet.
2. **Mock (`mock`)**: Offline testing provider that returns deterministic canned responses, unsuitable for dynamic semantic classification, real risk gating, or context-aware skill routing in production workflows.

Developers increasingly demand **100% offline, zero-latency, private, and free local inference** for agent decision-making. However, a single local engine cannot optimally serve all hardware environments:
- **Apple Silicon Macs** need ultra-low latency (< 15ms) and minimal memory footprints (< 1GB RAM) to run high-frequency turn checks without draining battery or causing memory pressure.
- **Linux, Windows, and CUDA/ROCm Workstations** need a cross-platform engine with broad architecture support, large context windows (up to 8,192 tokens), and deep reasoning capabilities for intricate policy analysis.

To solve this, `ce-ai` must introduce a **Tiered Local Decision Provider Architecture** supporting both **`kev`** ([jaredpalmer/kev](https://github.com/jaredpalmer/kev)) and **`laya-mlx`** ([mizorewww/laya-mlx](https://github.com/mizorewww/laya-mlx)) alongside the existing `jev` and `mock` providers (Issue [#403](https://github.com/mastepanoski/ce-ai/issues/403)).

---

## In-Scope

1. **Pluggable Local Provider Implementations**:
   - **`KevProvider` (`src/decisions/kev.rs`)**:
     - Connects to local/remote Kev HTTP daemon (default: `http://127.0.0.1:8009/v1`).
     - Wire protocol: implements TypeSafe System One API schema (`POST /v1/systemone` and `GET /v1/models`).
     - Translates domain `DecisionRequest` (`Boolean`, `Choice`, `Score`) into `noul`, `choice`, and `score` questions.
     - Provides health checks with fallback handling.
   - **`LayaMlxProvider` (`src/decisions/laya.rs`)**:
     - Specialized for Apple Silicon (`aarch64-apple-darwin`) with native MLX inference.
     - Connects to local HTTP daemon (default: `http://127.0.0.1:8080/v1`) or Unix Domain Socket (`~/.config/ce-ai/laya.sock`).
     - Maps domain decision queries to bidirectional encoder decision heads with sub-15ms latency.

2. **State & Configuration Schema (`state.json`)**:
   - Extend `DecisionsConfig` with `KevConfig` and `LayaConfig`:
     - `kev`: `{ endpoint, model, timeout_ms }`
     - `laya`: `{ endpoint, socket_path, model, timeout_ms }`
   - Atomic state persistence using `crate::state::write_atomic`.

3. **CLI Setup Presets & Provider Management**:
   - `ce-ai decisions setup`:
     - Add `--provider kev` and `--preset kev`.
     - Add `--provider laya` and `--preset laya` / `--preset mlx`.
     - Allow specifying `--endpoint <url>` to target custom ports or remote hosts.
   - Support `ce-ai decisions test` verifying connectivity against active provider.

4. **Health Diagnostics in `ce-ai doctor`**:
   - Probe active local provider endpoints.
   - Report measured round-trip latency.
   - On macOS Apple Silicon: suggest `laya` or `kev` with clear startup commands.
   - On Linux / Windows: check platform compatibility (flagging that `laya-mlx` is macOS-only) and verify `kev`.

5. **Graceful Fallback & Zero-Crash Resilience**:
   - If local daemon is stopped or unreachable, smoothly fallback to deterministic defaults (exit code 0, fallback indicator active).

---

## Out-of-Scope

1. **Packaging Python/MLX Binaries Inside Rust**:
   - `ce-ai` will not bundle Python, PyTorch, or MLX inside the Rust binary. The local engines run as standalone developer services (`python -m kev.serve` or `laya-mlx`), and `ce-ai` connects over standard local HTTP/UDS.
2. **Model Training & Fine-Tuning CLI**:
   - Fine-tuning weights is delegated to the respective upstreams (`kev.train`, `modal_app.py`, or `laya`).
3. **Third-Party Model Arbitrary Ingestion**:
   - Only models exposing the TypeSafe System One specification (`choice`, `score`, `noul`) are routed.

---

## Risk Evaluation

| Risk | Likelihood | Impact | Mitigation Strategy |
| :--- | :---: | :---: | :--- |
| **Local Daemon Not Running** | High | Low | Graceful fallback to deterministic policies; clear diagnostic output in `ce-ai doctor` with exact startup commands. |
| **User Attempts to Run Laya on Linux/Windows** | Medium | Medium | Platform gate in `ce-ai doctor` and `decisions setup`: inform user that `laya-mlx` requires macOS Apple Silicon, and auto-recommend `kev`. |
| **Timeout or Latency Spike with Heavy LLMs (Kev-9B)** | Medium | Medium | Configurable `timeout_ms` (default 2000ms for Kev, 250ms for Laya) with non-blocking fallback. |
| **Configuration Clobbering in state.json** | Low | High | Use atomic serialization (`write_atomic`) with default deserializers for existing configurations. |

---

## Success Criteria

1. Running `ce-ai decisions setup --provider kev` configures `state.json` and connects to `http://127.0.0.1:8009/v1/systemone`.
2. Running `ce-ai decisions setup --provider laya` configures `state.json` and targets the low-latency Apple Silicon daemon.
3. `ce-ai doctor` cleanly identifies and probes `kev` and `laya` health, providing actionable setup hints if unreachable.
4. 100% test coverage across request translation, response parsing, and error fallback.
5. All CI matrix jobs (Linux, macOS, Windows) pass green.
