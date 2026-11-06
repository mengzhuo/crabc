# Node.js

**Status: ✅ passes under crabc**

| | |
|---|---|
| Package | `nodejs-24.22.1-r0` (Alpine 3.23) |
| Version | `v24.22.1` |
| Baseline | same package with stock musl 1.2.6 |

## What was tested

Node.js combines libstdc++, V8, libuv, OpenSSL, zlib, cares, ICU and
nghttp2 in one address space, creates several worker threads at startup, and
loads native modules with `dlopen`.

- `node -v`;
- a JavaScript program using `process.version`, `os.platform()`;
- `fs.writeFileSync` / `readFileSync`;
- an HTTP GET with the `http` module;
- `zlib.gzipSync`;
- `crypto.createHash("sha256")`.

## Commands

```sh
export PATH=/usr/bin:/bin
node -v
node -e 'console.log("core", process.version, 1+1, require("os").platform())'
node -e 'const fs = require("fs")
fs.writeFileSync("/tmp/n.txt", "hello")
console.log("fs", fs.readFileSync("/tmp/n.txt", "utf8"))'
node -e 'const http = require("http")
http.get("http://127.0.0.1:18080/", r => {
  let d = ""; r.on("data", c => d += c)
  r.on("end", () => console.log("http", d.trim()))
})'
node -e 'const zlib = require("zlib")
console.log("zlib", zlib.gzipSync("hello").length)'
node -e 'const crypto = require("crypto")
console.log("crypto",
  crypto.createHash("sha256").update("x").digest("hex").slice(0, 8))'
```

## Output under crabc

```text
v24.22.1
core v24.22.1 2 linux
fs hello
http hello from crabc workload test
zlib 25
crypto 2d711642
NODE_OK
```

## Baseline comparison

Identical output and exit status under musl.

## Notes

Node was the hardest workload and exposed several loader bugs:

- **TLS symbol resolution.** Undefined TLS symbols (`STT_TLS` with
  `st_shndx == SHN_UNDEF`) were resolved to the *referencing* module instead
  of the defining one, so `__once_call` read from the wrong offset. V8's
  `std::call_once` path crashed at startup until this was fixed.
- **`dlopen` TLS layout.** Appending a module with no previous TLS modules
  could shrink `TLS_TOTAL_SIZE`, making the new block smaller than the old one
  and corrupting the copy. The layout now clamps the append point to the old
  total.
- **`pthread_getattr_np` for the main thread** reported a zero stack base,
  which made V8 think it was running on a foreign stack and attempt a stack
  switch. It now reports the mapping that contains the current stack pointer,
  and the loader gives the main thread an 8 MiB stack (or `RLIMIT_STACK`).
- **Thread TLS lifetime.** V8 keeps pointers into a joined thread's TLS and
  reads them during teardown. Unmapping the block (as musl does) turns that
  into `SIGSEGV`. crabc retains the TLS mapping after `pthread_join`; the
  trade-off is documented in this file and the matrix.

## Known issue

Exited worker threads retain their TLS mapping (`TLS_TOTAL_SIZE + TCB_SIZE`,
about 8 KiB each). The block is no longer used by crabc but stays mapped for
the lifetime of the process. This is required by V8's shutdown behaviour in
Node 24. It does not affect correctness or steady-state memory for a fixed
thread pool.
