# Agent Handoff

## Mission

`crabc` is a Rust `no_std` musl-compatible libc plus a dynamic linker. Goal: run unmodified musl-linked x86_64/aarch64/riscv64 ELF binaries. Port behavior from `/home/root/musl`; validate against `/home/root/libc-test`.

## Repo Layout

| Path | What it is |
|------|------------|
| `src/` | `loader` binary (package `crabc`): maps a standalone PIE/static ELF and jumps to its entry. It is **not** the dynamic linker. |
| `libc/` | Package `crabc-libc`; `[lib] name = "c"` produces `target/debug/libc.so` and `libc.a`. |
| `ldso/` | Package `crabc-ldso`; produces `target/debug/libldso.so`. `_start` is `global_asm!` in `ldso/src/lib.rs`. |
| `include/` | Hand-maintained C headers used by fixtures. There is no header generator or `make` target. |
| `tests/` | Root-package integration tests; C fixtures live in `tests/fixtures/`. |
| `libc-test-harness/` | Shell runner for upstream `libc-test`; reports in `libc-test-harness/reports/`. |
| `scripts/local-ci.sh` | Reproduces CI build/test in podman/docker (it does not run the harness). |
| `scripts/riscv-smoke.sh` | Cross-builds and runs riscv64 fixtures under `qemu-riscv64`. |

## Build And Test

```bash
cargo build --workspace      # required before tests: builds loader, libc.so/a, libldso.so
cargo test --workspace       # all integration tests
cargo test --test math       # one test file, after the build above
```

- **Bare `cargo build` only builds the root `crabc` package (the loader).** The root `Cargo.toml` is both workspace and package, so `libc.so`/`libldso.so` are not rebuilt. Use `--workspace`, or `-p crabc-libc` / `-p crabc-ldso`.
- `-p libc` and `-p ldso` do not exist. The artifact names come from `[lib] name` (`c`, `ldso`), not the package names.
- Even `cargo test --workspace` does not build `libc.so`/`libldso.so`; integration tests assert those files exist. Always run `cargo build --workspace` first.
- Tests and the harness hardcode `target/debug`; `cargo test --release` still looks for debug artifacts. Stick to the dev profile.
- Cargo stops at the first failing test binary; use `cargo test --workspace --no-fail-fast` when you need the full picture.
- Toolchain is nightly (`rust-toolchain.toml`) and requires `musl-gcc`. `static_hello` also needs host CRT objects under `/usr/lib/<arch>-linux-musl/`.
- Dev profile is intentional: `opt-level = 2`, `overflow-checks = false`, `panic = "abort"`. `.cargo/config.toml` sets `RUST_TEST_THREADS=1` and `-C link-dead-code`. Do not change these.
- New fixture binaries are written into `tests/fixtures/`; add each output path to the manual list in `.gitignore`.

## Test Pattern

Each `tests/<name>.rs` (canonical: `tests/math.rs`):

1. compiles `tests/fixtures/<name>_test.c` with `musl-gcc -fPIE -pie -I include/ ... -Wl,--dynamic-linker target/debug/libldso.so -L target/debug -lc -Wl,--allow-shlib-undefined`;
2. runs it with `LD_LIBRARY_PATH=target/debug`;
3. asserts exact stdout (for example `"math ok\n"`).

Math fixtures add `-frounding-math`; on x86_64 also `-mlong-double-64`. `tests/crypt.rs` writes its C source inline, so edit the Rust string, not `tests/fixtures/crypt_test.c`. `tests/wave4.rs` and `tests/wave5.rs` shell out to `libc-test-harness/run.sh` and assert aggregate PASS/FAIL counts; they are slow and depend on the harness setup.

`libc` and `ldso` set `test = false`; all coverage is through these root integration tests.

## Adding A libc Function

1. Implement it in `libc/src/lib.rs` or a `libc/src/<area>.rs` file, porting musl's algorithm literally.
2. If the file is new, add `include!("<area>.rs");` in `libc/src/lib.rs` at the right point. Order matters (`math_helpers.rs` comes before the math implementations, and several includes are near the end of the file).
3. Add the prototype to the matching `include/*.h`; headers are hand-written.
4. Add coverage to `tests/fixtures/<area>_test.c` and the expected stdout in `tests/<area>.rs`.
5. Run `cargo build --workspace && cargo test --test <area>`, then `libc-test-harness/run.sh <subset>` for broader checking.

## Implementation Notes

- `libc/src/lib.rs` is a single large `no_std` file. Subsystems are `include!`-ed, not `mod`-declared. The compatibility surface is split across area files: `netdb_extra.rs`, `compat_extra.rs`, `stdio_extra.rs`, `ns_parse.rs`, `fts.rs`, `ifaddrs.rs`, `dl_extra.rs`, `path_extra.rs`, `wordexp.rs`, `des.rs`, and `getopt_impl.rs`.
- `libc` and `ldso` are `no_std`; only the `src/` loader uses `std`.
- C ABI exports need `#[no_mangle] pub unsafe extern "C" fn ...`. A plain Rust function will not appear in `libc.so`/`libc.a`.
- `libm = "0.2"` is only a stopgap for functions not yet ported. When porting one, remove its `libm::` call.
- `long double`: crabc's x86_64 ABI is 64-bit (fixtures pass `-mlong-double-64`); aarch64/riscv64 use `f128` signatures, though `math_long_double.rs` currently computes through `f64`.
- `ldso/build.rs` links with `-nostartfiles -nostdlib -e _start -Wl,-Bsymbolic`; keep `_start` as the `global_asm!` entry in `ldso/src/lib.rs`.
- CI (`.github/workflows/ci.yml`) runs build + `cargo test --workspace` natively on x86_64 and aarch64 (`ubuntu-24.04-arm`). The native riscv64 entry (RISE `ubuntu-24.04-riscv`) is currently commented out because the runner does not come up; the x86-hosted `riscv64-cross` job still cross-builds the musl target and runs the qemu smoke suite (non-blocking, 10 minute cap).
- `scripts/local-ci.sh` accepts `CI_RUNNER` (podman/docker) and `CI_IMAGE` overrides.
- All public musl function symbols are now exported. The resolver (`res_*`) and utmp/utmpx are real implementations; `_init`, `_fini`, and `_dl_debug_*` are linker internals and are intentionally absent from `libc.so`.
- `tests/compat.rs` exercises the non-libc-test ABI surface (resolver, netdb, passwd/group/shadow, utmp, AIO, glob/nftw/realpath, wordexp, fopencookie, DES crypt, dladdr, statfs, sysinfo, posix_getdents, pthread join variants, futimesat, mremap, etc.).

## Releasing

Releases are published by `.github/workflows/release.yml` on `v*` tags using crates.io Trusted Publishing (OIDC, no long-lived token). The job uses the GitHub environment `release`.

One-time setup on crates.io, for **each** of `crabc`, `crabc-libc`, and `crabc-ldso`:

- open the crate's **Settings → Trusted Publishing → Add a new Trusted Publisher**
- GitHub repository owner: `mengzhuo`
- GitHub repository name: `crabc`
- Workflow filename: `release.yml`
- Environment: `release`

To cut a release, bump the version in `Cargo.toml`, `libc/Cargo.toml`, and `ldso/Cargo.toml`, commit, then push a matching tag:

```bash
git tag vX.Y.Z
git push origin main vX.Y.Z
```

The workflow verifies that all three manifest versions equal the tag before publishing. It publishes `crabc-libc`, `crabc-ldso`, then `crabc`, and creates a GitHub release.

If publishing manually, crates.io verifies each package standalone, where the workspace `panic = "abort"` is absent. Use an explicit abort strategy for the no_std crates:

```bash
RUSTFLAGS='-C panic=abort' cargo publish -p crabc-libc --registry crates-io
RUSTFLAGS='-C panic=abort' cargo publish -p crabc-ldso --registry crates-io
cargo publish -p crabc --registry crates-io
```

The default registry in this environment is a mirror; pass `--registry crates-io` to publish to crates.io.

## riscv64

- Cross-build: `CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_MUSL_LINKER=riscv64-linux-gnu-gcc cargo build --workspace --target riscv64gc-unknown-linux-musl`.
- `scripts/riscv-smoke.sh` cross-builds if needed and runs hello, tiny-PIE/loader, TLS/DSO, math, pthread, socket, net, and other fixtures under `qemu-riscv64`. Requires `gcc-riscv64-linux-gnu` and `qemu-user`. On native riscv64 it uses `musl-gcc` and runs the fixtures directly (set `RISCV_CC`/`QEMU` to override).
- `c_char` is `u8` on riscv64 (it is `i8` on x86_64). Use `b'x' as c_char`, not `'x' as i8`, in code compiled for both.
- `struct stat` is architecture-specific: x86_64 uses the older order; aarch64/riscv64 use the generic 128-byte layout. Keep Rust `Stat` and `include/sys/stat.h` in sync.
- `nlink_t` and `blksize_t` are 32-bit on aarch64/riscv64 but 64-bit on x86_64; see `include/sys/types.h`.

## libc-test Harness

```bash
cd libc-test-harness
./run.sh              # functional (default)
./run.sh math         # math subset
./run.sh regression
./run.sh api          # compile-only header/API checks
./run.sh all
```

- Defaults to `LIBC_TEST_DIR=/home/root/libc-test`; override it for another checkout.
- It symlinks `fake-libs/{libc,libpthread,libm,...}.so` to our `libc.so`, builds `runtest.exe`, and writes `reports/latest-summary.txt` / `latest-raw.txt`.
- When a test exits 0, any non-empty stderr line not starting with `ldso:` or `JUMP ` is still treated as FAIL.
- Its fallback build is bare `cargo build`, which will not build `libc.so`/`libldso.so`; run `cargo build --workspace` first.
- It creates symlinks in `$LIBC_TEST_DIR/src/<subset>/`, so it mutates the external libc-test checkout.
- Run one subset at a time. All subsets share `build/`, `fake-libs/`, and `reports/`; concurrent runs race and corrupt counts.
- `math` intentionally SKIPs `acoshl`, `asinhl`, `lgammal`, and `tgammal`: upstream uses 80-bit long-double vectors while crabc's x86_64 ABI is 64-bit. This is recorded in `SKIP_TESTS` in `run.sh`.

## Current Baseline

Verified on 2026-11-04:

```text
cargo test --workspace          # green
./run.sh all                    # 419 total: 415 PASS, 4 SKIP, 0 FAIL
./run.sh api                    # 79/79
./run.sh functional             # 73/73
./run.sh regression             # 68/68
./run.sh math                   # 195 PASS, 4 SKIP, 0 FAIL
scripts/riscv-smoke.sh          # riscv64 cross-build + qemu fixtures, 0 failures
```

- If a run shows a different result, first re-run sequentially and compare against the previous report in `libc-test-harness/reports/`.
- The four math SKIPs are the only known expected non-passes; do not "fix" them by weakening the implementation.
