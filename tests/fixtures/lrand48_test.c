#include <stdio.h>
#include <stdlib.h>

int main(void) {
    srand48(1);
    long l1 = lrand48();
    long l2 = lrand48();
    long l3 = lrand48();
    long m1 = mrand48();
    long m2 = mrand48();

    if (l1 != 89400484) return 1;
    if (l2 != 976015093) return 2;
    if (l3 != 1792756325) return 3;
    if (m1 != 1443049011) return 4;
    if (m2 != -1866208802) return 5;

    unsigned short def[3] = {0, 0, 0};
    long n1 = nrand48(def);
    long n2 = nrand48(def);
    long n3 = nrand48(def);
    if (n1 != 0) return 6;
    if (n2 != 2116118) return 7;
    if (n3 != 89401895) return 8;

    unsigned short s2[3] = {0x330e, 1, 0};
    long j1 = jrand48(s2);
    if (j1 != 178800969) return 9;

    srand48(42);
    unsigned short fresh[3] = {0x1234, 0x5678, 0x9abc};
    unsigned short *old = seed48(fresh);
    if (old[0] != 0x330e || old[1] != 42 || old[2] != 0) return 10;

    puts("lrand48 ok");
    return 0;
}
