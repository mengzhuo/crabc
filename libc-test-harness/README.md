# libc-test Integration Harness

Builds and runs the [libc-test](https://git.musl-libc.org/cgit/libc-test) suite against crabc's libc.so to produce a categorized failure report.

## How It Works

1. Builds `crabc/libc` into `target/debug/libc.so` (via `cargo build --workspace`).
2. Creates a `fake-libs/` directory with symlinks so the linker resolves `-lc`, `-lpthread`, `-lm`, etc. against our libc.so instead of musl's.
3. Builds libc-test's `runtest.exe` and `libtest.a` as host tools (linked against musl).
4. Compiles and links each test binary with `musl-gcc -L fake-libs/`, then runs it via `LD_LIBRARY_PATH=fake-libs/`.
5. Categorizes results: **PASS**, **FAIL**, **BUILDERROR** (compile/link failure), **TIMEOUT** (30s).

## Usage

```bash
./run.sh              # functional tests only (default)
./run.sh math         # math tests only
./run.sh regression   # regression tests only
./run.sh api          # API/header tests only
./run.sh all          # all categories
```

Reports are saved to `reports/`. Symlinks `reports/latest-summary.txt` and `reports/latest-raw.txt` always point to the most recent run.

## Requirements

- musl-gcc (`apt install musl-tools`)
- Rust nightly toolchain (for building crabc)
- libc-test source at `/home/root/libc-test` (override with `LIBC_TEST_DIR` env var)

## Known Limitations

- **Run subsets sequentially** — `build/`, `fake-libs/`, and `reports/` are shared, so concurrent runs race.
- **Long-double math tests:** `acoshl`, `asinhl`, `lgammal`, and `tgammal` are skipped because upstream uses 80-bit vectors while crabc's x86_64 `long double` ABI is 64-bit.
- **dlopen is only partially implemented** — tests that require a full `dlopen`/`dlsym` implementation may not exercise it.
- **Static linking is not tested** — only dynamic-linked binaries are built.
- **Only the functional subset runs by default** — use `./run.sh all` for everything.

Current baseline: `functional` 73/73, `regression` 68/68, `api` 79/79, `math` 195 PASS + 4 SKIP.

## Initial Report

See `initial-report.txt` for the first baseline run (functional tests): 13 PASS, 63 BUILDERROR, 0 FAIL, 0 TIMEOUT.
