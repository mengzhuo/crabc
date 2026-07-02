#include <math.h>
#include <stdio.h>

static int check(double got, double want, double eps) {
    return (got > want - eps && got < want + eps) ? 0 : 1;
}

static int checkf(float got, float want, float eps) {
    return (got > want - eps && got < want + eps) ? 0 : 1;
}

int main(void) {
    int e;
    double iptr;
    float iptrf;

    if (check(sqrt(4.0), 2.0, 1e-9)) return 1;
    if (check(fabs(-3.5), 3.5, 1e-9)) return 2;
    if (check(floor(2.7), 2.0, 1e-9)) return 3;
    if (check(ceil(2.1), 3.0, 1e-9)) return 4;
    if (check(sin(0.0), 0.0, 1e-9)) return 5;
    if (check(cos(0.0), 1.0, 1e-9)) return 6;
    if (check(pow(2.0, 3.0), 8.0, 1e-9)) return 7;
    if (check(log(exp(1.0)), 1.0, 1e-9)) return 8;
    if (checkf(sqrtf(4.0f), 2.0f, 1e-6f)) return 9;

    if (check(trunc(2.9), 2.0, 1e-9)) return 10;
    if (check(trunc(-2.9), -2.0, 1e-9)) return 11;
    if (check(round(2.4), 2.0, 1e-9)) return 12;
    if (check(round(2.6), 3.0, 1e-9)) return 13;
    if (check(round(-2.5), -3.0, 1e-9)) return 14;
    if (check(copysign(1.0, -2.0), -1.0, 1e-9)) return 15;
    if (check(copysign(-1.0, 2.0), 1.0, 1e-9)) return 16;
    if (check(scalbn(1.5, 2), 6.0, 1e-9)) return 17;
    if (check(ldexp(1.5, 2), 6.0, 1e-9)) return 18;

    if (check(frexp(6.0, &e), 0.75, 1e-9)) return 19;
    if (e != 3) return 20;
    if (check(modf(3.75, &iptr), 0.75, 1e-9)) return 21;
    if (check(iptr, 3.0, 1e-9)) return 22;
    if (check(modf(-3.75, &iptr), -0.75, 1e-9)) return 23;
    if (check(iptr, -3.0, 1e-9)) return 24;

    if (checkf(truncf(2.9f), 2.0f, 1e-6f)) return 25;
    if (checkf(truncf(-2.9f), -2.0f, 1e-6f)) return 26;
    if (checkf(roundf(2.4f), 2.0f, 1e-6f)) return 27;
    if (checkf(roundf(2.6f), 3.0f, 1e-6f)) return 28;
    if (checkf(roundf(-2.5f), -3.0f, 1e-6f)) return 29;
    if (checkf(copysignf(1.0f, -2.0f), -1.0f, 1e-6f)) return 30;
    if (checkf(scalbnf(1.5f, 2), 6.0f, 1e-6f)) return 31;
    if (checkf(ldexpf(1.5f, 2), 6.0f, 1e-6f)) return 32;

    if (checkf(frexpf(6.0f, &e), 0.75f, 1e-6f)) return 33;
    if (e != 3) return 34;
    if (checkf(modff(3.75f, &iptrf), 0.75f, 1e-6f)) return 35;
    if (checkf(iptrf, 3.0f, 1e-6f)) return 36;
    if (checkf(modff(-3.75f, &iptrf), -0.75f, 1e-6f)) return 37;
    if (checkf(iptrf, -3.0f, 1e-6f)) return 38;

    if (check(sqrt(2.0), 1.4142135623730951, 1e-12)) return 39;
    if (checkf(sqrtf(2.0f), 1.4142135f, 1e-5f)) return 40;
    if (check(fmod(5.5, 2.0), 1.5, 1e-12)) return 41;
    if (checkf(fmodf(5.5f, 2.0f), 1.5f, 1e-6f)) return 42;
    if (check(sin(0.5235987755982989), 0.5, 1e-12)) return 43;
    if (check(cos(0.5235987755982989), 0.8660254037844386, 1e-12)) return 44;
    if (check(tan(0.7853981633974483), 1.0, 1e-12)) return 45;
    if (check(sin(-0x1.5f9f1bdb17192p+749), 0.623779899189803, 1e-12)) return 46;
    if (checkf(sinf(-0x1.a206fp+2f), -0.24593880772590637f, 1e-5f)) return 47;
    if (checkf(cosf(-0x1.a206fp+2f), 0.9692853689193726f, 1e-5f)) return 48;
    if (checkf(tanf(-0x1.a206fp+2f), -0.2537320852279663f, 1e-5f)) return 49;

    /* Wave 3: exp/log/pow exact identities and edge cases */
    if (check(exp(0.0), 1.0, 1e-12)) return 50;
    if (check(exp(1.0), 2.718281828459045, 1e-12)) return 51;
    if (check(log(1.0), 0.0, 1e-12)) return 52;
    if (check(log2(1.0), 0.0, 1e-12)) return 53;
    if (check(log10(1.0), 0.0, 1e-12)) return 54;
    if (check(pow(2.0, 10.0), 1024.0, 1e-9)) return 55;
    if (check(pow(0.0, 3.0), 0.0, 1e-9)) return 56;
    if (check(pow(-2.0, 3.0), -8.0, 1e-9)) return 57;
    if (check(pow(2.0, -1074.0), 0x1p-1074, 1e-20)) return 58;
    if (check(pow(2.0, -1075.0), 0.0, 1e-20)) return 59;
    if (check(pow(0x1p-1072, 1.0), 0x1p-1072, 1e-20)) return 60;
    if (check(pow(0x1p-537, 2.0), 0x1p-1074, 1e-20)) return 61;
    if (check(pow(0x1p+1023, -1.0), 0x1p-1023, 1e-20)) return 62;
    if (checkf(expf(0.0f), 1.0f, 1e-6f)) return 63;
    if (checkf(logf(1.0f), 0.0f, 1e-6f)) return 64;
    if (checkf(log2f(1.0f), 0.0f, 1e-6f)) return 65;
    if (checkf(powf(2.0f, 10.0f), 1024.0f, 1e-3f)) return 66;
    if (checkf(powf(-2.0f, 3.0f), -8.0f, 1e-5f)) return 67;
    if (checkf(powf(0x1p-63f, 2.0f), 0x1p-126f, 1e-7f)) return 68;

    /* Wave 4: hyperbolic, inverse trig, hypot, lrint family */
    if (check(hypot(3.0, 4.0), 5.0, 1e-12)) return 69;
    if (check(hypot(1e200, 1e200), 1.4142135623730951e200, 1e185)) return 70;
    if (checkf(hypotf(3.0f, 4.0f), 5.0f, 1e-6f)) return 71;
    if (check(sinh(0.0), 0.0, 1e-12)) return 72;
    if (check(cosh(0.0), 1.0, 1e-12)) return 73;
    if (check(tanh(0.0), 0.0, 1e-12)) return 74;
    if (check(sinh(0.881373587019543), 1.0, 1e-12)) return 75;
    if (check(cosh(0.881373587019543), 1.4142135623730951, 1e-12)) return 76;
    if (check(tanh(0.5493061443340549), 0.5, 1e-12)) return 77;
    if (checkf(sinhf(0.0f), 0.0f, 1e-6f)) return 78;
    if (checkf(coshf(0.0f), 1.0f, 1e-6f)) return 79;
    if (checkf(tanhf(0.0f), 0.0f, 1e-6f)) return 80;
    if (check(asin(0.0), 0.0, 1e-12)) return 81;
    if (check(asin(1.0), 1.5707963267948966, 1e-12)) return 82;
    if (check(acos(1.0), 0.0, 1e-12)) return 83;
    if (check(acos(0.0), 1.5707963267948966, 1e-12)) return 84;
    if (check(atan(1.0), 0.7853981633974483, 1e-12)) return 85;
    if (check(atan2(1.0, 0.0), 1.5707963267948966, 1e-12)) return 86;
    if (checkf(asinf(0.0f), 0.0f, 1e-6f)) return 87;
    if (checkf(acosf(1.0f), 0.0f, 1e-6f)) return 88;
    if (checkf(atanf(1.0f), 0.7853981633974483f, 1e-6f)) return 89;
    if (checkf(atan2f(1.0f, 0.0f), 1.5707963267948966f, 1e-6f)) return 90;
    if (lrint(2.3) != 2) return 91;
    if (lrint(2.7) != 3) return 92;
    if (llrint(2.7) != 3) return 93;
    if (lrintf(2.3f) != 2) return 94;
    if (llrintf(2.7f) != 3) return 95;
    if (lrintl(2.3L) != 2) return 96;
    if (llrintl(2.7L) != 3) return 97;
    if (check(sinh(0x1.d3e0d2f5d98d6p-2), 0x1.e45428082fb8cp-2, 1e-15)) return 98;

    printf("math ok\n");
    return 0;
}
