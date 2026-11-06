# nginx

**Status: ✅ passes under crabc**

| | |
|---|---|
| Package | `nginx-1.28.3-r0` (Alpine 3.23) |
| Version | `nginx/1.28.3` |
| Baseline | same package with stock musl 1.2.6 |

## What was tested

nginx runs as a master process plus a worker, with a control socket between
them. That makes it a strong test of signals, `recvmsg`/`sendmsg` ancillary
data, `epoll`, `fork`, `chown`, `setuid`/`setgid`, and graceful shutdown.

The test uses a minimal configuration on port `18081`:

- `nginx -v` and `nginx -t` (configuration test);
- start with `daemon off` and one worker;
- serve a static file from `/work/web`;
- serve a fixed body from `/api`;
- check the HTTP status code;
- send `SIGQUIT` and wait for a clean exit.

## Configuration

```nginx
daemon off;
worker_processes 1;
error_log /tmp/nginx-error.log info;
pid /tmp/nginx.pid;
events { worker_connections 64; }
http {
    access_log off;
    client_body_temp_path /tmp/nginx-body;
    proxy_temp_path /tmp/nginx-proxy;
    fastcgi_temp_path /tmp/nginx-fastcgi;
    uwsgi_temp_path /tmp/nginx-uwsgi;
    scgi_temp_path /tmp/nginx-scgi;
    server {
        listen 18081;
        location / { root /work/web; index index.html; }
        location /api { return 200 'nginx-ok\n'; }
    }
}
```

## Commands

```sh
export PATH=/usr/sbin:/usr/bin:/bin
mkdir -p /tmp/logs
nginx -v
nginx -t -c /work/nginx.conf -p /tmp
nginx -c /work/nginx.conf -p /tmp &
P=$!
sleep 1
curl -sS http://127.0.0.1:18081/
curl -sS http://127.0.0.1:18081/api
curl -sS -o /dev/null -w 'code=%{http_code}\n' http://127.0.0.1:18081/index.html
kill -QUIT $P
wait $P
```

## Output under crabc

```text
nginx version: nginx/1.28.3
nginx: configuration file /work/nginx.conf test is successful
hello from crabc workload test
nginx-ok
code=200
NGINX_OK
```

## Baseline comparison

Identical responses and exit status under musl. The graceful-shutdown path was
checked by waiting for the master process; the error log records the worker
exiting with code 0.

## Notes

- The workload requires an `nginx` user/group (uid/gid 1002 in the test
  rootfs) and a writable `/tmp`; this is normal package setup, not a crabc
  workaround.
- nginx exposed two libc bugs during the port:
  - `chown` packed uid/gid into one argument instead of using the five-argument
    `fchownat` ABI, producing `EINVAL` in `chown("/tmp/nginx-body", …)`;
  - `recvmsg` passed musl's 32-bit `msg_iovlen`/`msg_controllen` and their
    padding straight to the kernel. The padding was often non-zero, so the
    kernel saw a huge control length and returned `EMSGSIZE`; the worker never
    received the shutdown message and nginx hung. crabc now copies the header,
    zeroes the pads, and passes a kernel-layout `msghdr`.
- TLS termination is not covered here; see curl and Python for the OpenSSL
  path.
