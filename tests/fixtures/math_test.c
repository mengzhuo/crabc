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

    printf("math ok\n");
    return 0;
}
