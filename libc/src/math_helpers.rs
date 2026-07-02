

#[inline]
const fn asuint(f: f32) -> u32 {
    f.to_bits()
}

#[inline]
const fn asfloat(i: u32) -> f32 {
    f32::from_bits(i)
}

#[inline]
const fn asuint64(f: f64) -> u64 {
    f.to_bits()
}

#[inline]
const fn asdouble(i: u64) -> f64 {
    f64::from_bits(i)
}

#[inline]
unsafe fn extract_words(hi: *mut u32, lo: *mut u32, d: f64) {
    let u = asuint64(d);
    *hi = (u >> 32) as u32;
    *lo = u as u32;
}

#[inline]
unsafe fn get_high_word(d: f64) -> u32 {
    (asuint64(d) >> 32) as u32
}

#[inline]
unsafe fn get_low_word(d: f64) -> u32 {
    asuint64(d) as u32
}

#[inline]
unsafe fn insert_words(d: *mut f64, hi: u32, lo: u32) {
    *d = asdouble(((hi as u64) << 32) | (lo as u64));
}

#[inline]
unsafe fn set_high_word(d: *mut f64, hi: u32) {
    let lo = asuint64(*d) as u32;
    *d = asdouble(((hi as u64) << 32) | (lo as u64));
}

#[inline]
unsafe fn set_low_word(d: *mut f64, lo: u32) {
    let hi = asuint64(*d) >> 32;
    *d = asdouble((hi << 32) | (lo as u64));
}

#[inline]
unsafe fn get_float_word(f: f32) -> u32 {
    asuint(f)
}

#[inline]
unsafe fn set_float_word(f: *mut f32, w: u32) {
    *f = asfloat(w);
}

#[inline]
fn eval_as_float(x: f32) -> f32 {
    x
}

#[inline]
fn eval_as_double(x: f64) -> f64 {
    x
}

#[inline]
fn fp_barrierf(x: f32) -> f32 {
    unsafe { core::ptr::read_volatile(&x) }
}

#[inline]
fn fp_barrier(x: f64) -> f64 {
    unsafe { core::ptr::read_volatile(&x) }
}

#[inline]
fn fp_force_evalf(x: f32) {
    unsafe {
        core::ptr::read_volatile(&x);
    }
}

#[inline]
fn fp_force_eval(x: f64) {
    unsafe {
        core::ptr::read_volatile(&x);
    }
}

#[inline]
fn fp_force_evall(x: f64) {
    unsafe {
        core::ptr::read_volatile(&x);
    }
}

#[inline]
fn force_eval<T: Copy>(x: T) {
    unsafe {
        core::ptr::read_volatile(&x);
    }
}

#[inline]
fn predict_true(x: bool) -> bool {
    x
}

#[inline]
fn predict_false(x: bool) -> bool {
    x
}


#[inline]
pub fn __math_invalid(x: f64) -> f64 {
    (x - x) / (x - x)
}

#[inline]
pub fn __math_invalidf(x: f32) -> f32 {
    (x - x) / (x - x)
}

#[inline]
pub fn __math_divzero(sign: u32) -> f64 {
    let x = if sign != 0 { -1.0f64 } else { 1.0f64 };
    x / 0.0
}

#[inline]
pub fn __math_divzerof(sign: u32) -> f32 {
    let x = if sign != 0 { -1.0f32 } else { 1.0f32 };
    x / 0.0
}

#[inline]
pub fn __math_xflow(sign: u32, y: f64) -> f64 {
    // musl: (sign ? -y : y) * y  (preserves sign of infinity/zero)
    if sign != 0 {
        -y * y
    } else {
        y * y
    }
}

#[inline]
pub fn __math_xflowf(sign: u32, y: f32) -> f32 {
    if sign != 0 {
        -y * y
    } else {
        y * y
    }
}

#[inline]
pub fn __math_oflow(sign: u32) -> f64 {
    __math_xflow(sign, 1e231f64)
}

#[inline]
pub fn __math_oflowf(sign: u32) -> f32 {
    __math_xflowf(sign, 1.7e38f32)
}

#[inline]
pub fn __math_uflow(sign: u32) -> f64 {
    __math_xflow(sign, 1e-231f64)
}

#[inline]
pub fn __math_uflowf(sign: u32) -> f32 {
    __math_xflowf(sign, 1.2e-38f32)
}

#[no_mangle]
pub static mut signgam: c_int = 0;

#[no_mangle]
pub static mut __signgam: c_int = 0;

pub const FP_NAN: c_int = 0;
pub const FP_INFINITE: c_int = 1;
pub const FP_ZERO: c_int = 2;
pub const FP_SUBNORMAL: c_int = 3;
pub const FP_NORMAL: c_int = 4;

pub const FP_ILOGB0: c_int = -2147483648i32;
pub const FP_ILOGBNAN: c_int = 2147483647i32;

#[inline]
pub fn __fpclassify(x: f64) -> c_int {
    let u = asuint64(x);
    let e = (u >> 52) & 0x7ff;
    if e == 0 {
        if (u << 1) != 0 { FP_SUBNORMAL } else { FP_ZERO }
    } else if e == 0x7ff {
        if (u << 12) != 0 { FP_NAN } else { FP_INFINITE }
    } else {
        FP_NORMAL
    }
}

#[inline]
pub fn __fpclassifyf(x: f32) -> c_int {
    let u = asuint(x);
    let e = (u >> 23) & 0xff;
    if e == 0 {
        if (u << 1) != 0 { FP_SUBNORMAL } else { FP_ZERO }
    } else if e == 0xff {
        if (u << 9) != 0 { FP_NAN } else { FP_INFINITE }
    } else {
        FP_NORMAL
    }
}

#[inline]
pub fn __signbit(x: f64) -> c_int {
    ((asuint64(x) >> 63) & 1) as c_int
}

#[inline]
pub fn __signbitf(x: f32) -> c_int {
    ((asuint(x) >> 31) & 1) as c_int
}

#[inline]
pub fn is_nan(x: f64) -> bool {
    let u = asuint64(x);
    ((u >> 52) & 0x7ff) == 0x7ff && (u << 12) != 0
}

#[inline]
pub fn is_nanf(x: f32) -> bool {
    let u = asuint(x);
    ((u >> 23) & 0xff) == 0xff && (u << 9) != 0
}

#[inline]
pub fn is_inf(x: f64) -> bool {
    let u = asuint64(x);
    ((u >> 52) & 0x7ff) == 0x7ff && (u << 12) == 0
}

#[inline]
pub fn is_inff(x: f32) -> bool {
    let u = asuint(x);
    ((u >> 23) & 0xff) == 0xff && (u << 9) == 0
}

#[inline]
pub fn is_finite(x: f64) -> bool {
    let u = asuint64(x);
    ((u >> 52) & 0x7ff) != 0x7ff
}

#[inline]
pub fn is_finitef(x: f32) -> bool {
    let u = asuint(x);
    ((u >> 23) & 0xff) != 0xff
}

#[inline]
pub fn toint_value() -> f64 {
    4503599627370496.0
}

#[inline]
pub fn tointf_value() -> f32 {
    8388608.0f32
}
