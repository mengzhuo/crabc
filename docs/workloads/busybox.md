# BusyBox

**Status: ✅ passes under crabc**

| | |
|---|---|
| Package | `busybox-1.37.0-r30` (Alpine 3.23) |
| Version | `BusyBox v1.37.0 (2025-12-16 14:19:28 UTC) multi-call binary` |
| Interpreter | crabc `libldso.so` + `libc.so` |
| Baseline | same package with stock musl 1.2.6 |

## What was tested

BusyBox is the first thing that runs in an Alpine userspace and exercises a
large part of the libc surface through one binary. The test covers:

- version banner and applet enumeration;
- `sh` (fork/exec, redirection, pipelines, builtins);
- `ls`, `grep`, `sed`, `awk`, `find`, `ps`;
- `wget` against a local HTTP server;
- file I/O, `stat`, `opendir`/`readdir`, `getpw*`, and process listing.

## Commands

```sh
export PATH=/usr/bin:/bin
busybox | head -1
echo "applets: $(busybox --list | wc -l)"
busybox sh -c 'echo shell-ok; ls / >/dev/null && echo ls-ok'
busybox grep -c root /etc/passwd
busybox awk 'BEGIN { print "awk", 1+1 }'
busybox sed -n '1p' /etc/passwd
busybox find /etc -maxdepth 1 -name passwd
busybox wget -qO- http://127.0.0.1:18080/
busybox ps | head -2
```

## Output under crabc

```text
BusyBox v1.37.0 (2025-12-16 14:19:28 UTC) multi-call binary.
applets: 304
shell-ok
ls-ok
1
awk 2
root:x:0:0:root:/root:/bin/sh
/etc/passwd
hello from crabc workload test
PID   USER     TIME  COMMAND
    1 root      0:04 {systemd} /sbin/init
BUSYBOX_OK
```

The `ps` line shows the host's PID 1 because `/proc` is bind-mounted into the
chroot; this is identical under the musl baseline.

## Baseline comparison

The musl run produces byte-for-byte the same output and exits 0
(`BUSYBOX_OK`). See `docs/workloads.md` for the matrix.

## Notes

- The applet count (304) is the same under both runtimes.
- `busybox wget` is a useful smoke test for crabc's socket + DNS + stdio
  stack; it passes without special configuration.
