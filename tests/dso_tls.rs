use std::process::Command;

#[test]
fn dso_tls_works_in_main_and_pthread_threads() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = manifest_dir.join("tests/fixtures");
    let include = manifest_dir.join("include");

    let ldso_path = manifest_dir.join("target/debug/libldso.so");
    let libc_path = manifest_dir.join("target/debug/libc.so");
    assert!(ldso_path.exists(), "libldso.so not found");
    assert!(libc_path.exists(), "libc.so not found");

    let libtls_src = fixtures.join("libtls.c");
    let libtls_so = manifest_dir.join("target/debug/libtls.so");
    let status = Command::new("musl-gcc")
        .args([
            "-shared",
            "-fPIC",
            libtls_src.to_str().unwrap(),
            "-o",
            libtls_so.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc for libtls.so");
    assert!(status.success(), "musl-gcc libtls.so compilation failed");

    let src = fixtures.join("dso_tls_test.c");
    let bin = fixtures.join("dso_tls_test");
    let status = Command::new("musl-gcc")
        .args([
            "-fPIE",
            "-pie",
            "-I",
            include.to_str().unwrap(),
            "-Wl,--dynamic-linker",
            ldso_path.to_str().unwrap(),
            "-L",
            manifest_dir.join("target/debug").to_str().unwrap(),
            src.to_str().unwrap(),
            "-Wl,--allow-shlib-undefined",
            "-ltls",
            "-lc",
            "-o",
            bin.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc for dso_tls_test");
    assert!(status.success(), "musl-gcc dso_tls_test compilation failed");

    let readelf = Command::new("readelf")
        .args(["-r", libtls_so.to_str().unwrap()])
        .output()
        .expect("failed to run readelf on libtls.so");
    eprintln!("libtls.so relocations:\n{}", String::from_utf8_lossy(&readelf.stdout));

    let objdump = Command::new("objdump")
        .args(["-d", libtls_so.to_str().unwrap()])
        .output()
        .expect("failed to run objdump on libtls.so");
    eprintln!("libtls.so disassembly:\n{}", String::from_utf8_lossy(&objdump.stdout));

    let readelf = Command::new("readelf")
        .args(["-r", bin.to_str().unwrap()])
        .output()
        .expect("failed to run readelf");
    eprintln!("dso_tls_test relocations:\n{}", String::from_utf8_lossy(&readelf.stdout));

    let objdump = Command::new("objdump")
        .args(["-d", bin.to_str().unwrap()])
        .output()
        .expect("failed to run objdump");
    eprintln!("dso_tls_test disassembly:\n{}", String::from_utf8_lossy(&objdump.stdout));

    let output = Command::new(&bin)
        .env("LD_LIBRARY_PATH", manifest_dir.join("target/debug").to_str().unwrap())
        .output()
        .expect("failed to run dso_tls_test");

    assert!(
        output.status.success(),
        "dso_tls_test exited with {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "dso tls ok\n");
}
