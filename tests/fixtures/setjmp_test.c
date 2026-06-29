#include "setjmp.h"
#include "stdio.h"

static jmp_buf env;
static int count = 0;

int main(void) {
    int r = setjmp(env);
    if (r == 0) {
        if (++count != 1) return 1;
        longjmp(env, 42);
        return 2;
    }
    if (r != 42) return 3;

    count = 0;
    r = setjmp(env);
    if (r == 0) {
        if (++count != 1) return 4;
        longjmp(env, 0);
        return 5;
    }
    if (r != 1) return 6;

    puts("setjmp ok");
    return 0;
}
