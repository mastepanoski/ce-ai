//! Docker E2E Gate (DG-1..DG-3).
//! Probes Docker availability, builds test container with Linux release binary, and runs full lifecycle test in isolated HOME.

use std::process::Command;

#[test]
#[ignore = "expensive containerized E2E test; execute via make e2e or cargo test --test e2e -- --ignored"]
fn test_docker_e2e_gate() {
    // DG-3: Probe Docker availability. If unavailable or running on Windows, exit 0 with skip message.
    if cfg!(windows) {
        println!("SKIPPED: Docker E2E gate targets Linux containers; skipping on Windows host environment.");
        return;
    }

    let probe = Command::new("docker").arg("info").output();
    match probe {
        Ok(output) if output.status.success() => {
            println!("Docker daemon is active. Proceeding with E2E gate execution.");
        }
        _ => {
            println!("SKIPPED: Docker daemon is unavailable on host environment.");
            return;
        }
    }

    // 1. Build Docker E2E image (multi-stage compiles Linux release binary)
    println!("[E2E] Building Docker E2E image...");
    let docker_build = Command::new("docker")
        .args(["build", "-t", "ce-ai-e2e", "-f", "Dockerfile.e2e", "."])
        .status()
        .expect("Failed to execute docker build");
    assert!(docker_build.success(), "docker build failed");

    // 2. Run containerized E2E test suite
    println!("[E2E] Running Docker containerized test runner...");
    let docker_run = Command::new("docker")
        .args(["run", "--rm", "ce-ai-e2e"])
        .status()
        .expect("Failed to execute docker run");
    assert!(docker_run.success(), "Docker E2E test execution failed");
}
