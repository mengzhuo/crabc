// Port of musl lrint/llrint/lrintf/llrintf/lrintl/llrintl
//
// On x86_64: c_long = i64 (LONG_MAX >= 2^53), c_longlong = i64, long double = f64.
// All six functions reduce to: round-to-integer-using-current-FP-mode, then cast.
//
// The rint/rintf helpers use musl's `x + toint - toint` trick which rounds via
// the current FP rounding mode (MXCSR on x86_64), unlike Rust's `as` which
// truncates toward zero.

// musl rint: rounds f64 to integer using current FP rounding mode
#[inline]
fn rint(x: f64) -> f64 {
    let u = asuint64(x);
    let e = ((u >> 52) & 0x7ff) as i32;
    let s = u >> 63;

    // Already an integer (exponent >= 52 means no fractional bits)
    if e >= 0x3ff + 52 {
        return x;
    }

    // toint = 1/DBL_EPSILON = 2^52 = 4503599627370496.0
    let toint = 4503599627370496.0f64;
    let y = if s != 0 {
        x - toint + toint
    } else {
        x + toint - toint
    };

    if y == 0.0 {
        if s != 0 { -0.0 } else { 0.0 }
    } else {
        y
    }
}

// musl rintf: rounds f32 to integer using current FP rounding mode
#[inline]
fn rintf(x: f32) -> f32 {
    let u = asuint(x);
    let e = ((u >> 23) & 0xff) as i32;
    let s = u >> 31;

    // Already an integer (exponent >= 23 means no fractional bits)
    if e >= 0x7f + 23 {
        return x;
    }

    // toint = 1/FLT_EPSILON = 2^23 = 8388608.0
    let toint = 8388608.0f32;
    let y = if s != 0 {
        x - toint + toint
    } else {
        x + toint - toint
    };

    if y == 0.0 {
        if s != 0 { -0.0f32 } else { 0.0f32 }
    } else {
        y
    }
}

// On x86_64 with c_long = i64 (LONG_MAX >= 2^53), the musl slow path
// (lrint_slow with FE_INEXACT handling) is not needed. Simple cast suffices
// because: (1) rint returns an exact integer as f64, (2) the largest integer
// representable as f64 is 2^53 which fits in i64, (3) for |x| >= 2^52 rint
// returns x unchanged and the saturating `as` cast handles overflow.

#[no_mangle]
pub extern "C" fn lrint(x: f64) -> c_long {
    rint(x) as c_long
}

#[no_mangle]
pub extern "C" fn llrint(x: f64) -> c_longlong {
    rint(x) as c_longlong
}

#[no_mangle]
pub extern "C" fn lrintf(x: f32) -> c_long {
    rintf(x) as c_long
}

#[no_mangle]
pub extern "C" fn llrintf(x: f32) -> c_longlong {
    rintf(x) as c_longlong
}

// On x86_64 with -mlong-double-64: long double = f64, delegate to lrint/llrint.
// On aarch64: long double = f128 (IEEE quad), cast to f64 then round.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub extern "C" fn lrintl(x: f64) -> c_long {
    lrint(x)
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub extern "C" fn llrintl(x: f64) -> c_longlong {
    llrint(x)
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub extern "C" fn lrintl(x: f128) -> c_long {
    lrint(x as f64)
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub extern "C" fn llrintl(x: f128) -> c_longlong {
    llrint(x as f64)
}

#[cfg(target_arch = "riscv64")]
#[no_mangle]
pub extern "C" fn lrintl(x: f128) -> c_long {
    lrint(x as f64)
}

#[cfg(target_arch = "riscv64")]
#[no_mangle]
pub extern "C" fn llrintl(x: f128) -> c_longlong {
    llrint(x as f64)
}
