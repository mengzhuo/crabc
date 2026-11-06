# Performance

This is a microbenchmark comparison between crabc and musl 1.2.6. The numbers
are a snapshot from one machine, not an application benchmark. Use the ratios
and the reproduction scripts rather than the absolute values.

## Method

- Host (x86\_64): AMD EPYC 7532, Linux 6.12, 64 available CPUs.
- `crabc` built with `cargo build --release --workspace`.
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
- Five interleaved rounds, pinned to one CPU; the median is reported. The
  host is shared, so absolute values can move by 10-20% between sessions.

## x86\_64 results

`crabc/musl` below 1.0 means crabc is faster.

| Benchmark | musl | crabc | crabc/musl |
|---|---:|---:|---:|
| `memcpy` 64B | 10.9 ns | 6.8 ns | 0.62x |
| `memcpy` 4KB | 59.0 ns | 43.4 ns | 0.74x |
| `memset` 4KB | 63.4 ns | 48.4 ns | 0.76x |
| `memcmp` 4KB | 869 ns | 43.8 ns | 0.05x |
| `strlen` 4KB | 316 ns | 55.2 ns | 0.17x |
| `strcmp` (short) | 10.3 ns | 6.1 ns | 0.60x |
| `strstr` | 54.2 ns | 7.5 ns | 0.14x |
| `snprintf` | 435 ns | 232 ns | 0.53x |
| `fprintf` → `/dev/null` | 416 ns | 227 ns | 0.55x |
| `malloc`/`free` 16B | 110 ns | 13.1 ns | 0.12x |
| `malloc`/`free` 256B | 109 ns | 13.0 ns | 0.12x |
| `calloc` 1KB | 174 ns | 23.2 ns | 0.13x |
| `realloc` (grow) | 21.9 ns | 9.3 ns | 0.42x |
| `sin` | 5.19 ns | 5.25 ns | 1.01x |
| `exp` | 6.57 ns | 6.59 ns | 1.00x |
| `log` | 8.58 ns | 8.90 ns | 1.04x |
| `pow` | 21.9 ns | 23.3 ns | 1.07x |
| `sqrt` | 2.91 ns | 2.94 ns | 1.01x |
| `getpid` | 183 ns | 183 ns | 1.00x |
| `clock_gettime` | 27.9 ns | 27.8 ns | 1.00x |
| `open`+`close` | 3481 ns | 3409 ns | 0.98x |
| `stat` | 1469 ns | 1466 ns | 1.00x |
| `getenv` | 89.7 ns | 26.3 ns | 0.29x |
| `pthread_mutex` lock/unlock | 12.4 ns | 7.2 ns | 0.58x |
| `pthread_create`+`join` | 39.3 µs | 40.7 µs | 1.04x |
| Process startup | 338 µs | 417 µs | 1.23x |

The thread benchmark creates and joins 2000 threads; all 2000 ran in every
crabc round.

## riscv64 results

Cross-compiled with `riscv64-linux-gnu-gcc` and run under `qemu-riscv64`.
Both binaries use the same crt objects and differ only in libc/interpreter.
QEMU/TCG numbers are not hardware numbers; only the ratios are meaningful.

| Benchmark | crabc/musl |
|---|---:|
| `memcpy` 4KB | 0.52x |
| `memset` 4KB | 0.98x |
| `memcmp` 4KB | 0.97x |
| `strlen` 4KB | 0.95x |
| `strcmp` (short) | 1.14x |
| `malloc`/`free` 16B | 0.11x |
| `calloc` 1KB | 0.37x |
| `clock_gettime` | 1.04x |

## What changed

- **SIMD string/memory paths.** `strlen`, `memcpy`, `memset`, `memcmp`,
  `memmove` and `strcmp` were rewritten. x86\_64 uses SSE2 (part of the
  baseline) with a runtime-dispatched AVX2 path; aarch64/riscv64 use word
  loops. The riscv64 path uses single `ld`/`sd` instructions (LLVM otherwise
  splits unaligned 64-bit accesses into bytes) and offset-addressed blocks.
- **vDSO `clock_gettime`.** crabc resolves `__vdso_clock_gettime` from
  `AT_SYSINFO_EHDR` and calls it directly; the syscall remains the fallback.
- **Packed relative relocations.** `libc.so` is linked with
  `-z pack-relative-relocs`; the loader applies both `DT_RELA` and `DT_RELR`,
  which also covers binaries produced by distro toolchains that default to
  packed relocations.
- **Dynamic linker.** Symbol lookup now uses each object's GNU/SysV hash
  table instead of a linear walk over every symbol. The loader uses the
  kernel's existing `AT_PHDR` executable mapping instead of re-reading
  `/proc/self/exe` and remapping every segment, defers the `$ORIGIN`
  readlink until a RUNPATH needs it, and no longer processes its own
  already-applied relocations.
- **Symbol visibility.** The interpreter used to export its private
  `memcpy`/`strlen`/... helpers, so programs (and libc itself) resolved those
  names to the loader instead of libc. The loader is now registered last in
  the symbol search order.
- **`log`.** The near-1 branch is out of line, keeping the common path
  contiguous.

## Remaining gaps

- **Process startup, 1.23x.** musl's dynamic linker *is* its libc, so one
  file is mapped by the kernel. crabc maps `libldso.so` and then loads and
  relocates a separate `libc.so`; the remaining difference is mostly that
  second DSO's relocation and first-touch page-fault cost.
- **`pow`, 1.07x.** The port keeps exact `x^1` / `x^-1` identities (see
  `AGENTS.md`), which adds a small amount of work to the front of the
  function.
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
