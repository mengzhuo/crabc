// Translated from musl: sin, cos, tan, sinf, cosf, tanf,
// __sin, __cos, __tan, __rem_pio2, __rem_pio2f, __rem_pio2_large,
// __sindf, __cosdf, __tandf

const INIT_JK: [i32; 4] = [3, 4, 4, 6];

const IPIO2: [i32; 66] = [
    0xA2F983, 0x6E4E44, 0x1529FC, 0x2757D1, 0xF534DD, 0xC0DB62,
    0x95993C, 0x439041, 0xFE5163, 0xABDEBB, 0xC561B7, 0x246E3A,
    0x424DD2, 0xE00649, 0x2EEA09, 0xD1921C, 0xFE1DEB, 0x1CB129,
    0xA73EE8, 0x8235F5, 0x2EBB44, 0x84E99C, 0x7026B4, 0x5F7E41,
    0x3991D6, 0x398353, 0x39F49C, 0x845F8B, 0xBDF928, 0x3B1FF8,
    0x97FFDE, 0x05980F, 0xEF2F11, 0x8B5A0A, 0x6D1F6D, 0x367ECF,
    0x27CB09, 0xB74F46, 0x3F669E, 0x5FEA2D, 0x7527BA, 0xC7EBE5,
    0xF17B3D, 0x0739F7, 0x8A5292, 0xEA6BFB, 0x5FB11F, 0x8D5D08,
    0x560330, 0x46FC7B, 0x6BABF0, 0xCFBC20, 0x9AF436, 0x1DA9E3,
    0x91615E, 0xE61B08, 0x659985, 0x5F14A0, 0x68408D, 0xFFD880,
    0x4D7327, 0x310606, 0x1556CA, 0x73A8C9, 0x60E27B, 0xC08C6B,
];

const PIO2: [f64; 8] = [
    asdouble(0x3FF921FB40000000),
    asdouble(0x3E74442D00000000),
    asdouble(0x3CF8469880000000),
    asdouble(0x3B78CC5160000000),
    asdouble(0x39F01B8380000000),
    asdouble(0x387A252040000000),
    asdouble(0x36E3822280000000),
    asdouble(0x3569F31D00000000),
];

const S1PIO2: f64 = asdouble(0x3FF921FB54442D18);
const S2PIO2: f64 = asdouble(0x400921FB54442D18);
const S3PIO2: f64 = asdouble(0x4012D97C7F3321D2);
const S4PIO2: f64 = asdouble(0x401921FB54442D18);

#[allow(non_snake_case)]
fn __rem_pio2_large(x: &[f64], y: &mut [f64], e0: i32, nx: i32, prec: i32) -> i32 {
    let mut iq = [0i32; 20];
    let mut f = [0.0f64; 30];
    let mut fq = [0.0f64; 20];
    let mut q = [0.0f64; 20];

    let jk = INIT_JK[prec as usize];
    let jp = jk;
    let jx = nx - 1;
    let mut jv = (e0 - 3) / 24;
    if jv < 0 { jv = 0; }
    let mut q0 = e0 - 24 * (jv + 1);

    let mut j = jv - jx;
    let m = jx + jk;
    for ii in 0..=m {
        f[ii as usize] = if j < 0 { 0.0 } else { IPIO2[j as usize] as f64 };
        j += 1;
    }

    for ii in 0..=jk {
        let mut fw = 0.0f64;
        for jj in 0..=jx {
            fw += x[jj as usize] * f[(jx + ii - jj) as usize];
        }
        q[ii as usize] = fw;
    }

    let mut jz = jk;
    let mut need_recompute = true;
    let mut n_val = 0i32;
    let mut ih_val = 0i32;
    let mut z = 0.0f64;

    while need_recompute {
        need_recompute = false;
        // distill q[] into iq[]
        z = q[jz as usize];
        j = jz;
        let mut ii = 0i32;
        while j > 0 {
            let fw = (z * p2d(-24)) as i32;
            iq[ii as usize] = (z - fw as f64 * p2d(24)) as i32;
            z = q[(j - 1) as usize] + fw as f64;
            ii += 1;
            j -= 1;
        }

        z = scalbn(z, q0);
        z -= 8.0 * floor(z * 0.125);
        n_val = z as i32;
        z -= n_val as f64;
        ih_val = 0;
        if q0 > 0 {
            let i_part = iq[(jz - 1) as usize] >> (24 - q0);
            n_val += i_part;
            iq[(jz - 1) as usize] -= i_part << (24 - q0);
            ih_val = iq[(jz - 1) as usize] >> (23 - q0);
        } else if q0 == 0 {
            ih_val = iq[(jz - 1) as usize] >> 23;
        } else if z >= 0.5 {
            ih_val = 2;
        }

        if ih_val > 0 {
            n_val += 1;
            let mut carry = 0i32;
            for idx in 0..(jz as usize) {
                let jv = iq[idx];
                if carry == 0 {
                    if jv != 0 {
                        carry = 1;
                        iq[idx] = 0x1000000 - jv;
                    }
                } else {
                    iq[idx] = 0xffffff - jv;
                }
            }
            if q0 > 0 {
                match q0 {
                    1 => { iq[(jz - 1) as usize] &= 0x7fffff; }
                    2 => { iq[(jz - 1) as usize] &= 0x3fffff; }
                    _ => {}
                }
            }
            if ih_val == 2 {
                z = 1.0 - z;
                if carry != 0 {
                    z -= scalbn(1.0, q0);
                }
            }
        }

        // check recomputation
        if z == 0.0 {
            j = 0;
            let mut idx = jz - 1;
            while idx >= jk {
                j |= iq[idx as usize];
                idx -= 1;
            }
            if j == 0 {
                let mut k = 1i32;
                while iq[(jk - k) as usize] == 0 { k += 1; }
                for idx2 in (jz + 1)..=(jz + k) {
                    f[(jx + idx2) as usize] = IPIO2[(jv + idx2) as usize] as f64;
                    let mut fw = 0.0f64;
                    for jj in 0..=jx {
                        fw += x[jj as usize] * f[(jx + idx2 - jj) as usize];
                    }
                    q[idx2 as usize] = fw;
                }
                jz += k;
                need_recompute = true;
            }
        }
    }

    // chop off zero terms
    if z == 0.0 {
        jz -= 1;
        q0 -= 24;
        while iq[jz as usize] == 0 {
            jz -= 1;
            q0 -= 24;
        }
    } else {
        z = scalbn(z, -q0);
        if z >= p2d(24) {
            let fw = (z * p2d(-24)) as i32;
            iq[jz as usize] = (z - fw as f64 * p2d(24)) as i32;
            jz += 1;
            q0 += 24;
            iq[jz as usize] = fw;
        } else {
            iq[jz as usize] = z as i32;
        }
    }

    // convert integer chunks to float
    let mut fw = scalbn(1.0, q0);
    for idx in (0..=(jz as usize)).rev() {
        q[idx] = fw * iq[idx] as f64;
        fw *= p2d(-24);
    }

    // compute PIo2 * q
    for idx in (0..=(jz as usize)).rev() {
        fw = 0.0;
        for k in 0..=(jp as usize) {
            if k <= (jz as usize - idx) {
                fw += PIO2[k] * q[idx + k];
            }
        }
        fq[(jz as usize) - idx] = fw;
    }

    // compress fq[] into y[]
    match prec {
        0 => {
            fw = 0.0;
            for idx in (0..=(jz as usize)).rev() {
                fw += fq[idx];
            }
            y[0] = if ih_val == 0 { fw } else { -fw };
        }
        1 | 2 => {
            fw = 0.0;
            for idx in (0..=(jz as usize)).rev() {
                fw += fq[idx];
            }
            fw = fw; // double_t = f64 when FLT_EVAL_METHOD==0
            y[0] = if ih_val == 0 { fw } else { -fw };
            fw = fq[0] - fw;
            for idx in 1..=(jz as usize) {
                fw += fq[idx];
            }
            y[1] = if ih_val == 0 { fw } else { -fw };
        }
        _ => {
            for idx in (1..=(jz as usize)).rev() {
                let tmp = fq[idx - 1] + fq[idx];
                fq[idx] += fq[idx - 1] - tmp;
                fq[idx - 1] = tmp;
            }
            for idx in (2..=(jz as usize)).rev() {
                let tmp = fq[idx - 1] + fq[idx];
                fq[idx] += fq[idx - 1] - tmp;
                fq[idx - 1] = tmp;
            }
            fw = 0.0;
            for idx in (2..=(jz as usize)).rev() {
                fw += fq[idx];
            }
            if ih_val == 0 {
                y[0] = fq[0]; y[1] = fq[1]; y[2] = fw;
            } else {
                y[0] = -fq[0]; y[1] = -fq[1]; y[2] = -fw;
            }
        }
    }
    n_val & 7
}

const REM_PIO2_TOINT: f64 = asdouble(0x4338000000000000);
const REM_PIO2_PIO4: f64 = asdouble(0x3FE921FB54442D18);
const REM_PIO2_INVPIO2: f64 = asdouble(0x3FE45F306DC9C883);
const REM_PIO2_PIO2_1: f64 = asdouble(0x3FF921FB54400000);
const REM_PIO2_PIO2_1T: f64 = asdouble(0x3DD0B4611A626331);
const REM_PIO2_PIO2_2: f64 = asdouble(0x3DD0B4611A600000);
const REM_PIO2_PIO2_2T: f64 = asdouble(0x3BA3198A2E037073);
const REM_PIO2_PIO2_3: f64 = asdouble(0x3BA3198A2E000000);
const REM_PIO2_PIO2_3T: f64 = asdouble(0x397B839A252049C1);

fn __rem_pio2(x: f64, y: &mut [f64; 2]) -> i32 {
    let ix64 = asuint64(x);
    let sign = (ix64 >> 63) as i32;
    let ix = ((ix64 >> 32) & 0x7fffffff) as u32;

    if ix <= 0x400f6a7a {
        if (ix & 0xfffff) == 0x921fb {
            // cancellation -- use medium case
            return rem_pio2_medium(x, sign, ix, y);
        }
        if ix <= 0x4002d97c {
            if sign == 0 {
                let z = x - REM_PIO2_PIO2_1;
                y[0] = z - REM_PIO2_PIO2_1T;
                y[1] = (z - y[0]) - REM_PIO2_PIO2_1T;
                return 1;
            } else {
                let z = x + REM_PIO2_PIO2_1;
                y[0] = z + REM_PIO2_PIO2_1T;
                y[1] = (z - y[0]) + REM_PIO2_PIO2_1T;
                return -1;
            }
        } else {
            if sign == 0 {
                let z = x - 2.0 * REM_PIO2_PIO2_1;
                y[0] = z - 2.0 * REM_PIO2_PIO2_1T;
                y[1] = (z - y[0]) - 2.0 * REM_PIO2_PIO2_1T;
                return 2;
            } else {
                let z = x + 2.0 * REM_PIO2_PIO2_1;
                y[0] = z + 2.0 * REM_PIO2_PIO2_1T;
                y[1] = (z - y[0]) + 2.0 * REM_PIO2_PIO2_1T;
                return -2;
            }
        }
    }
    if ix <= 0x401c463b {
        if ix <= 0x4015fdbc {
            if ix == 0x4012d97c {
                return rem_pio2_medium(x, sign, ix, y);
            }
            if sign == 0 {
                let z = x - 3.0 * REM_PIO2_PIO2_1;
                y[0] = z - 3.0 * REM_PIO2_PIO2_1T;
                y[1] = (z - y[0]) - 3.0 * REM_PIO2_PIO2_1T;
                return 3;
            } else {
                let z = x + 3.0 * REM_PIO2_PIO2_1;
                y[0] = z + 3.0 * REM_PIO2_PIO2_1T;
                y[1] = (z - y[0]) + 3.0 * REM_PIO2_PIO2_1T;
                return -3;
            }
        } else {
            if ix == 0x401921fb {
                return rem_pio2_medium(x, sign, ix, y);
            }
            if sign == 0 {
                let z = x - 4.0 * REM_PIO2_PIO2_1;
                y[0] = z - 4.0 * REM_PIO2_PIO2_1T;
                y[1] = (z - y[0]) - 4.0 * REM_PIO2_PIO2_1T;
                return 4;
            } else {
                let z = x + 4.0 * REM_PIO2_PIO2_1;
                y[0] = z + 4.0 * REM_PIO2_PIO2_1T;
                y[1] = (z - y[0]) + 4.0 * REM_PIO2_PIO2_1T;
                return -4;
            }
        }
    }
    if ix < 0x413921fb {
        return rem_pio2_medium(x, sign, ix, y);
    }
    // all other (large) arguments
    if ix >= 0x7ff00000 {
        y[0] = x - x;
        y[1] = y[0];
        return 0;
    }
    // set z = scalbn(|x|,-ilogb(x)+23)
    let mut ux = ix64;
    ux &= (-1i64 as u64) >> 12;
    ux |= ((0x3ff + 23) as u64) << 52;
    let mut z = asdouble(ux);
    let mut tx = [0.0f64; 3];
    for i in 0..2 {
        tx[i] = z as i32 as f64;
        z = (z - tx[i]) * p2d(24);
    }
    tx[2] = z;
    let mut i = 2i32;
    while tx[i as usize] == 0.0 {
        i -= 1;
    }
    let n = __rem_pio2_large(&tx, y, (ix >> 20) as i32 - (0x3ff + 23), i + 1, 1);
    if sign != 0 {
        y[0] = -y[0];
        y[1] = -y[1];
        return -(n as i32);
    }
    n as i32
}

fn rem_pio2_medium(x: f64, _sign: i32, ix: u32, y: &mut [f64; 2]) -> i32 {
    let mut fn_val = x as f64 * REM_PIO2_INVPIO2 + REM_PIO2_TOINT - REM_PIO2_TOINT;
    let mut n = fn_val as i32;
    let mut r = x - fn_val * REM_PIO2_PIO2_1;
    let mut w = fn_val * REM_PIO2_PIO2_1T;
    if predict_false(r - w < -REM_PIO2_PIO4) {
        n -= 1;
        fn_val -= 1.0;
        r = x - fn_val * REM_PIO2_PIO2_1;
        w = fn_val * REM_PIO2_PIO2_1T;
    } else if predict_false(r - w > REM_PIO2_PIO4) {
        n += 1;
        fn_val += 1.0;
        r = x - fn_val * REM_PIO2_PIO2_1;
        w = fn_val * REM_PIO2_PIO2_1T;
    }
    y[0] = r - w;
    let ey = ((asuint64(y[0]) >> 52) & 0x7ff) as i32;
    let ex = (ix >> 20) as i32;
    if ex - ey > 16 {
        let t = r;
        w = fn_val * REM_PIO2_PIO2_2;
        r = t - w;
        w = fn_val * REM_PIO2_PIO2_2T - ((t - r) - w);
        y[0] = r - w;
        let ey = ((asuint64(y[0]) >> 52) & 0x7ff) as i32;
        if ex - ey > 49 {
            let t = r;
            w = fn_val * REM_PIO2_PIO2_3;
            r = t - w;
            w = fn_val * REM_PIO2_PIO2_3T - ((t - r) - w);
            y[0] = r - w;
        }
    }
    y[1] = (r - y[0]) - w;
    n
}

const REMPIO2F_TOINT: f64 = asdouble(0x4338000000000000);
const REMPIO2F_PIO4: f64 = asdouble(0x3FE921FB60000000);
const REMPIO2F_INVPIO2: f64 = asdouble(0x3FE45F306DC9C883);
const REMPIO2F_PIO2_1: f64 = asdouble(0x3FF921FB50000000);
const REMPIO2F_PIO2_1T: f64 = asdouble(0x3E5110B4611A6263);

fn __rem_pio2f(x: f32, y: &mut f64) -> i32 {
    let ix = asuint(x) & 0x7fffffff;
    if ix < 0x4dc90fdb {
        let mut fn_val = x as f64 * REMPIO2F_INVPIO2 + REMPIO2F_TOINT - REMPIO2F_TOINT;
        let mut n = fn_val as i32;
        *y = x as f64 - fn_val * REMPIO2F_PIO2_1 - fn_val * REMPIO2F_PIO2_1T;
        if predict_false(*y < -REMPIO2F_PIO4) {
            n -= 1;
            fn_val -= 1.0;
            *y = x as f64 - fn_val * REMPIO2F_PIO2_1 - fn_val * REMPIO2F_PIO2_1T;
        } else if predict_false(*y > REMPIO2F_PIO4) {
            n += 1;
            fn_val += 1.0;
            *y = x as f64 - fn_val * REMPIO2F_PIO2_1 - fn_val * REMPIO2F_PIO2_1T;
        }
        return n;
    }
    if ix >= 0x7f800000 {
        *y = x as f64 - x as f64;
        return 0;
    }
    let sign = (asuint(x) >> 31) as i32;
    let e0 = ((ix >> 23) as i32) - (0x7f + 23);
    let ux = ix - ((e0 as u32) << 23);
    let tx = [asfloat(ux) as f64];
    let mut ty = [0.0f64];
    let n = __rem_pio2_large(&tx, &mut ty, e0, 1, 0);
    if sign != 0 {
        *y = -ty[0];
        return -(n as i32);
    }
    *y = ty[0];
    n as i32
}

// __sin kernel
const SIN_S1: f64 = asdouble(0xBFC5555555555549);
const SIN_S2: f64 = asdouble(0x3F8111111110F8A6);
const SIN_S3: f64 = asdouble(0xBF2A01A019C161D5);
const SIN_S4: f64 = asdouble(0x3EC71DE357B1FE7D);
const SIN_S5: f64 = asdouble(0xBE5AE5E68A2B9CEB);
const SIN_S6: f64 = asdouble(0x3DE5D93A5ACFD57C);

fn __sin(x: f64, y: f64, iy: i32) -> f64 {
    let z = x * x;
    let w = z * z;
    let r = SIN_S2 + z * (SIN_S3 + z * SIN_S4) + z * w * (SIN_S5 + z * SIN_S6);
    let v = z * x;
    if iy == 0 {
        x + v * (SIN_S1 + z * r)
    } else {
        x - ((z * (0.5 * y - v * r) - y) - v * SIN_S1)
    }
}

// __cos kernel
const COS_C1: f64 = asdouble(0x3FA555555555554C);
const COS_C2: f64 = asdouble(0xBF56C16C16C15177);
const COS_C3: f64 = asdouble(0x3EFA01A019CB1590);
const COS_C4: f64 = asdouble(0xBE927E4F809C52AD);
const COS_C5: f64 = asdouble(0x3E21EE9EBDB4B1C4);
const COS_C6: f64 = asdouble(0xBDA8FAE9BE8838D4);

fn __cos(x: f64, y: f64) -> f64 {
    let z = x * x;
    let w = z * z;
    let r = z * (COS_C1 + z * (COS_C2 + z * COS_C3)) + w * w * (COS_C4 + z * (COS_C5 + z * COS_C6));
    let hz = 0.5 * z;
    let w = 1.0 - hz;
    w + (((1.0 - w) - hz) + (z * r - x * y))
}

// __tan kernel
const TAN_T: [f64; 13] = [
    asdouble(0x3FD5555555555563),
    asdouble(0x3FC111111110FE7A),
    asdouble(0x3FABA1BA1BB341FE),
    asdouble(0x3F9664F48406D637),
    asdouble(0x3F8226E3E96E8493),
    asdouble(0x3F6D6D22C9560328),
    asdouble(0x3F57DBC8FEE08315),
    asdouble(0x3F4344D8F2F26501),
    asdouble(0x3F3026F71A8D1068),
    asdouble(0x3F147E88A03792A6),
    asdouble(0x3F12B80F32F0A7E9),
    asdouble(0xBEF375CBDB605373),
    asdouble(0x3EFB2A7074BF7AD4),
];
const TAN_PIO4: f64 = asdouble(0x3FE921FB54442D18);
const TAN_PIO4LO: f64 = asdouble(0x3C81A62633145C07);

fn __tan(x: f64, y: f64, odd: i32) -> f64 {
    let hx = (asuint64(x) >> 32) as u32;
    let big = (hx & 0x7fffffff) >= 0x3FE59428;
    let (x, y, sign) = if big {
        let sign = hx >> 31;
        let (nx, ny) = if sign != 0 { (-x, -y) } else { (x, y) };
        ((TAN_PIO4 - nx) + (TAN_PIO4LO - ny), 0.0, sign)
    } else {
        (x, y, 0u32)
    };
    let z = x * x;
    let w = z * z;
    let r = TAN_T[1] + w * (TAN_T[3] + w * (TAN_T[5] + w * (TAN_T[7] + w * (TAN_T[9] + w * TAN_T[11]))));
    let v = z * (TAN_T[2] + w * (TAN_T[4] + w * (TAN_T[6] + w * (TAN_T[8] + w * (TAN_T[10] + w * TAN_T[12])))));
    let s = z * x;
    let r = y + z * (s * (r + v) + y) + s * TAN_T[0];
    let w = x + r;
    if big {
        let s_val = 1.0 - 2.0 * odd as f64;
        let v = s_val - 2.0 * (x + (r - w * w / (w + s_val)));
        return if sign != 0 { -v } else { v };
    }
    if odd == 0 {
        return w;
    }
    // -1.0/(x+r) has up to 2ulp error, so compute it accurately
    let w0 = {
        let mut tmp = w;
        unsafe { set_low_word(&mut tmp, 0); }
        tmp
    };
    let v = r - (w0 - x);
    let mut a0 = -1.0 / w;
    let a = a0;
    unsafe { set_low_word(&mut a0, 0); }
    a0 + a * (1.0 + a0 * w0 + a0 * v)
}

// __sindf kernel
const SINDF_S1: f64 = asdouble(0xBFC5555554CBAC77);
const SINDF_S2: f64 = asdouble(0x3F811110896EFBB2);
const SINDF_S3: f64 = asdouble(0xBF2A00F9E2CAE774);
const SINDF_S4: f64 = asdouble(0x3EC6CD878C3B46A7);

fn __sindf(x: f64) -> f32 {
    let z = x * x;
    let w = z * z;
    let r = SINDF_S3 + z * SINDF_S4;
    let s = z * x;
    ((x + s * (SINDF_S1 + z * SINDF_S2)) + s * w * r) as f32
}

// __cosdf kernel
const COSDF_C0: f64 = asdouble(0xBFDFFFFFFD0C5E81);
const COSDF_C1: f64 = asdouble(0x3FA55553E1053A42);
const COSDF_C2: f64 = asdouble(0xBF56C087E80F1E27);
const COSDF_C3: f64 = asdouble(0x3EF99342E0EE5069);

fn __cosdf(x: f64) -> f32 {
    let z = x * x;
    let w = z * z;
    let r = COSDF_C2 + z * COSDF_C3;
    ((1.0 + z * COSDF_C0) + w * COSDF_C1 + (w * z) * r) as f32
}

// __tandf kernel
const TANDF_T: [f64; 6] = [
    asdouble(0x3FD5554D3418C99F),
    asdouble(0x3FC112FD38999F72),
    asdouble(0x3FAB54C91D865AFE),
    asdouble(0x3F991DF3908C33CE),
    asdouble(0x3F685DADFCECF44E),
    asdouble(0x3F8362B9BF971BCD),
];

fn __tandf(x: f64, odd: i32) -> f32 {
    let z = x * x;
    let r = TANDF_T[4] + z * TANDF_T[5];
    let t = TANDF_T[2] + z * TANDF_T[3];
    let w = z * z;
    let s = z * x;
    let u = TANDF_T[0] + z * TANDF_T[1];
    let r = (x + s * u) + (s * w) * (t + w * r);
    if odd != 0 { (-1.0 / r) as f32 } else { r as f32 }
}

// ============================================================
// sin, cos, tan (double precision)
// ============================================================

#[no_mangle]
pub extern "C" fn sin(x: f64) -> f64 {
    let mut ix = ((asuint64(x) >> 32) as u32);
    ix &= 0x7fffffff;

    if ix <= 0x3fe921fb {
        if ix < 0x3e500000 {
            force_eval(if ix < 0x00100000 { x / p2f(120) as f64 } else { x + p2f(120) as f64 });
            return x;
        }
        return __sin(x, 0.0, 0);
    }
    if ix >= 0x7ff00000 {
        return x - x;
    }
    let mut y = [0.0f64; 2];
    let n = __rem_pio2(x, &mut y);
    match n & 3 {
        0 =>  __sin(y[0], y[1], 1),
        1 =>  __cos(y[0], y[1]),
        2 => -__sin(y[0], y[1], 1),
        _ => -__cos(y[0], y[1]),
    }
}

#[no_mangle]
pub extern "C" fn cos(x: f64) -> f64 {
    let mut ix = ((asuint64(x) >> 32) as u32);
    ix &= 0x7fffffff;

    if ix <= 0x3fe921fb {
        if ix < 0x3e46a09e {
            force_eval(x + p2f(120) as f64);
            return 1.0;
        }
        return __cos(x, 0.0);
    }
    if ix >= 0x7ff00000 {
        return x - x;
    }
    let mut y = [0.0f64; 2];
    let n = __rem_pio2(x, &mut y);
    match n & 3 {
        0 =>  __cos(y[0], y[1]),
        1 => -__sin(y[0], y[1], 1),
        2 => -__cos(y[0], y[1]),
        _ =>  __sin(y[0], y[1], 1),
    }
}

#[no_mangle]
pub extern "C" fn tan(x: f64) -> f64 {
    let mut ix = ((asuint64(x) >> 32) as u32);
    ix &= 0x7fffffff;

    if ix <= 0x3fe921fb {
        if ix < 0x3e400000 {
            force_eval(if ix < 0x00100000 { x / p2f(120) as f64 } else { x + p2f(120) as f64 });
            return x;
        }
        return __tan(x, 0.0, 0);
    }
    if ix >= 0x7ff00000 {
        return x - x;
    }
    let mut y = [0.0f64; 2];
    let n = __rem_pio2(x, &mut y);
    __tan(y[0], y[1], n & 1)
}

// ============================================================
// sinf, cosf, tanf (single precision)
// ============================================================

#[no_mangle]
pub extern "C" fn sinf(x: f32) -> f32 {
    let ix = asuint(x);
    let sign = ix >> 31;
    let ix = ix & 0x7fffffff;

    if ix <= 0x3f490fda {
        if ix < 0x39800000 {
            force_eval(if ix < 0x00800000 { x / p2f(120) } else { x + p2f(120) });
            return x;
        }
        return __sindf(x as f64);
    }
    if ix <= 0x407b53d1 {
        if ix <= 0x4016cbe3 {
            return if sign != 0 { -__cosdf(x as f64 + S1PIO2) } else { __cosdf(x as f64 - S1PIO2) };
        }
        return __sindf(if sign != 0 { -(x as f64 + S2PIO2) } else { -(x as f64 - S2PIO2) });
    }
    if ix <= 0x40e231d5 {
        if ix <= 0x40afeddf {
            return if sign != 0 { __cosdf(x as f64 + S3PIO2) } else { -__cosdf(x as f64 - S3PIO2) };
        }
        return __sindf(if sign != 0 { x as f64 + S4PIO2 } else { x as f64 - S4PIO2 });
    }
    if ix >= 0x7f800000 {
        return x - x;
    }
    let mut y = 0.0f64;
    let n = __rem_pio2f(x, &mut y);
    match n & 3 {
        0 =>  __sindf(y),
        1 =>  __cosdf(y),
        2 =>  __sindf(-y),
        _ => -__cosdf(y),
    }
}

#[no_mangle]
pub extern "C" fn cosf(x: f32) -> f32 {
    let ix = asuint(x);
    let sign = ix >> 31;
    let ix = ix & 0x7fffffff;

    if ix <= 0x3f490fda {
        if ix < 0x39800000 {
            force_eval(x + p2f(120));
            return 1.0f32;
        }
        return __cosdf(x as f64);
    }
    if ix <= 0x407b53d1 {
        if ix > 0x4016cbe3 {
            return -__cosdf(if sign != 0 { x as f64 + S2PIO2 } else { x as f64 - S2PIO2 });
        } else {
            return if sign != 0 { __sindf(x as f64 + S1PIO2) } else { __sindf(S1PIO2 - x as f64) };
        }
    }
    if ix <= 0x40e231d5 {
        if ix > 0x40afeddf {
            return __cosdf(if sign != 0 { x as f64 + S4PIO2 } else { x as f64 - S4PIO2 });
        } else {
            return if sign != 0 { __sindf(-x as f64 - S3PIO2) } else { __sindf(x as f64 - S3PIO2) };
        }
    }
    if ix >= 0x7f800000 {
        return x - x;
    }
    let mut y = 0.0f64;
    let n = __rem_pio2f(x, &mut y);
    match n & 3 {
        0 =>  __cosdf(y),
        1 =>  __sindf(-y),
        2 => -__cosdf(y),
        _ =>  __sindf(y),
    }
}

#[no_mangle]
pub extern "C" fn tanf(x: f32) -> f32 {
    let ix = asuint(x);
    let sign = ix >> 31;
    let ix = ix & 0x7fffffff;

    if ix <= 0x3f490fda {
        if ix < 0x39800000 {
            force_eval(if ix < 0x00800000 { x / p2f(120) } else { x + p2f(120) });
            return x;
        }
        return __tandf(x as f64, 0);
    }
    if ix <= 0x407b53d1 {
        if ix <= 0x4016cbe3 {
            return __tandf(if sign != 0 { x as f64 + S1PIO2 } else { x as f64 - S1PIO2 }, 1);
        } else {
            return __tandf(if sign != 0 { x as f64 + S2PIO2 } else { x as f64 - S2PIO2 }, 0);
        }
    }
    if ix <= 0x40e231d5 {
        if ix <= 0x40afeddf {
            return __tandf(if sign != 0 { x as f64 + S3PIO2 } else { x as f64 - S3PIO2 }, 1);
        } else {
            return __tandf(if sign != 0 { x as f64 + S4PIO2 } else { x as f64 - S4PIO2 }, 0);
        }
    }
    if ix >= 0x7f800000 {
        return x - x;
    }
    let mut y = 0.0f64;
    let n = __rem_pio2f(x, &mut y);
    __tandf(y, n & 1)
}
