# Agent Handoff

## Current Mission
Build a Rust musl-compatible libc (`librc`) with a dynamic linker that runs existing unmodified musl-linked x86_64 Linux binaries, implementing remaining libc subsystems incrementally against `/home/root/musl` and validating with `/home/root/libc-test`.

## Recent Commits
- `de6c95b` Fix sinh accuracy for small arguments using x87 80-bit precision
- `b658053` Wave 4: port hypot, lrint family, hyperbolic, inverse trig from musl
- `1e329a2` Wave 3: port musl exp/expf, log/logf/log10/log10f/log2/log2f, pow/powf

## Verification Status
- `cargo test --test math` passes.
- libc-test math subset: 66 PASS, 0 FAIL (previously 65/1 with sinh failing).
- `cargo test --workspace` still fails the pre-existing `new_functions` test because `/dev/null` is a regular file in this environment, not a char device. This is environment-specific and unrelated to recent changes.

## Active Work
- `sinh` libc-test failure fixed by routing `2^-10 <= |x| < ln2` through an x86_64 x87 helper.
- All Wave 4 math functions (hypot, lrint family, hyperbolic, inverse trig) are committed and validated.

## Next Steps
1. Port remaining math subsystems from musl: `expm1`/`expm1f`, `exp2`/`exp2f`, `log1p`/`log1pf`, `cbrt`, `acosh`/`asinh`/`atanh`, `erf`/`erfc`, `tgamma`/`lgamma`, Bessel functions, `remainder`/`remquo`, `ilogb`/`logb`, `nextafter`, etc.
2. Implement `long double` variants (currently 64-bit ABI with `-mlong-double-64`).
3. Implement `dlopen`/DSO support.
4. Continue functional libc subsystems beyond math.

## Critical Notes
- Each feature must have tests and a commit.
- Port musl algorithms literally; no `libm` crate wrappers for implemented functions.
- Compile libc-test math cases with `-mlong-double-64` in `libc-test-harness/run.sh`.
- x87 inline asm used in `libc/src/math_hyperbolic.rs` for `sinh` small-argument path.

## Key Files
- `/home/root/librc/libc/src/math_*.rs`: math implementations.
- `/home/root/librc/libc/src/lib.rs`: module wiring and syscall wrappers.
- `/home/root/librc/include/math.h`: math declarations.
- `/home/root/librc/tests/fixtures/math_test.c`: C integration regression tests.
- `/home/root/librc/libc-test-harness/run.sh`: libc-test harness script.
- `/home/root/musl/src/math/`: upstream source for ports.

## Environment Quirks
- `/dev/null` is a regular file, causing `tests/new_functions.rs` to fail.
- `RUST_TEST_THREADS=1` is set to serialize integration tests.
- `overflow-checks = false` in dev profile to match musl's unsigned wraparound arithmetic.
