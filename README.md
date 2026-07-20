# crabc

A Rust `no_std` musl-compatible libc with a dynamic linker that runs existing,
unmodified musl-linked x86_64, aarch64, and riscv64 Linux binaries.

## Why crabc over musl?

| | musl | crabc |
|---|---|---|
| **Language** | C (~60k lines) | Rust `no_std` (~15k lines) |
| **Memory safety** | Manual — depends on developer discipline | Compiler-guaranteed — no buffer overflows, use-after-free, or UB |
| **Dynamic linker** | Separate `ld-musl-*.so` | Built-in `libldso.so` — runs musl binaries directly |
| **Embedded/kernel** | Requires cross-compile + link | `no_std` — `use crabc_libc` directly in Rust kernels |
| **Architecture** | 10+ arches | x86\_64, aarch64, riscv64 (growing) |
| **Symbol exports** | ~1420 | ~350 (growing incrementally) |

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

## Project layout

| Path | Description |
|------|-------------|
| `src/` | `loader` binary — minimal static-PIE ELF runner |
| `libc/` | `libc.so` / `libc.a` — `no_std` libc implementation |
| `ldso/` | `libldso.so` — dynamic linker |
| `include/` | Public C headers |
| `tests/` | Rust integration tests and C fixtures |
| `libc-test-harness/` | Shell harness for upstream musl `libc-test` |

## Notes

- `long double` ABI is currently 64-bit; math fixtures are compiled with
  `-mlong-double-64`.
- `tests/new_functions.rs` may fail in containers where `/dev/null` is a
  regular file instead of a character device.
- `libc-test` reports many `BUILDERROR`s until the full musl symbol set is
  exported; this is expected.

## License

MIT OR Apache-2.0
