use std::process::Command;

#[test]
fn packed_relative_relocations_work() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = manifest_dir.join("tests/fixtures");
    let include = manifest_dir.join("include");
    let debug = manifest_dir.join("target/debug");

    let ldso = debug.join("libldso.so");
    let libc = debug.join("libc.so");
    assert!(ldso.exists(), "libldso.so not found");
    assert!(libc.exists(), "libc.so not found");

    let dso = fixtures.join("librelr_dso.so");
    let dso_src = fixtures.join("relr_dso.c");
    let status = Command::new("musl-gcc")
        .args([
            "-shared",
            "-fPIC",
            "-nostdlib",
            "-Wl,-z,pack-relative-relocs",
            dso_src.to_str().unwrap(),
            "-o",
            dso.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc for the RELR DSO");
    if !status.success() {
        eprintln!("skipping: linker does not support -z pack-relative-relocs");
        return;
    }

    let exe = fixtures.join("relr_test");
    let exe_src = fixtures.join("relr_test.c");
    let status = Command::new("musl-gcc")
        .args([
            "-fPIE",
            "-pie",
            "-I",
            include.to_str().unwrap(),
            "-Wl,-z,pack-relative-relocs",
            "-Wl,--dynamic-linker",
            ldso.to_str().unwrap(),
            "-L",
            debug.to_str().unwrap(),
            "-L",
            fixtures.to_str().unwrap(),
            exe_src.to_str().unwrap(),
            "-Wl,--allow-shlib-undefined",
            "-lrelr_dso",
            "-lc",
            "-o",
            exe.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc for the RELR test");
    if !status.success() {
        eprintln!("skipping: linker does not support -z pack-relative-relocs");
        return;
    }

    // Make sure the binary really carries packed relocations.
    let readelf = Command::new("readelf")
        .args(["-d", exe.to_str().unwrap()])
        .output()
        .expect("failed to run readelf");
    assert!(
        String::from_utf8_lossy(&readelf.stdout).contains("(RELR)"),
        "test binary has no DT_RELR section"
    );

    let path = format!(
        "{}:{}",
        debug.to_str().unwrap(),
        fixtures.to_str().unwrap()
    );
    let output = Command::new(&exe)
        .env("LD_LIBRARY_PATH", path)
        .output()
        .expect("failed to run the RELR test");
    assert!(
        output.status.success(),
        "RELR test exited with {:?}, stdout: {}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "relr ok 42\n");
}
