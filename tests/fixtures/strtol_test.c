#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void)
{
    char *end;
    long r;
    unsigned long ur;

    errno = 0;
    r = strtol("9223372036854775808", &end, 0);
    if (r != LONG_MAX || errno != ERANGE) {
        printf("FAIL overflow positive: r=%ld errno=%d\n", r, errno);
        return 1;
    }

    errno = 0;
    r = strtol("-9223372036854775809", &end, 0);
    if (r != LONG_MIN || errno != ERANGE) {
        printf("FAIL overflow negative: r=%ld errno=%d\n", r, errno);
        return 2;
    }

    errno = 0;
    r = strtol("0xz", &end, 16);
    if (r != 0 || end[0] != 'x' || errno != 0) {
        printf("FAIL 0xz base16: r=%ld end=%s errno=%d\n", r, end, errno);
        return 3;
    }

    errno = 0;
    r = strtol("123", &end, 37);
    if (r != 0 || end[0] != '1' || errno != EINVAL) {
        printf("FAIL invalid base: r=%ld end=%s errno=%d\n", r, end, errno);
        return 4;
    }

    errno = 0;
    ur = strtoul("-18446744073709551616", &end, 0);
    if (ur != ULONG_MAX || errno != ERANGE) {
        printf("FAIL unsigned negative overflow: ur=%lu errno=%d\n", ur, errno);
        return 5;
    }

    errno = 0;
    ur = strtoul("-1", &end, 0);
    if (ur != ULONG_MAX || errno != 0) {
        printf("FAIL unsigned negative one: ur=%lu errno=%d\n", ur, errno);
        return 6;
    }

    printf("OK\n");
    return 0;
}
