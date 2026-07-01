#[inline]
fn p2d(e: i32) -> f64 {
    f64::from_bits(((0x3ff + e) as u64) << 52)
}

#[inline]
fn p2f(e: i32) -> f32 {
    f32::from_bits(((0x7f + e) as u32) << 23)
}

#[no_mangle]
pub extern "C" fn fabs(x: f64) -> f64 {
    f64::from_bits(x.to_bits() & ((-1i64 as u64) / 2))
}

#[no_mangle]
pub extern "C" fn fabsf(x: f32) -> f32 {
    f32::from_bits(x.to_bits() & 0x7fffffff)
}

#[no_mangle]
pub extern "C" fn copysign(x: f64, y: f64) -> f64 {
    let mut ux = x.to_bits();
    ux &= (-1i64 as u64) / 2;
    ux |= y.to_bits() & (1u64 << 63);
    f64::from_bits(ux)
}

#[no_mangle]
pub extern "C" fn copysignf(x: f32, y: f32) -> f32 {
    let mut ux = x.to_bits();
    ux &= 0x7fffffff;
    ux |= y.to_bits() & 0x80000000;
    f32::from_bits(ux)
}

#[no_mangle]
pub extern "C" fn trunc(x: f64) -> f64 {
    let ix = x.to_bits();
    let mut e = ((ix >> 52) & 0x7ff) as i32 - 0x3ff + 12;

    if e >= 52 + 12 {
        return x;
    }
    if e < 12 {
        e = 1;
    }
    let m = (-1i64 as u64) >> e;
    if (ix & m) == 0 {
        return x;
    }
    force_eval(x + p2d(120));
    f64::from_bits(ix & !m)
}

#[no_mangle]
pub extern "C" fn truncf(x: f32) -> f32 {
    let ix = x.to_bits();
    let mut e = ((ix >> 23) & 0xff) as i32 - 0x7f + 9;

    if e >= 23 + 9 {
        return x;
    }
    if e < 9 {
        e = 1;
    }
    let m = (-1i32 as u32) >> e;
    if (ix & m) == 0 {
        return x;
    }
    force_eval(x + p2f(120));
    f32::from_bits(ix & !m)
}

#[no_mangle]
pub extern "C" fn floor(x: f64) -> f64 {
    let ix = x.to_bits();
    let e = ((ix >> 52) & 0x7ff) as i32;

    if e >= 0x3ff + 52 || x == 0.0 {
        return x;
    }
    let y = if ix >> 63 != 0 {
        x - p2d(52) + p2d(52) - x
    } else {
        x + p2d(52) - p2d(52) - x
    };
    if e <= 0x3ff - 1 {
        force_eval(y);
        return if ix >> 63 != 0 { -1.0 } else { 0.0 };
    }
    if y > 0.0 {
        x + y - 1.0
    } else {
        x + y
    }
}

#[no_mangle]
pub extern "C" fn floorf(x: f32) -> f32 {
    let mut ix = x.to_bits();
    let e = ((ix >> 23) & 0xff) as i32 - 0x7f;

    if e >= 23 {
        return x;
    }
    if e >= 0 {
        let m = 0x007fffff >> e;
        if (ix & m) == 0 {
            return x;
        }
        force_eval(x + p2f(120));
        if ix >> 31 != 0 {
            ix += m;
        }
        ix &= !m;
    } else {
        force_eval(x + p2f(120));
        if ix >> 31 == 0 {
            ix = 0;
        } else if ix << 1 != 0 {
            return -1.0f32;
        }
    }
    f32::from_bits(ix)
}

#[no_mangle]
pub extern "C" fn ceil(x: f64) -> f64 {
    let ix = x.to_bits();
    let e = ((ix >> 52) & 0x7ff) as i32;

    if e >= 0x3ff + 52 || x == 0.0 {
        return x;
    }
    let y = if ix >> 63 != 0 {
        x - p2d(52) + p2d(52) - x
    } else {
        x + p2d(52) - p2d(52) - x
    };
    if e <= 0x3ff - 1 {
        force_eval(y);
        return if ix >> 63 != 0 { -0.0 } else { 1.0 };
    }
    if y < 0.0 {
        x + y + 1.0
    } else {
        x + y
    }
}

#[no_mangle]
pub extern "C" fn ceilf(x: f32) -> f32 {
    let mut ix = x.to_bits();
    let e = ((ix >> 23) & 0xff) as i32 - 0x7f;

    if e >= 23 {
        return x;
    }
    if e >= 0 {
        let m = 0x007fffff >> e;
        if (ix & m) == 0 {
            return x;
        }
        force_eval(x + p2f(120));
        if ix >> 31 == 0 {
            ix += m;
        }
        ix &= !m;
    } else {
        force_eval(x + p2f(120));
        if ix >> 31 != 0 {
            return -0.0f32;
        } else if ix << 1 != 0 {
            return 1.0f32;
        }
    }
    f32::from_bits(ix)
}

#[no_mangle]
pub extern "C" fn round(x: f64) -> f64 {
    let ix = x.to_bits();
    let e = ((ix >> 52) & 0x7ff) as i32;
    let mut ax = x;

    if e >= 0x3ff + 52 {
        return x;
    }
    let neg = ix >> 63 != 0;
    if neg {
        ax = -ax;
    }
    if e < 0x3ff - 1 {
        force_eval(ax + p2d(52));
        return 0.0 * x;
    }
    let y = ax + p2d(52) - p2d(52) - ax;
    let y = if y > 0.5 {
        y + ax - 1.0
    } else if y <= -0.5 {
        y + ax + 1.0
    } else {
        y + ax
    };
    if neg { -y } else { y }
}

#[no_mangle]
pub extern "C" fn roundf(x: f32) -> f32 {
    let ix = x.to_bits();
    let e = ((ix >> 23) & 0xff) as i32;
    let mut ax = x;

    if e >= 0x7f + 23 {
        return x;
    }
    let neg = ix >> 31 != 0;
    if neg {
        ax = -ax;
    }
    if e < 0x7f - 1 {
        force_eval(ax + p2f(23));
        return 0.0f32 * x;
    }
    let y = ax + p2f(23) - p2f(23) - ax;
    let y = if y > 0.5f32 {
        y + ax - 1.0f32
    } else if y <= -0.5f32 {
        y + ax + 1.0f32
    } else {
        y + ax
    };
    if neg { -y } else { y }
}

#[no_mangle]
pub extern "C" fn scalbn(x: f64, n: c_int) -> f64 {
    let mut y = x;
    let mut n = n;

    if n > 1023 {
        y *= p2d(1023);
        n -= 1023;
        if n > 1023 {
            y *= p2d(1023);
            n -= 1023;
            if n > 1023 {
                n = 1023;
            }
        }
    } else if n < -1022 {
        y *= p2d(-1022) * p2d(53);
        n += 1022 - 53;
        if n < -1022 {
            y *= p2d(-1022) * p2d(53);
            n += 1022 - 53;
            if n < -1022 {
                n = -1022;
            }
        }
    }
    f64::from_bits(((0x3ff + n) as u64) << 52) * y
}

#[no_mangle]
pub extern "C" fn scalbnf(x: f32, n: c_int) -> f32 {
    let mut y = x;
    let mut n = n;

    if n > 127 {
        y *= p2f(127);
        n -= 127;
        if n > 127 {
            y *= p2f(127);
            n -= 127;
            if n > 127 {
                n = 127;
            }
        }
    } else if n < -126 {
        y *= p2f(-126) * p2f(24);
        n += 126 - 24;
        if n < -126 {
            y *= p2f(-126) * p2f(24);
            n += 126 - 24;
            if n < -126 {
                n = -126;
            }
        }
    }
    f32::from_bits(((0x7f + n) as u32) << 23) * y
}

#[no_mangle]
pub extern "C" fn scalbln(x: f64, n: c_long) -> f64 {
    let n = if n > c_int::MAX as c_long {
        c_int::MAX
    } else if n < c_int::MIN as c_long {
        c_int::MIN
    } else {
        n as c_int
    };
    scalbn(x, n)
}

#[no_mangle]
pub extern "C" fn scalblnf(x: f32, n: c_long) -> f32 {
    let n = if n > c_int::MAX as c_long {
        c_int::MAX
    } else if n < c_int::MIN as c_long {
        c_int::MIN
    } else {
        n as c_int
    };
    scalbnf(x, n)
}

#[no_mangle]
pub extern "C" fn ldexp(x: f64, n: c_int) -> f64 {
    scalbn(x, n)
}

#[no_mangle]
pub extern "C" fn ldexpf(x: f32, n: c_int) -> f32 {
    scalbnf(x, n)
}

#[no_mangle]
pub unsafe extern "C" fn frexp(x: f64, e: *mut c_int) -> f64 {
    let mut ix = x.to_bits();
    let ee = ((ix >> 52) & 0x7ff) as i32;

    if ee == 0 {
        if x != 0.0 {
            let r = frexp(x * p2d(64), e);
            if !e.is_null() {
                *e -= 64;
            }
            return r;
        }
        if !e.is_null() {
            *e = 0;
        }
        return x;
    }
    if ee == 0x7ff {
        if !e.is_null() {
            *e = 0;
        }
        return x;
    }
    if !e.is_null() {
        *e = ee - 0x3fe;
    }
    ix &= 0x800fffffffffffffu64;
    ix |= 0x3fe0000000000000u64;
    f64::from_bits(ix)
}

#[no_mangle]
pub unsafe extern "C" fn frexpf(x: f32, e: *mut c_int) -> f32 {
    let mut ix = x.to_bits();
    let ee = ((ix >> 23) & 0xff) as i32;

    if ee == 0 {
        if x != 0.0f32 {
            let r = frexpf(x * p2f(64), e);
            if !e.is_null() {
                *e -= 64;
            }
            return r;
        }
        if !e.is_null() {
            *e = 0;
        }
        return x;
    }
    if ee == 0xff {
        if !e.is_null() {
            *e = 0;
        }
        return x;
    }
    if !e.is_null() {
        *e = ee - 0x7e;
    }
    ix &= 0x807fffffu32;
    ix |= 0x3f000000u32;
    f32::from_bits(ix)
}

#[no_mangle]
pub unsafe extern "C" fn modf(x: f64, iptr: *mut f64) -> f64 {
    let mut ix = x.to_bits();
    let e = ((ix >> 52) & 0x7ff) as i32 - 0x3ff;

    if e >= 52 {
        if !iptr.is_null() {
            *iptr = x;
        }
        if e == 0x400 && (ix << 12) != 0 {
            return x;
        }
        return f64::from_bits(ix & (1u64 << 63));
    }
    if e < 0 {
        if !iptr.is_null() {
            *iptr = f64::from_bits(ix & (1u64 << 63));
        }
        return x;
    }
    let mask = (-1i64 as u64) >> 12 >> e;
    if (ix & mask) == 0 {
        if !iptr.is_null() {
            *iptr = x;
        }
        return f64::from_bits(ix & (1u64 << 63));
    }
    ix &= !mask;
    let i = f64::from_bits(ix);
    if !iptr.is_null() {
        *iptr = i;
    }
    x - i
}

#[no_mangle]
pub unsafe extern "C" fn modff(x: f32, iptr: *mut f32) -> f32 {
    let mut ix = x.to_bits();
    let e = ((ix >> 23) & 0xff) as i32 - 0x7f;

    if e >= 23 {
        if !iptr.is_null() {
            *iptr = x;
        }
        if e == 0x80 && (ix << 9) != 0 {
            return x;
        }
        return f32::from_bits(ix & 0x80000000u32);
    }
    if e < 0 {
        if !iptr.is_null() {
            *iptr = f32::from_bits(ix & 0x80000000u32);
        }
        return x;
    }
    let mask = 0x007fffffu32 >> e;
    if (ix & mask) == 0 {
        if !iptr.is_null() {
            *iptr = x;
        }
        return f32::from_bits(ix & 0x80000000u32);
    }
    ix &= !mask;
    let i = f32::from_bits(ix);
    if !iptr.is_null() {
        *iptr = i;
    }
    x - i
}
