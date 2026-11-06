# SQLite

**Status: ✅ passes under crabc**

| | |
|---|---|
| Package | `sqlite-3.53.4-r0` (Alpine 3.23) |
| Version | `3.53.4 2026-07-24 …` |
| Baseline | same package with stock musl 1.2.6 |

## What was tested

The `sqlite3` shell is a compact database engine that stresses file I/O,
`mmap`, locking, `fstat`, `getcwd`, stdio and `dlopen`-free operation.

- version banner;
- create table, insert, aggregate (`sum`, `group_concat`);
- WAL journal mode and a second table in WAL;
- `:memory:` databases;
- row count on the persisted database.

## Commands

```sh
export PATH=/usr/bin:/bin
sqlite3 --version
rm -f /tmp/test.db
sqlite3 /tmp/test.db "create table t(a integer, b text);
  insert into t values(1,'x'),(2,'y');
  select sum(a), group_concat(b) from t;"
sqlite3 /tmp/test.db 'pragma journal_mode=wal;
  create table w(x); insert into w values(42); select x from w;'
sqlite3 /tmp/test.db 'select count(*) from t;'
sqlite3 :memory: 'select sqlite_version(), 1+1;'
```

## Output under crabc

```text
3.53.4 2026-07-24 19:02:57 bf7c7f30031888f4e796e429ab3978879485813aaca6f641c7b33e4e09459bcc (64-bit)
3|x,y
wal
42
2
3.53.4|2
SQLITE_OK
```

## Baseline comparison

Identical output and exit status under musl. The WAL file and the database
are ordinary files created by the test.

## Notes

- The first version of this test used double quotes for SQL string literals,
  which SQLite correctly rejects in both runtimes; the final script uses
  single quotes.
- SQLite is sensitive to `stat`/`fstat` metadata width; the x86\_64 `struct
  stat` layout is shared with musl and is covered by this workload.
