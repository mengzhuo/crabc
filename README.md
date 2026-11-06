# crabc libc

crabc is an MIT OR Apache-2.0 licensed implementation of the standard C
library and dynamic linker, targeting the Linux syscall API and the musl ABI,
suitable for running musl-linked binaries unmodified. crabc offers efficient
static and dynamic linking support, lightweight code and low runtime overhead,
and correctness in the sense of standards conformance and tested compatibility
with musl. crabc is built on the principle that these goals are best achieved
through simple code that is easy to understand and maintain, written in Rust
with `unsafe` confined to the syscall and pointer boundary.

crabc targets x86\_64, aarch64 and riscv64 and tracks the musl 1.2.6
interface, covering the ISO C99 and POSIX 2008 base interfaces along with the
non-standard Linux, BSD and glibc-compatible interfaces that musl provides.
The full public musl symbol surface is exported (1,685 symbols), and the
dynamic linker is built from the same workspace as the libc, so the two are
developed and tested as one unit.

Correctness is measured with musl's own tests rather than with crabc-specific
expectations. libc-test reports 415 PASS / 4 SKIP / 0 FAIL, and a differential
harness runs the same binaries under musl 1.2.6 and crabc and compares exit
status, stdout and stderr: 0 unexpected differences. The four skips are
x86\_64 long-double tests; crabc intentionally uses a 64-bit `long double`
ABI on that target.

crabc is pre-1.0: the C ABI and the tested musl surface are stable, while the
Rust crate layout and internal APIs may still change.

For basic build and test instructions, including the riscv64 cross build and
the Alpine package flow, see the included AGENTS.md file. Benchmark
methodology and results are in `docs/performance.md`. Release artifacts are
produced by `.github/workflows/release.yml` from `v*` tags, and the workspace
is published as the `crabc`, `crabc-libc` and `crabc-ldso` crates.

crabc is licensed under MIT OR Apache-2.0.
