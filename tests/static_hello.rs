use std::process::Command;

#[test]
fn static_hello_links_against_libc_a() {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixtures = manifest_dir.join("tests/fixtures");

    let build = Command::new("cargo")
        .args(["build", "-p", "libc"])
        .status()
        .expect("cargo build -p libc");
    assert!(build.success(), "cargo build -p libc failed");

    let libc_a = manifest_dir.join("target/debug/libc.a");
    assert!(libc_a.exists(), "libc.a not found at {}", libc_a.display());

    let hello_src = fixtures.join("hello.c");
    let hello_bin = fixtures.join("hello_static");

    let status = Command::new("musl-gcc")
        .args([
            "-static",
            "-nostdlib",
            "-fno-stack-protector",
            "/usr/lib/x86_64-linux-musl/crt1.o",
            "/usr/lib/x86_64-linux-musl/crti.o",
            hello_src.to_str().unwrap(),
            libc_a.to_str().unwrap(),
            "/usr/lib/x86_64-linux-musl/crtn.o",
            "-o",
            hello_bin.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run musl-gcc");
    assert!(status.success(), "static link with libc.a failed");

    let file_out = Command::new("file")
        .arg(&hello_bin)
        .output()
        .expect("file command failed");
    let file_info = String::from_utf8_lossy(&file_out.stdout);
    assert!(
        file_info.contains("statically linked"),
        "binary is not statically linked: {}",
        file_info
    );

    let output = Command::new(&hello_bin)
        .output()
        .expect("failed to run hello_static");
    assert!(
        output.status.success(),
        "hello_static exited with {:?}, stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello\n");
}
