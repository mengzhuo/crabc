use std::process::Command;

#[test]
fn ldso_startup_argv_env() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = manifest_dir.join("tests/fixtures");
    let include = manifest_dir.join("include");

    let ldso_path = manifest_dir.join("target/debug/libldso.so");
    let libc_path = manifest_dir.join("target/debug/libc.so");
    assert!(ldso_path.exists(), "libldso.so not found");
    assert!(libc_path.exists(), "libc.so not found");

    let src = fixtures.join("startup_argv_env.c");
    let bin = fixtures.join("startup_argv_env");
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
        .expect("failed to run musl-gcc for startup_argv_env");
    assert!(
        status.success(),
        "musl-gcc startup_argv_env compilation failed"
    );

    let output = Command::new(&bin)
        .env(
            "LD_LIBRARY_PATH",
            manifest_dir.join("target/debug").to_str().unwrap(),
        )
        .env("TEST_STARTUP_VAR", "hello_world")
        .output()
        .expect("failed to run startup_argv_env");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "startup_argv_env exited with {:?}, stderr: {}",
        output.status.code(),
        stderr
    );
    assert!(
        stdout.contains("argc=1"),
        "expected argc=1 in: {}",
        stdout
    );
    assert!(
        !stdout.contains("argv0=") || !stdout.contains("argv0=\n"),
        "argv[0] should not be empty: {}",
        stdout
    );
    assert!(
        stdout.contains("env=hello_world"),
        "expected env=hello_world in: {}",
        stdout
    );
}
