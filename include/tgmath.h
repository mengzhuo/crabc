#ifndef _TGMATH_H
#define _TGMATH_H

#include <math.h>

#define lrint(x) _Generic((x), \
    float: lrintf, \
    long double: lrintl, \
    default: lrint \
)(x)

#define llrint(x) _Generic((x), \
    float: llrintf, \
    long double: llrintl, \
    default: llrint \
)(x)

#endif
