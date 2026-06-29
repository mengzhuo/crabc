use std::process::Command;

#[test]
fn ldso_runs_real_printf_binary() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = manifest_dir.join("tests/fixtures");

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
    assert!(ldso_path.exists(), "libldso.so not found at {}", ldso_path.display());

    let libc_path = manifest_dir.join("target/debug/libc.so");
    assert!(libc_path.exists(), "libc.so not found at {}", libc_path.display());

    let hello_src = fixtures.join("hello.c");
    let hello_bin = fixtures.join("hello");
    let status = Command::new("musl-gcc")
        .args([
            "-fPIE",
            "-pie",
            "-Wl,--dynamic-linker",
            ldso_path.to_str().unwrap(),
            "-L",
            manifest_dir.join("target/debug").to_str().unwrap(),
            hello_src.to_str().unwrap(),
            "-Wl,--allow-shlib-undefined",
                        "-lc",
            "-o",
            hello_bin.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc for hello");
    assert!(status.success(), "musl-gcc hello compilation failed");

    let output = Command::new(&hello_bin)
        .env("LD_LIBRARY_PATH", manifest_dir.join("target/debug").to_str().unwrap())
        .output()
        .expect("failed to run hello");

    assert!(
        output.status.success(),
        "hello exited with {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
}
