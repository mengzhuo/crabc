#ifndef _STDLIB_H
#define _STDLIB_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifndef NULL
#define NULL ((void*)0)
#endif

#ifndef __crabc_wchar_t_defined
#define __crabc_wchar_t_defined
#ifndef __cplusplus
#ifndef __DEFINED_wchar_t
#define __DEFINED_wchar_t
typedef int wchar_t;
#endif
#endif
#endif

typedef struct { int quot, rem; } div_t;
typedef struct { long quot, rem; } ldiv_t;
typedef struct { long long quot, rem; } lldiv_t;

#define EXIT_FAILURE 1
#define EXIT_SUCCESS 0

#define RAND_MAX (0x7fffffff)

#ifndef WIFEXITED
#define WNOHANG    1
#define WUNTRACED  2

#define WEXITSTATUS(s) (((s) & 0xff00) >> 8)
#define WTERMSIG(s) ((s) & 0x7f)
#define WSTOPSIG(s) WEXITSTATUS(s)
#define WIFEXITED(s) (!WTERMSIG(s))
#define WIFSTOPPED(s) ((short)((((s)&0xffff)*0x10001U)>>8) > 0x7f00)
#define WIFSIGNALED(s) (((s)&0xffff)-1U < 0xffu)
#endif

size_t __ctype_get_mb_cur_max(void);
#define MB_CUR_MAX (__ctype_get_mb_cur_max())

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

int rand(void);
void srand(unsigned);
int rand_r(unsigned *);

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
int posix_memalign(void **, size_t, size_t);

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
int mbtowc(wchar_t *, const char *, size_t);
int wctomb(char *, wchar_t);
size_t mbstowcs(wchar_t *, const char *, size_t);
size_t wcstombs(char *, const wchar_t *, size_t);

char *mkdtemp(char *);
int mkstemp(char *);
int mkstemps(char *, int);
int mkostemp(char *, int);
int mkostemps(char *, int, int);
int getsubopt(char **, char *const *, char **);
char *realpath(const char *, char *);
int posix_openpt(int);
int grantpt(int);
int unlockpt(int);
char *ptsname(int);
int ptsname_r(int, char *, size_t);

#ifdef __cplusplus
}
#endif

#endif
