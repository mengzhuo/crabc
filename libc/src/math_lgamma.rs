// Translated from musl: lgamma_r.c (lgamma_r, lgamma).
// Constants are the exact IEEE-754 bit patterns from the C source.

const LG_PI: f64 = asdouble(0x400921FB54442D18);
const LG_A0: f64 = asdouble(0x3FB3C467E37DB0C8);
const LG_A1: f64 = asdouble(0x3FD4A34CC4A60FAD);
const LG_A2: f64 = asdouble(0x3FB13E001A5562A7);
const LG_A3: f64 = asdouble(0x3F951322AC92547B);
const LG_A4: f64 = asdouble(0x3F7E404FB68FEFE8);
const LG_A5: f64 = asdouble(0x3F67ADD8CCB7926B);
const LG_A6: f64 = asdouble(0x3F538A94116F3F5D);
const LG_A7: f64 = asdouble(0x3F40B6C689B99C00);
const LG_A8: f64 = asdouble(0x3F2CF2ECED10E54D);
const LG_A9: f64 = asdouble(0x3F1C5088987DFB07);
const LG_A10: f64 = asdouble(0x3EFA7074428CFA52);
const LG_A11: f64 = asdouble(0x3F07858E90A45837);
const LG_TC: f64 = asdouble(0x3FF762D86356BE3F);
const LG_TF: f64 = asdouble(0xBFBF19B9BCC38A42);
const LG_TT: f64 = asdouble(0xBC50C7CAA48A971F);
const LG_T0: f64 = asdouble(0x3FDEF72BC8EE38A2);
const LG_T1: f64 = asdouble(0xBFC2E4278DC6C509);
const LG_T2: f64 = asdouble(0x3FB08B4294D5419B);
const LG_T3: f64 = asdouble(0xBFA0C9A8DF35B713);
const LG_T4: f64 = asdouble(0x3F9266E7970AF9EC);
const LG_T5: f64 = asdouble(0xBF851F9FBA91EC6A);
const LG_T6: f64 = asdouble(0x3F78FCE0E370E344);
const LG_T7: f64 = asdouble(0xBF6E2EFFB3E914D7);
const LG_T8: f64 = asdouble(0x3F6282D32E15C915);
const LG_T9: f64 = asdouble(0xBF56FE8EBF2D1AF1);
const LG_T10: f64 = asdouble(0x3F4CDF0CEF61A8E9);
const LG_T11: f64 = asdouble(0xBF41A6109C73E0EC);
const LG_T12: f64 = asdouble(0x3F34AF6D6C0EBBF7);
const LG_T13: f64 = asdouble(0xBF347F24ECC38C38);
const LG_T14: f64 = asdouble(0x3F35FD3EE8C2D3F4);
const LG_U0: f64 = asdouble(0xBFB3C467E37DB0C8);
const LG_U1: f64 = asdouble(0x3FE4401E8B005DFF);
const LG_U2: f64 = asdouble(0x3FF7475CD119BD6F);
const LG_U3: f64 = asdouble(0x3FEF497644EA8450);
const LG_U4: f64 = asdouble(0x3FCD4EAEF6010924);
const LG_U5: f64 = asdouble(0x3F8B678BBF2BAB09);
const LG_V1: f64 = asdouble(0x4003A5D7C2BD619C);
const LG_V2: f64 = asdouble(0x40010725A42B18F5);
const LG_V3: f64 = asdouble(0x3FE89DFBE45050AF);
const LG_V4: f64 = asdouble(0x3FBAAE55D6537C88);
const LG_V5: f64 = asdouble(0x3F6A5ABB57D0CF61);
const LG_S0: f64 = asdouble(0xBFB3C467E37DB0C8);
const LG_S1: f64 = asdouble(0x3FCB848B36E20878);
const LG_S2: f64 = asdouble(0x3FD4D98F4F139F59);
const LG_S3: f64 = asdouble(0x3FC2BB9CBEE5F2F7);
const LG_S4: f64 = asdouble(0x3F9B481C7E939961);
const LG_S5: f64 = asdouble(0x3F5E26B67368F239);
const LG_S6: f64 = asdouble(0x3F00BFECDD17E945);
const LG_R1: f64 = asdouble(0x3FF645A762C4AB74);
const LG_R2: f64 = asdouble(0x3FE71A1893D3DCDC);
const LG_R3: f64 = asdouble(0x3FC601EDCCFBDF27);
const LG_R4: f64 = asdouble(0x3F9317EA742ED475);
const LG_R5: f64 = asdouble(0x3F497DDACA41A95B);
const LG_R6: f64 = asdouble(0x3EDEBAF7A5B38140);
const LG_W0: f64 = asdouble(0x3FDACFE390C97D69);
const LG_W1: f64 = asdouble(0x3FB555555555553B);
const LG_W2: f64 = asdouble(0xBF66C16C16B02E5C);
const LG_W3: f64 = asdouble(0x3F4A019F98CF38B6);
const LG_W4: f64 = asdouble(0xBF4380CB8C0FE741);
const LG_W5: f64 = asdouble(0x3F4B67BA4CDAD5D1);
const LG_W6: f64 = asdouble(0xBF5AB89D0B9E43E4);

// sin(pi*x) assuming x > 2^-100, if sin(pi*x)==0 the sign is arbitrary.
#[inline]
fn lg_sin_pi(x: f64) -> f64 {
    // spurious inexact if odd int
    let x = 2.0 * (x * 0.5 - floor(x * 0.5)); // x mod 2.0
    let mut n = (x * 4.0) as i32;
    n = (n + 1) / 2;
    let x = x - n as f64 * 0.5;
    let x = x * LG_PI;
    match n {
        1 => __cos(x, 0.0),
        2 => __sin(-x, 0.0, 0),
        3 => -__cos(x, 0.0),
        _ => __sin(x, 0.0, 0),
    }
}

pub(crate) unsafe fn lgamma_r_impl(x: f64, signgamp: *mut i32) -> f64 {
    let u = x.to_bits();
    let mut x = x;
    let sign = u >> 63;
    let ix = ((u >> 32) as u32) & 0x7fffffff;

    *signgamp = 1;
    // purge off +-inf, NaN, +-0, tiny and negative arguments
    if ix >= 0x7ff00000 {
        return x * x;
    }
    if ix < (0x3ff - 70) << 20 {
        // |x| < 2**-70, return -log(|x|)
        if sign != 0 {
            x = -x;
            *signgamp = -1;
        }
        return -log(x);
    }

    let mut nadj = 0.0;
    if sign != 0 {
        x = -x;
        let mut t = lg_sin_pi(x);
        if t == 0.0 {
            // -integer
            return 1.0 / (x - x);
        }
        if t > 0.0 {
            *signgamp = -1;
        } else {
            t = -t;
        }
        nadj = log(LG_PI / (t * x));
    }

    // purge off 1 and 2
    let r = if (ix == 0x3ff00000 || ix == 0x40000000) && (u as u32) == 0 {
        0.0
    } else if ix < 0x40000000 {
        // for x < 2.0
        let mut r = 0.0;
        let y;
        let i;
        if ix <= 0x3feccccc {
            // lgamma(x) = lgamma(x+1)-log(x)
            r = -log(x);
            if ix >= 0x3FE76944 {
                y = 1.0 - x;
                i = 0;
            } else if ix >= 0x3FCDA661 {
                y = x - (LG_TC - 1.0);
                i = 1;
            } else {
                y = x;
                i = 2;
            }
        } else {
            if ix >= 0x3FFBB4C3 {
                // [1.7316,2]
                y = 2.0 - x;
                i = 0;
            } else if ix >= 0x3FF3B4C4 {
                // [1.23,1.73]
                y = x - LG_TC;
                i = 1;
            } else {
                y = x - 1.0;
                i = 2;
            }
        }
        match i {
            0 => {
                let z = y * y;
                let p1 = LG_A0
                    + z * (LG_A2 + z * (LG_A4 + z * (LG_A6 + z * (LG_A8 + z * LG_A10))));
                let p2 = z
                    * (LG_A1
                        + z * (LG_A3 + z * (LG_A5 + z * (LG_A7 + z * (LG_A9 + z * LG_A11)))));
                let p = y * p1 + p2;
                r += p - 0.5 * y;
            }
            1 => {
                let z = y * y;
                let w = z * y;
                let p1 = LG_T0 + w * (LG_T3 + w * (LG_T6 + w * (LG_T9 + w * LG_T12)));
                let p2 = LG_T1 + w * (LG_T4 + w * (LG_T7 + w * (LG_T10 + w * LG_T13)));
                let p3 = LG_T2 + w * (LG_T5 + w * (LG_T8 + w * (LG_T11 + w * LG_T14)));
                let p = z * p1 - (LG_TT - w * (p2 + y * p3));
                r += LG_TF + p;
            }
            _ => {
                let p1 = y
                    * (LG_U0
                        + y * (LG_U1
                            + y * (LG_U2 + y * (LG_U3 + y * (LG_U4 + y * LG_U5)))));
                let p2 = 1.0
                    + y * (LG_V1
                        + y * (LG_V2 + y * (LG_V3 + y * (LG_V4 + y * LG_V5))));
                r += -0.5 * y + p1 / p2;
            }
        }
        r
    } else if ix < 0x40200000 {
        // x < 8.0
        let i = x as i32;
        let y = x - i as f64;
        let p = y
            * (LG_S0
                + y * (LG_S1
                    + y * (LG_S2 + y * (LG_S3 + y * (LG_S4 + y * (LG_S5 + y * LG_S6))))));
        let q = 1.0
            + y * (LG_R1
                + y * (LG_R2 + y * (LG_R3 + y * (LG_R4 + y * (LG_R5 + y * LG_R6)))));
        let mut r = 0.5 * y + p / q;
        let mut z = 1.0;
        // lgamma(1+s) = log(s) + lgamma(s)
        if (3..=7).contains(&i) {
            let mut k = i;
            while k >= 3 {
                z *= y + (k as f64 - 1.0);
                k -= 1;
            }
            r += log(z);
        }
        r
    } else if ix < 0x43900000 {
        // 8.0 <= x < 2**58
        let t = log(x);
        let z = 1.0 / x;
        let y = z * z;
        let w = LG_W0
            + z * (LG_W1 + y * (LG_W2 + y * (LG_W3 + y * (LG_W4 + y * (LG_W5 + y * LG_W6)))));
        (x - 0.5) * (t - 1.0) + w
    } else {
        // 2**58 <= x <= inf
        x * (log(x) - 1.0)
    };

    if sign != 0 { nadj - r } else { r }
}

pub(crate) unsafe fn lgamma_impl(x: f64) -> f64 {
    lgamma_r_impl(x, core::ptr::addr_of_mut!(crate::signgam))
}
