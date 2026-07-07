#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#include <stdio.h>
#include "string.h"

int main(void) {
    int fail = 0;

    if (strverscmp("", "") != 0) { puts("fail: empty==empty"); fail++; }
    if (strverscmp("a", "a") != 0) { puts("fail: a==a"); fail++; }

    if (strverscmp("a", "b") >= 0) { puts("fail: a<b"); fail++; }
    if (strverscmp("b", "a") <= 0) { puts("fail: b>a"); fail++; }

    if (strverscmp("000", "00") >= 0) { puts("fail: 000<00"); fail++; }
    if (strverscmp("00", "000") <= 0) { puts("fail: 00>000"); fail++; }

    if (strverscmp("9", "10") >= 0) { puts("fail: 9<10"); fail++; }
    if (strverscmp("1.2", "1.10") <= 0) { puts("fail: 1.2<1.10"); fail++; }
    if (strverscmp("a1b2", "a1b10") <= 0) { puts("fail: a1b2<a1b10"); fail++; }

    if (strverscmp("01", "010") >= 0) { puts("fail: 01<010"); fail++; }
    if (strverscmp("010", "09") >= 0) { puts("fail: 010<09"); fail++; }
    if (strverscmp("09", "0") >= 0) { puts("fail: 09<0"); fail++; }

    if (strverscmp("00", "01") >= 0) { puts("fail: 00<01"); fail++; }
    if (strverscmp("a0", "a") <= 0) { puts("fail: a0>a"); fail++; }
    if (strverscmp("0a", "0") <= 0) { puts("fail: 0a>0"); fail++; }

    if (strverscmp("foobar-1.1.2", "foobar-1.1.3") >= 0) { puts("fail: 1.1.2<1.1.3"); fail++; }
    if (strverscmp("foobar-1.1.2", "foobar-1.01.3") <= 0) { puts("fail: 1.1.2>1.01.3"); fail++; }

    if (strverscmp("foo", "foo~") >= 0) { puts("fail: foo<foo~"); fail++; }
    if (strverscmp("foo~", "foo") <= 0) { puts("fail: foo~<foo"); fail++; }

    if (fail == 0) {
        puts("strverscmp ok");
    }
    return fail;
}
