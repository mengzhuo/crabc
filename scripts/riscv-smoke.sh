#!/usr/bin/env bash
# Cross-build and smoke-test crabc for riscv64 under qemu-user.
#
# Requirements:
#   rustup target add riscv64gc-unknown-linux-musl
#   apt install gcc-riscv64-linux-gnu qemu-user
#
# Usage:
#   scripts/riscv-smoke.sh
#   RISCV_CC=riscv64-linux-gnu-gcc QEMU=qemu-riscv64 scripts/riscv-smoke.sh
set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TARGET_NAME=riscv64gc-unknown-linux-musl
CARGO_TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
T="$CARGO_TARGET/$TARGET_NAME/debug"
CC_BIN="${RISCV_CC:-riscv64-linux-gnu-gcc}"
QEMU_BIN="${QEMU:-qemu-riscv64}"
GCCINC="$("$CC_BIN" -print-file-name=include)"
TMP="$(mktemp -d /tmp/crabc-riscv-smoke.XXXXXX)"
trap 'rm -rf "$TMP"' EXIT

if [ ! -f "$T/libc.so" ] || [ ! -f "$T/libldso.so" ] || [ ! -x "$T/loader" ]; then
    echo ">>> cross-building crabc for $TARGET_NAME"
    (cd "$ROOT" && CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_MUSL_LINKER="$CC_BIN" \
        cargo build --workspace --target "$TARGET_NAME") || exit 1
fi

LIBS="$TMP/libs"
mkdir -p "$LIBS"
for l in libc libpthread libm librt libcrypt libdl libresolv libutil; do
    ln -sf "$T/libc.so" "$LIBS/$l.so"
done
"$CC_BIN" -shared -fPIC -nostdlib "$ROOT/tests/fixtures/libfoo.c" -o "$LIBS/libfoo.so"
"$CC_BIN" -shared -fPIC -nostdinc -isystem "$GCCINC" -I "$ROOT/include" -D_GNU_SOURCE \
    -L"$LIBS" "$ROOT/tests/fixtures/libtls.c" -Wl,--allow-shlib-undefined \
    -lc -o "$LIBS/libtls.so"

CC=("$CC_BIN" -fPIE -pie -fno-builtin -nostdinc -isystem "$GCCINC"
    -I "$ROOT/include" -D_GNU_SOURCE -Wl,--no-as-needed)
FAILS=0

run() {
    local name=$1 src=$2
    shift 2
    local out="$TMP/$name"
    if ! (cd "$ROOT" && "${CC[@]}" "$@" -Wl,--dynamic-linker="$T/libldso.so" \
        -L"$LIBS" "$src" -Wl,--allow-shlib-undefined -lc -o "$out") \
        >"$TMP/$name.build" 2>&1; then
        echo "BUILDFAIL $name: $(grep -m1 -E 'error:|undefined reference' "$TMP/$name.build" | head -c 160)"
        FAILS=$((FAILS + 1))
        return
    fi
    if ! LD_LIBRARY_PATH="$LIBS" "$QEMU_BIN" "$out" >"$TMP/$name.out" 2>"$TMP/$name.err"; then
        echo "RUNFAIL $name: $(head -c 160 "$TMP/$name.err" | tr '\n' ' ')"
        FAILS=$((FAILS + 1))
        return
    fi
    echo "PASS $name: $(head -c 60 "$TMP/$name.out" | tr '\n' ' ')"
}

run hello "$ROOT/tests/fixtures/hello.c"
for n in ctype string stdlib strtol fenv setjmp signal process socket net statvfs \
         pthread pthread_atfork lrand48 locale iconv fnmatch math pthread_full \
         stdio_full va_list_abi daemon crypt new_functions strverscmp compat; do
    run "$n" "$ROOT/tests/fixtures/${n}_test.c"
done

# Static PIE through the userspace loader.
"$CC_BIN" -fPIE -pie -nostdlib -nostartfiles "$ROOT/tests/fixtures/tiny.c" -o "$TMP/tiny"
if [ "$("$QEMU_BIN" -L /usr/riscv64-linux-gnu "$T/loader" "$TMP/tiny" 2>/dev/null)" = Hello ]; then
    echo "PASS tiny-loader"
else
    echo "FAIL tiny-loader"
    FAILS=$((FAILS + 1))
fi

# TLS, DSO dependency, and startup environment.
run tlstest "$ROOT/tests/fixtures/tlstest.c" -nostdlib -nostartfiles -Wl,-e,_start
run dso_tls "$ROOT/tests/fixtures/dso_tls_test.c" -L"$LIBS" -ltls
run needfoo "$ROOT/tests/fixtures/needfoo.c" -nostdlib -nostartfiles -Wl,-e,_start -L"$LIBS" -lfoo
TEST_STARTUP_VAR=hello_world
export TEST_STARTUP_VAR
run startup_argv_env "$ROOT/tests/fixtures/startup_argv_env.c"
if ! grep -q 'env=hello_world' "$TMP/startup_argv_env.out"; then
    echo "FAIL startup_argv_env: TEST_STARTUP_VAR not visible"
    FAILS=$((FAILS + 1))
fi

echo "riscv64 smoke: FAILS=$FAILS"
exit "$FAILS"
