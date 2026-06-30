#ifndef MATH_H
#define MATH_H

#define M_E        2.71828182845904523536
#define M_LOG2E    1.44269504088896340736
#define M_LOG10E   0.434294481903251827651
#define M_LN2      0.693147180559945309417
#define M_LN10     2.30258509299404568402
#define M_PI       3.14159265358979323846
#define M_PI_2     1.57079632679489661923
#define M_PI_4     0.785398163397448309616
#define M_1_PI     0.318309886183790671538
#define M_2_PI     0.636619772367581343076
#define M_2_SQRTPI 1.12837916709551257390
#define M_SQRT2    1.41421356237309504880
#define M_SQRT1_2  0.707106781186547524401

#ifdef __cplusplus
extern "C" {
#endif

double acos(double);
double asin(double);
double atan(double);
double atan2(double, double);
double ceil(double);
double cos(double);
double cosh(double);
double exp(double);
double fabs(double);
double floor(double);
double fmod(double, double);
double frexp(double, int *);
double ldexp(double, int);
double log(double);
double log10(double);
double log2(double);
double modf(double, double *);
double pow(double, double);
double round(double);
double sin(double);
double sinh(double);
double sqrt(double);
double tan(double);
double tanh(double);
double trunc(double);
double hypot(double, double);

float acosf(float);
float asinf(float);
float atanf(float);
float atan2f(float, float);
float ceilf(float);
float cosf(float);
float coshf(float);
float expf(float);
float fabsf(float);
float floorf(float);
float fmodf(float, float);
float frexpf(float, int *);
float ldexpf(float, int);
float logf(float);
float log10f(float);
float log2f(float);
float modff(float, float *);
float powf(float, float);
float roundf(float);
float sinf(float);
float sinhf(float);
float sqrtf(float);
float tanf(float);
float tanhf(float);
float truncf(float);
float hypotf(float, float);

long lrint(double);
long lrintf(float);
long lrintl(long double);
long long llrint(double);
long long llrintf(float);
long long llrintl(long double);

#ifdef __cplusplus
}
#endif

#endif
