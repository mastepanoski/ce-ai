# Design: Real Long-Running `sync --watch` Loop & Drift Recovery

## Architecture

### 1. Polling & Safe Signal Handler Registration
In `src/commands/sync.rs`:
```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Once;

static INIT_CTRLC: Once = Once::new();
static RUNNING: AtomicBool = AtomicBool::new(true);

fn setup_ctrlc() {
    INIT_CTRLC.call_once(|| {
        let _ = ctrlc::set_handler(move || {
            RUNNING.store(false, Ordering::SeqCst);
        });
    });
    RUNNING.store(true, Ordering::SeqCst);
}

pub fn run_watch(
    ctx: &Context,
    args: &Args,
    source_root: &Path,
    manifest: &InstallManifest,
) -> Result<(), CeError> {
    setup_ctrlc();

    let interval = std::time::Duration::from_millis(args.interval_ms.unwrap_or(2000));
    let mut passes = 0;
    let mut repaired_count = 0;

    if !ctx.quiet {
        println!("ce-ai sync --watch: monitoring managed paths for drift...");
    }

    while RUNNING.load(Ordering::SeqCst) {
        if let Some(max) = args.max_passes {
            if passes >= max {
                break;
            }
        }

        std::thread::sleep(interval);
        if !RUNNING.load(Ordering::SeqCst) {
            break;
        }

        match check_and_repair_drift(ctx, source_root, manifest) {
            Ok(true) => {
                repaired_count += 1;
                if !ctx.quiet {
                    println!(
                        "ce-ai sync --watch: repaired drift at {}",
                        chrono::Utc::now().to_rfc3339()
                    );
                }
            }
            Ok(false) => {}
            Err(err) => {
                eprintln!("notice: sync pass error: {err} — retrying on next pass");
            }
        }
        passes += 1;
    }

    if !ctx.quiet {
        println!(
            "ce-ai sync --watch: stopped after {passes} pass(es) ({repaired_count} drift repair(s))."
        );
    }
    Ok(())
}
```

### 2. High-Performance Pre-Check (`check_and_repair_drift`)
To avoid running full disk copies, manifest rewrites, and state serialization on every 2000ms tick when zero drift exists:
```rust
fn check_and_repair_drift(
    ctx: &Context,
    source_root: &Path,
    manifest: &InstallManifest,
) -> Result<bool, CeError> {
    // 1. Pre-check: In-memory hash comparison of managed files
    let managed_dir = ctx.opencode_config_dir.join(MANAGED_DIR);
    let mut desired: BTreeMap<String, String> = BTreeMap::new();
    for (rel, hash) in read_local_tree(source_root)? {
        if MANAGED_PREFIXES.iter().any(|p| rel.starts_with(p)) {
            let managed_rel = rel.trim_start_matches(".opencode/").to_string();
            desired.insert(managed_rel, hash);
        }
    }
    let installed: BTreeMap<String, String> = manifest
        .files
        .iter()
        .map(|f| (f.path.clone(), f.sha256.clone()))
        .collect();

    let plan = diff::diff(&desired, &installed, &managed_dir);
    if plan.actions.is_empty() {
        return Ok(false);
    }

    // 2. Execute sync repair only when actions exist
    if ctx.dry_run {
        println!("plan: dry-run watch detected {} drift action(s)", plan.actions.len());
        for action in &plan.actions {
            let (verb, path) = plan_verb(action);
            println!("plan: {verb} {path}");
        }
        return Ok(false);
    }

    sync_with(ctx, source_root, &manifest.version, manifest.source.clone())?;
    Ok(true)
}
```

### 3. Testing Flags
- `--watch`: Long-running watcher mode.
- `--interval-ms <MS>`: Polling interval in milliseconds (default: 2000).
- `--max-passes <N>`: Maximum iterations (used in unit & integration tests).
