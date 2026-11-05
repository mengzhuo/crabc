#!/usr/bin/env bash
# Build Alpine Linux .apk packages (apk-tools v3 / ADB format) from release
# artifacts.
#
# Usage: VERSION=0.5.0 scripts/package-apk.sh
#        CONTAINER_RUNTIME=podman scripts/package-apk.sh
#
# Expects the release artifacts produced by:
#   cargo build --release -p crabc-libc -p crabc-ldso --target x86_64-unknown-linux-gnu
#   CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc \
#     cargo build --release -p crabc --target x86_64-unknown-linux-musl
#   CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
#     cargo build --release -p crabc-libc -p crabc-ldso --target aarch64-unknown-linux-gnu
#   CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc \
#     cargo build --release -p crabc --target aarch64-unknown-linux-musl
#   CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_MUSL_LINKER=riscv64-linux-gnu-gcc \
#     cargo build --release --workspace --target riscv64gc-unknown-linux-musl
#
# Requires Docker or podman: the package is assembled with apk-tools 3
# (Alpine 3.23+), which writes the current v3/ADB format.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)}"
DIST="${DIST:-$ROOT/dist}"
REVISION="${REVISION:-r0}"
PKGVER="${VERSION}-${REVISION}"
IMAGE="${APK_BUILDER_IMAGE:-alpine:3.23}"

if [ -z "${CONTAINER_RUNTIME:-}" ]; then
    if command -v docker >/dev/null 2>&1; then
        CONTAINER_RUNTIME=docker
    elif command -v podman >/dev/null 2>&1; then
        CONTAINER_RUNTIME=podman
    else
        echo "error: docker or podman is required to run apk mkpkg" >&2
        exit 1
    fi
fi

mkdir -p "$DIST"

package() {
    local arch="$1" libc="$2" ldso="$3" loader="$4"
    local dir
    dir="$(mktemp -d)"

    install -d "$dir/usr/lib/crabc" "$dir/usr/bin" "$dir/usr/share/doc/crabc"
    install -m 755 "$ROOT/$libc" "$dir/usr/lib/crabc/libc.so"
    install -m 755 "$ROOT/$ldso" "$dir/usr/lib/crabc/libldso.so"
    install -m 755 "$ROOT/$loader" "$dir/usr/bin/crabc-loader"
    install -m 644 "$ROOT/README.md" "$dir/usr/share/doc/crabc/README.md"

    "$CONTAINER_RUNTIME" run --rm \
        -e V="$PKGVER" -e ARCH="$arch" -e SHA="${GITHUB_SHA:-}" \
        -v "$dir:/src:ro" -v "$DIST:/dist" "$IMAGE" sh -eu -c '
            rm -rf /pkg
            cp -a /src /pkg
            chown -R 0:0 /pkg
            chmod 755 /pkg
            set -- -F /pkg \
                -I "name:crabc" \
                -I "version:$V" \
                -I "description:A Rust no_std musl-compatible libc with a dynamic linker" \
                -I "arch:$ARCH" \
                -I "license:MIT OR Apache-2.0" \
                -I "origin:crabc" \
                -I "maintainer:Meng Zhuo <mengzhuo@iscas.ac.cn>" \
                -I "url:https://github.com/mengzhuo/crabc" \
                -I "build-time:$(date +%s)"
            if [ -n "$SHA" ]; then
                set -- "$@" -I "repo-commit:$SHA"
            fi
            set -- "$@" -o "/dist/crabc-$V.$ARCH.apk"
            apk mkpkg "$@"
        '
    rm -rf "$dir"
    echo "built $DIST/crabc-${PKGVER}.${arch}.apk"
}

# x86_64: the no_std libraries are built for the host (gnu) target, the
# loader (std) for musl so it runs on Alpine.
package x86_64 \
    target/x86_64-unknown-linux-gnu/release/libc.so \
    target/x86_64-unknown-linux-gnu/release/libldso.so \
    target/x86_64-unknown-linux-musl/release/loader

# aarch64: the musl target forces crt-static and cannot build a cdylib with
# this toolchain, so build the no_std libraries for gnu and the loader for
# musl (the libraries do not depend on the C library).
package aarch64 \
    target/aarch64-unknown-linux-gnu/release/libc.so \
    target/aarch64-unknown-linux-gnu/release/libldso.so \
    target/aarch64-unknown-linux-musl/release/loader

package riscv64 \
    target/riscv64gc-unknown-linux-musl/release/libc.so \
    target/riscv64gc-unknown-linux-musl/release/libldso.so \
    target/riscv64gc-unknown-linux-musl/release/loader

(cd "$DIST" && sha256sum ./*.apk > SHA256SUMS)
echo "wrote $DIST/SHA256SUMS"
