#!/usr/bin/env bash
#
# Compare crabc and musl on riscv64 under qemu-user.
#
# The two benchmark binaries are compiled from the same source with the same
# crt objects and differ only in which libc/interpreter they load. Absolute
# numbers are qemu/TCG numbers; use the ratios.
#
# Usage:
#   scripts/bench-riscv.sh
#
# Environment:
#   MUSL_RISCV   riscv64 musl install with crt*.o and libc.so
#                (default /home/root/musl/lib)
#   CC           cross compiler (default riscv64-linux-gnu-gcc)
#   QEMU         qemu user binary (default qemu-riscv64)
#   ROUNDS       interleaved rounds (default 3)
#   BENCH_DIR    output directory (default target/bench-riscv)
#
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET=riscv64gc-unknown-linux-musl
CC="${CC:-riscv64-linux-gnu-gcc}"
QEMU="${QEMU:-qemu-riscv64}"
MUSL_RISCV="${MUSL_RISCV:-/home/root/musl/lib}"
ROUNDS="${ROUNDS:-3}"
OUT="${BENCH_DIR:-$ROOT/target/bench-riscv}"
TR="$ROOT/target/$TARGET/release"

for f in Scrt1.o crti.o crtn.o libc.so; do
    if [ ! -f "$MUSL_RISCV/$f" ]; then
        echo "error: $MUSL_RISCV/$f missing (set MUSL_RISCV to a riscv64 musl install)" >&2
        exit 1
    fi
done
if ! command -v "$QEMU" >/dev/null; then
    echo "error: $QEMU not found" >&2
    exit 1
fi

echo ">>> building crabc (riscv64 release)..."
(cd "$ROOT" && CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_MUSL_LINKER="$CC" \
    cargo build --release --workspace --target "$TARGET") || exit 1

mkdir -p "$OUT"
CFLAGS=(-O2 -fno-builtin -nostdlib -fPIE -pie)
CRT=("$MUSL_RISCV/Scrt1.o" "$MUSL_RISCV/crti.o" "$MUSL_RISCV/crtn.o")

"$CC" "${CFLAGS[@]}" "${CRT[@]}" -I "$ROOT/include" \
    "$ROOT/scripts/bench/riscv_perf.c" -L "$MUSL_RISCV" -lc -lgcc \
    -Wl,--dynamic-linker="$MUSL_RISCV/libc.so" -o "$OUT/perf_musl" || exit 1
"$CC" "${CFLAGS[@]}" "${CRT[@]}" -I "$ROOT/include" \
    "$ROOT/scripts/bench/riscv_perf.c" -L "$TR" -lc -lgcc \
    -Wl,--dynamic-linker="$TR/libldso.so" -o "$OUT/perf_crabc" || exit 1

echo ">>> measuring ($ROUNDS rounds)..."
rm -f "$OUT"/musl_*.txt "$OUT"/crabc_*.txt
for r in $(seq 1 "$ROUNDS"); do
    "$QEMU" "$OUT/perf_musl" > "$OUT/musl_$r.txt" 2>/dev/null || exit 1
    LD_LIBRARY_PATH="$TR" "$QEMU" "$OUT/perf_crabc" > "$OUT/crabc_$r.txt" 2>/dev/null || exit 1
done

median() {
    sort -n | awk '
        { a[NR] = $1 }
        END {
            if (NR == 0) { print "nan"; exit }
            if (NR % 2) printf "%.0f\n", a[(NR + 1) / 2]
            else printf "%.0f\n", (a[NR / 2] + a[NR / 2 + 1]) / 2
        }'
}

keys=$(awk -F'\t' '{ print $1 }' "$OUT/musl_1.txt")
echo
printf "%-18s %14s %14s %12s\n" "benchmark" "musl ps/op" "crabc ps/op" "crabc/musl"
printf "%-18s %14s %14s %12s\n" "------------------" "--------------" "--------------" "------------"
for k in $keys; do
    m=$(awk -F'\t' -v k="$k" '$1 == k { print $2 }' "$OUT"/musl_*.txt | median)
    c=$(awk -F'\t' -v k="$k" '$1 == k { print $2 }' "$OUT"/crabc_*.txt | median)
    ratio=$(awk -v m="$m" -v c="$c" 'BEGIN { printf "%.2f", c / m }')
    printf "%-18s %14s %14s %11sx\n" "$k" "$m" "$c" "$ratio"
done
echo
echo "baseline: $MUSL_RISCV/libc.so"
echo "crabc:    $TR/libldso.so"
echo "results:  $OUT"
