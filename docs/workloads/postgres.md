# PostgreSQL

**Status: ✅ passes under crabc**

| | |
|---|---|
| Package | `postgresql-18.6-r0` / `postgresql-client` (Alpine 3.23) |
| Version | `PostgreSQL 18.6 on x86_64-alpine-linux-musl` |
| Baseline | same package with stock musl 1.2.6 |

## What was tested

PostgreSQL is a multi-process server (postmaster + backends) and uses fork,
SysV shared memory and semaphores, signals, `sigsetjmp`/`siglongjmp`, sockets,
file locking and a large amount of formatting and locale code.

- `initdb --no-locale -U postgres` (bootstrap mode runs as a child process);
- start with `pg_ctl -w`, unix socket in `/tmp`, TCP disabled;
- `psql` version query and `SELECT 1+1`;
- `CREATE TABLE`, multi-row `INSERT`, aggregate query
  (`sum`, `string_agg`);
- `CREATE INDEX` and an indexed lookup;
- `pg_database_size(current_database()) > 0`;
- fast shutdown with `pg_ctl -m fast -w stop`.

The server runs as uid/gid 1000 (the `postgres` user), as required by the
package; it refuses to run as root.

## Commands

```sh
export PATH=/usr/libexec/postgresql18:/usr/bin:/bin
export PGDATA=/var/lib/postgresql/data
export PGHOST=/tmp PGPORT=15432 PGUSER=postgres

initdb -D "$PGDATA" --no-locale -U postgres
pg_ctl -D "$PGDATA" -o "-p 15432 -k /tmp -c listen_addresses=''" \
       -l /tmp/pg.log -w start
psql -h /tmp -p 15432 -U postgres -Atc 'select version();'
psql -h /tmp -p 15432 -U postgres -Atc 'select 1+1;'
psql -h /tmp -p 15432 -U postgres \
     -c 'create table t(a int, b text);'
psql -h /tmp -p 15432 -U postgres \
     -c "insert into t values (1,'alpha'),(2,'beta'),(3,'gamma');"
psql -h /tmp -p 15432 -U postgres \
     -Atc 'select sum(a), string_agg(b, chr(44) order by a) from t;'
psql -h /tmp -p 15432 -U postgres \
     -Atc 'create index ti on t(a); select count(*) from t where a > 1;'
psql -h /tmp -p 15432 -U postgres \
     -Atc 'select pg_database_size(current_database()) > 0;'
pg_ctl -D "$PGDATA" -m fast -w stop
```

## Output under crabc

```text
INITDB_OK
waiting for server to start.... done
server started
pg_ctl start rc=0
PostgreSQL 18.6 on x86_64-alpine-linux-musl, compiled by cc (Alpine 15.2.0) 15.2.0, 64-bit
2
CREATE TABLE
INSERT 0 3
6|alpha,beta,gamma
CREATE INDEX
2
t
waiting for server to shut down.... done
server stopped
pg_ctl stop rc=0
PG_OK
```

## Baseline comparison

Identical output and exit status under musl, including the `initdb` bootstrap
and the fast shutdown.

## Notes

- The first crabc attempt crashed in the `initdb` bootstrap child with
  `SIGSEGV` (`si_code=SI_KERNEL`) inside an AVX `memset`. The cause was
  `sigsetjmp`: it pushed `rbx` before calling `setjmp`, so the saved stack
  pointer was 8 bytes off. After `longjmp`, the caller ran with a misaligned
  stack and `vmovdqa` faulted. `sigsetjmp` now preserves the musl stack
  layout, and a standalone SIGSEGV/SIGALRM test covers it.
- `initdb` needs a writable `PGDATA` owned by uid 1000, and `/tmp` writable
  for the unix socket.
- Locale support is minimal (`--no-locale`); ICU collations and TLS
  connections to the server were not exercised.
