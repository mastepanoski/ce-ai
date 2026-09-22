# Exploration: Local Decision Engine Architecture (Kev vs Laya-MLX)

## Architectural Investigation

The purpose of this exploration is to evaluate local decision runtimes to serve as offline, zero-cost, private backends for `ce-ai`'s System 1 Decision Engine.

### Option 1: Standardizing Exclusively on `jaredpalmer/kev`
- **Architecture**: Decoder LLM (Qwen3.5-0.8B, 4B, 9B) with rank-16 LoRA adapter + pointer head. Hybrid recurrent linear attention (Gated DeltaNet) + causal self-attention.
- **Portability**: Cross-platform (CUDA, ROCm, MPS, MLX, CPU).
- **Transport**: Native built-in HTTP server (`python -m kev.serve`) serving `POST /v1/systemone` and `GET /v1/models`.
- **Latency**:
  - Kev-0.8B on M5: ~149 ms cold / ~28 ms cached prefix.
  - Kev-4B on M5: ~721 ms cold / ~136 ms cached prefix.
  - Kev-4B on CUDA (L4): ~118 ms.
- **Memory Footprint**: 2 GB to 10 GB+ RAM.
- **Context Window**: 8,192 tokens.
- **Pros**: Runs on any OS (Linux, Windows, macOS), 8k token context, directly implements TypeSafe System One wire API.
- **Cons**: 100–700ms latency is noticeable when running 20+ tool-risk checks in agent loops; high memory consumption for lightweight developer laptops (e.g. 8GB/16GB MacBook Air).

### Option 2: Standardizing Exclusively on `mizorewww/laya-mlx`
- **Architecture**: Dedicated bidirectional encoder (ModernBERT-large 421M, mmBERT-base 322M) with custom decision heads.
- **Portability**: Strictly macOS Apple Silicon (`aarch64-apple-darwin`) via Apple MLX.
- **Transport**: Python library and batch CLI (`laya-mlx predict`). Needs a lightweight HTTP/UDS daemon sidecar for daemonized invocation.
- **Latency**: 7.4 ms (Multilingual) to 13.4 ms (ModernBERT) on M3 Max. Sub-perceptual.
- **Memory Footprint**: 687 MiB to 943 MiB peak RAM.
- **Context Window**: 512 – 1,024 tokens.
- **Pros**: Blazing fast (< 15ms), extremely low memory footprint, zero battery strain.
- **Cons**: Completely unavailable on Linux and Windows; short context window.

### Option 3: Tiered Dual-Provider Strategy (Selected Option)
By supporting both `kev` and `laya`:
1. `ce-ai` leverages the fact that both engines implement the exact same **TypeSafe System One question paradigm** (`noul`, `choice`, `score`).
2. Linux, Windows, DevContainer, and Cloud-runner developers use **`kev`** for cross-platform local decision-making and 8k token capacity.
3. Apple Silicon developers who want maximum responsiveness in autonomous turn loops use **`laya`** for 7–14ms latency and sub-1GB RAM usage.
4. Users who prefer cloud execution without running local daemons retain **`jev`**.

---

## Architectural Tradeoffs Matrix

| Feature | `kev` | `laya-mlx` | Selected Approach |
| :--- | :--- | :--- | :--- |
| **OS Compatibility** | Linux, macOS, Windows | macOS Apple Silicon only | **Dual**: `kev` universal, `laya` specialized |
| **Median Latency** | 120–700 ms | 7–14 ms | User selects based on speed vs context requirement |
| **Memory Usage** | 2–10 GB | < 1 GB | Laya for low RAM, Kev for workstations |
| **Context Window** | 8,192 tokens | 1,024 tokens | Kev for large diffs, Laya for rapid briefs |
| **Server Protocol** | `POST /v1/systemone` | `POST /v1/systemone` (daemon) | Unified `SystemOneWireRequest` serializer |
