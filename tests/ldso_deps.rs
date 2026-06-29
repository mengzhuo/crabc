use std::process::Command;

#[test]
fn ldso_runs_pie_with_dependency() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = manifest_dir.join("tests/fixtures");

    // Build ldso
    let build_status = Command::new("cargo")
        .args(["build", "-p", "ldso"])
        .status()
        .expect("failed to run cargo build -p ldso");
    assert!(build_status.success(), "cargo build -p ldso failed");

    let ldso_path = manifest_dir.join("target/debug/libldso.so");
    assert!(ldso_path.exists(), "libldso.so not found at {}", ldso_path.display());

    // Build libfoo.so
    let libfoo_src = fixtures.join("libfoo.c");
    let libfoo_so = fixtures.join("libfoo.so");
    let status = Command::new("musl-gcc")
        .args([
            "-shared",
            "-fPIC",
            libfoo_src.to_str().unwrap(),
            "-o",
            libfoo_so.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc for libfoo.so");
    assert!(status.success(), "musl-gcc libfoo.so compilation failed");

    // Build needfoo (PIE, nostdlib, nostartfiles, dynamic-linker=ldso, -lfoo)
    let needfoo_src = fixtures.join("needfoo.c");
    let needfoo_bin = fixtures.join("needfoo");
    let status = Command::new("musl-gcc")
        .args([
            "-fPIE",
            "-pie",
            "-nostdlib",
            "-nostartfiles",
            "-Wl,--dynamic-linker",
            ldso_path.to_str().unwrap(),
            "-L",
            fixtures.to_str().unwrap(),
            needfoo_src.to_str().unwrap(),
            "-lfoo",
            "-o",
            needfoo_bin.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc for needfoo");
    assert!(status.success(), "musl-gcc needfoo compilation failed");

    // Run needfoo with LD_LIBRARY_PATH pointing at fixtures
    let output = Command::new(&needfoo_bin)
        .env("LD_LIBRARY_PATH", fixtures.to_str().unwrap())
        .output()
        .expect("failed to run needfoo");

    assert!(
        output.status.success(),
        "needfoo exited with {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
}
