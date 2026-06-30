use std::process::Command;

#[test]
fn wave4_libc_test_regression_zero_failures() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let harness = manifest_dir.join("libc-test-harness/run.sh");

    let output = Command::new(&harness)
        .arg("regression")
        .output()
        .expect("failed to run libc-test regression harness");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "regression harness exited with {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout,
        stderr
    );

    assert!(
        stdout.contains("FAIL:       0"),
        "expected zero FAIL in regression harness summary:\n{}",
        stdout
    );
}
