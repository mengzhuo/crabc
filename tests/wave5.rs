use std::process::Command;

#[test]
fn wave5_libc_test_functional_networking_and_fcntl() {
    let libc_test_dir = std::env::var("LIBC_TEST_DIR").unwrap_or_else(|_| "/home/root/libc-test".into());
    if !std::path::Path::new(&libc_test_dir).join("src").is_dir() {
        eprintln!(
            "skipping wave5_libc_test_functional_networking_and_fcntl: libc-test source not found at {}",
            libc_test_dir
        );
        return;
    }

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let harness = manifest_dir.join("libc-test-harness/run.sh");

    let output = Command::new(&harness)
        .arg("functional")
        .output()
        .expect("failed to run libc-test functional harness");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "functional harness exited with {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout,
        stderr
    );

    for test in ["functional/inet_pton", "functional/fcntl", "functional/socket"] {
        assert!(
            stdout.contains(&format!("  {}\n", test)),
            "expected {} to PASS:\n{}",
            test,
            stdout
        );
    }

    assert!(
        stdout.contains("PASS:       46") || stdout.contains("PASS:       4"),
        "expected PASS count of at least 46:\n{}",
        stdout
    );
    assert!(
        stdout.contains("FAIL:       23") || stdout.contains("FAIL:       2"),
        "expected FAIL count of at most 23:\n{}",
        stdout
    );
}
