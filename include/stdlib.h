#ifndef _STDLIB_H
#define _STDLIB_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define NULL ((void*)0)

typedef struct { int quot, rem; } div_t;
typedef struct { long quot, rem; } ldiv_t;
typedef struct { long long quot, rem; } lldiv_t;

#define EXIT_FAILURE 1
#define EXIT_SUCCESS 0

size_t __ctype_get_mb_cur_max(void);
#define MB_CUR_MAX (__ctype_get_mb_cur_max())

#define RAND_MAX (0x7fffffff)

int atoi(const char *);
long atol(const char *);
long long atoll(const char *);
long strtol(const char *, char **, int);
unsigned long strtoul(const char *, char **, int);
long long strtoll(const char *, char **, int);
unsigned long long strtoull(const char *, char **, int);
double strtod(const char *, char **);
float strtof(const char *, char **);
long double strtold(const char *, char **);
double atof(const char *);

int abs(int);
long labs(long);
long long llabs(long long);

div_t div(int, int);
ldiv_t ldiv(long, long);
lldiv_t lldiv(long long, long long);

void srand(unsigned);
int rand(void);

long lrand48(void);
long mrand48(void);
long nrand48(unsigned short *);
long jrand48(unsigned short *);
void srand48(long);
unsigned short *seed48(unsigned short *);
double drand48(void);
double erand48(unsigned short *);
void lcong48(unsigned short *);
void srandom(unsigned);
long random(void);
char *initstate(unsigned, char *, size_t);
char *setstate(char *);

void *malloc(size_t);
void *calloc(size_t, size_t);
void *realloc(void *, size_t);
void free(void *);

_Noreturn void abort(void);
int atexit(void (*)(void));
_Noreturn void exit(int);
_Noreturn void _Exit(int);

char *getenv(const char *);
int setenv(const char *, const char *, int);
int putenv(char *);
int unsetenv(const char *);
int clearenv(void);

int system(const char *);

void *bsearch(const void *, const void *, size_t, size_t, int (*)(const void *, const void *));
void qsort(void *, size_t, size_t, int (*)(const void *, const void *));
void qsort_r(void *, size_t, size_t, int (*)(const void *, const void *, void *), void *);

int mblen(const char *, size_t);
int mbtowc(int *__restrict, const char *__restrict, size_t);
int wctomb(char *, int);

#ifdef __cplusplus
}
#endif

#endif
