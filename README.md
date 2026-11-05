# crabc

[![CI](https://github.com/mengzhuo/crabc/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/mengzhuo/crabc/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/crabc.svg)](https://crates.io/crates/crabc)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**A musl-compatible libc and dynamic linker written in Rust.**

crabc is a from-scratch implementation of the musl C library ABI and its
dynamic linker. It runs musl-linked ELF binaries on x86\_64, aarch64, and
riscv64 when crabc's `libldso.so` is used as their interpreter, and it can be
linked as `libc.a` into static Linux programs.

> **Status:** pre-1.0. The C ABI and the tested musl surface are stable; the
> Rust crate layout and internal APIs may still change.

## At a glance

| Attribute | Value |
|---|---|
| **Architectures** | x86\_64, aarch64, riscv64 |
| **Target ABI** | musl 1.2.6 |
| **Exported symbols** | 1,685 |
| **libc-test** | 415 PASS / 4 SKIP / 0 FAIL |
| **Differential parity** | 340 cases, 0 unexpected differences |
| **Distribution** | crates.io and Alpine Linux packages |
| **License** | MIT OR Apache-2.0 |

## Why crabc

| | musl | crabc |
|---|---|---|
| **Implementation** | C | Rust `no_std` |
| **Memory safety** | Manual review | Safe Rust for the bulk; `unsafe` is explicit and confined to syscall/pointer glue |
| **Dynamic linker** | Separate `ld-musl-*.so` | Built from the same workspace as `libldso.so` |
| **Integration** | C toolchain | C ABI, plus `libc.a` for static links |
| **Architectures** | Many | x86\_64, aarch64, riscv64 |
| **Symbol surface** | ~1,650 | 1,685 |

crabc is **not** a glibc replacement. It implements the musl ABI, so software
built for musl can use it without source changes. The dynamic linker is part
of the project, which means the libc and loader are tested together as one
unit.

A `no_std` implementation also makes crabc a practical C ABI layer for
projects that need a small static libc: link `libc.a` when you need stdio,
pthreads, or POSIX calls on Linux without pulling in a hosted libc.

## Quick start

### Build from source

Requirements:

- Rust **nightly**
- `musl-gcc` (from `musl-tools` / `musl-dev`)
- Docker or podman only when building Alpine packages

```bash
git clone https://github.com/mengzhuo/crabc
cd crabc
cargo build --workspace
```

The build produces:

- `target/debug/libc.so`
- `target/debug/libldso.so`
- `target/debug/libc.a`
- `target/debug/loader` (a minimal standalone ELF loader)

### Run a program on crabc

```c
/* hello.c */
#include <stdio.h>

int main(void) {
    puts("hello from crabc");
    return 0;
}
```

Link against crabc's libc and dynamic linker, then run it:

```bash
musl-gcc -fPIE -pie \
    -Wl,--dynamic-linker="$PWD/target/debug/libldso.so" \
    -L "$PWD/target/debug" hello.c \
    -Wl,--allow-shlib-undefined -lc -o hello

LD_LIBRARY_PATH="$PWD/target/debug" ./hello
```

```text
hello from crabc
```

### Point an existing musl binary at crabc

An already-built musl binary only needs its interpreter path changed; the
program itself is not recompiled:

```bash
patchelf --set-interpreter /usr/lib/crabc/libldso.so ./program
LD_LIBRARY_PATH=/usr/lib/crabc ./program
```

### Install on Alpine

Every release ships unsigned Alpine packages for x86\_64, aarch64, and
riscv64 in apk-tools v3 / ADB format (Alpine 3.23+):

```bash
version=0.4.1
arch=x86_64   # or aarch64 / riscv64
base=https://github.com/mengzhuo/crabc/releases/download/v${version}

curl -LO "$base/crabc-${version}-r0.${arch}.apk"
curl -LO "$base/SHA256SUMS"
grep "crabc-${version}-r0.${arch}.apk" SHA256SUMS | sha256sum -c -
apk add --allow-untrusted "./crabc-${version}-r0.${arch}.apk"
```

The package is installed in a private prefix and does not replace the
system musl:

- `/usr/lib/crabc/libc.so`
- `/usr/lib/crabc/libldso.so`
- `/usr/bin/crabc-loader`

`crabc-loader` is the minimal standalone ELF loader used by the test suite; it
is not a general runner for static binaries. For normal musl programs, use
`libldso.so` as the interpreter (or link with it directly, as shown above).

### Use the crates

The workspace is published as three crates:

| Crate | Contents |
|---|---|
| [`crabc`](https://crates.io/crates/crabc) | The `loader` binary |
| [`crabc-libc`](https://crates.io/crates/crabc-libc) | `libc.so` / `libc.a` |
| [`crabc-ldso`](https://crates.io/crates/crabc-ldso) | `libldso.so` |

Releases are published from
[`.github/workflows/release.yml`](.github/workflows/release.yml) on `v*` tags
using crates.io Trusted Publishing (OIDC, no long-lived token).

## Compatibility and verification

### What works

crabc implements the musl symbol surface needed by real programs, including:

- stdio, stdlib, string, ctype, locale, iconv, regex, and glob
- pthreads, TLS, semaphores, mutexes, condition variables, and robust locks
- sockets, `getaddrinfo`, netdb, and the DNS resolver
- passwd/group/shadow, utmp/utmpx, mntent, and syslog
- AIO, `wordexp`, `realpath`, `nftw`, `fopencookie`, and `open_wmemstream`
- `dlopen`/`dlsym`/`dladdr`/`dl_iterate_phdr`
- DES and modern `crypt`, `getopt_long`, and the Linux syscall wrappers
- the double/float math library, with parity-sensitive algorithms ported from
  musl

All public musl function symbols are exported. The only absent symbols are the
ELF/loader internals `_init`, `_fini`, `_dl_debug_addr`, and `_dl_debug_state`,
which are not part of the public libc ABI.

### Test results

`libc-test` (the upstream musl test suite) passes across functional, regression,
math, and API checks:

```text
functional + regression + math + api: 415 PASS / 4 SKIP / 0 FAIL
```

The functional and regression subsets also run in CI on x86\_64 and aarch64;
the full suite is reproduced locally with `libc-test-harness/run.sh all`.

The four skips are long-double tests whose upstream vectors use 80-bit values;
crabc's x86\_64 ABI uses 64-bit `long double`.

The differential parity check compares crabc's observable behaviour with musl
1.2.6 by running the same musl-built test binary under both libcs:

```text
340 cases: 278 identical, 3 documented differences, 59 long-double skips,
0 unexpected differences
```

### Verify it yourself

```bash
# Integration tests and C fixtures
cargo test --workspace

# Upstream musl libc-test
cd libc-test-harness
./run.sh all

# Differential parity and exported-symbol comparison
# (build musl 1.2.6 first so the baseline matches the ported behaviour)
(cd /path/to/musl-1.2.6 && ./configure && make -j)
MUSL_LIBC=/path/to/musl-1.2.6/lib/libc.so ./scripts/parity-check.sh all
MUSL_LIBC=/path/to/musl-1.2.6/lib/libc.so ./scripts/parity-check.sh symbols
```

### Known differences

- **x86\_64 `long double` is 64-bit.** musl uses 80-bit; aarch64 and riscv64
  use 128-bit IEEE quad. C code that depends on 80-bit precision should be
  built with `-mlong-double-64` or use the `double` APIs.
- **Four ELF/loader-internal symbols are absent.** `_init`, `_fini`,
  `_dl_debug_addr`, and `_dl_debug_state` are not part of the public libc ABI.
- **`strptime` is a small superset.** crabc implements the `%s` and two-digit
  `%z` conversions expected by `libc-test`; musl 1.2.6 leaves the `tm` fields
  of `%s` unspecified and requires four digits for `%z`.
- **`pow` / `powf` keep exact `x^1` / `x^-1` identities.** This avoids musl's
  spurious `INEXACT`/`UNDERFLOW` flags for subnormal results. Numeric results
  are unchanged; musl's own `powf` round-up test is the only case that fails
  under musl.
- **APK packages are unsigned.** Verify them with `SHA256SUMS` and install
  with `--allow-untrusted`.
- **riscv64 CI is cross-built.** The native RISE runner is disabled; the
  non-blocking cross job builds riscv64 and runs the fixtures under
  `qemu-riscv64`.

## How it works

```text
musl-linked program
        │
        ▼
  libldso.so        relocations, TLS, constructors, symbol resolution
        │
        ▼
  libc.so           musl C ABI, syscalls, stdio, pthreads, math
```

- `libc.so` is a Rust `no_std` shared library exporting the C ABI.
- `libldso.so` is the dynamic linker and is the ELF interpreter for programs
  linked against crabc.
- `libc.a` supports static links.
- `loader` is a small standalone ELF loader used by the test suite.

## Project layout

| Path | Description |
|---|---|
| `src/` | `loader` binary: minimal userspace ELF loader |
| `libc/` | `libc.so` / `libc.a`: the `no_std` libc implementation |
| `ldso/` | `libldso.so`: dynamic linker |
| `include/` | Public C headers |
| `tests/` | Rust integration tests and C fixtures |
| `libc-test-harness/` | Harness for upstream musl `libc-test` |
| `scripts/parity-check.sh` | Differential behaviour check against real musl |
| `scripts/package-apk.sh` | Alpine `.apk` packaging |
| `scripts/riscv-smoke.sh` | riscv64 cross-build and qemu smoke tests |

## Development

See [`AGENTS.md`](AGENTS.md) for the full build/test invariants. The short
version:

```bash
cargo build --workspace     # required before tests: builds libc.so / libldso.so
cargo test --workspace
cd libc-test-harness && ./run.sh all
```

Cross-build and smoke-test riscv64:

```bash
rustup target add riscv64gc-unknown-linux-musl
./scripts/riscv-smoke.sh
```

Build the Alpine packages from release artifacts:

```bash
CONTAINER_RUNTIME=podman ./scripts/package-apk.sh
```

## Contributing

Issues and pull requests are welcome. Run `cargo test --workspace` and the
`libc-test` harness before submitting a change; see [`AGENTS.md`](AGENTS.md)
for the build/test invariants and architecture notes.

## License

MIT OR Apache-2.0.
