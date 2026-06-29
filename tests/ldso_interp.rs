use std::process::Command;

#[test]
fn ldso_runs_tiny_pie() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = manifest_dir.join("tests/fixtures");
    let tiny_src = fixtures.join("tiny.c");
    let tiny_bin = fixtures.join("tiny_ldso");

    let build_status = Command::new("cargo")
        .args(["build", "-p", "ldso"])
        .status()
        .expect("failed to run cargo build -p ldso");
    assert!(build_status.success(), "cargo build -p ldso failed");

    let ldso_path = manifest_dir.join("target/debug/libldso.so");
    assert!(ldso_path.exists(), "libldso.so not found at {}", ldso_path.display());

    let status = Command::new("musl-gcc")
        .args([
            "-fPIE",
            "-pie",
            "-nostdlib",
            "-nostartfiles",
            "-Wl,--dynamic-linker",
            ldso_path.to_str().unwrap(),
            tiny_src.to_str().unwrap(),
            "-Wl,--allow-shlib-undefined",
            "-o",
            tiny_bin.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc");
    assert!(status.success(), "musl-gcc compilation failed");

    let output = Command::new(&tiny_bin)
        .output()
        .expect("failed to run tiny_ldso");

    assert!(
        output.status.success(),
        "tiny_ldso exited with {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello\n");
}
