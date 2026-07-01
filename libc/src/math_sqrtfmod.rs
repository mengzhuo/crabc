// Translated from musl: sqrt.c, sqrtf.c, fmod.c, fmodf.c, sqrt_data.c
// Uses only integer/bit ops and f64::from_bits/to_bits.
// No Rust float methods except final return.

// ============================================================
// sqrt_data: __rsqrt_tab lookup table
// ============================================================

const RSQRT_TAB: [u16; 128] = [
    0xb451,0xb2f0,0xb196,0xb044,0xaef9,0xadb6,0xac79,0xab43,
    0xaa14,0xa8eb,0xa7c8,0xa6aa,0xa592,0xa480,0xa373,0xa26b,
    0xa168,0xa06a,0x9f70,0x9e7b,0x9d8a,0x9c9d,0x9bb5,0x9ad1,
    0x99f0,0x9913,0x983a,0x9765,0x9693,0x95c4,0x94f8,0x9430,
    0x936b,0x92a9,0x91ea,0x912e,0x9075,0x8fbe,0x8f0a,0x8e59,
    0x8daa,0x8cfe,0x8c54,0x8bac,0x8b07,0x8a64,0x89c4,0x8925,
    0x8889,0x87ee,0x8756,0x86c0,0x862b,0x8599,0x8508,0x8479,
    0x83ec,0x8361,0x82d8,0x8250,0x81c9,0x8145,0x80c2,0x8040,
    0xff02,0xfd0e,0xfb25,0xf947,0xf773,0xf5aa,0xf3ea,0xf234,
    0xf087,0xeee3,0xed47,0xebb3,0xea27,0xe8a3,0xe727,0xe5b2,
    0xe443,0xe2dc,0xe17a,0xe020,0xdecb,0xdd7d,0xdc34,0xdaf1,
    0xd9b3,0xd87b,0xd748,0xd61a,0xd4f1,0xd3cd,0xd2ad,0xd192,
    0xd07b,0xcf69,0xce5b,0xcd51,0xcc4a,0xcb48,0xca4a,0xc94f,
    0xc858,0xc764,0xc674,0xc587,0xc49d,0xc3b7,0xc2d4,0xc1f4,
    0xc116,0xc03c,0xbf65,0xbe90,0xbdbe,0xbcef,0xbc23,0xbb59,
    0xba91,0xb9cc,0xb90a,0xb84a,0xb78c,0xb6d0,0xb617,0xb560,
];

// ============================================================
// sqrt helpers
// ============================================================

#[inline]
fn mul32(a: u32, b: u32) -> u32 {
    ((a as u64 * b as u64) >> 32) as u32
}

#[inline]
fn mul64(a: u64, b: u64) -> u64 {
    let ahi = a >> 32;
    let alo = a & 0xffffffff;
    let bhi = b >> 32;
    let blo = b & 0xffffffff;
    ahi * bhi + (ahi * blo >> 32) + (alo * bhi >> 32)
}

// ============================================================
// sqrt
// ============================================================

#[no_mangle]
pub extern "C" fn sqrt(x: f64) -> f64 {
    let ix = asuint64(x);
    let mut top = ix >> 52;

    // special case handling
    if predict_false(top.wrapping_sub(0x001) >= 0x7ff - 0x001) {
        // x < 0x1p-1022 or inf or nan
        if ix.wrapping_mul(2) == 0 {
            return x;
        }
        if ix == 0x7ff0000000000000 {
            return x;
        }
        if ix > 0x7ff0000000000000 {
            return __math_invalid(x);
        }
        // x is subnormal, normalize it
        let ix2 = asuint64(x * p2d(52));
        top = ix2 >> 52;
        top -= 52;
        // use normalized ix
        return sqrt_inner(ix2, top);
    }
    sqrt_inner(ix, top)
}

#[inline]
fn sqrt_inner(ix: u64, top: u64) -> f64 {
    let mut top = top;
    let mut m;

    let even = top & 1;
    m = (ix << 11) | 0x8000000000000000u64;
    if even != 0 {
        m >>= 1;
    }
    top = (top + 0x3ff) >> 1;

    let three = 0xc0000000u64;

    let i = ((ix >> 46) % 128) as usize;
    let mut r = (RSQRT_TAB[i] as u64) << 16;
    // |r sqrt(m) - 1| < 0x1.fdp-9
    let mut s = mul32((m >> 32) as u32, r as u32) as u64;
    // |s/sqrt(m) - 1| < 0x1.fdp-9
    let mut d = mul32(s as u32, r as u32) as u64;
    let mut u = three.wrapping_sub(d);
    r = (mul32(r as u32, u as u32) as u64) << 1;
    // |r sqrt(m) - 1| < 0x1.7bp-16
    s = (mul32(s as u32, u as u32) as u64) << 1;
    // |s/sqrt(m) - 1| < 0x1.7bp-16
    d = mul32(s as u32, r as u32) as u64;
    u = three.wrapping_sub(d);
    r = (mul32(r as u32, u as u32) as u64) << 1;
    // |r sqrt(m) - 1| < 0x1.3704p-29
    r = r << 32;
    s = mul64(m, r);
    d = mul64(s, r);
    u = (three << 32).wrapping_sub(d);
    s = mul64(s, u);
    // -0x1p-57 < s - sqrt(m) < 0x1.8001p-61
    s = (s.wrapping_sub(2)) >> 9;

    let d0 = m.wrapping_shl(42).wrapping_sub(s.wrapping_mul(s));
    let d1 = s.wrapping_sub(d0);
    let d2 = d1.wrapping_add(s).wrapping_add(1);
    let s = s.wrapping_add((d1 >> 63) & 1);
    let mut s = s & 0x000fffffffffffff;
    s |= top << 52;
    let mut y = asdouble(s);
    // handle rounding modes and inexact exception
    let tiny = if predict_false(d2 == 0) { 0u64 } else { 0x0010000000000000u64 };
    let tiny = tiny | ((d1 ^ d2) & 0x8000000000000000);
    let t = asdouble(tiny);
    y = eval_as_double(y + t);
    y
}

// ============================================================
// sqrtf
// ============================================================

#[no_mangle]
pub extern "C" fn sqrtf(x: f32) -> f32 {
    let ix = asuint(x);
    if predict_false(ix.wrapping_sub(0x00800000) >= 0x7f800000 - 0x00800000) {
        // x < 0x1p-126 or inf or nan
        if ix.wrapping_mul(2) == 0 {
            return x;
        }
        if ix == 0x7f800000 {
            return x;
        }
        if ix > 0x7f800000 {
            return __math_invalidf(x);
        }
        // x is subnormal, normalize it
        let ix2 = asuint(x * p2f(23));
        return sqrtf_inner(ix2.wrapping_sub(23 << 23));
    }
    sqrtf_inner(ix)
}

#[inline]
fn sqrtf_inner(ix: u32) -> f32 {
    let even = ix & 0x00800000;
    let m1 = (ix << 8) | 0x80000000u32;
    let m0 = (ix << 7) & 0x7fffffff;
    let m = if even != 0 { m0 } else { m1 };

    // 2^e is the exponent part of the return value
    let mut ey = ix >> 1;
    ey = ey.wrapping_add(0x3f800000 >> 1);
    ey &= 0x7f800000;

    let three = 0xc0000000u32;
    let i = ((ix >> 17) % 128) as usize;
    let mut r = (RSQRT_TAB[i] as u32) << 16;
    let mut s = mul32(m, r);
    let mut d = mul32(s, r);
    let mut u = three.wrapping_sub(d);
    r = mul32(r, u) << 1;
    s = mul32(s, u) << 1;
    d = mul32(s, r);
    u = three.wrapping_sub(d);
    s = mul32(s, u);
    // -0x1.03p-28 < s/sqrt(m) - 1 < 0x1.fp-31
    s = (s.wrapping_sub(1)) >> 6;

    let d0 = m.wrapping_shl(16).wrapping_sub(s.wrapping_mul(s));
    let d1 = s.wrapping_sub(d0);
    let d2 = d1.wrapping_add(s).wrapping_add(1);
    let s = s.wrapping_add((d1 >> 31) & 1);
    let mut s = s & 0x007fffff;
    s |= ey;
    let mut y = asfloat(s);
    // handle rounding and inexact exception
    let tiny = if predict_false(d2 == 0) { 0u32 } else { 0x01000000u32 };
    let tiny = tiny | ((d1 ^ d2) & 0x80000000);
    let t = asfloat(tiny);
    y = eval_as_float(y + t);
    y
}

// ============================================================
// fmod
// ============================================================

#[no_mangle]
pub extern "C" fn fmod(x: f64, y: f64) -> f64 {
    let uxi = asuint64(x);
    let uyi = asuint64(y);
    let ex = ((uxi >> 52) & 0x7ff) as i32;
    let ey = ((uyi >> 52) & 0x7ff) as i32;
    let sx = uxi >> 63;

    if uyi << 1 == 0 || is_nan(y) || ex == 0x7ff {
        return (x * y) / (x * y);
    }
    if uxi << 1 <= uyi << 1 {
        if uxi << 1 == uyi << 1 {
            return 0.0 * x;
        }
        return x;
    }

    // normalize x and y
    let mut uxi = uxi;
    let mut uyi = uyi;
    let mut ex = ex;
    let mut ey = ey;

    if ex == 0 {
        let mut i = uxi << 12;
        while (i as i64) >= 0 {
            ex -= 1;
            i <<= 1;
        }
        uxi <<= (-ex + 1) as u64;
    } else {
        uxi &= (-1i64 as u64) >> 12;
        uxi |= 1u64 << 52;
    }
    if ey == 0 {
        let mut i = uyi << 12;
        while (i as i64) >= 0 {
            ey -= 1;
            i <<= 1;
        }
        uyi <<= (-ey + 1) as u64;
    } else {
        uyi &= (-1i64 as u64) >> 12;
        uyi |= 1u64 << 52;
    }

    // x mod y
    while ex > ey {
        let i = uxi.wrapping_sub(uyi);
        if (i as i64) >= 0 {
            if i == 0 {
                return 0.0 * x;
            }
            uxi = i;
        }
        uxi <<= 1;
        ex -= 1;
    }
    let i = uxi.wrapping_sub(uyi);
    if (i as i64) >= 0 {
        if i == 0 {
            return 0.0 * x;
        }
        uxi = i;
    }
    while uxi >> 52 == 0 {
        uxi <<= 1;
        ex -= 1;
    }

    // scale result
    if ex > 0 {
        uxi -= 1u64 << 52;
        uxi |= (ex as u64) << 52;
    } else {
        uxi >>= (-ex + 1) as u64;
    }
    uxi |= sx << 63;
    asdouble(uxi)
}

// ============================================================
// fmodf
// ============================================================

#[no_mangle]
pub extern "C" fn fmodf(x: f32, y: f32) -> f32 {
    let uxi = asuint(x);
    let uyi = asuint(y);
    let ex = ((uxi >> 23) & 0xff) as i32;
    let ey = ((uyi >> 23) & 0xff) as i32;
    let sx = uxi & 0x80000000;

    if uyi << 1 == 0 || is_nanf(y) || ex == 0xff {
        return (x * y) / (x * y);
    }
    if uxi << 1 <= uyi << 1 {
        if uxi << 1 == uyi << 1 {
            return 0.0f32 * x;
        }
        return x;
    }

    // normalize x and y
    let mut uxi = uxi;
    let mut uyi = uyi;
    let mut ex = ex;
    let mut ey = ey;

    if ex == 0 {
        let mut i = uxi << 9;
        while (i as i32) >= 0 {
            ex -= 1;
            i <<= 1;
        }
        uxi <<= (-ex + 1) as u32;
    } else {
        uxi &= (-1i32 as u32) >> 9;
        uxi |= 1u32 << 23;
    }
    if ey == 0 {
        let mut i = uyi << 9;
        while (i as i32) >= 0 {
            ey -= 1;
            i <<= 1;
        }
        uyi <<= (-ey + 1) as u32;
    } else {
        uyi &= (-1i32 as u32) >> 9;
        uyi |= 1u32 << 23;
    }

    // x mod y
    while ex > ey {
        let i = uxi.wrapping_sub(uyi);
        if (i as i32) >= 0 {
            if i == 0 {
                return 0.0f32 * x;
            }
            uxi = i;
        }
        uxi <<= 1;
        ex -= 1;
    }
    let i = uxi.wrapping_sub(uyi);
    if (i as i32) >= 0 {
        if i == 0 {
            return 0.0f32 * x;
        }
        uxi = i;
    }
    while uxi >> 23 == 0 {
        uxi <<= 1;
        ex -= 1;
    }

    // scale result up
    if ex > 0 {
        uxi -= 1u32 << 23;
        uxi |= (ex as u32) << 23;
    } else {
        uxi >>= (-ex + 1) as u32;
    }
    uxi |= sx;
    asfloat(uxi)
}
