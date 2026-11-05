// Translated from musl: erf.c (erf, erfc).
// Constants are the exact IEEE-754 bit patterns from the C source.

const ERX: f64 = asdouble(0x3FEB0AC160000000);
const EFX8: f64 = asdouble(0x3FF06EBA8214DB69);
const PP0: f64 = asdouble(0x3FC06EBA8214DB68);
const PP1: f64 = asdouble(0xBFD4CD7D691CB913);
const PP2: f64 = asdouble(0xBF9D2A51DBD7194F);
const PP3: f64 = asdouble(0xBF77A291236668E4);
const PP4: f64 = asdouble(0xBEF8EAD6120016AC);
const QQ1: f64 = asdouble(0x3FD97779CDDADC09);
const QQ2: f64 = asdouble(0x3FB0A54C5536CEBA);
const QQ3: f64 = asdouble(0x3F74D022C4D36B0F);
const QQ4: f64 = asdouble(0x3F215DC9221C1A10);
const QQ5: f64 = asdouble(0xBED09C4342A26120);
const PA0: f64 = asdouble(0xBF6359B8BEF77538);
const PA1: f64 = asdouble(0x3FDA8D00AD92B34D);
const PA2: f64 = asdouble(0xBFD7D240FBB8C3F1);
const PA3: f64 = asdouble(0x3FD45FCA805120E4);
const PA4: f64 = asdouble(0xBFBC63983D3E28EC);
const PA5: f64 = asdouble(0x3FA22A36599795EB);
const PA6: f64 = asdouble(0xBF61BF380A96073F);
const QA1: f64 = asdouble(0x3FBB3E6618EEE323);
const QA2: f64 = asdouble(0x3FE14AF092EB6F33);
const QA3: f64 = asdouble(0x3FB2635CD99FE9A7);
const QA4: f64 = asdouble(0x3FC02660E763351F);
const QA5: f64 = asdouble(0x3F8BEDC26B51DD1C);
const QA6: f64 = asdouble(0x3F888B545735151D);
const RA0: f64 = asdouble(0xBF843412600D6435);
const RA1: f64 = asdouble(0xBFE63416E4BA7360);
const RA2: f64 = asdouble(0xC0251E0441B0E726);
const RA3: f64 = asdouble(0xC04F300AE4CBA38D);
const RA4: f64 = asdouble(0xC0644CB184282266);
const RA5: f64 = asdouble(0xC067135CEBCCABB2);
const RA6: f64 = asdouble(0xC054526557E4D2F2);
const RA7: f64 = asdouble(0xC023A0EFC69AC25C);
const SA1: f64 = asdouble(0x4033A6B9BD707687);
const SA2: f64 = asdouble(0x4061350C526AE721);
const SA3: f64 = asdouble(0x407B290DD58A1A71);
const SA4: f64 = asdouble(0x40842B1921EC2868);
const SA5: f64 = asdouble(0x407AD02157700314);
const SA6: f64 = asdouble(0x405B28A3EE48AE2C);
const SA7: f64 = asdouble(0x401A47EF8E484A93);
const SA8: f64 = asdouble(0xBFAEEFF2EE749A62);
const RB0: f64 = asdouble(0xBF84341239E86F4A);
const RB1: f64 = asdouble(0xBFE993BA70C285DE);
const RB2: f64 = asdouble(0xC031C209555F995A);
const RB3: f64 = asdouble(0xC064145D43C5ED98);
const RB4: f64 = asdouble(0xC083EC881375F228);
const RB5: f64 = asdouble(0xC09004616A2E5992);
const RB6: f64 = asdouble(0xC07E384E9BDC383F);
const SB1: f64 = asdouble(0x403E568B261D5190);
const SB2: f64 = asdouble(0x40745CAE221B9F0A);
const SB3: f64 = asdouble(0x409802EB189D5118);
const SB4: f64 = asdouble(0x40A8FFB7688C246A);
const SB5: f64 = asdouble(0x40A3F219CEDF3BE6);
const SB6: f64 = asdouble(0x407DA874E79FE763);
const SB7: f64 = asdouble(0xC03670E242712D62);

#[inline]
fn erf_abs(x: f64) -> f64 {
    f64::from_bits(x.to_bits() & 0x7FFFFFFFFFFFFFFF)
}

#[inline]
fn erfc1(x: f64) -> f64 {
    let s = erf_abs(x) - 1.0;
    let p = PA0
        + s * (PA1 + s * (PA2 + s * (PA3 + s * (PA4 + s * (PA5 + s * PA6)))));
    let q = 1.0
        + s * (QA1 + s * (QA2 + s * (QA3 + s * (QA4 + s * (QA5 + s * QA6)))));
    1.0 - ERX - p / q
}

#[inline]
fn erfc2(ix: u32, x: f64) -> f64 {
    if ix < 0x3ff40000 {
        return erfc1(x);
    }
    let x = erf_abs(x);
    let s = 1.0 / (x * x);
    let (r, q);
    if ix < 0x4006db6d {
        r = RA0
            + s * (RA1
                + s * (RA2
                    + s * (RA3 + s * (RA4 + s * (RA5 + s * (RA6 + s * RA7))))));
        q = 1.0
            + s * (SA1
                + s * (SA2
                    + s * (SA3
                        + s * (SA4 + s * (SA5 + s * (SA6 + s * (SA7 + s * SA8)))))));
    } else {
        r = RB0 + s * (RB1 + s * (RB2 + s * (RB3 + s * (RB4 + s * (RB5 + s * RB6)))));
        q = 1.0
            + s * (SB1
                + s * (SB2
                    + s * (SB3 + s * (SB4 + s * (SB5 + s * (SB6 + s * SB7))))));
    }
    // SET_LOW_WORD(z, 0)
    let z = f64::from_bits(x.to_bits() & 0xFFFFFFFF00000000);
    exp(-z * z - 0.5625) * exp((z - x) * (z + x) + r / q) / x
}

pub(crate) fn erf_impl(x: f64) -> f64 {
    let bits = x.to_bits();
    let mut ix = (bits >> 32) as u32;
    let sign = ix >> 31;
    ix &= 0x7fffffff;
    if ix >= 0x7ff00000 {
        // erf(nan)=nan, erf(+-inf)=+-1
        return 1.0 - 2.0 * sign as f64 + 1.0 / x;
    }
    if ix < 0x3feb0000 {
        // |x| < 0.84375
        if ix < 0x3e300000 {
            // |x| < 2**-28, avoid underflow
            return 0.125 * (8.0 * x + EFX8 * x);
        }
        let z = x * x;
        let r = PP0 + z * (PP1 + z * (PP2 + z * (PP3 + z * PP4)));
        let s = 1.0 + z * (QQ1 + z * (QQ2 + z * (QQ3 + z * (QQ4 + z * QQ5))));
        let y = r / s;
        return x + x * y;
    }
    let y = if ix < 0x40180000 {
        1.0 - erfc2(ix, x)
    } else {
        1.0 - asdouble(0x0010000000000000)
    };
    if sign != 0 { -y } else { y }
}

pub(crate) fn erfc_impl(x: f64) -> f64 {
    let bits = x.to_bits();
    let mut ix = (bits >> 32) as u32;
    let sign = ix >> 31;
    ix &= 0x7fffffff;
    if ix >= 0x7ff00000 {
        // erfc(nan)=nan, erfc(+-inf)=0,2
        return 2.0 * sign as f64 + 1.0 / x;
    }
    if ix < 0x3feb0000 {
        // |x| < 0.84375
        if ix < 0x3c700000 {
            // |x| < 2**-56
            return 1.0 - x;
        }
        let z = x * x;
        let r = PP0 + z * (PP1 + z * (PP2 + z * (PP3 + z * PP4)));
        let s = 1.0 + z * (QQ1 + z * (QQ2 + z * (QQ3 + z * (QQ4 + z * QQ5))));
        let y = r / s;
        if sign != 0 || ix < 0x3fd00000 {
            // x < 1/4
            return 1.0 - (x + x * y);
        }
        return 0.5 - (x - 0.5 + x * y);
    }
    if ix < 0x403c0000 {
        // 0.84375 <= |x| < 28
        return if sign != 0 { 2.0 - erfc2(ix, x) } else { erfc2(ix, x) };
    }
    if sign != 0 {
        2.0 - asdouble(0x0010000000000000)
    } else {
        asdouble(0x0010000000000000) * asdouble(0x0010000000000000)
    }
}
