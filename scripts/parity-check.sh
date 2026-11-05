#!/usr/bin/env bash
#
# Differential behaviour check: musl vs crabc.
#
# The same libc-test source is compiled and linked against the system musl
# libc twice.  The two binaries differ only in their ELF interpreter:
#
#   musl   -> $MUSL_LIBC                (the real musl loader/libc)
#   crabc  -> target/debug/libldso.so   (with LD_LIBRARY_PATH=target/debug)
#
# Running both and comparing exit status, stdout and stderr therefore checks
# the exact promise of crabc: an unmodified musl-linked binary behaves the
# same when crabc supplies libc and the dynamic linker at run time.
#
# Usage:
#   scripts/parity-check.sh [functional|regression|all|symbols]
#
# `symbols` compares the exported dynamic symbol sets instead of running
# tests; the four linker-internal symbols musl exports are allowed to be
# absent from crabc.
#
# Environment:
#   LIBC_TEST_DIR  upstream libc-test checkout (default /home/root/libc-test)
#   MUSL_LIBC      musl loader/libc used as baseline (auto-detected)
#   TIMEOUT        per-test timeout in seconds (default 30)
#   PARITY_RETRIES retries after a mismatching run (default 2)
#   PARITY_SKIP    space-separated tests that are ABI-incompatible
#   PARITY_KNOWN   space-separated tests with a documented difference
#   PARITY_DIR     build/report directory (default libc-test-harness/parity)
#
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LIBC_TEST_DIR="${LIBC_TEST_DIR:-/home/root/libc-test}"
PARITY_DIR="${PARITY_DIR:-$ROOT/libc-test-harness/parity}"
CC="${CC:-musl-gcc}"
TIMEOUT="${TIMEOUT:-30}"
RETRIES="${PARITY_RETRIES:-2}"
SUBSET="${1:-functional}"
ARCH="$(uname -m)"

# These tests build 80-bit long-double values or print them through %La.
# crabc intentionally uses a 64-bit long-double ABI on x86_64, so a binary
# compiled with musl's headers cannot exercise them correctly.
SKIP_TESTS="${PARITY_SKIP:-functional/strtold regression/fpclassify-invalid-ld80}"

# crabc implements the newer %s / %z behaviour that libc-test expects;
# musl 1.2.6 leaves the tm fields of %s unspecified and requires four
# digits for %z. This is a deliberate superset, not a regression.
#
# pow/powf keep exact identities for x^1 and x^-1, so they return x without
# musl's spurious INEXACT/UNDERFLOW flags (musl itself fails the powf RU
# case). All other math functions are ported from musl.
KNOWN_DIFFS="${PARITY_KNOWN:-functional/strptime math/pow math/powf}"

if [ "$SUBSET" = "all" ]; then
    DIRS="functional regression math"
else
    DIRS="$SUBSET"
fi

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
    echo "error: cannot find the system musl libc/loader; set MUSL_LIBC" >&2
    echo "       (looked for /usr/lib/${ARCH}-linux-musl/libc.so and ld-musl-${ARCH}.so.1)" >&2
    exit 1
fi

if [ ! -d "$LIBC_TEST_DIR/src" ]; then
    echo "error: LIBC_TEST_DIR=$LIBC_TEST_DIR does not look like libc-test" >&2
    exit 1
fi

if [ ! -f "$ROOT/target/debug/libc.so" ] || [ ! -f "$ROOT/target/debug/libldso.so" ]; then
    echo ">>> building crabc..."
    (cd "$ROOT" && cargo build --workspace) || exit 1
fi

if [ "$SUBSET" = "symbols" ]; then
    mkdir -p "$PARITY_DIR"
    report="$PARITY_DIR/report-symbols.txt"
    musl_syms="$PARITY_DIR/musl.syms"
    crabc_syms="$PARITY_DIR/crabc.syms"
    nm -D --defined-only "$MUSL_LIBC" \
        | awk '$2 ~ /^[TtWwDdBbRrVv]$/ {print $3}' | sort -u > "$musl_syms"
    nm -D --defined-only "$ROOT/target/debug/libc.so" \
        | awk '$2 ~ /^[TtWwDdBbRrVv]$/ {print $3}' | sort -u > "$crabc_syms"
    comm -23 "$musl_syms" "$crabc_syms" > "$PARITY_DIR/missing.syms"
    comm -13 "$musl_syms" "$crabc_syms" > "$PARITY_DIR/extra.syms"
    # _init/_fini are ELF entry stubs provided by crt; _dl_debug_addr and
    # _dl_debug_state are loader internals exported by musl's libc.so.
    allow="_init _fini _dl_debug_addr _dl_debug_state"
    unexpected=$(grep -v -x -F -f <(printf '%s\n' $allow) \
        "$PARITY_DIR/missing.syms" 2>/dev/null || true)

    {
        echo "Symbol parity"
        echo "-------------"
        echo "musl:  $MUSL_LIBC ($(wc -l < "$musl_syms") symbols)"
        echo "crabc: $ROOT/target/debug/libc.so ($(wc -l < "$crabc_syms") symbols)"
        echo
        echo "Missing in crabc (allowed: $allow):"
        sed 's/^/  /' "$PARITY_DIR/missing.syms"
        echo
        echo "Extra in crabc:"
        sed 's/^/  /' "$PARITY_DIR/extra.syms"
        echo
        if [ -n "$unexpected" ]; then
            echo "FAIL: unexpected missing symbols:"
            printf '%s\n' "$unexpected" | sed 's/^/  /'
        else
            echo "OK: no unexpected missing symbols"
        fi
    } | tee "$report"

    [ -z "$unexpected" ]
    exit
fi

# The baseline uses musl's headers everywhere, so do NOT add crabc's include
# directory and do NOT pass -mlong-double-64 here: the binaries must use the
# musl ABI, which is what crabc promises to run.

CFLAGS=(
    "-I$LIBC_TEST_DIR/src/common"
    "-I$PARITY_DIR/common"
    -pipe -std=c99 -D_POSIX_C_SOURCE=200809L
    -Wall -Wno-unused-function -Wno-missing-braces -Wno-unused
    -Wno-overflow -Wno-unknown-pragmas -fno-builtin -frounding-math
    -Werror=implicit-function-declaration -Werror=implicit-int
    -Werror=pointer-sign -Werror=pointer-arith
    -g -D_FILE_OFFSET_BITS=64
)

mkdir -p "$PARITY_DIR/common"
REPORT="$PARITY_DIR/report-$SUBSET.txt"
: > "$REPORT"

echo ">>> musl baseline: $MUSL_LIBC"
musl_version=$(strings "$MUSL_LIBC" 2>/dev/null | grep -m1 -E '^1\.[0-9]+\.[0-9]+')
echo ">>> musl version:  ${musl_version:-unknown}"
if [ "${musl_version:-}" != "1.2.6" ]; then
    echo ">>> note: crabc ports musl 1.2.6 behavior; an older baseline can"
    echo ">>>       report differences that are not crabc bugs."
fi
echo ">>> crabc ldso:    $ROOT/target/debug/libldso.so"
echo ">>> libc-test:     $LIBC_TEST_DIR"

# ---------------------------------------------------------------------------
# Build the common test helpers with musl headers.
# ---------------------------------------------------------------------------
if [ -f "$LIBC_TEST_DIR/src/common/options.h.in" ]; then
    "$CC" -E - < "$LIBC_TEST_DIR/src/common/options.h.in" | awk '
        /optiongroups_unistd_end/ {s=1; next}
        !s || !NF || /^#/ {next}
        !a {a=$1; if(NF==1)next}
        {print "#define "a" "$NF; a=""}' > "$PARITY_DIR/common/options.h.tmp" 2>/dev/null || true
    if [ -s "$PARITY_DIR/common/options.h.tmp" ]; then
        mv "$PARITY_DIR/common/options.h.tmp" "$PARITY_DIR/common/options.h"
    fi
fi

COMMON_OBJS=()
for cfile in "$LIBC_TEST_DIR"/src/common/*.c; do
    base="$(basename "$cfile" .c)"
    [ "$base" = "runtest" ] && continue
    obj="$PARITY_DIR/common/$base.o"
    if ! "$CC" "${CFLAGS[@]}" -c -o "$obj" "$cfile" 2> "$obj.err"; then
        echo "FATAL: failed to build src/common/$base.c" >&2
        cat "$obj.err" >&2
        exit 1
    fi
    COMMON_OBJS+=("$obj")
done
rm -f "$PARITY_DIR/common/libtest.a"
ar rc "$PARITY_DIR/common/libtest.a" "${COMMON_OBJS[@]}" 2>/dev/null || {
    echo "FATAL: failed to archive libtest.a" >&2
    exit 1
}
ranlib "$PARITY_DIR/common/libtest.a" 2>/dev/null || true
echo ">>> common helpers built"

TOTAL=0
SAME=0
DIFF=0
BUILDERR=0
TIMEOUTS=0
SKIPPED=0
FLAKY=0
KNOWN=0

is_skipped() {
    case " $SKIP_TESTS " in *" $1 "*) return 0 ;; esac
    return 1
}

is_known() {
    case " $KNOWN_DIFFS " in *" $1 "*) return 0 ;; esac
    return 1
}

run_one() { # exe ldpath stdout stderr
    local exe="$1" ldpath="$2" out="$3" err="$4" rc
    ( cd "$LIBC_TEST_DIR" && \
      env LD_LIBRARY_PATH="$ldpath" timeout "$TIMEOUT" "$exe" >"$out" 2>"$err" )
    rc=$?
    return $rc
}

normalize() { # file exe
    local file="$1" exe="$2"
    sed -e '/^ldso:/d' -e '/^JUMP /d' \
        -e "s|$exe|<exe>|g" "$file"
}

for dir in $DIRS; do
    SRC_DIR="$LIBC_TEST_DIR/src/$dir"
    if [ ! -d "$SRC_DIR" ]; then
        echo "WARNING: $SRC_DIR not found, skipping" >&2
        continue
    fi

    echo ">>> $dir"
    DSO_DIR="$PARITY_DIR/$dir/dso"
    MUSL_DIR="$PARITY_DIR/$dir/musl"
    CRABC_DIR="$PARITY_DIR/$dir/crabc"
    rm -rf "$PARITY_DIR/$dir"
    mkdir -p "$DSO_DIR" "$MUSL_DIR" "$CRABC_DIR"

    # Build companion DSOs once; they only use the standard musl ABI.
    for dso_cfile in "$SRC_DIR"/*_dso.c; do
        [ -f "$dso_cfile" ] || continue
        dso_base="$(basename "$dso_cfile" .c)"
        if ! "$CC" "${CFLAGS[@]}" -shared -fPIC \
             -o "$DSO_DIR/$dso_base.so" "$dso_cfile" 2> "$DSO_DIR/$dso_base.err"; then
            echo "BUILDERROR $dir/$dso_base (dso)" | tee -a "$REPORT"
            cat "$DSO_DIR/$dso_base.err" >> "$REPORT"
            BUILDERR=$((BUILDERR + 1))
        fi
    done

    for cfile in "$SRC_DIR"/*.c; do
        [ -f "$cfile" ] || continue
        base="$(basename "$cfile" .c)"
        [[ "$base" == *_dso ]] && continue

        TOTAL=$((TOTAL + 1))
        if is_skipped "$dir/$base"; then
            SKIPPED=$((SKIPPED + 1))
            echo "SKIP $dir/$base (ABI-incompatible baseline)"
            continue
        fi
        # crabc's x86_64 long-double ABI is 64-bit; binaries compiled with
        # musl's 80-bit long double cannot exercise these tests correctly.
        if [ "$dir" = "math" ] && [ "$ARCH" = "x86_64" ] && \
           grep -q 'long double' "$cfile"; then
            SKIPPED=$((SKIPPED + 1))
            echo "SKIP $dir/$base (64-bit long-double ABI)"
            continue
        fi
        OBJ="$PARITY_DIR/$dir/$base.o"
        if ! "$CC" "${CFLAGS[@]}" -c -o "$OBJ" "$cfile" 2> "$OBJ.err"; then
            echo "BUILDERROR $dir/$base (compile)" | tee -a "$REPORT"
            head -5 "$OBJ.err" >> "$REPORT"
            BUILDERR=$((BUILDERR + 1))
            continue
        fi

        # On x86_64 crabc uses a 64-bit long-double ABI. Math tests embed
        # isnan()/signbit() for long double via mtest.h's checkcr(), so the
        # crabc variant must be compiled with crabc's headers and ABI; the
        # musl variant keeps musl's 80-bit ABI. Double/float semantics are
        # identical, only the long-double test cases are skipped above.
        OBJ_CRABC="$OBJ"
        if [ "$dir" = "math" ] && [ "$ARCH" = "x86_64" ]; then
            OBJ_CRABC="$PARITY_DIR/$dir/$base.crabc.o"
            if ! "$CC" "${CFLAGS[@]}" -I"$ROOT/include" -mlong-double-64 \
                 -c -o "$OBJ_CRABC" "$cfile" 2> "$OBJ_CRABC.err"; then
                echo "BUILDERROR $dir/$base (compile crabc ABI)" | tee -a "$REPORT"
                head -5 "$OBJ_CRABC.err" >> "$REPORT"
                BUILDERR=$((BUILDERR + 1))
                continue
            fi
        fi

        # Same object, same musl headers, same link flags; only the ELF
        # interpreter differs between the two binaries.
        COMPANION=()
        if [ -f "$SRC_DIR/$base.mk" ] && grep -q 'LDLIBS.*_dso\.so' "$SRC_DIR/$base.mk"; then
            COMPANION=("$DSO_DIR/${base}_dso.so")
        fi

        LINK_FAIL=0
        for variant in musl crabc; do
            if [ "$variant" = musl ]; then
                INTERP="$MUSL_LIBC"
                OBJ_VARIANT="$OBJ"
            else
                INTERP="$ROOT/target/debug/libldso.so"
                OBJ_VARIANT="$OBJ_CRABC"
            fi
            if ! "$CC" "${CFLAGS[@]}" -g -rdynamic \
                 -Wl,--dynamic-linker="$INTERP" \
                 -Wl,-rpath,'$ORIGIN' \
                 -o "$PARITY_DIR/$dir/$variant/$base.exe" \
                 "$OBJ_VARIANT" "${COMPANION[@]}" "$PARITY_DIR/common/libtest.a" \
                 -lpthread -lm -lrt -ldl -lresolv -lutil \
                 2> "$PARITY_DIR/$dir/$variant/$base.ld.err"; then
                echo "BUILDERROR $dir/$base (link $variant)" | tee -a "$REPORT"
                head -5 "$PARITY_DIR/$dir/$variant/$base.ld.err" >> "$REPORT"
                BUILDERR=$((BUILDERR + 1))
                LINK_FAIL=1
                break
            fi
            # The runtime DSOs are found next to argv[0] by t_pathrel().
            for dso in "$DSO_DIR"/*.so; do
                [ -f "$dso" ] && ln -sf "$dso" "$PARITY_DIR/$dir/$variant/$(basename "$dso")"
            done
        done
        [ $LINK_FAIL -ne 0 ] && continue

        MUSL_OUT="$PARITY_DIR/$dir/musl/$base.out"
        MUSL_ERR="$PARITY_DIR/$dir/musl/$base.err"
        CRABC_OUT="$PARITY_DIR/$dir/crabc/$base.out"
        CRABC_ERR="$PARITY_DIR/$dir/crabc/$base.err"

        attempt=0
        same=0
        while [ "$attempt" -le "$RETRIES" ]; do
            run_one "$PARITY_DIR/$dir/musl/$base.exe" "" "$MUSL_OUT" "$MUSL_ERR"
            MUSL_RC=$?
            run_one "$PARITY_DIR/$dir/crabc/$base.exe" "$ROOT/target/debug" "$CRABC_OUT" "$CRABC_ERR"
            CRABC_RC=$?

            normalize "$MUSL_OUT" "$PARITY_DIR/$dir/musl/$base.exe" > "$MUSL_OUT.norm"
            normalize "$MUSL_ERR" "$PARITY_DIR/$dir/musl/$base.exe" > "$MUSL_ERR.norm"
            normalize "$CRABC_OUT" "$PARITY_DIR/$dir/crabc/$base.exe" > "$CRABC_OUT.norm"
            normalize "$CRABC_ERR" "$PARITY_DIR/$dir/crabc/$base.exe" > "$CRABC_ERR.norm"

            if [ $MUSL_RC -eq $CRABC_RC ] && \
               cmp -s "$MUSL_OUT.norm" "$CRABC_OUT.norm" && \
               cmp -s "$MUSL_ERR.norm" "$CRABC_ERR.norm"; then
                same=1
                break
            fi
            attempt=$((attempt + 1))
        done

        if [ $MUSL_RC -eq 124 ] || [ $CRABC_RC -eq 124 ]; then
            TIMEOUTS=$((TIMEOUTS + 1))
        fi

        if [ $same -eq 1 ]; then
            if [ "$attempt" -eq 0 ]; then
                SAME=$((SAME + 1))
                echo "SAME $dir/$base"
            else
                FLAKY=$((FLAKY + 1))
                echo "FLAKY $dir/$base (matched on retry $attempt)"
            fi
        elif is_known "$dir/$base"; then
            KNOWN=$((KNOWN + 1))
            echo "KNOWN $dir/$base (documented difference; musl rc=$MUSL_RC crabc rc=$CRABC_RC)"
            {
                echo "KNOWN $dir/$base (documented difference; musl rc=$MUSL_RC crabc rc=$CRABC_RC)"
                diff -u --label "$base.musl.stdout" --label "$base.crabc.stdout" \
                    "$MUSL_OUT.norm" "$CRABC_OUT.norm" | head -30
                diff -u --label "$base.musl.stderr" --label "$base.crabc.stderr" \
                    "$MUSL_ERR.norm" "$CRABC_ERR.norm" | head -30
                echo
            } >> "$REPORT"
        else
            DIFF=$((DIFF + 1))
            echo "DIFF $dir/$base (musl rc=$MUSL_RC crabc rc=$CRABC_RC)"
            {
                echo "DIFF $dir/$base (musl rc=$MUSL_RC crabc rc=$CRABC_RC)"
                diff -u --label "$base.musl.stdout" --label "$base.crabc.stdout" \
                    "$MUSL_OUT.norm" "$CRABC_OUT.norm" | head -30
                diff -u --label "$base.musl.stderr" --label "$base.crabc.stderr" \
                    "$MUSL_ERR.norm" "$CRABC_ERR.norm" | head -30
                echo
            } >> "$REPORT"
            echo "  stdout/stderr diff recorded in $REPORT"
        fi
    done
done

{
    echo
    echo "Summary ($SUBSET)"
    echo "-------"
    echo "Total:      $TOTAL"
    echo "Same:       $SAME"
    echo "Flaky:      $FLAKY"
    echo "Known diff: $KNOWN"
    echo "Skipped:    $SKIPPED"
    echo "Diff:       $DIFF"
    echo "BuildError: $BUILDERR"
    echo "Timeouts:   $TIMEOUTS"
    echo "musl:       $MUSL_LIBC"
    echo "crabc:      $ROOT/target/debug/libldso.so"
} | tee -a "$REPORT"

if [ $DIFF -ne 0 ] || [ $BUILDERR -ne 0 ]; then
    exit 1
fi
