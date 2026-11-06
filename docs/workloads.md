# Real workload matrix

This page answers a single question: **do real, unmodified musl-linked programs
run under crabc?** Nine applications from Alpine 3.23 are executed in two
identical chroots — one with the stock musl loader, one with crabc's
`libldso.so` + `libc.so` — and the observable output is compared.

Everything below is a snapshot from **2026-11-06** on x86\_64. The results are
reproducible with the commands in each report.

## Result

✅ **All nine workloads pass.** For every application, crabc produces the same
exit status and the same relevant stdout as the musl baseline.

| Workload | Version | Status | Report | Scope |
|---|---|---:|:---:|---|
| BusyBox | 1.37.0 | ✅ | [busybox.md](workloads/busybox.md) | 304 applets, `sh`, `ls`, `grep`, `awk`, `sed`, `find`, `wget`, `ps` |
| curl | 8.23.0 | ✅ | [curl.md](workloads/curl.md) | `file://`, HTTP, HTTPS (OpenSSL), proxy |
| Git | 2.52.0 | ✅ | [git.md](workloads/git.md) | init, add, commit, log, diff, status, clone, fsck |
| SQLite | 3.53.4 | ✅ | [sqlite.md](workloads/sqlite.md) | SQL, WAL mode, indexes, in-memory DB |
| nginx | 1.28.3 | ✅ | [nginx.md](workloads/nginx.md) | config test, master/worker, HTTP 1.1, graceful shutdown |
| Redis | 8.4.2 | ✅ | [redis.md](workloads/redis.md) | strings, counters, lists, hashes, `SAVE`, shutdown |
| CPython | 3.12.16 | ✅ | [python.md](workloads/python.md) | pure Python, `ssl`, `sqlite3`, `ctypes`, `zlib`/`bz2`/`lzma` |
| Node.js | 24.22.1 | ✅ | [node.md](workloads/node.md) | JS runtime, `fs`, HTTP client, `zlib`, `crypto` |
| PostgreSQL | 18.6 | ✅ | [postgres.md](workloads/postgres.md) | initdb, server start/stop, DDL, DML, index, `psql` |

## Environment

- Alpine Linux **3.23**, x86\_64, musl 1.2.6 ABI.
- All binaries are the stock Alpine packages (`apk add`), unmodified.
- crabc **0.4.2** with the loader/libc changes in this branch.
- Two chroots:
  - `alpine-root`: pristine musl loader and libc.
  - `alpine-crabc`: same files, but `/lib/ld-musl-x86_64.so.1` is crabc's
    `libldso.so`, `/lib/libc.musl-x86_64.so.1` points at crabc's `libc.so`,
    and `LD_LIBRARY_PATH=/opt/crabc`.
- `/proc`, `/dev`, `/sys` and a shared `/work` directory are bind-mounted into
  both chroots. PostgreSQL and Redis run as uid/gid 1000/1001 as required by
  their packages.

## Method

1. Install the packages into a clean rootfs with `apk --root ... --initdb`.
2. Clone the rootfs; replace only the loader and libc in the second copy.
3. Run the same shell script in both chroots.
4. A workload passes when it exits 0, prints its completion marker, and the
   functional output matches the musl baseline.

The scripts use only the packaged binaries; no recompilation, no patches and no
crabc-specific build flags are involved. The one exception is the test harness
itself, which is a plain shell script.

## Known issues and scope

There are **no passing-to-failing differences** in this matrix. The notes below
describe what was *not* exercised, so the matrix is not over-read:

- **nginx**: TLS termination was not exercised (plaintext HTTP only). The
  packaged OpenSSL path is covered by curl and Python.
- **PostgreSQL**: `initdb` uses `--no-locale`; client authentication is local
  (`-k /tmp`, no TCP password setup). Replication and extension loading were
  not exercised.
- **Git**: only local repositories (`init`, `clone` over `file://`); network
  transports (`git://`, SSH, HTTP) were not exercised.
- **curl**: HTTPS was tested through the lab proxy; no client certificates.
- **BusyBox**: a representative set of applets, not all 304.
- **Node.js**: exited worker threads keep their TLS mapping alive in the
  current implementation. This is deliberate: V8 retains pointers into a
  joined thread's TLS and dereferences them during teardown, so unmapping the
  block turns shutdown into `SIGSEGV`. The retained block is small
  (`TLS_TOTAL_SIZE + TCB_SIZE`, about 8 KiB per thread). It is a compatibility
  trade-off, not a functional failure.
- **Redis** prints the stock `vm.overcommit_memory` warning in the container;
  the musl baseline prints it too.

## Reproducing

The general shape of the setup:

```bash
# 1. Build crabc (dev or release; release is what the container used)
cargo build --release -p crabc-libc -p crabc-ldso --target x86_64-unknown-linux-gnu

# 2. Create the pristine rootfs (Docker is only used for apk)
docker run --rm --network host -v "$PWD/alpine-root:/rootfs" alpine:3.23 \
  sh -c 'apk --root /rootfs --initdb --arch x86_64 -U --allow-untrusted \
    -X https://dl-cdn.alpinelinux.org/alpine/v3.23/main \
    -X https://dl-cdn.alpinelinux.org/alpine/v3.23/community \
    add alpine-baselayout busybox curl git sqlite nginx redis python3 nodejs \
        postgresql postgresql-client openssl ca-certificates'

# 3. Clone it and install crabc as the runtime
cp -a alpine-root alpine-crabc
mkdir -p alpine-crabc/opt/crabc
cp target/x86_64-unknown-linux-gnu/release/libc.so   alpine-crabc/opt/crabc/
cp target/x86_64-unknown-linux-gnu/release/libldso.so alpine-crabc/opt/crabc/
cp alpine-crabc/lib/ld-musl-x86_64.so.1 alpine-crabc/lib/ld-musl-x86_64.so.1.musl
cp alpine-crabc/opt/crabc/libldso.so alpine-crabc/lib/ld-musl-x86_64.so.1
ln -sfn /opt/crabc/libc.so alpine-crabc/lib/libc.musl-x86_64.so.1

# 4. Run a script in a chroot
LD_LIBRARY_PATH=/opt/crabc chroot alpine-crabc /bin/sh /work/busybox.sh
chroot alpine-root /bin/sh /work/busybox.sh   # baseline
```

Each report lists the exact commands and the expected output.
