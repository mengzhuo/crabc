# CPython

**Status: ✅ passes under crabc**

| | |
|---|---|
| Package | `python3-3.12.16-r0` (Alpine 3.23) |
| Version | `Python 3.12.16` |
| Baseline | same package with stock musl 1.2.6 |

## What was tested

CPython is the best test of `dlopen`-based extension modules: the standard
library loads `_json`, `_ssl`, `_sqlite3`, `_ctypes`, `zlib`, `_bz2` and
`_lzma` at runtime, each a separate `.so` with its own relocations and TLS.

- `python3 -V`;
- pure-Python modules (`sys`, `os`, `json`, `hashlib`, `re`, `datetime`);
- `_ssl` (`ssl.OPENSSL_VERSION`);
- `_sqlite3` (in-memory table, insert, select);
- `_ctypes` (`CDLL(None).getpid()` through `RTLD_DEFAULT`);
- compression modules (`zlib`, `bz2`, `lzma`).

## Commands

```sh
export PATH=/usr/bin:/bin
python3 -V
python3 -c 'import sys, os, json, hashlib, re, datetime
print("core", sys.version_info[:2], json.dumps({"a": 1}),
      hashlib.sha256(b"x").hexdigest()[:8])'
python3 -c 'import ssl; print("ssl", ssl.OPENSSL_VERSION)'
python3 -c 'import sqlite3
c = sqlite3.connect(":memory:")
c.execute("create table t(x)")
c.execute("insert into t values(7)")
print("sqlite3", c.execute("select x from t").fetchone()[0])'
python3 -c 'import ctypes; print("ctypes", ctypes.CDLL(None).getpid() > 0)'
python3 -c 'import zlib, bz2, lzma
print("compress", len(zlib.compress(b"x"*100)),
      len(bz2.compress(b"x"*100)), len(lzma.compress(b"x"*100)))'
```

## Output under crabc

```text
Python 3.12.16
core (3, 12) {"a": 1} 2d711642
ssl OpenSSL 3.5.9 29 Sep 2026
sqlite3 7
ctypes True
compress 12 39 68
PY_OK
```

## Baseline comparison

Identical output and exit status under musl.

## Notes

- The extension modules are loaded with `dlopen`, so this workload covers
  crabc's `dlopen` path: recursive `DT_NEEDED` loading, relocation of only the
  newly added objects, TLS layout updates, and constructor order.
- `ctypes.CDLL(None)` initially failed with `dlsym: symbol not found` because
  startup dependencies were not part of the global lookup scope. They now are.
- C++ `thread_local` destructors were verified separately with a small
  `g++` fixture; Python itself does not depend on C++ TLS.
