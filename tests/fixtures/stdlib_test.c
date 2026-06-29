#include <stdio.h>
#include "stdlib.h"

int main(void) {
    if (atoi("123") != 123) return 1;
    if (atoi("-42") != -42) return 2;
    if (atol("9999999999") != 9999999999L) return 3;

    char *end;
    if (strtol("0x1a", &end, 0) != 26 || *end != 0) return 4;
    if (strtol("077", &end, 0) != 63 || *end != 0) return 5;
    if (strtol("abc", &end, 16) != 0xabc || *end != 0) return 6;
    if (strtol("123abc", &end, 10) != 123 || *end != 'a') return 7;

    if (strtoul("-1", NULL, 10) != (unsigned long)-1) return 8;
    if (strtoul("0xFF", NULL, 0) != 255) return 9;

    if (atoll("-9223372036854775807") != -9223372036854775807LL) return 10;

    if (abs(-7) != 7) return 11;
    if (labs(-7000000000L) != 7000000000L) return 12;
    if (llabs(-9000000000000000000LL) != 9000000000000000000LL) return 13;

    srand(1);
    int r1 = rand();
    int r2 = rand();
    if (r1 == r2) return 14;
    if (r1 < 0 || r1 > RAND_MAX) return 15;
    srand(1);
    if (rand() != r1) return 16;

    puts("stdlib ok");
    return 0;
}
