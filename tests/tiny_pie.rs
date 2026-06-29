use std::process::Command;

#[test]
fn loader_runs_tiny_pie() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let tiny_src = fixtures.join("tiny.c");
    let tiny_bin = fixtures.join("tiny");

    let status = Command::new("musl-gcc")
        .args([
            "-fPIE",
            "-pie",
            "-nostdlib",
            "-nostartfiles",
            tiny_src.to_str().unwrap(),
            "-o",
            tiny_bin.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc");
    assert!(status.success(), "musl-gcc compilation failed");

    let loader_bin = env!("CARGO_BIN_EXE_loader");
    let output = Command::new(loader_bin)
        .arg(&tiny_bin)
        .output()
        .expect("failed to run loader");

    assert!(
        output.status.success(),
        "loader exited with {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello\n");
}
