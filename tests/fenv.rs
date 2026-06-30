use std::process::Command;

#[test]
fn fenv_round_and_except_under_libldso() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = manifest_dir.join("tests/fixtures");
    let include = manifest_dir.join("include");

    let ldso_path = manifest_dir.join("target/debug/libldso.so");
    let lib_dir = manifest_dir.join("target/debug");
    assert!(ldso_path.exists(), "libldso.so not found");

    let src = fixtures.join("fenv_test.c");
    let bin = fixtures.join("fenv_test");
    let status = Command::new("musl-gcc")
        .args([
            "-fPIE",
            "-pie",
            "-I",
            include.to_str().unwrap(),
            "-Wl,--dynamic-linker",
            ldso_path.to_str().unwrap(),
            "-L",
            lib_dir.to_str().unwrap(),
            src.to_str().unwrap(),
            "-Wl,--allow-shlib-undefined",
            "-lc",
            "-o",
            bin.to_str().unwrap(),
        ])
        .status()
        .expect("musl-gcc failed");
    assert!(status.success(), "musl-gcc fenv_test compilation failed");

    let output = Command::new(&bin)
        .env("LD_LIBRARY_PATH", lib_dir.to_str().unwrap())
        .output()
        .expect("failed to run fenv_test");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "fenv_test exited with {:?}, stdout: {}, stderr: {}",
        output.status.code(),
        stdout,
        stderr
    );
    assert_eq!(stdout, "OK\n", "unexpected output: {}", stdout);
}
