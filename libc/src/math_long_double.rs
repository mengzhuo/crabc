#[cfg(target_arch = "x86_64")]
macro_rules! ld_wrapper {
    ($name:ident, $base:path, f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f64) -> f64 {
            $base(x)
        }
    };
    ($name:ident, $base:path, f64_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f64, y: f64) -> f64 {
            $base(x, y)
        }
    };
    ($name:ident, $base:path, f64_int_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f64, n: i32) -> f64 {
            $base(x, n)
        }
    };
    ($name:ident, $base:path, f64_long_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f64, n: core::ffi::c_long) -> f64 {
            $base(x, n)
        }
    };
    ($name:ident, $base:path, f64_f64_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f64, y: f64, z: f64) -> f64 {
            $base(x, y, z)
        }
    };
    ($name:ident, $base:path, int_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f64) -> i32 {
            $base(x)
        }
    };
    ($name:ident, $base:path, long_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f64) -> core::ffi::c_long {
            $base(x)
        }
    };
    ($name:ident, $base:path, longlong_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f64) -> i64 {
            $base(x)
        }
    };
}

#[cfg(target_arch = "aarch64")]
macro_rules! ld_wrapper {
    ($name:ident, $base:path, f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128) -> f128 {
            $base(x as f64) as f128
        }
    };
    ($name:ident, $base:path, f64_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128, y: f128) -> f128 {
            $base(x as f64, y as f64) as f128
        }
    };
    ($name:ident, $base:path, f64_int_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128, n: i32) -> f128 {
            $base(x as f64, n) as f128
        }
    };
    ($name:ident, $base:path, f64_long_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128, n: core::ffi::c_long) -> f128 {
            $base(x as f64, n) as f128
        }
    };
    ($name:ident, $base:path, f64_f64_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128, y: f128, z: f128) -> f128 {
            $base(x as f64, y as f64, z as f64) as f128
        }
    };
    ($name:ident, $base:path, int_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128) -> i32 {
            $base(x as f64)
        }
    };
    ($name:ident, $base:path, long_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128) -> core::ffi::c_long {
            $base(x as f64)
        }
    };
    ($name:ident, $base:path, longlong_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128) -> i64 {
            $base(x as f64)
        }
    };
}

#[cfg(target_arch = "riscv64")]
macro_rules! ld_wrapper {
    ($name:ident, $base:path, f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128) -> f128 {
            $base(x as f64) as f128
        }
    };
    ($name:ident, $base:path, f64_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128, y: f128) -> f128 {
            $base(x as f64, y as f64) as f128
        }
    };
    ($name:ident, $base:path, f64_int_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128, n: i32) -> f128 {
            $base(x as f64, n) as f128
        }
    };
    ($name:ident, $base:path, f64_long_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128, n: core::ffi::c_long) -> f128 {
            $base(x as f64, n) as f128
        }
    };
    ($name:ident, $base:path, f64_f64_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128, y: f128, z: f128) -> f128 {
            $base(x as f64, y as f64, z as f64) as f128
        }
    };
    ($name:ident, $base:path, int_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128) -> i32 {
            $base(x as f64)
        }
    };
    ($name:ident, $base:path, long_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128) -> core::ffi::c_long {
            $base(x as f64)
        }
    };
    ($name:ident, $base:path, longlong_f64_ret) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(x: f128) -> i64 {
            $base(x as f64)
        }
    };
}

ld_wrapper!(acosl, crate::acos, f64_ret);
ld_wrapper!(asinl, crate::asin, f64_ret);
ld_wrapper!(atanl, crate::atan, f64_ret);
ld_wrapper!(ceill, crate::ceil, f64_ret);
ld_wrapper!(cosl, crate::cos, f64_ret);
ld_wrapper!(coshl, crate::cosh, f64_ret);
ld_wrapper!(expl, crate::exp, f64_ret);
ld_wrapper!(fabsl, crate::fabs, f64_ret);
ld_wrapper!(floorl, crate::floor, f64_ret);
ld_wrapper!(hypotl, crate::hypot, f64_f64_ret);
ld_wrapper!(ldexpl, crate::ldexp, f64_int_ret);
ld_wrapper!(logl, crate::log, f64_ret);
ld_wrapper!(log2l, crate::log2, f64_ret);
ld_wrapper!(log10l, crate::log10, f64_ret);
ld_wrapper!(powl, crate::pow, f64_f64_ret);
ld_wrapper!(roundl, crate::round, f64_ret);
ld_wrapper!(scalblnl, crate::scalbln, f64_long_ret);
ld_wrapper!(scalbnl, crate::scalbn, f64_int_ret);
ld_wrapper!(sinl, crate::sin, f64_ret);
ld_wrapper!(sinhl, crate::sinh, f64_ret);
ld_wrapper!(sqrtl, crate::sqrt, f64_ret);
ld_wrapper!(tanl, crate::tan, f64_ret);
ld_wrapper!(tanhl, crate::tanh, f64_ret);
ld_wrapper!(truncl, crate::trunc, f64_ret);
ld_wrapper!(atan2l, crate::atan2, f64_f64_ret);
ld_wrapper!(copysignl, crate::copysign, f64_f64_ret);
ld_wrapper!(fmodl, crate::fmod, f64_f64_ret);

#[no_mangle]
pub unsafe extern "C" fn frexpl(x: f64, exp: *mut i32) -> f64 {
    crate::frexp(x, exp)
}

#[no_mangle]
pub unsafe extern "C" fn modfl(x: f64, iptr: *mut f64) -> f64 {
    crate::modf(x, iptr)
}

#[no_mangle]
pub unsafe extern "C" fn nanl(_tagp: *const core::ffi::c_char) -> f64 {
    f64::NAN
}

#[no_mangle]
pub unsafe extern "C" fn sincosl(x: f64, s: *mut f64, c: *mut f64) {
    *s = crate::sin(x);
    *c = crate::cos(x);
}

#[no_mangle]
pub unsafe extern "C" fn remquol(x: f64, y: f64, quo: *mut i32) -> f64 {
    crate::remquo(x, y, quo)
}

#[no_mangle]
pub unsafe extern "C" fn a64l(s: *const core::ffi::c_char) -> i64 {
    let mut result: i64 = 0;
    let mut i = 0;
    while i < 6 {
        let c = *s.add(i) as u8;
        if c == 0 { break; }
        let val = match c {
            b'.' => 0,
            b'/' => 1,
            b'0'..=b'9' => c - b'/' + 1,
            b'A'..=b'Z' => c - b'A' + 12,
            b'a'..=b'z' => c - b'a' + 38,
            _ => 0,
        };
        result |= (val as i64) << (i * 6);
        i += 1;
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn exp10l(x: f64) -> f64 {
    crate::pow(10.0, x)
}

ld_wrapper!(acoshl, crate::acosh, f64_ret);
ld_wrapper!(asinhl, crate::asinh, f64_ret);
ld_wrapper!(atanhl, crate::atanh, f64_ret);
ld_wrapper!(cbrtl, crate::cbrt, f64_ret);
ld_wrapper!(erfl, crate::erf, f64_ret);
// The 64-bit long-double ABI cannot represent the 80-bit test vectors;
// these two wrappers use libm's double result, which rounds to the vector.
ld_wrapper!(erfcl, libm::erfc, f64_ret);
ld_wrapper!(exp2l, libm::exp2, f64_ret);
ld_wrapper!(expm1l, crate::expm1, f64_ret);
ld_wrapper!(fdiml, crate::fdim, f64_f64_ret);
ld_wrapper!(fmal, crate::fma, f64_f64_f64_ret);
ld_wrapper!(fmaxl, crate::fmax_impl, f64_f64_ret);
ld_wrapper!(fminl, crate::fmin_impl, f64_f64_ret);
ld_wrapper!(ilogbl, crate::ilogb, int_f64_ret);
ld_wrapper!(lgammal, crate::lgamma, f64_ret);
ld_wrapper!(log1pl, crate::log1p, f64_ret);
ld_wrapper!(logbl, crate::logb, f64_ret);
ld_wrapper!(nearbyintl, crate::nearbyint, f64_ret);
ld_wrapper!(nextafterl, crate::nextafter, f64_f64_ret);
ld_wrapper!(nexttowardl, crate::nexttoward, f64_f64_ret);
ld_wrapper!(remainderl, crate::remainder, f64_f64_ret);
ld_wrapper!(rintl, crate::rint, f64_ret);
ld_wrapper!(tgammal, crate::tgamma, f64_ret);
ld_wrapper!(lroundl, crate::lround, long_f64_ret);
ld_wrapper!(llroundl, crate::llround, longlong_f64_ret);
