# libc-test Integration Harness

Builds and runs the [libc-test](https://git.musl-libc.org/cgit/libc-test) suite against crabc's libc.so to produce a categorized failure report.

## How It Works

1. Builds `crabc/libc` into `target/debug/libc.so` (via `cargo build`).
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

- **dlopen tests are skipped** — our libc.so doesn't export `dlopen`/`dlsym`.
- **Most tests will BUILDERROR** — our libc.so exports ~351 symbols vs musl's ~1420. Missing symbols cause link failures.
- **Static linking is not tested** — only dynamic-linked binaries are built.
- **Only functional subset is tested by default** — use `./run.sh all` for everything.

## Initial Report

See `initial-report.txt` for the first baseline run (functional tests): 13 PASS, 63 BUILDERROR, 0 FAIL, 0 TIMEOUT.
