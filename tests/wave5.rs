use std::process::Command;

fn parse_count(stdout: &str, label: &str) -> i32 {
    stdout
        .lines()
        .find(|l| l.trim_start().starts_with(label))
        .and_then(|l| {
            l.splitn(2, label)
                .nth(1)
                .map(|v| v.trim().parse::<i32>().unwrap_or(0))
        })
        .unwrap_or(0)
}

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

    let pass = parse_count(&stdout, "PASS:");
    let fail = parse_count(&stdout, "FAIL:");
    assert!(pass >= 58, "expected PASS count of at least 58, got {}", pass);
    assert!(fail <= 11, "expected FAIL count of at most 11, got {}", fail);
}
