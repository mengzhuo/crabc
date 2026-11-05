# Performance

This is a microbenchmark comparison between crabc and musl 1.2.6. The numbers
are a snapshot from one machine, not an application benchmark. Use the ratios
and the reproduction script rather than the absolute values.

## Method

- Host: AMD EPYC 7532, Linux 6.12, 64 available CPUs.
- `crabc` built with `cargo build --release --workspace`.
- Baseline: musl 1.2.6 built from source, used as the ELF interpreter.
- The same C source is compiled twice against musl. The two binaries differ
  only in their ELF interpreter:
  - musl: `musl-1.2.6/lib/libc.so`
  - crabc: `target/release/libldso.so` with
    `LD_LIBRARY_PATH=target/release`
- Microbenchmarks: `clock_gettime(CLOCK_MONOTONIC)` around a batch, `-O2
  -fno-builtin`, so calls go through the libc.
- Startup: `fork` + `exec` + `wait` of a minimal dynamically linked program.
- Five interleaved rounds, pinned to one CPU; the median is reported. The
  host is shared, so absolute values can move by 10-20% between sessions.

## Results

`crabc/musl` below 1.0 means crabc is faster.

| Benchmark | musl | crabc | crabc/musl |
|---|---:|---:|---:|
| `memcpy` 64B | 10.9 ns | 6.8 ns | 0.62x |
| `memcpy` 4KB | 59.1 ns | 84.2 ns | 1.43x |
| `memset` 4KB | 63.7 ns | 80.5 ns | 1.26x |
| `memcmp` 4KB | 868 ns | 636 ns | 0.73x |
| `strlen` 4KB | 315 ns | 1261 ns | 4.00x |
| `strcmp` (short) | 9.6 ns | 8.0 ns | 0.83x |
| `strstr` | 54.8 ns | 6.4 ns | 0.12x |
| `snprintf` | 436 ns | 331 ns | 0.76x |
| `fprintf` → `/dev/null` | 418 ns | 283 ns | 0.68x |
| `malloc`/`free` 16B | 111.8 ns | 12.9 ns | 0.12x |
| `malloc`/`free` 256B | 111.6 ns | 12.9 ns | 0.12x |
| `calloc` 1KB | 175 ns | 30.1 ns | 0.17x |
| `realloc` (grow) | 21.8 ns | 8.5 ns | 0.39x |
| `sin` | 5.21 ns | 5.16 ns | 0.99x |
| `exp` | 6.63 ns | 6.56 ns | 0.99x |
| `log` | 8.59 ns | 10.11 ns | 1.18x |
| `pow` | 21.8 ns | 23.5 ns | 1.08x |
| `sqrt` | 2.87 ns | 2.95 ns | 1.03x |
| `getpid` | 182.6 ns | 182.3 ns | 1.00x |
| `clock_gettime` | 27.8 ns | 271 ns | 9.74x |
| `open`+`close` | 3431 ns | 3368 ns | 0.98x |
| `stat` | 1470 ns | 1462 ns | 0.99x |
| `getenv` | 96.4 ns | 25.8 ns | 0.27x |
| `pthread_mutex` lock/unlock | 12.3 ns | 7.3 ns | 0.59x |
| `pthread_create`+`join` | 41.1 µs | 41.4 µs | 1.01x |
| Process startup | 337 µs | 947 µs | 2.81x |

The thread benchmark creates and joins 2000 threads; all 2000 ran in every
crabc round.

## What this says

### Strengths

- **Allocator**: after replacing the mmap-per-allocation allocator with
  segregated free lists, small `malloc`/`free` is roughly 8x faster than musl
  in this single-threaded benchmark, and `calloc`/`realloc` are also faster.
- **Math**: the ported functions are within noise of musl (`sin`, `exp`,
  `sqrt`) or a few percent slower (`log`, `pow`).
- **stdio**: formatting is faster, likely because the output path is simpler
  than musl's.
- **Some string/environment paths**: `strstr`, `memcmp`, `getenv`, and
  uncontended mutex operations are faster.

### Remaining gaps

- **`strlen` 4x slower.** crabc uses a scalar word loop; musl uses SIMD.
- **`memcpy`/`memset` 1.3-1.4x slower.** Same scalar-vs-SIMD gap.
- **`clock_gettime` 9.7x slower.** crabc enters the kernel; musl uses the
  vDSO. This is the next high-value fix for timer-heavy programs.
- **Process startup 2.8x slower.** The dynamic linker still spends more time
  on relocation, TLS, and symbol setup than musl. Needs profiling.
- **Thread capacity.** Joinable thread slots are now reused, but the table is
  still capped at 64 concurrent threads. Detached-thread cleanup and a larger
  table are future work.

## Reproduce

```bash
# Build musl 1.2.6 once, then:
MUSL_LIBC=/path/to/musl-1.2.6/lib/libc.so \
BENCH_CPU=0 ROUNDS=5 scripts/bench.sh
```

The script builds the release artifacts, compiles the benchmark binaries,
runs interleaved rounds, and prints the median table. Results are written to
`target/bench/`.
