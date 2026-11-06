# curl

**Status: ✅ passes under crabc**

| | |
|---|---|
| Package | `curl-8.23.0-r0` (Alpine 3.23) |
| Version | `curl 8.23.0 (x86_64-alpine-linux-musl)` |
| Libraries | libcurl/OpenSSL 3.5.9, zlib, brotli, zstd, c-ares, libidn2, libpsl, nghttp2 |
| Baseline | same package with stock musl 1.2.6 |

## What was tested

curl is a good end-to-end test because it pulls in a deep dependency graph
through the loader (`libcurl`, `libssl`, `libcrypto`, c-ares, nghttp2, …),
uses threads, DNS, sockets, TLS, and stdio.

- `curl --version` — verifies recursive `DT_NEEDED` loading and symbol
  resolution across the whole graph;
- `file://` reads;
- plain HTTP GET against a local server;
- an HTTPS GET through the lab proxy, exercising OpenSSL and the system trust
  store.

## Commands

```sh
export PATH=/usr/bin:/bin
curl --version | head -1
curl -sS file:///etc/passwd | head -1
curl -sS http://127.0.0.1:18080/
curl -sS -o /dev/null -w 'http_code=%{http_code}\n' http://127.0.0.1:18080/
https_proxy=http://10.0.0.1:8123 http_proxy=http://10.0.0.1:8123 \
  curl -sS -o /dev/null -w 'https_code=%{http_code}\n' https://example.com/
```

## Output under crabc

```text
curl 8.23.0 (x86_64-alpine-linux-musl) libcurl/8.23.0 OpenSSL/3.5.9 \
zlib/1.3.2 brotli/1.2.1 zstd/1.5.7 c-ares/1.34.9 libidn2/2.3.8 \
libpsl/0.21.5 nghttp2/1.69.0
root:x:0:0:root:/root:/bin/sh
hello from crabc workload test
http_code=200
https_code=200
CURL_OK
```

(The version line is wrapped here for readability; it is one line in the log.)

## Baseline comparison

The musl baseline prints the same version banner and the same three responses,
and exits 0. The HTTPS code is `200` in both runs.

## Notes

- This workload is what forced crabc's loader to load transitive `DT_NEEDED`
  dependencies and to run constructors in dependency order. Both behaviours
  are now part of the loader.
- OpenSSL's RNG initially failed because crabc's `getentropy` did not set
  `errno` correctly; that was fixed and is covered by this test.
- Client certificates, HTTP/3 and proxy authentication were not exercised.
