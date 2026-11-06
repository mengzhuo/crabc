# Git

**Status: ✅ passes under crabc**

| | |
|---|---|
| Package | `git-2.52.0-r0` (Alpine 3.23) |
| Version | `git version 2.52.0` |
| Baseline | same package with stock musl 1.2.6 |

## What was tested

Git exercises process spawning (`fork`/`exec`, `popen`-style helpers), file
status, recursive directory walks, `mmap`, `readdir`, locks and object
compression.

The scenario creates a repository from scratch:

- `git init` (`.git` layout, hooks, refs);
- `add`, `commit`, `log`, `diff`, `status`;
- `git clone` of the local repository;
- `git fsck` in the clone.

Dates and the default branch are pinned in the script so the commit hash is
stable.

## Commands

```sh
export PATH=/usr/bin:/bin
export HOME=/tmp GIT_PAGER=cat PAGER=cat GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_DATE='2026-01-01T00:00:00 +0000'
export GIT_COMMITTER_DATE='2026-01-01T00:00:00 +0000'
rm -rf /tmp/gitrepo /tmp/gitclone
mkdir -p /tmp/gitrepo && cd /tmp/gitrepo
git --version
git -c init.defaultBranch=main init -q
git config user.email test@example.com
git config user.name Test
echo hello > a.txt
git add a.txt
git commit -qm first
git log --oneline | head -1
echo change >> a.txt
git diff --stat
git status --porcelain
git clone -q /tmp/gitrepo /tmp/gitclone
cd /tmp/gitclone
git log --oneline | head -1
git fsck --no-progress
```

## Output under crabc

```text
git version 2.52.0
33d215a first
 a.txt | 1 +
 1 file changed, 1 insertion(+)
 M a.txt
33d215a first
GIT_OK
```

`git fsck --no-progress` prints nothing on a healthy repository, so the log
ends with `GIT_OK`.

## Baseline comparison

The musl baseline produces the same commit (`33d215a`), the same diff and
status, and exits 0. Git's output is deterministic because the timestamps are
fixed.

## Notes

- Early in the port, `git init` failed with
  `cannot stat '.git/info': File exists`. The cause was `lstat`/`stat`
  returning raw `-errno` instead of `-1` with `errno` set. All public I/O
  wrappers now normalise syscall results the way musl's `__syscall_ret` does.
- Directory streams were originally a process-wide singleton, which broke
  nested `opendir`/`readdir` during `git clone`. Each `DIR` now owns its fd
  and read buffer.
- Network transports (SSH, `git://`, HTTP) were not exercised; only local
  repositories are covered.
