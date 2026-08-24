//! Docker E2E Gate (DG-1..DG-3).
//! Probes Docker availability, builds a test container with the Linux
//! release binary, and runs the full lifecycle suite in an isolated HOME.
//!
//! Fail-closed by design (#165): a gate that passes by skipping is not a
//! gate. Docker absence is a hard failure with remediation guidance.

use std::process::Command;

#[test]
#[ignore = "expensive containerized E2E test; execute via make e2e or cargo test --test e2e -- --ignored"]
fn test_docker_e2e_gate() {
    // DG-3 fail-closed probe (#165). The E2E gate must EXECUTE — never skip.
    if cfg!(windows) {
        panic!(
            "FAILED-TO-RUN: the Docker E2E gate targets Linux containers; execute it from a Linux or macOS host with a running Docker daemon (`make e2e`)."
        );
    }

    match Command::new("docker").arg("info").output() {
        Ok(output) if output.status.success() => {
            println!("[E2E] Docker daemon active — executing containerized gate.");
        }
        Ok(output) => panic!(
            "FAILED-TO-RUN: Docker daemon unreachable (docker info exited with {}). Start the Docker daemon and re-run `make e2e`. A gate that skips is not a gate.",
            output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".into())
        ),
        Err(err) => panic!(
            "FAILED-TO-RUN: docker CLI not found ({err}). Install Docker and re-run `make e2e`."
        ),
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

    println!("[E2E] GATE PASSED");
}
