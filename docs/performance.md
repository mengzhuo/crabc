# Performance

This is a microbenchmark comparison between crabc and musl 1.2.6. The numbers
are a snapshot from one machine, not an application benchmark. Use the ratios
and the reproduction scripts rather than the absolute values.

## Method

- Host (x86\_64): AMD EPYC 7532, Linux 6.12, 64 available CPUs.
- `crabc` release artifacts use the size-oriented release profile:
  `opt-level = "s"`, `strip = "symbols"`, `panic = "abort"`, and no unwind
  tables. This costs a few percent on stdio/math versus `-O3` and shrinks
  `libc.so` by about 30%.
- Baseline: musl 1.2.6 built from the pristine release tarball, used as the
  ELF interpreter/libc.
- The same C source is compiled twice against musl. The two binaries differ
  only in their ELF interpreter:
  - musl: `musl-1.2.6/lib/libc.so`
  - crabc: `target/release/libldso.so` with
    `LD_LIBRARY_PATH=target/release`
- Microbenchmarks: `clock_gettime(CLOCK_MONOTONIC)` around a batch, `-O2
  -fno-builtin`, so calls go through the libc.
- Startup: `fork` + `exec` + `wait` of a minimal dynamically linked program.
- Interleaved rounds, pinned to one CPU; the median is reported. The host is
  shared, so absolute values can move by 10-20% between sessions.

## x86\_64 results

`crabc/musl` below 1.0 means crabc is faster.

| Benchmark | musl | crabc | crabc/musl |
|---|---:|---:|---:|
| `memcpy` 64B | 10.9 ns | 6.8 ns | 0.62x |
| `memcpy` 4KB | 59.0 ns | 45.4 ns | 0.77x |
| `memset` 4KB | 63.3 ns | 50.2 ns | 0.79x |
| `memcmp` 4KB | 861 ns | 36.7 ns | 0.04x |
| `strlen` 4KB | 314 ns | 53.2 ns | 0.17x |
| `strcmp` (short) | 9.3 ns | 4.8 ns | 0.52x |
| `strstr` | 55.1 ns | 8.4 ns | 0.15x |
| `snprintf` | 434 ns | 272 ns | 0.63x |
| `fprintf` → `/dev/null` | 414 ns | 252 ns | 0.61x |
| `malloc`/`free` 16B | 112 ns | 13.1 ns | 0.12x |
| `malloc`/`free` 256B | 112 ns | 12.6 ns | 0.11x |
| `calloc` 1KB | 171 ns | 25.8 ns | 0.15x |
| `realloc` (grow) | 22.3 ns | 9.2 ns | 0.41x |
| `sin` | 5.19 ns | 5.01 ns | 0.96x |
| `exp` | 6.59 ns | 6.53 ns | 0.98x |
| `log` | 8.57 ns | 8.60 ns | 1.00x |
| `pow` | 21.8 ns | 23.9 ns | 1.10x |
| `sqrt` | 2.95 ns | 2.87 ns | 1.00x |
| `getpid` | 183 ns | 182 ns | 1.00x |
| `clock_gettime` | 27.7 ns | 27.7 ns | 1.00x |
| `open`+`close` | 3482 ns | 3404 ns | 0.98x |
| `stat` | 1457 ns | 1455 ns | 1.00x |
| `getenv` | 88.6 ns | 40.9 ns | 0.46x |
| `pthread_mutex` lock/unlock | 12.2 ns | 6.5 ns | 0.53x |
| `pthread_create`+`join` | 38.8 µs | 40.4 µs | 1.04x |
| Process startup | 333 µs | 391 µs | 1.17x |

Geometric mean over the 26 benchmarks: **0.50x musl's runtime**.

The thread benchmark creates and joins 2000 threads; all 2000 ran in every
crabc round.

## riscv64 results

Cross-compiled with `riscv64-linux-gnu-gcc` and run under `qemu-riscv64`.
Both binaries use the same crt objects and differ only in libc/interpreter.
QEMU/TCG numbers are not hardware numbers; only the ratios are meaningful.

| Benchmark | crabc/musl |
|---|---:|
| `memcpy` 4KB | 0.53x |
| `memset` 4KB | 1.01x |
| `memcmp` 4KB | 0.43x |
| `strlen` 4KB | 0.96x |
| `strcmp` (short) | 0.58x |
| `malloc`/`free` 16B | 0.11x |
| `calloc` 1KB | 0.36x |
| `clock_gettime` | 1.06x |

Geometric mean: **0.52x**.

## Binary size

Release artifacts are stripped. musl's dynamic linker is its libc, so its
runtime footprint is one file; crabc needs `libc.so` plus `libldso.so`.

| | x86\_64 | aarch64 | riscv64 |
|---|---:|---:|---:|
| `libc.so` | 680 KB | 731 KB | 662 KB |
| `libldso.so` | 26 KB | 67 KB | 30 KB |
| Runtime total | **706 KB** | **798 KB** | **692 KB** |
| musl `libc.so` | 666 KB | — | — |
| Alpine `.apk` | 606 KB | 596 KB | 570 KB |

For comparison, the 0.4.2 packages were 1.40 MB / 1.41 MB / 1.32 MB.

Static links use `libc.a` (9.7 MB as a Rust archive). Link with
`-Wl,--gc-sections`: a static hello is 26.5 KB versus musl's 17.8 KB, but
without `--gc-sections` it grows to 1.23 MB.

## What changed

- **SIMD string/memory paths.** `strlen`, `memcpy`, `memset`, `memcmp`,
  `memmove` and `strcmp` were rewritten. x86\_64 uses SSE2 (part of the
  baseline) with a runtime-dispatched AVX2 path; aarch64/riscv64 use word
  loops. The riscv64 path uses single `ld`/`sd` instructions (LLVM otherwise
  splits unaligned 64-bit accesses into bytes) and offset-addressed 32B
  blocks.
- **vDSO `clock_gettime`.** crabc resolves `__vdso_clock_gettime` from
  `AT_SYSINFO_EHDR` and calls it directly; the syscall remains the fallback.
- **Packed relative relocations.** `libc.so` is linked with
  `-z pack-relative-relocs`; the loader applies both `DT_RELA` and `DT_RELR`.
- **Dynamic linker.** Symbol lookup uses each object's GNU/SysV hash table
  instead of a linear walk. The loader uses the kernel's existing `AT_PHDR`
  executable mapping instead of re-reading `/proc/self/exe` and remapping
  every segment, defers `$ORIGIN` until a RUNPATH needs it, and no longer
  processes its own already-applied relocations.
- **Symbol visibility.** The interpreter used to export its private
  `memcpy`/`strlen`/... helpers, so programs resolved those names to the
  loader instead of libc. The loader is now registered last in the symbol
  search order.
- **`log` / `pow`.** The near-1 and special-case branches are out of line,
  keeping the common paths compact.
- **Release profile.** `opt-level = "s"` with `strip` and no unwind tables
  trades a few percent of stdio/math throughput for a much smaller package;
  the previous `-O3` build was about 2.3x musl's size, the current one is
  1.06x.

## Remaining gaps

- **Process startup, 1.17x.** musl's dynamic linker *is* its libc, so one
  file is mapped by the kernel. crabc maps `libldso.so` and then loads and
  relocates a separate `libc.so`.
- **`pow`, 1.10x.** The port keeps exact `x^1` / `x^-1` identities (see
  `AGENTS.md`).
- **`snprintf` / `fprintf`, 0.61-0.63x.** Still faster than musl, but slower
  than the `-O3` build (0.53x) because of the size profile.
- **`pthread_create`+`join`, 1.04x.** Within run-to-run noise on this host.

## Reproduce

x86\_64:

```bash
# Build musl 1.2.6 once, then:
MUSL_LIBC=/path/to/musl-1.2.6/lib/libc.so \
BENCH_CPU=0 ROUNDS=5 scripts/bench.sh
```

riscv64 (needs `gcc-riscv64-linux-gnu` and `qemu-user`):

```bash
MUSL_RISCV=/path/to/riscv64/musl/lib scripts/bench-riscv.sh
```

The scripts write raw results to `target/bench/` and
`target/bench-riscv/`.
