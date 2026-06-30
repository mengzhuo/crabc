use std::process::Command;

#[test]
fn lrand48_musl_vectors_under_libldso() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = manifest_dir.join("tests/fixtures");
    let include = manifest_dir.join("include");

    let build_status = Command::new("cargo")
        .args(["build", "-p", "ldso"])
        .status()
        .expect("failed to run cargo build -p ldso");
    assert!(build_status.success(), "cargo build -p ldso failed");

    let build_status = Command::new("cargo")
        .args(["build", "-p", "libc"])
        .status()
        .expect("failed to run cargo build -p libc");
    assert!(build_status.success(), "cargo build -p libc failed");

    let ldso_path = manifest_dir.join("target/debug/libldso.so");
    let libc_path = manifest_dir.join("target/debug/libc.so");
    assert!(ldso_path.exists(), "libldso.so not found");
    assert!(libc_path.exists(), "libc.so not found");

    let src = fixtures.join("lrand48_test.c");
    let bin = fixtures.join("lrand48_test");
    let status = Command::new("musl-gcc")
        .args([
            "-fPIE",
            "-pie",
            "-D_GNU_SOURCE",
            "-I",
            include.to_str().unwrap(),
            "-Wl,--dynamic-linker",
            ldso_path.to_str().unwrap(),
            "-L",
            manifest_dir.join("target/debug").to_str().unwrap(),
            src.to_str().unwrap(),
            "-Wl,--allow-shlib-undefined",
            "-lc",
            "-o",
            bin.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc for lrand48_test");
    assert!(status.success(), "musl-gcc lrand48_test compilation failed");

    let output = Command::new(&bin)
        .env(
            "LD_LIBRARY_PATH",
            manifest_dir.join("target/debug").to_str().unwrap(),
        )
        .output()
        .expect("failed to run lrand48_test");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "lrand48_test exited with {:?}, stderr: {}",
        output.status.code(),
        stderr
    );
    assert_eq!(stdout.trim(), "lrand48 ok", "expected lrand48 ok in: {}", stdout);
}
