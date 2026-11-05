# crabc

A Rust `no_std` musl-compatible libc with a dynamic linker that runs existing,
unmodified musl-linked x86_64, aarch64, and riscv64 Linux binaries.

## Why crabc over musl?

| | musl | crabc |
|---|---|---|
| **Language** | C (~60k lines) | Rust `no_std` |
| **Memory safety** | Manual — depends on developer discipline | Compiler-guaranteed — no buffer overflows, use-after-free, or UB |
| **Dynamic linker** | Separate `ld-musl-*.so` | Built-in `libldso.so` — runs musl binaries directly |
| **Embedded/kernel** | Requires cross-compile + link | `no_std` — `use crabc_libc` directly in Rust kernels |
| **Architecture** | 10+ arches | x86\_64, aarch64, riscv64 |
| **Symbol exports** | ~1,420 | ~1,650 |

**The core advantage: a Rust libc can be integrated directly into Rust `no_std` projects without FFI or cross-compilation.**

## Requirements

- Rust **nightly**
- `musl-gcc` (from `musl-tools` / `musl-dev`)
- Docker or podman (only when building the Alpine packages)

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

## Install

### Alpine Linux packages

Every GitHub release ships unsigned Alpine packages for x86\_64, aarch64, and
riscv64 (apk-tools v3 / ADB format, Alpine 3.23+):

```bash
version=0.4.0
arch=x86_64   # or aarch64 / riscv64
base=https://github.com/mengzhuo/crabc/releases/download/v${version}
curl -LO "$base/crabc-${version}-r0.${arch}.apk"
curl -LO "$base/SHA256SUMS"
grep "crabc-${version}-r0.${arch}.apk" SHA256SUMS | sha256sum -c -
apk add --allow-untrusted "./crabc-${version}-r0.${arch}.apk"
```

The package is unsigned, so verify the checksum and use `--allow-untrusted`.
It installs into a private prefix and does not replace the system musl:

- `/usr/lib/crabc/libc.so`
- `/usr/lib/crabc/libldso.so`
- `/usr/bin/crabc-loader`

Use the loader for a static PIE:

```bash
crabc-loader ./program
```

Link a program against crabc's libc and dynamic linker:

```bash
musl-gcc -fPIE -pie -Wl,--dynamic-linker=/usr/lib/crabc/libldso.so \
    -L/usr/lib/crabc program.c -Wl,--allow-shlib-undefined -lc -o program
LD_LIBRARY_PATH=/usr/lib/crabc ./program
```

### crates.io

The workspace is published as three crates:

- [`crabc`](https://crates.io/crates/crabc) — the loader binary
- [`crabc-libc`](https://crates.io/crates/crabc-libc) — `libc.so` / `libc.a`
- [`crabc-ldso`](https://crates.io/crates/crabc-ldso) — `libldso.so`

Releases are published from `.github/workflows/release.yml` on `v*` tags using
crates.io Trusted Publishing (OIDC, no long-lived token).

## Project layout

| Path | Description |
|------|-------------|
| `src/` | `loader` binary — minimal userspace ELF loader |
| `libc/` | `libc.so` / `libc.a` — `no_std` libc implementation |
| `ldso/` | `libldso.so` — dynamic linker |
| `include/` | Public C headers |
| `tests/` | Rust integration tests and C fixtures |
| `libc-test-harness/` | Shell harness for upstream musl `libc-test` |
| `scripts/` | Local CI, riscv64 smoke test, and Alpine packaging |

## Notes

- `long double` ABI: x86_64 uses 64-bit (`-mlong-double-64`); aarch64 and riscv64 use 128-bit IEEE quad.
- `libc-test` passes `functional`, `regression`, and `api` completely. `math` passes 195/199; the four long-double tests that use upstream 80-bit vectors are skipped because crabc's x86_64 ABI is 64-bit.
- All public musl function symbols are exported and implemented; the resolver (`res_*`), utmp/utmpx, netdb, passwd/group/shadow, AIO, glob/nftw/realpath, wordexp, DES `crypt`, `dladdr`, and `getopt_long` are real implementations. `_init`, `_fini`, and `_dl_debug_*` are linker internals and are intentionally absent from `libc.so`.
- `tests/compat.rs` covers the extra ABI surface (resolver, netdb, passwd/group/shadow, utmp, AIO, glob/nftw/realpath, wordexp, fopencookie, DES crypt, dladdr, statfs, sysinfo, posix_getdents, pthread join variants, futimesat, mremap, and friends).
- riscv64 is cross-built and smoke-tested under `qemu-riscv64` by `scripts/riscv-smoke.sh` (hello, tiny PIE/loader, TLS/DSO, math, pthread, socket, net, and more).
- CI runs `cargo test --workspace` natively on x86_64 and aarch64; the riscv64 cross + qemu job is non-blocking and capped at 10 minutes.

## License

MIT OR Apache-2.0
