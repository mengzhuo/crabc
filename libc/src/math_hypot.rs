// Translated from musl: hypot.c, hypotf.c
// Uses only integer/bit ops and helper functions from math_helpers.rs.
// SPLIT constant for double_t (FLT_EVAL_METHOD == 0, standard x86-64).

const HYPOT_SPLIT: f64 = asdouble(0x41A0000002000000); // 2^27 + 1

// ============================================================
// hypotf (float)
// ============================================================

const HYPOTF_SCALE_UP: f32 = asfloat(0x6C800000); // 0x1p90f
const HYPOTF_SCALE_DN: f32 = asfloat(0x12800000); // 0x1p-90f

#[no_mangle]
pub extern "C" fn hypotf(x: f32, y: f32) -> f32 {
    let mut ux = asuint(x) & 0x7FFFFFFF;
    let mut uy = asuint(y) & 0x7FFFFFFF;

    if ux < uy {
        let ut = ux;
        ux = uy;
        uy = ut;
    }

    let mut x = asfloat(ux);
    let mut y = asfloat(uy);

    // hypot(inf, nan) == inf
    if uy == 0xFF << 23 {
        return y;
    }
    if ux >= 0xFF << 23 || uy == 0 || ux.wrapping_sub(uy) >= 25 << 23 {
        return x + y;
    }

    let mut z: f32 = 1.0;
    if ux >= (0x7F + 60) << 23 {
        z = HYPOTF_SCALE_UP;
        x *= HYPOTF_SCALE_DN;
        y *= HYPOTF_SCALE_DN;
    } else if uy < (0x7F - 60) << 23 {
        z = HYPOTF_SCALE_DN;
        x *= HYPOTF_SCALE_UP;
        y *= HYPOTF_SCALE_UP;
    }
    z * sqrtf(((x as f64) * (x as f64) + (y as f64) * (y as f64)) as f32)
}

// ============================================================
// hypot (double)
// ============================================================

const HYPOT_SCALE_UP: f64 = asdouble(0x6BB0000000000000); // 0x1p700
const HYPOT_SCALE_DN: f64 = asdouble(0x1430000000000000); // 0x1p-700

/// Extended precision square: computes x*x as (hi, lo) where hi+lo ≈ x*x
/// with much higher precision using Dekker splitting.
#[inline]
fn hypot_sq(x: f64) -> (f64, f64) {
    let xc = x * HYPOT_SPLIT;
    let xh = x - xc + xc;
    let xl = x - xh;
    let hi = x * x;
    let lo = xh * xh - hi + 2.0 * xh * xl + xl * xl;
    (hi, lo)
}

#[no_mangle]
pub extern "C" fn hypot(x: f64, y: f64) -> f64 {
    let mut ux = asuint64(x) & 0x7FFFFFFFFFFFFFFF;
    let mut uy = asuint64(y) & 0x7FFFFFFFFFFFFFFF;

    // arrange |x| >= |y|
    if ux < uy {
        let ut = ux;
        ux = uy;
        uy = ut;
    }

    let ex = (ux >> 52) as i32;
    let ey = (uy >> 52) as i32;
    let x = asdouble(ux);
    let y = asdouble(uy);

    // hypot(inf, nan) == inf
    if ey == 0x7FF {
        return y;
    }
    if ex == 0x7FF || uy == 0 {
        return x;
    }
    // 64 difference is enough for ld80 double_t
    if ex - ey > 64 {
        return x + y;
    }

    // precise sqrt argument in nearest rounding mode without overflow
    // xh*xh must not overflow and xl*xl must not underflow in sq
    let mut z: f64 = 1.0;
    let mut x = x;
    let mut y = y;
    if ex > 0x3FF + 510 {
        z = HYPOT_SCALE_UP;
        x *= HYPOT_SCALE_DN;
        y *= HYPOT_SCALE_DN;
    } else if ey < 0x3FF - 450 {
        z = HYPOT_SCALE_DN;
        x *= HYPOT_SCALE_UP;
        y *= HYPOT_SCALE_UP;
    }

    let (hx, lx) = hypot_sq(x);
    let (hy, ly) = hypot_sq(y);
    z * sqrt(ly + lx + hy + hx)
}
