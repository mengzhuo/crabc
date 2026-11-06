# LTP results

This page answers a narrower question than the workload matrix: **does crabc
pass the Linux Test Project's own test suite?** LTP is the stress/regression
suite used by the kernel and by musl itself. It exercises far more of the libc
ABI than a normal application does, including the corners that only a test
suite reaches.

Everything below is a snapshot from **2026-11-07** on x86\_64. The full
`syscalls` suite takes about 30-35 minutes per runtime; the smaller suites are
much faster.

## Result

The table compares the same LTP binaries in two chroots. `FAIL` counts
non-zero exits (including `TBROK`); `TCONF` counts tests that the LTP library
itself skipped because the environment does not provide a requirement.

| Suite | Tests | musl FAIL | crabc FAIL | musl TCONF | crabc TCONF |
|---|---:|---:|---:|---:|---:|
| `syscalls` | 1488† | 112 | 104 | 139 | 139 |
| `math` | 10 | 0 | 0 | 0 | 0 |
| `mm` | 74 | 2 | 2 | 0 | 0 |
| `sched` | 14 | 1 | 1 | 0 | 0 |
| `nptl` | 1 | 0 | 0 | 0 | 0 |
| `syscalls-ipc` | 57 | 6 | 6 | 3 | 3 |

† Three `syscalls` tags (`fmtmsg01`, `timer_create01`, `timer_create03`) cannot
be built against musl. They are replaced in both chroots by the same static
stub that exits with status 32, so LTP records them as `TCONF` and excludes
them from the pass/fail comparison.

For `math`, `mm`, `sched`, `nptl` and `syscalls-ipc` the **failure lists are
byte-for-byte identical** between musl and crabc. They are shared environment
failures (loop devices, cgroup layout, missing kernel config), not libc
regressions. For `syscalls`, the per-test diff is the important number: one
test, `preadv203`, fails only under crabc, and it is a timing-sensitive cache
test rather than an ABI failure. See
[Remaining differences](#remaining-differences).

## Method

- LTP **20250930**, the last release with the classic `runltp`/`ltp-pan`
  driver. LTP 20260930 removed `runltp` in favour of
  [kirk](https://github.com/linux-test-project/kirk); staying on 20250930 keeps
  the comparison against the documented upstream runner. The suite content is
  the same for the syscall tests used here.
- LTP is built **inside Alpine 3.23** with musl and installed as
  `/opt/ltp-2025`. The same installed tree is copied into both chroots, so the
  test binaries, runner and data files are identical.
- Two chroots:
  - `alpine-root`: stock musl loader and libc. This is the baseline.
  - `alpine-crabc`: same files; `/lib/ld-musl-x86_64.so.1` is crabc's
    `libldso.so`, `/lib/libc.musl-x86_64.so.1` points at crabc's `libc.so`,
    and tests run with `LD_LIBRARY_PATH=/opt/crabc`.
- A suite is run with `runltp -f <suite>` in one chroot, then the other.
  `-C` and `-T` record failing and skipped tags. The two tag lists are diffed
  with `comm`, not just compared by aggregate counts. Aggregate counts move
  when a timing-sensitive test flakes; the per-test diff does not.
- Before each run: set `kernel.sem="250 32000 32 128"`, `kernel.msgmni=32000`,
  `kernel.shmmni=4096`, clear SysV IPC with `ipcrm -a`, and detach stale loop
  devices with `losetup -d /dev/loopN`. LTP formats its own filesystems on
  `/dev/loop2` and above, so a stale loop device produces false failures.
- `TMPDIR=/tmp` and `LTPROOT=/opt/ltp-2025` are set explicitly. The tests run
  as root, matching a container/CI environment.

## Building LTP on musl

LTP's configure step needs `make autotools` first; the release tarball does not
ship `configure`. Two upstream tests do not build against musl:

- `fmtmsg01` calls `addseverity()`, which musl does not implement.
- `timer_create01`/`timer_create03` reference the glibc-internal
  `sigevent._sigev_un` member.

`make -k` builds everything else, but the recursive build stops descending at
the `timer_create` directory: without an explicit second pass, about 110
`syscalls` binaries are never built. We built each affected directory with its
own `make -C <dir> -k -j8` and then copied all executables into
`/opt/ltp-2025/testcases/bin`. This is a build-system issue in LTP, not a
crabc failure.

Verify the install before running:

```bash
grep -c "execvp of" /tmp/ltp-*.out    # must be 0
```

The three unbuildable tags stay in `runtest/syscalls`; they are replaced by a
static stub that exits 32 (`TCONF`), so both sides treat them identically.

## What LTP found and what was fixed

The first full run exposed a long list of real ABI bugs. The final tree
includes fixes for, among others:

- **Thread-local `errno`.** `errno` was a process-global `static mut`, so
  concurrent `mkdir`/`rmdir` threads clobbered each other's error codes. That
  is what made `mkdir09` fail intermittently. Dynamic programs now store it in
  the loader-allocated TCB; static programs use a single-threaded fallback
  because there is no loader to build a TCB. This also removed the last
  dependency on a stub `__tls_get_addr` in `libc.so`.
- **stdio.** `fscanf` ignored length modifiers (`%ld`/`%lu` wrote only 32
  bits), scansets did not support ranges (`%[a-z]`), `%llx` did not consume
  its argument (which shifted every later `%s` and crashed `fanotify09/16`),
  `fgetc` read one byte per syscall (breaking procfs files that only return
  data on the first read), `remove()` did not fall back to `rmdir`, and
  `vdprintf()` issued one `write()` per literal fragment, which cgroupfs
  rejects.
- **Process and signal APIs.** `execv`/`execvp` dropped the environment and
  did not search `PATH`; `clone()` used the glibc argument order and the child
  fell through to `hlt` instead of exiting with the function result;
  `sigset()` was a no-op; `signal()` did not set `SA_RESTART`;
  `sched_getaffinity()` returned the copied byte count instead of 0;
  `ptrace(PTRACE_PEEK*)` ignored the result buffer; `waitid()` now passes the
  kernel's fifth (`rusage`) argument, and `uname()` passes the caller buffer
  to the kernel so an invalid address produces `EFAULT` instead of a local
  copy fault.
- **Syscall ABI details.** `renameat()` passed an undefined fifth argument to
  `renameat2` (EINVAL); `readahead()` returned `int` although musl declares
  `ssize_t`; `creat()` included `O_EXCL`; `dup2()` rejected equal descriptors;
  `alarm()`, `nice()` and `getpriority()` did not match musl's rounding and
  conversions; `posix_fadvise()`/`posix_fallocate()` returned `-1` instead of
  the error number; `preadv`/`pwritev`/`preadv2`/`pwritev2` split the 64-bit
  offset into `pos_l`/`pos_h` incorrectly (the high half was undefined);
  `fchmodat(AT_SYMLINK_NOFOLLOW)` used the wrong syscall.
- **SysV IPC and POSIX message queues.** `semctl()` did not consume the
  `union semun` argument; `mq_*` were a file-descriptor emulation instead of
  the kernel `mq_*` syscalls; `SIGEV_THREAD` notification is now implemented
  with a netlink socket and a helper thread, as musl does.
- **Thread stacks.** `pthread_create()` now carves the TLS block out of the
  top of an application-provided stack, as musl does. A separate TLS `mmap`
  can land directly below a `MAP_GROWSDOWN` stack and prevent it from growing;
  this made `mmap18` fail intermittently. `__rc_init_thread_tls_at` in
  `libldso.so` initializes the embedded block.
- **Loader.** GNU symbol versioning now prefers the default version
  (`foo@@VER`) instead of the first old ABI (`foo@VER`). This mattered for
  libnuma, which exports incompatible 1.1 and 1.2 implementations of
  `numa_get_membind`.

The full set of code changes is in the repository history; this page records
only what the LTP comparison demonstrated.

## Remaining differences

After the final run:

- **One `syscalls` tag fails only under crabc: `preadv203`.** It is an
  inherently timing-sensitive test: three threads race a `RWF_NOWAIT` reader
  against a writer and a cache dropper, and the test fails if it does not
  observe `EAGAIN` within its runtime. On ext2/ext4 the crabc reader performs
  about ten times as many reads in the same window (46M vs 4.4M), keeping the
  pages warm before the dropper can evict them. `strace` confirms that
  `RWF_NOWAIT` is passed and the kernel does return `EAGAIN` on cold pages;
  ext3 and xfs pass. The same test also timed out under the musl baseline in
  other runs, so this is a cache/timing sensitivity, not an ABI regression.
- `clock_settime03` is similarly timing-sensitive and fails intermittently
  under **both** runtimes: it sets `CLOCK_REALTIME` to just before the Y2038
  boundary and waits for an absolute timer that sometimes does not fire before
  LTP's 4-second limit. It appears in both final failure lists.
- The remaining shared `syscalls` failures come from the host/kernel setup:
  module loading (`init_module*`, `finit_module*`, `delete_module*`), quota,
  eventfd/io_uring/cgroup restrictions, missing `/boot/config-*`, and
  tests that assume a non-container sysctl policy. They fail identically
  under musl and are not libc regressions.
- The shared `mm` failures are `mmapstress07` (needs a larger temporary file
  and is sensitive to the host's `/tmp` size) and `vma05.sh` (needs
  `/proc/self/maps` behaviour that this kernel exposes differently).
- The shared `sched` failure is `sched_cli_serv`, which needs a kernel built
  with `CONFIG_SCHED_DEBUG`.
- The shared `syscalls-ipc` failures are the tests that require kernel config
  (`msgget04`, `msgget05`, `shmget02`, `shmget05`, `shmget06`) and
  `msgrcv03`; all fail identically under musl.
- Many `syscalls` TCONFs come from the missing `/boot/config-*` (LTP cannot
  parse a kernel config) and from container restrictions on module loading,
  quota, swap, and certain special filesystems. These are counted separately
  from failures.

Two earlier crabc-only failures are worth calling out because they were real
and are now fixed:

- `fanotify09`/`fanotify16` crashed with `SIGSEGV` while printing an event:
  `%llx` did not consume its argument, so the following `%s` used `fd = -1`
  as a string pointer.
- `mkdir09` failed only under concurrency because of the global `errno`; it
  now passes 3/3 runs.

`ioctl_ficlone*` and `statx05` were never libc failures: the crabc chroot was
missing `mkfs.xfs`/`mkfs.ext4`, so LTP's version probe aborted the test. After
installing the same `e2fsprogs`/`xfsprogs` packages as the baseline, they
match.

## Reproducing

```bash
# 1. Fetch and build LTP 20250930 inside the Alpine chroot.
cd /work/ltp-2025
make autotools
./configure --prefix=/opt/ltp-2025
make -k -j8
make -k install || true          # then build skipped dirs / install TCONF stubs (see above)

# 2. Install crabc into the second chroot.
cp target/x86_64-unknown-linux-gnu/release/libc.so   /path/alpine-crabc/opt/crabc/
cp target/x86_64-unknown-linux-gnu/release/libldso.so /path/alpine-crabc/opt/crabc/
cp /path/alpine-crabc/opt/crabc/libldso.so /path/alpine-crabc/lib/ld-musl-x86_64.so.1
ln -sfn /opt/crabc/libc.so /path/alpine-crabc/lib/libc.musl-x86_64.so.1

# 3. Set the IPC sysctls and detach stale loop devices (see Method above).

# 4. Run a suite in each chroot and diff the failing tag lists.
for r in alpine-root alpine-crabc; do
  lp=; [ "$r" = alpine-crabc ] && lp="LD_LIBRARY_PATH=/opt/crabc"
  chroot "$r" /bin/sh -c "cd /opt/ltp-2025 && export LTPROOT=/opt/ltp-2025; \
    $lp TMPDIR=/tmp ./runltp -f syscalls \
    -l /tmp/ltp-$r.log -o /tmp/ltp-$r.out \
    -C /tmp/ltp-$r.fail -T /tmp/ltp-$r.conf"
done
comm -13 <(awk '{print $1}' alpine-root/tmp/ltp-alpine-root.fail | sort -u) \
         <(awk '{print $1}' alpine-crabc/tmp/ltp-alpine-crabc.fail | sort -u)
```

## Caveats

- LTP mutates the external checkout (it symlinks test sources into
  `src/<subset>/`), so do not share one checkout between concurrent runs.
- The `runltp` driver is deprecated in 20250930 and removed in 20260930; the
  warnings printed at the end of each run are expected.
- The numbers above are a single snapshot on one host. Re-run the suite
  sequentially and compare per-test tags before treating a count change as a
  regression.
