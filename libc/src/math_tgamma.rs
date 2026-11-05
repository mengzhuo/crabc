// Translated from musl: tgamma.c.

const TG_PI: f64 = 3.141592653589793238462643383279502884;
const TG_GMHALF: f64 = 5.524680040776729583740234375;
const TG_N: usize = 12;

const TG_SNUM: [f64; TG_N + 1] = [
    23531376880.410759688572007674451636754734846804940,
    42919803642.649098768957899047001988850926355848959,
    35711959237.355668049440185451547166705960488635843,
    17921034426.037209699919755754458931112671403265390,
    6039542586.3520280050642916443072979210699388420708,
    1439720407.3117216736632230727949123939715485786772,
    248874557.86205415651146038641322942321632125127801,
    31426415.585400194380614231628318205362874684987640,
    2876370.6289353724412254090516208496135991145378768,
    186056.26539522349504029498971604569928220784236328,
    8071.6720023658162106380029022722506138218516325024,
    210.82427775157934587250973392071336271166969580291,
    2.5066282746310002701649081771338373386264310793408,
];

const TG_SDEN: [f64; TG_N + 1] = [
    0.0, 39916800.0, 120543840.0, 150917976.0, 105258076.0, 45995730.0, 13339535.0,
    2637558.0, 357423.0, 32670.0, 1925.0, 66.0, 1.0,
];

const TG_FACT: [f64; 23] = [
    1.0,
    1.0,
    2.0,
    6.0,
    24.0,
    120.0,
    720.0,
    5040.0,
    40320.0,
    362880.0,
    3628800.0,
    39916800.0,
    479001600.0,
    6227020800.0,
    87178291200.0,
    1307674368000.0,
    20922789888000.0,
    355687428096000.0,
    6402373705728000.0,
    121645100408832000.0,
    2432902008176640000.0,
    51090942171709440000.0,
    1124000727777607680000.0,
];

// sin(pi*x) with x > 0x1p-100, if sin(pi*x)==0 the sign is arbitrary.
#[inline]
fn tg_sinpi(mut x: f64) -> f64 {
    // argument reduction: x = |x| mod 2
    // spurious inexact when x is odd int
    x *= 0.5;
    x = 2.0 * (x - floor(x));

    // reduce x into [-.25,.25]
    let mut n = (4.0 * x) as i32;
    n = (n + 1) / 2;
    x -= n as f64 * 0.5;

    let x = x * TG_PI;
    match n {
        1 => __cos(x, 0.0),
        2 => __sin(-x, 0.0, 0),
        3 => -__cos(x, 0.0),
        _ => __sin(x, 0.0, 0),
    }
}

// S(x) rational function for positive x.
#[inline]
fn tg_s(x: f64) -> f64 {
    let mut num = 0.0;
    let mut den = 0.0;
    if x < 8.0 {
        let mut i = TG_N as i32;
        while i >= 0 {
            num = num * x + TG_SNUM[i as usize];
            den = den * x + TG_SDEN[i as usize];
            i -= 1;
        }
    } else {
        for i in 0..=TG_N {
            num = num / x + TG_SNUM[i];
            den = den / x + TG_SDEN[i];
        }
    }
    num / den
}

pub(crate) unsafe fn tgamma_impl(x: f64) -> f64 {
    let u = x.to_bits();
    let ix = ((u >> 32) as u32) & 0x7fffffff;
    let sign = u >> 63;

    // special cases
    if ix >= 0x7ff00000 {
        // tgamma(nan)=nan, tgamma(inf)=inf, tgamma(-inf)=nan with invalid
        return x + f64::INFINITY;
    }
    if ix < (0x3ff - 54) << 20 {
        // |x| < 2^-54: tgamma(x) ~ 1/x, +-0 raises div-by-zero
        return 1.0 / x;
    }

    // integer arguments
    // raise inexact when non-integer
    if x == floor(x) {
        if sign != 0 {
            feraiseexcept(FE_INVALID);
            return 0.0 / 0.0;
        }
        if x <= TG_FACT.len() as f64 {
            return TG_FACT[x as usize - 1];
        }
    }

    // x >= 172: tgamma(x)=inf with overflow
    // x =< -184: tgamma(x)=+-0 with underflow
    if ix >= 0x40670000 {
        // |x| >= 184
        if sign != 0 {
            fp_force_evalf((f64::from_bits(0x3810000000000000) / x) as f32);
            if floor(x) * 0.5 == floor(x * 0.5) {
                return 0.0;
            }
            return -0.0;
        }
        return x * f64::from_bits(0x7FE0000000000000);
    }

    let absx = if sign != 0 { -x } else { x };

    // handle the error of x + g - 0.5
    let y = absx + TG_GMHALF;
    let mut dy;
    if absx > TG_GMHALF {
        dy = y - absx;
        dy -= TG_GMHALF;
    } else {
        dy = y - TG_GMHALF;
        dy -= absx;
    }

    let mut z = absx - 0.5;
    let mut r = tg_s(absx) * exp(-y);
    if x < 0.0 {
        // reflection formula for negative x
        // sinpi(absx) is not 0, integers are already handled
        r = -TG_PI / (tg_sinpi(absx) * absx * r);
        dy = -dy;
        z = -z;
    }
    r += dy * (TG_GMHALF + 0.5) * r / y;
    z = pow(y, 0.5 * z);
    r * z * z
}
