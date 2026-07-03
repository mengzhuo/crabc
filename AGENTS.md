# Agent Handoff

## Mission

Build a Rust musl-compatible libc (`crabc`) with a dynamic linker that runs existing, unmodified musl-linked x86_64 Linux binaries. Implement remaining libc subsystems incrementally against `/home/root/musl` and validate with `/home/root/libc-test`.

## Repo Layout

| Path | What it is |
|------|------------|
| `src/` | `loader` binary — minimal static-PIE ELF runner (`src/main.rs` + `src/loader_core.rs`). |
| `libc/` | `libc.so` / `libc.a` — monolithic `no_std` libc implementation. Output crate name is `c`. |
| `ldso/` | `libldso.so` — dynamic linker (`_start` entry, self-relocating, loads DT_NEEDED, handles TLS). |
| `include/` | Public C headers. `include/math.h` declares the full math API. |
| `tests/` | Rust integration tests. Each compiles a C fixture and runs it under `libldso.so`. |
| `tests/fixtures/` | C sources for the integration tests (e.g., `math_test.c`). |
| `libc-test-harness/` | Shell harness that builds/runs the upstream musl `libc-test` suite against our `libc.so`. |
| `wave1/`, `wave2/`, `wave3/` | Staged subsystems `include!`-ed into `libc/src/lib.rs`. |

## Build

```bash
# Build all artifacts: target/debug/libc.so, target/debug/libldso.so, target/debug/loader
cargo build

# Build only one member
cargo build -p libc
cargo build -p ldso
```

- Requires **Rust nightly** (uses `#![feature(c_variadic)]` and `#![feature(linkage)]`).
- Requires **musl-gcc** for compiling C test fixtures.
- Dev profile: `panic = "abort"`, `overflow-checks = false` (matches musl's unsigned wraparound semantics).
- `.cargo/config.toml` sets `RUST_TEST_THREADS=1` and `-C link-dead-code`.
- `ldso/build.rs` links with `-nostartfiles -nostdlib -e _start -Wl,-Bsymbolic`.

## Test

```bash
# All integration tests
cargo test --workspace

# Single subsystem
cargo test --test math
cargo test --test ctype
cargo test --test string
cargo test --test ldso_real_binary
```

Integration-test pattern (see `tests/math.rs`):
1. Compile `tests/fixtures/<name>_test.c` with `musl-gcc -fPIE -pie -I include/ ...`.
2. Link with `-Wl,--dynamic-linker target/debug/libldso.so -L target/debug -lc -Wl,--allow-shlib-undefined`.
3. Run the binary with `LD_LIBRARY_PATH=target/debug`.
4. Assert stdout matches an expected string (e.g., `"math ok\n"`).

Math fixtures additionally need `-frounding-math -mlong-double-64`.

## libc-test Harness

```bash
cd libc-test-harness
./run.sh              # functional subset (default)
./run.sh math         # math subset
./run.sh regression   # regression subset
./run.sh api          # API/header compile checks
./run.sh all          # everything
```

Reports go to `libc-test-harness/reports/`; `latest-summary.txt` and `latest-raw.txt` symlink to the most recent run. The harness creates `fake-libs/` with symlinks (`libpthread.so`, `libm.so`, etc.) all pointing at our `libc.so`. `LIBC_TEST_DIR` overrides the default `/home/root/libc-test` path.

## Adding a Math Function

1. Implement in `libc/src/math_*.rs` using musl's algorithm literally. Do **not** call the `libm` crate for functions we have implemented ourselves.
2. Add the `include!("math_*.rs");` line in `libc/src/lib.rs` if it's a new file.
3. Add declarations to `include/math.h` only if missing.
4. Add `CHECK` cases to `tests/fixtures/math_test.c`.
5. Run `cargo test --test math` and `libc-test-harness/run.sh math`.

## Critical Conventions

- Port musl algorithms literally; no `libm` crate wrappers for implemented functions.
- Each feature must have tests and a commit.
- `long double` ABI is currently 64-bit; compile math cases with `-mlong-double-64`.
- `libc/src/lib.rs` is a single monolithic `no_std` file. Math is split into `include!`-ed modules; most other subsystems are inline.

## Known Environment Quirks

- `tests/new_functions.rs` fails because `/dev/null` is a regular file here, not a char device. This is environment-specific and unrelated to code changes.
- `RUST_TEST_THREADS=1` is forced by `.cargo/config.toml` to serialize integration tests.
- libc-test will report many `BUILDERROR`s until more symbols are exported; our `libc.so` currently exports far fewer than musl's full symbol set.
