use std::fs::File;
use std::io::Write;
use std::process::Command;

#[test]
fn stdio_input_functions_under_libc_so() {
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

    let src = fixtures.join("stdio_input_test.c");
    let bin = fixtures.join("stdio_input_test");
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
                        "-lc",
            "-o",
            bin.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc for stdio_input_test");
    assert!(status.success(), "musl-gcc stdio_input_test compilation failed");

    let input_path = fixtures.join("stdio_input_test.txt");
    {
        let mut f = File::create(&input_path).expect("create input file");
        f.write_all(b"hi\nxy\nz").expect("write input");
    }

    let output = Command::new(&bin)
        .env("LD_LIBRARY_PATH", manifest_dir.join("target/debug").to_str().unwrap())
        .stdin(File::open(&input_path).expect("open input"))
        .output()
        .expect("failed to run stdio_input_test");

    assert!(
        output.status.success(),
        "stdio_input_test exited with {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "stdio input ok\n"
    );
}
