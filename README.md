# crabc libc

crabc is an MIT/Apache-2.0 licensed implementation of the musl C library ABI
and dynamic linker for Linux, written in Rust. It runs unmodified musl-linked
binaries on x86\_64, aarch64 and riscv64 when `libldso.so` is used as the
interpreter, and provides `libc.a` for static linking. crabc is built on the
principle that a libc should be small, easy to audit and safe to maintain;
`unsafe` is confined to syscalls and pointer glue.

crabc tracks musl 1.2.6 and exports its full public symbol surface. It is
validated with musl's own libc-test and a differential harness that runs the
same binaries under musl and crabc, and it is faster than musl on both
x86\_64 and riscv64 (qemu-user) in the microbenchmark suite; process startup
is the one structural exception. See `docs/performance.md`.

Build and cross-compilation instructions are in `AGENTS.md`. Releases are
produced from `v*` tags and the workspace is published as the `crabc`,
`crabc-libc` and `crabc-ldso` crates.

crabc is licensed under MIT OR Apache-2.0.
