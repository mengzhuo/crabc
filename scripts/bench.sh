#!/usr/bin/env bash
#
# Compare crabc against musl with a small set of microbenchmarks and a
# process-startup benchmark.
#
# The same C source is compiled twice against musl; the binaries differ only
# in their ELF interpreter, so the same code runs on musl and with crabc's
# libldso.so/libc.so at run time.
#
# Usage:
#   scripts/bench.sh
#
# Environment:
#   MUSL_LIBC    musl loader/libc used as the baseline (auto-detected)
#   ROUNDS       interleaved measurement rounds (default 5)
#   SPAWN_RUNS   fork+exec runs per startup measurement (default 1000)
#   BENCH_CPU    optional CPU number for taskset
#   BENCH_DIR    build/output directory (default target/bench)
#   CC           compiler (default musl-gcc)
#
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ARCH="$(uname -m)"
CC="${CC:-musl-gcc}"
ROUNDS="${ROUNDS:-5}"
SPAWN_RUNS="${SPAWN_RUNS:-1000}"
BENCH_DIR="${BENCH_DIR:-$ROOT/target/bench}"
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
case "$TARGET_DIR" in
    /*) ;;
    *) TARGET_DIR="$ROOT/$TARGET_DIR" ;;
esac

if [ -z "${MUSL_LIBC:-}" ]; then
    for candidate in \
        "/usr/lib/${ARCH}-linux-musl/libc.so" \
        "/lib/ld-musl-${ARCH}.so.1" \
        "/usr/lib/ld-musl-${ARCH}.so.1" \
        "/usr/lib/musl/lib/libc.so"
    do
        if [ -x "$candidate" ]; then
            MUSL_LIBC="$candidate"
            break
        fi
    done
fi
if [ -z "${MUSL_LIBC:-}" ] || [ ! -x "$MUSL_LIBC" ]; then
    echo "error: cannot find a musl libc; set MUSL_LIBC" >&2
    exit 1
fi

PIN=()
if [ -n "${BENCH_CPU:-}" ]; then
    PIN=(taskset -c "$BENCH_CPU")
fi
run_musl() { "${PIN[@]}" env -u LD_LIBRARY_PATH "$@"; }
run_crabc() { "${PIN[@]}" env LD_LIBRARY_PATH="$TARGET_DIR/release" "$@"; }

echo ">>> building crabc (release)..."
(cd "$ROOT" && cargo build --release --workspace) || exit 1

mkdir -p "$BENCH_DIR"
CFLAGS=(-O2 -fno-builtin -pthread -fPIE -pie)

echo ">>> compiling benchmarks..."
"$CC" "${CFLAGS[@]}" -Wl,--dynamic-linker="$MUSL_LIBC" \
    "$ROOT/scripts/bench/perf_test.c" -o "$BENCH_DIR/perf_musl" -lm || exit 1
"$CC" "${CFLAGS[@]}" -Wl,--dynamic-linker="$TARGET_DIR/release/libldso.so" \
    "$ROOT/scripts/bench/perf_test.c" -o "$BENCH_DIR/perf_crabc" -lm || exit 1
"$CC" -O2 -static "$ROOT/scripts/bench/spawn_bench.c" -o "$BENCH_DIR/spawn_bench" || exit 1
"$CC" -O2 -fPIE -pie -Wl,--dynamic-linker="$MUSL_LIBC" \
    "$ROOT/scripts/bench/hello_main.c" -o "$BENCH_DIR/hello_musl" || exit 1
"$CC" -O2 -fPIE -pie -Wl,--dynamic-linker="$TARGET_DIR/release/libldso.so" \
    "$ROOT/scripts/bench/hello_main.c" -o "$BENCH_DIR/hello_crabc" || exit 1

rm -f "$BENCH_DIR"/musl_*.txt "$BENCH_DIR"/crabc_*.txt
: > "$BENCH_DIR/startup_musl.txt"
: > "$BENCH_DIR/startup_crabc.txt"

echo ">>> measuring ($ROUNDS rounds, $SPAWN_RUNS startups each)..."
for r in $(seq 1 "$ROUNDS"); do
    run_musl timeout 300 "$BENCH_DIR/perf_musl" > "$BENCH_DIR/musl_$r.txt" || exit 1
    run_crabc timeout 300 "$BENCH_DIR/perf_crabc" > "$BENCH_DIR/crabc_$r.txt" || exit 1
    m=$(run_musl "$BENCH_DIR/spawn_bench" "$BENCH_DIR/hello_musl" "$SPAWN_RUNS") || exit 1
    c=$(run_crabc "$BENCH_DIR/spawn_bench" "$BENCH_DIR/hello_crabc" "$SPAWN_RUNS") || exit 1
    echo "$m" >> "$BENCH_DIR/startup_musl.txt"
    echo "$c" >> "$BENCH_DIR/startup_crabc.txt"
done

median() {
    sort -n | awk '
        { a[NR] = $1 }
        END {
            if (NR == 0) { print "nan"; exit }
            if (NR % 2) printf "%.3f\n", a[(NR + 1) / 2]
            else printf "%.3f\n", (a[NR / 2] + a[NR / 2 + 1]) / 2
        }'
}

keys=$(awk -F'\t' '$1 != "sink" && $1 != "thread_runs" { print $1 }' \
    "$BENCH_DIR/musl_1.txt")

echo
printf "%-24s %14s %14s %12s\n" "benchmark" "musl ns/op" "crabc ns/op" "crabc/musl"
printf "%-24s %14s %14s %12s\n" "------------------------" "--------------" "--------------" "------------"
for k in $keys; do
    m=$(awk -F'\t' -v k="$k" '$1 == k { print $2 }' "$BENCH_DIR"/musl_*.txt | median)
    c=$(awk -F'\t' -v k="$k" '$1 == k { print $2 }' "$BENCH_DIR"/crabc_*.txt | median)
    ratio=$(awk -v m="$m" -v c="$c" 'BEGIN { printf "%.2f", c / m }')
    printf "%-24s %14s %14s %11sx\n" "$k" "$m" "$c" "$ratio"
done
m=$(median < "$BENCH_DIR/startup_musl.txt")
c=$(median < "$BENCH_DIR/startup_crabc.txt")
ratio=$(awk -v m="$m" -v c="$c" 'BEGIN { printf "%.2f", c / m }')
printf "%-24s %14s %14s %11sx\n" "process_startup" "$m" "$c" "$ratio"

runs=$(awk -F'\t' '$1 == "thread_runs" { print $2 }' "$BENCH_DIR"/crabc_*.txt | sort -u | tr '\n' ' ')
echo
echo "baseline: $MUSL_LIBC"
echo "crabc:    $TARGET_DIR/release/libldso.so"
echo "threads:  $runs (expected 2000 per round)"
echo "results:  $BENCH_DIR"
