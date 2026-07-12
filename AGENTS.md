# Agent Handoff

## Mission

Build a Rust musl-compatible libc (`crabc`) with a dynamic linker that runs existing, unmodified musl-linked x86_64 Linux binaries. Implement remaining libc subsystems incrementally against `/home/root/musl` and validate with `/home/root/libc-test`.

## Repo Layout

| Path | What it is |
|------|------------|
| `src/` | `loader` binary — minimal static-PIE ELF runner. |
| `libc/` | `libc.so` / `libc.a` — monolithic `no_std` libc. Crate name is `c`. |
| `ldso/` | `libldso.so` — dynamic linker (`_start` entry, self-relocating, loads DT_NEEDED, handles TLS). |
| `include/` | Public C headers (36 files). |
| `tests/` | Rust integration tests (39 files). Each compiles a C fixture and runs it under `libldso.so`. |
| `tests/fixtures/` | C sources for the integration tests. |
| `libc-test-harness/` | Shell harness that builds/runs the upstream musl `libc-test` suite against our `libc.so`. |
| `scripts/local-ci.sh` | Reproduces the GitHub Actions CI environment locally via podman/docker. |

Subsystems live in `libc/src/*.rs` and are `include!`-ed into `libc/src/lib.rs` (a single ~14k-line `no_std` file). There are no `wave*/` directories — those were consolidated.

## Build

```bash
cargo build                 # all artifacts
cargo build -p libc         # libc only
cargo build -p ldso         # ldso only
```

**Requirements:** Rust nightly (`rust-toolchain.toml` enforces this), musl-gcc.

**Dev profile quirks:** `opt-level = 2` + `overflow-checks = false` + `panic = "abort"` — matches musl's unsigned wraparound semantics. Also `RUST_TEST_THREADS=1` and `-C link-dead-code` from `.cargo/config.toml`.

**ldso linking:** `build.rs` passes `-nostartfiles -nostdlib -e _start -Wl,-Bsymbolic`.

**Cargo.lock** is in `.gitignore` (not committed).

## Test

```bash
cargo test --workspace           # all integration tests
cargo test --test math           # single subsystem
cargo test --test ctype
cargo test --test string
cargo test --test ldso_real_binary
```

All integration tests follow the same pattern (see `tests/math.rs`):
1. Compile `tests/fixtures/<name>_test.c` with `musl-gcc -fPIE -pie -I include/ ...`
2. Link with `-Wl,--dynamic-linker target/debug/libldso.so -L target/debug -lc -Wl,--allow-shlib-undefined`
3. Run the binary with `LD_LIBRARY_PATH=target/debug`
4. Assert stdout matches expected string

Math fixtures additionally need `-frounding-math -mlong-double-64`. Non-math tests use `-fno-builtin` or nothing extra.

## libc-test Harness

```bash
cd libc-test-harness
./run.sh              # functional subset (default) — what CI runs
./run.sh math         # math subset
./run.sh regression   # regression subset
./run.sh api          # API/header compile checks
./run.sh all          # everything
```

Reports: `libc-test-harness/reports/latest-summary.txt` and `latest-raw.txt`. The harness creates `fake-libs/` with symlinks (`libpthread.so`, `libm.so`, etc.) → `libc.so`. `LIBC_TEST_DIR` overrides `/home/root/libc-test`.

## CI

`.github/workflows/ci.yml` runs on push/PR to `main`. Matrix: x86_64 (ubuntu-latest) + aarch64 (ubuntu-24.04-arm, `continue-on-error: true`). Steps: build, cargo test, then `./run.sh functional` (informational only, not a gate).

## Adding a Math Function

1. Implement in `libc/src/math_*.rs` using musl's algorithm literally. Do **not** call the `libm` crate for functions we have implemented ourselves. The `libm` dependency is for functions we haven't ported yet.
2. Add `include!("math_*.rs");` in `libc/src/lib.rs` if it's a new file.
3. Add declarations to `include/math.h` only if missing (`make header` task).
4. Add `CHECK` cases to `tests/fixtures/math_test.c`.
5. Run `cargo test --test math` and `libc-test-harness/run.sh math`.

## Critical Conventions

- Port musl algorithms literally; no `libm` crate wrappers for implemented functions.
- Each feature must have tests and a commit.
- `long double` ABI is currently 64-bit; compile math cases with `-mlong-double-64`.
- `libc/src/lib.rs` is a single monolithic `no_std` file (~14k lines). Subsystems are in `libc/src/*.rs` and pulled in via `include!()`.
- The `libc` crate depends on `libm = "0.2"` for unimplemented math functions. When you port one, stop calling `libm` for it.

## Known Environment Quirks

- `tests/new_functions.rs` fails because `/dev/null` is a regular file here, not a char device. Pre-existing, not a code issue.
- `RUST_TEST_THREADS=1` is forced to serialize integration tests.
- libc-test reports many `BUILDERROR`s — our `libc.so` exports ~351 symbols vs musl's ~1420. Missing symbols cause link failures. This is expected and decreases as we add functions.
- `opt-level = 2` in dev profile is intentional (matches musl behavior). Do not change.
