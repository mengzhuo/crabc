use std::process::Command;

#[test]
fn ldso_sets_up_tls() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = manifest_dir.join("tests/fixtures");
    let tlstest_src = fixtures.join("tlstest.c");
    let tlstest_bin = fixtures.join("tlstest");

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
            tlstest_src.to_str().unwrap(),
            "-o",
            tlstest_bin.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc for tlstest");
    assert!(status.success(), "musl-gcc tlstest compilation failed");

    let output = Command::new(&tlstest_bin)
        .output()
        .expect("failed to run tlstest");

    assert!(
        output.status.success(),
        "tlstest exited with {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
}
