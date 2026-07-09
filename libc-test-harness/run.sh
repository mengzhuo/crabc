#!/usr/bin/env bash
#
# Integration harness: build libc-test against crabc's libc.so
# Usage: ./run.sh [subset]
#   subset: functional (default), math, regression, api, all
#
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CRABC_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
LIBC_TEST_DIR="${LIBC_TEST_DIR:-/home/root/libc-test}"
LIBC_SO="${CRABC_DIR}/target/debug/libc.so"
LDSO_SO="${CRABC_DIR}/target/debug/libldso.so"
FAKE_LIBS="${SCRIPT_DIR}/fake-libs"
BUILD_DIR="${SCRIPT_DIR}/build"
REPORT_DIR="${SCRIPT_DIR}/reports"

SUBSET="${1:-functional}"

mkdir -p "$FAKE_LIBS" "$BUILD_DIR" "$REPORT_DIR"


if [ ! -f "$LIBC_SO" ] || [ ! -f "$LDSO_SO" ]; then
    echo ">>> Building crabc..."
    (cd "$CRABC_DIR" && cargo build 2>&1) || { echo "FATAL: cargo build failed"; exit 1; }
fi
echo ">>> Using libc.so: $LIBC_SO"
echo ">>> Using libldso.so: $LDSO_SO"

echo ">>> Setting up fake-libs..."
for lib in libc libpthread libm librt libcrypt libdl libresolv libutil; do
    ln -sf "$LIBC_SO" "$FAKE_LIBS/${lib}.so"
done

WRAPPER_CC="${SCRIPT_DIR}/cc-wrapper.sh"
cat > "$WRAPPER_CC" <<WRAPPER
#!/usr/bin/env bash
exec musl-gcc -L${FAKE_LIBS} "\$@"
WRAPPER
chmod +x "$WRAPPER_CC"

echo ">>> Building runtest.exe (host tool)..."
COMMON_BUILD="$BUILD_DIR/common"
mkdir -p "$COMMON_BUILD"

CFLAGS="-I${LIBC_TEST_DIR}/src/common -pipe -std=c99 -D_POSIX_C_SOURCE=200809L -Wall -Wno-unused-function -Wno-missing-braces -Wno-unused -Wno-overflow -Wno-unknown-pragmas -fno-builtin -frounding-math -Werror=implicit-function-declaration -Werror=implicit-int -Werror=pointer-sign -Werror=pointer-arith -g -D_FILE_OFFSET_BITS=64"

for f in "${LIBC_TEST_DIR}"/src/common/*.c; do
    base=$(basename "$f" .c)
    if [ "$base" = "mtest" ]; then
        musl-gcc $CFLAGS -I"${CRABC_DIR}/include" -c -o "$COMMON_BUILD/$base.o" "$f" 2>"$COMMON_BUILD/$base.o.err" || true
    else
        musl-gcc $CFLAGS -c -o "$COMMON_BUILD/$base.o" "$f" 2>"$COMMON_BUILD/$base.o.err" || true
    fi
done

ar rc "$COMMON_BUILD/libtest.a" \
    "$COMMON_BUILD"/{fdfill,memfill,mtest,path,print,rand,setrlim,utf8,vmfill}.o 2>/dev/null
ranlib "$COMMON_BUILD/libtest.a"

musl-gcc -g -o "$COMMON_BUILD/runtest.exe" \
    "$COMMON_BUILD/runtest.o" "$COMMON_BUILD/libtest.a" \
    -lpthread -lm -lrt 2>"$COMMON_BUILD/runtest.err" || true

if [ ! -x "$COMMON_BUILD/runtest.exe" ]; then
    echo "FATAL: failed to build runtest.exe"
    cat "$COMMON_BUILD/runtest.err"
    exit 1
fi
echo ">>> runtest.exe built OK"

echo ">>> Building and running $SUBSET tests..."

case "$SUBSET" in
    all) DIRS="functional math regression api" ;;
    *)   DIRS="$SUBSET" ;;
esac

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RAW_REPORT="$REPORT_DIR/raw_${TIMESTAMP}.txt"
SUMMARY_REPORT="$REPORT_DIR/summary_${TIMESTAMP}.txt"

> "$RAW_REPORT"

TOTAL=0
BUILDERROR=0
FAIL=0
PASS=0
TIMEOUT=0
OTHER=0

for dir in $DIRS; do
    SRC_DIR="${LIBC_TEST_DIR}/src/$dir"
    if [ ! -d "$SRC_DIR" ]; then
        echo "WARNING: $SRC_DIR not found, skipping"
        continue
    fi

    echo ">>> Processing $dir..."
    DIR_BUILD="$BUILD_DIR/$dir"
    mkdir -p "$DIR_BUILD"

    for cfile in "$SRC_DIR"/*.c; do
        [ -f "$cfile" ] || continue
        base=$(basename "$cfile" .c)

        if echo "$base" | grep -q "dlopen"; then
            echo "BUILDERROR $dir/$base: skipped (dlopen)" >> "$RAW_REPORT"
            BUILDERROR=$((BUILDERROR + 1))
            TOTAL=$((TOTAL + 1))
            continue
        fi

        TOTAL=$((TOTAL + 1))

        OBJ="$DIR_BUILD/${base}.o"
        COMPILE_RC=0
        EXTRA_CFLAGS="-mlong-double-64"
        if [ "$dir" = "math" ]; then
            EXTRA_CFLAGS="$EXTRA_CFLAGS -I${CRABC_DIR}/include"
        fi
        musl-gcc $CFLAGS $EXTRA_CFLAGS -c -o "$OBJ" "$cfile" 2>"$DIR_BUILD/${base}.o.err" || COMPILE_RC=$?

        if [ $COMPILE_RC -ne 0 ]; then
            echo "BUILDERROR $dir/$base: compile failed" >> "$RAW_REPORT"
            cat "$DIR_BUILD/${base}.o.err" >> "$RAW_REPORT" 2>/dev/null
            BUILDERROR=$((BUILDERROR + 1))
            continue
        fi

        # API tests are compile-only checks (no main), skip link/run
        if [ "$dir" = "api" ]; then
            echo "PASS $dir/$base" >> "$RAW_REPORT"
            PASS=$((PASS + 1))
            continue
        fi

        EXE="$DIR_BUILD/${base}.exe"
        LINK_RC=0
        musl-gcc -L"$FAKE_LIBS" -g -o "$EXE" \
            -Wl,--dynamic-linker="$LDSO_SO" \
            "$OBJ" "$COMMON_BUILD/libtest.a" \
            -lpthread -lm -lrt -lcrypt -ldl -lresolv -lutil \
            2>"$DIR_BUILD/${base}.ld.err" || LINK_RC=$?

        if [ $LINK_RC -ne 0 ]; then
            echo "BUILDERROR $dir/$base: link failed" >> "$RAW_REPORT"
            grep -i "undefined reference\|cannot find" "$DIR_BUILD/${base}.ld.err" >> "$RAW_REPORT" 2>/dev/null || true
            BUILDERROR=$((BUILDERROR + 1))
            continue
        fi

        ERRFILE="$DIR_BUILD/${base}.err"
        RUN_RC=0
        timeout 30 env LD_LIBRARY_PATH="$FAKE_LIBS" \
            "$COMMON_BUILD/runtest.exe" -w '' "$EXE" > "$ERRFILE" 2>&1 || RUN_RC=$?

        if [ $RUN_RC -eq 124 ]; then
            echo "TIMEOUT $dir/$base" >> "$RAW_REPORT"
            TIMEOUT=$((TIMEOUT + 1))
        elif [ $RUN_RC -eq 0 ] && [ ! -s "$ERRFILE" ]; then
            echo "PASS $dir/$base" >> "$RAW_REPORT"
            PASS=$((PASS + 1))
        else
            echo "FAIL $dir/$base" >> "$RAW_REPORT"
            head -5 "$ERRFILE" >> "$RAW_REPORT" 2>/dev/null || true
            FAIL=$((FAIL + 1))
        fi
    done
done

cat > "$SUMMARY_REPORT" <<EOF
libc-test Integration Report
============================
Date: $(date)
libc.so: $LIBC_SO
Symbols: $(nm -D "$LIBC_SO" 2>/dev/null | grep -c " T " || echo "unknown")
Subset: $SUBSET

Results
-------
Total:      $TOTAL
PASS:       $PASS
FAIL:       $FAIL
BUILDERROR: $BUILDERROR
TIMEOUT:    $TIMEOUT
Other:      $OTHER

Failure Breakdown (by category):
EOF

echo "" >> "$SUMMARY_REPORT"

if grep -qa "^BUILDERROR" "$RAW_REPORT" 2>/dev/null; then
    echo "BUILDERROR tests:" >> "$SUMMARY_REPORT"
    grep -a "^BUILDERROR" "$RAW_REPORT" | sed 's/BUILDERROR /  /' >> "$SUMMARY_REPORT"
    echo "" >> "$SUMMARY_REPORT"
fi

if grep -qa "^FAIL" "$RAW_REPORT" 2>/dev/null; then
    echo "FAIL tests:" >> "$SUMMARY_REPORT"
    grep -a "^FAIL" "$RAW_REPORT" | sed 's/FAIL /  /' >> "$SUMMARY_REPORT"
    echo "" >> "$SUMMARY_REPORT"
fi

if grep -qa "^TIMEOUT" "$RAW_REPORT" 2>/dev/null; then
    echo "TIMEOUT tests:" >> "$SUMMARY_REPORT"
    grep -a "^TIMEOUT" "$RAW_REPORT" | sed 's/TIMEOUT /  /' >> "$SUMMARY_REPORT"
    echo "" >> "$SUMMARY_REPORT"
fi

if grep -qa "^PASS" "$RAW_REPORT" 2>/dev/null; then
    echo "PASS tests:" >> "$SUMMARY_REPORT"
    grep -a "^PASS" "$RAW_REPORT" | sed 's/PASS /  /' >> "$SUMMARY_REPORT"
fi

echo "" >> "$SUMMARY_REPORT"
echo "Raw report: $RAW_REPORT"

cat "$SUMMARY_REPORT"

ln -sf "$(basename "$SUMMARY_REPORT")" "$REPORT_DIR/latest-summary.txt"
ln -sf "$(basename "$RAW_REPORT")" "$REPORT_DIR/latest-raw.txt"

echo ""
echo ">>> Done. Reports in $REPORT_DIR/"
