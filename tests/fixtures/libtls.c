#include <stdio.h>

__thread int libvar = 99;

int get_libvar(void) {
    fprintf(stderr, "libtls: get_libvar called\n");
    fflush(stderr);
    int v = libvar;
    fprintf(stderr, "libtls: libvar=%d\n", v);
    fflush(stderr);
    return v;
}

void set_libvar(int v) {
    fprintf(stderr, "libtls: set_libvar(%d) called\n", v);
    fflush(stderr);
    libvar = v;
    fprintf(stderr, "libtls: set_libvar done\n");
    fflush(stderr);
}
