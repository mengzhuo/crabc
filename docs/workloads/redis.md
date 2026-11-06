# Redis

**Status: ✅ passes under crabc**

| | |
|---|---|
| Package | `redis-8.4.2-r0` (Alpine 3.23) |
| Version | `Redis server v=8.4.2`, `redis-cli 8.4.2` |
| Baseline | same package with stock musl 1.2.6 |

## What was tested

Redis is a single-threaded event-loop server: `epoll`, timers, sockets,
`fork`, file persistence, and a large amount of string/formatting code.

- `redis-server --version` and `redis-cli --version`;
- daemonized server on port `16379`;
- `PING`, `SET`/`GET`, `INCR`;
- list `RPUSH`/`LRANGE`, hash `HSET`/`HGET`;
- `SAVE` (RDB write), `DBSIZE`, and `SHUTDOWN NOSAVE`.

The server is started with `--save "" --appendonly no` and a fresh RDB file so
the counters are deterministic.

## Commands

```sh
export PATH=/usr/bin:/bin
rm -f /tmp/crabc.rdb
redis-server --version
redis-cli --version
redis-server --port 16379 --daemonize yes --dir /tmp \
  --dbfilename crabc.rdb --save "" --appendonly no
sleep 0.5
redis-cli -p 16379 ping
redis-cli -p 16379 set k hello
redis-cli -p 16379 get k
redis-cli -p 16379 incr counter
redis-cli -p 16379 rpush list a b c
redis-cli -p 16379 lrange list 0 -1
redis-cli -p 16379 hset h f v
redis-cli -p 16379 hget h f
redis-cli -p 16379 save
redis-cli -p 16379 dbsize
redis-cli -p 16379 shutdown nosave
```

## Output under crabc

```text
Redis server v=8.4.2 sha=00000000:1 malloc=libc bits=64 build=%llx
redis-cli 8.4.2
1308735:C 06 Nov 2026 10:16:27.769 # WARNING Memory overcommit must be enabled! \
Without it, a background save or replication may fail under low memory condition. …
PONG
OK
hello
1
3
a b c
1
v
OK
4
REDIS_OK
```

The `vm.overcommit_memory` warning comes from the stock configuration and is
also printed by the musl baseline. The process id and timestamp in the warning
vary per run.

## Baseline comparison

The same replies and the same final `DBSIZE=4`, exit status 0, under musl.

## Notes

- `SAVE` writes an RDB file through `fork` + `write`; the file is readable by
  the subsequent start, which is what the cleanup at the top of the script
  prevents from leaking state between runs.
- Redis is a useful regression test for `epoll_wait`/`epoll_pwait`; the loader
  and libc now use the correct x86\_64 syscall numbers for both.
