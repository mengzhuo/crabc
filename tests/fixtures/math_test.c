#include <math.h>
#include <stdio.h>

static int check(double got, double want, double eps) {
    return (got > want - eps && got < want + eps) ? 0 : 1;
}

int main(void) {
    if (check(sqrt(4.0), 2.0, 1e-9)) return 1;
    if (check(fabs(-3.5), 3.5, 1e-9)) return 2;
    if (check(floor(2.7), 2.0, 1e-9)) return 3;
    if (check(ceil(2.1), 3.0, 1e-9)) return 4;
    if (check(sin(0.0), 0.0, 1e-9)) return 5;
    if (check(cos(0.0), 1.0, 1e-9)) return 6;
    if (check(pow(2.0, 3.0), 8.0, 1e-9)) return 7;
    if (check(log(exp(1.0)), 1.0, 1e-9)) return 8;
    if (check(sqrtf(4.0f), 2.0f, 1e-6f)) return 9;

    printf("math ok\n");
    return 0;
}
