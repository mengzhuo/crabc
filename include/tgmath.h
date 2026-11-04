#ifndef _TGMATH_H
#define _TGMATH_H

/*
 * Type-generic math.  crabc has no complex arithmetic support, so the
 * complex-only helpers are provided as real-valued fallbacks while the
 * real functions dispatch on float/double/long double.
 */

#include <math.h>

#define __IS_FP(x) (sizeof((x)+1ULL) == sizeof((x)+1.0f))
#define __FLT(x) (__IS_FP(x) && sizeof(x) == sizeof(float))
#define __LDBL(x) (__IS_FP(x) && sizeof(x) == sizeof(long double) && sizeof(long double) != sizeof(double))

#define __type1(c,t) __typeof__(*(0?(t*)0:(void*)!(c)))
#define __type2(c,t1,t2) __typeof__(*(0?(__type1(c,t1)*)0:(__type1(!(c),t2)*)0))
#define __RETCAST(x) (__type2(__IS_FP(x), __typeof__(x), double))
#define __RETCAST_2(x,y) (__type2(__IS_FP(x) && __IS_FP(y), \
    __typeof__((x)+(y)), __typeof__((x)+(y)+1.0)))
#define __RETCAST_3(x,y,z) (__type2(__IS_FP(x) && __IS_FP(y) && __IS_FP(z), \
    __typeof__((x)+(y)+(z)), __typeof__((x)+(y)+(z)+1.0)))

#define __tg_real_nocast(fun, x) ( \
    __FLT(x) ? fun ## f (x) : \
    __LDBL(x) ? fun ## l (x) : \
    fun(x) )

#define __tg_real(fun, x) (__RETCAST(x)__tg_real_nocast(fun, x))

#define __tg_real_2_1(fun, x, y) (__RETCAST(x)( \
    __FLT(x) ? fun ## f (x, y) : \
    __LDBL(x) ? fun ## l (x, y) : \
    fun(x, y) ))

#define __tg_real_2(fun, x, y) (__RETCAST_2(x, y)( \
    __FLT(x) && __FLT(y) ? fun ## f (x, y) : \
    __LDBL((x)+(y)) ? fun ## l (x, y) : \
    fun(x, y) ))

#define __tg_real_remquo(x, y, z) (__RETCAST_2(x, y)( \
    __FLT(x) && __FLT(y) ? remquof(x, y, z) : \
    __LDBL((x)+(y)) ? remquol(x, y, z) : \
    remquo(x, y, z) ))

#define __tg_real_fma(x, y, z) (__RETCAST_3(x, y, z)( \
    __FLT(x) && __FLT(y) && __FLT(z) ? fmaf(x, y, z) : \
    __LDBL((x)+(y)+(z)) ? fmal(x, y, z) : \
    fma(x, y, z) ))

/* complex helpers, reduced to their real-valued meaning */
#define carg(x) ((double)(x) < 0 ? 3.14159265358979323846 : 0.0)
#define cimag(x) (0.0)
#define conj(x) (x)
#define cproj(x) (x)
#define creal(x) ((double)(x))

#define acos(x)         __tg_real(acos, (x))
#define acosh(x)        __tg_real(acosh, (x))
#define asin(x)         __tg_real(asin, (x))
#define asinh(x)        __tg_real(asinh, (x))
#define atan(x)         __tg_real(atan, (x))
#define atan2(x,y)      __tg_real_2(atan2, (x), (y))
#define atanh(x)        __tg_real(atanh, (x))
#define cbrt(x)         __tg_real(cbrt, (x))
#define ceil(x)         __tg_real(ceil, (x))
#define copysign(x,y)   __tg_real_2(copysign, (x), (y))
#define cos(x)          __tg_real(cos, (x))
#define cosh(x)         __tg_real(cosh, (x))
#define erf(x)          __tg_real(erf, (x))
#define erfc(x)         __tg_real(erfc, (x))
#define exp(x)          __tg_real(exp, (x))
#define exp2(x)         __tg_real(exp2, (x))
#define expm1(x)        __tg_real(expm1, (x))
#define fabs(x)         __tg_real(fabs, (x))
#define fdim(x,y)       __tg_real_2(fdim, (x), (y))
#define floor(x)        __tg_real(floor, (x))
#define fma(x,y,z)      __tg_real_fma((x), (y), (z))
#define fmax(x,y)       __tg_real_2(fmax, (x), (y))
#define fmin(x,y)       __tg_real_2(fmin, (x), (y))
#define fmod(x,y)       __tg_real_2(fmod, (x), (y))
#define frexp(x,y)      __tg_real_2_1(frexp, (x), (y))
#define hypot(x,y)      __tg_real_2(hypot, (x), (y))
#define ilogb(x)        __tg_real_nocast(ilogb, (x))
#define ldexp(x,y)      __tg_real_2_1(ldexp, (x), (y))
#define lgamma(x)       __tg_real(lgamma, (x))
#define llrint(x)       __tg_real_nocast(llrint, (x))
#define llround(x)      __tg_real_nocast(llround, (x))
#define log(x)          __tg_real(log, (x))
#define log10(x)        __tg_real(log10, (x))
#define log1p(x)        __tg_real(log1p, (x))
#define log2(x)         __tg_real(log2, (x))
#define logb(x)         __tg_real(logb, (x))
#define lrint(x)        __tg_real_nocast(lrint, (x))
#define lround(x)       __tg_real_nocast(lround, (x))
#define nearbyint(x)    __tg_real(nearbyint, (x))
#define nextafter(x,y)  __tg_real_2(nextafter, (x), (y))
#define nexttoward(x,y) __tg_real_2(nexttoward, (x), (y))
#define pow(x,y)        __tg_real_2(pow, (x), (y))
#define remainder(x,y)  __tg_real_2(remainder, (x), (y))
#define remquo(x,y,z)   __tg_real_remquo((x), (y), (z))
#define rint(x)         __tg_real(rint, (x))
#define round(x)        __tg_real(round, (x))
#define scalbln(x,y)    __tg_real_2_1(scalbln, (x), (y))
#define scalbn(x,y)     __tg_real_2_1(scalbn, (x), (y))
#define sin(x)          __tg_real(sin, (x))
#define sinh(x)         __tg_real(sinh, (x))
#define sqrt(x)         __tg_real(sqrt, (x))
#define tan(x)          __tg_real(tan, (x))
#define tanh(x)         __tg_real(tanh, (x))
#define tgamma(x)       __tg_real(tgamma, (x))
#define trunc(x)        __tg_real(trunc, (x))

#endif
