# crabc

A Rust `no_std` musl-compatible libc with a dynamic linker that runs existing,
unmodified musl-linked x86_64, aarch64, and riscv64 Linux binaries.

## Why crabc over musl?

| | musl | crabc |
|---|---|---|
| **Language** | C (~60k lines) | Rust `no_std` (~21k lines) |
| **Memory safety** | Manual — depends on developer discipline | Compiler-guaranteed — no buffer overflows, use-after-free, or UB |
| **Dynamic linker** | Separate `ld-musl-*.so` | Built-in `libldso.so` — runs musl binaries directly |
| **Embedded/kernel** | Requires cross-compile + link | `no_std` — `use crabc_libc` directly in Rust kernels |
| **Architecture** | 10+ arches | x86\_64, aarch64, riscv64 (growing) |
| **Symbol exports** | ~1420 | ~1500 (growing incrementally) |

**The core advantage: a Rust libc can be integrated directly into Rust `no_std` projects without FFI or cross-compilation.**

## Requirements

- Rust **nightly**
- `musl-gcc` (from `musl-tools` / `musl-dev`)

## Build

```bash
cargo build --workspace
```

This produces:

- `target/debug/libc.so`
- `target/debug/libldso.so`
- `target/debug/loader`

## Test

Run all integration tests:

```bash
cargo test --workspace
```

Run a single subsystem:

```bash
cargo test --test math
cargo test --test ctype
cargo test --test string
cargo test --test ldso_real_binary
```

Run the upstream musl `libc-test` harness:

```bash
cd libc-test-harness
./run.sh              # functional subset
./run.sh math         # math subset
./run.sh regression   # regression subset
./run.sh api          # API/header checks
./run.sh all          # everything
```

Cross-build and smoke-test riscv64 under qemu:

```bash
rustup target add riscv64gc-unknown-linux-musl
./scripts/riscv-smoke.sh
```

## Project layout

| Path | Description |
|------|-------------|
| `src/` | `loader` binary — minimal userspace ELF loader |
| `libc/` | `libc.so` / `libc.a` — `no_std` libc implementation |
| `ldso/` | `libldso.so` — dynamic linker |
| `include/` | Public C headers |
| `tests/` | Rust integration tests and C fixtures |
| `libc-test-harness/` | Shell harness for upstream musl `libc-test` |

## Notes

- `long double` ABI: x86_64 uses 64-bit (`-mlong-double-64`); aarch64 and riscv64 use 128-bit IEEE quad.
- `libc-test` currently passes `functional`, `regression`, and `api` completely. `math` passes 195/199; the four long-double tests that use upstream 80-bit vectors are skipped because crabc's x86_64 ABI is 64-bit.
- All public musl function symbols are exported. The resolver (`res_*`) and utmp/utmpx are real implementations; the `_init`, `_fini`, and `_dl_debug_*` linker internals are intentionally not in `libc.so`.
- `tests/compat.rs` covers the extra ABI surface (resolver, netdb, passwd/group/shadow, utmp, AIO, glob/nftw/realpath, wordexp, fopencookie, DES crypt, dladdr, statfs, sysinfo, posix_getdents, pthread join variants, futimesat, mremap, and friends).
- riscv64 is cross-built and smoke-tested under `qemu-riscv64` by `scripts/riscv-smoke.sh` (hello, tiny PIE/loader, TLS/DSO, math, pthread, socket, net, and more).

## License

MIT OR Apache-2.0
