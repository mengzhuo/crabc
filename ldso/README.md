# crabc-ldso

A `no_std` Rust dynamic linker (`ldso`) for musl-linked ELF binaries. Produces `libldso.so` which can be used as `--dynamic-linker` to run musl-linked executables.

## Usage

Build with:
```bash
cargo build -p crabc-ldso
```

Output is in `target/debug/libldso.so`.

Run a musl-linked binary:
```bash
LD_LIBRARY_PATH=target/debug ./target/debug/loader my_binary
```

Or directly:
```bash
./my_binary  # if ldso is set as PT_INTERP
```

## Features

- Self-relocating `_start` entry point
- Loads `DT_NEEDED` dependencies
- Handles TLS (Thread-Local Storage) for both x86_64 (TLS_BELOW_TP) and aarch64 (TLS_ABOVE_TP)
- Processes all standard ELF relocation types
- TLSDESC resolver for aarch64

## License

MIT OR Apache-2.0
