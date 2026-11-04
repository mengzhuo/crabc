#ifndef _WCHAR_H
#define _WCHAR_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdarg.h>
#include <stdio.h>
#include <locale.h>
#include <time.h>

#ifndef NULL
#define NULL ((void*)0)
#endif

#ifndef __crabc_wchar_t_defined
#define __crabc_wchar_t_defined
#ifndef __cplusplus
#ifndef __DEFINED_wchar_t
#define __DEFINED_wchar_t
#if defined(__aarch64__)
typedef unsigned int wchar_t;
#else
typedef int wchar_t;
#endif
#endif
#endif
#endif

#ifndef __crabc_wint_t_defined
#define __crabc_wint_t_defined
#ifndef __DEFINED_wint_t
#define __DEFINED_wint_t
typedef unsigned int wint_t;
#endif
#endif

#ifndef __crabc_mbstate_t_defined
#define __crabc_mbstate_t_defined
#ifndef __DEFINED_mbstate_t
#define __DEFINED_mbstate_t
typedef unsigned int mbstate_t;
#endif
#endif

#if L'\0'-1 > 0
#ifndef WCHAR_MAX
#define WCHAR_MAX (0xffffffffu+L'\0')
#endif
#ifndef WCHAR_MIN
#define WCHAR_MIN (0+L'\0')
#endif
#else
#ifndef WCHAR_MAX
#define WCHAR_MAX (0x7fffffff+L'\0')
#endif
#ifndef WCHAR_MIN
#define WCHAR_MIN (-1-0x7fffffff+L'\0')
#endif
#endif

#ifndef WEOF
#define WEOF 0xffffffffU
#endif

wchar_t *wcscpy(wchar_t *, const wchar_t *);
wchar_t *wcsncpy(wchar_t *, const wchar_t *, size_t);
wchar_t *wcpcpy(wchar_t *, const wchar_t *);
wchar_t *wcpncpy(wchar_t *, const wchar_t *, size_t);

wchar_t *wcscat(wchar_t *, const wchar_t *);
wchar_t *wcsncat(wchar_t *, const wchar_t *, size_t);

int wcscmp(const wchar_t *, const wchar_t *);
int wcsncmp(const wchar_t *, const wchar_t *, size_t);
int wcscasecmp(const wchar_t *, const wchar_t *);
int wcsncasecmp(const wchar_t *, const wchar_t *, size_t);

int wcscoll(const wchar_t *, const wchar_t *);
size_t wcsxfrm(wchar_t *, const wchar_t *, size_t);
int wcscoll_l(const wchar_t *, const wchar_t *, locale_t);
int wcscasecmp_l(const wchar_t *, const wchar_t *, locale_t);
int wcsncasecmp_l(const wchar_t *, const wchar_t *, size_t, locale_t);
size_t wcsxfrm_l(wchar_t *, const wchar_t *, size_t, locale_t);

wchar_t *wcschr(const wchar_t *, wchar_t);
wchar_t *wcsrchr(const wchar_t *, wchar_t);

size_t wcscspn(const wchar_t *, const wchar_t *);
size_t wcsspn(const wchar_t *, const wchar_t *);
wchar_t *wcspbrk(const wchar_t *, const wchar_t *);

wchar_t *wcstok(wchar_t *, const wchar_t *, wchar_t **);

size_t wcslen(const wchar_t *);
wchar_t *wcsdup(const wchar_t *);
size_t wcsnlen(const wchar_t *, size_t);

wchar_t *wcsstr(const wchar_t *, const wchar_t *);
wchar_t *wcswcs(const wchar_t *, const wchar_t *);

wchar_t *wmemchr(const wchar_t *, wchar_t, size_t);
int wmemcmp(const wchar_t *, const wchar_t *, size_t);
wchar_t *wmemcpy(wchar_t *, const wchar_t *, size_t);
wchar_t *wmemmove(wchar_t *, const wchar_t *, size_t);
wchar_t *wmemset(wchar_t *, wchar_t, size_t);

wint_t btowc(int);
int wctob(wint_t);

int mbsinit(const mbstate_t *);
size_t mbrtowc(wchar_t *, const char *, size_t, mbstate_t *);
size_t wcrtomb(char *, wchar_t, mbstate_t *);
size_t mbrlen(const char *, size_t, mbstate_t *);
size_t mbsrtowcs(wchar_t *, const char **, size_t, mbstate_t *);
size_t wcsrtombs(char *, const wchar_t **, size_t, mbstate_t *);
size_t mbsnrtowcs(wchar_t *, const char **, size_t, size_t, mbstate_t *);
size_t wcsnrtombs(char *, const wchar_t **, size_t, size_t, mbstate_t *);

float wcstof(const wchar_t *, wchar_t **);
double wcstod(const wchar_t *, wchar_t **);
long double wcstold(const wchar_t *, wchar_t **);

long wcstol(const wchar_t *, wchar_t **, int);
unsigned long wcstoul(const wchar_t *, wchar_t **, int);
long long wcstoll(const wchar_t *, wchar_t **, int);
unsigned long long wcstoull(const wchar_t *, wchar_t **, int);

long long wcstoimax(const wchar_t *, wchar_t **, int);
unsigned long long wcstoumax(const wchar_t *, wchar_t **, int);

int fwide(FILE *, int);

int wprintf(const wchar_t *, ...);
int fwprintf(FILE *, const wchar_t *, ...);
int swprintf(wchar_t *, size_t, const wchar_t *, ...);

int vwprintf(const wchar_t *, va_list);
int vfwprintf(FILE *, const wchar_t *, va_list);
int vswprintf(wchar_t *, size_t, const wchar_t *, va_list);

int wscanf(const wchar_t *, ...);
int fwscanf(FILE *, const wchar_t *, ...);
int swscanf(const wchar_t *, const wchar_t *, ...);

int vwscanf(const wchar_t *, va_list);
int vfwscanf(FILE *, const wchar_t *, va_list);
int vswscanf(const wchar_t *, const wchar_t *, va_list);

wint_t fgetwc(FILE *);
wint_t getwc(FILE *);
wint_t getwchar(void);
wint_t fputwc(wchar_t, FILE *);
wint_t putwc(wchar_t, FILE *);
wint_t putwchar(wchar_t);
wchar_t *fgetws(wchar_t *, int, FILE *);
int fputws(const wchar_t *, FILE *);
wint_t ungetwc(wint_t, FILE *);

size_t wcsftime(wchar_t *, size_t, const wchar_t *, const struct tm *);
size_t wcsftime_l(wchar_t *, size_t, const wchar_t *, const struct tm *, locale_t);

FILE *open_wmemstream(wchar_t **, size_t *);

int wcwidth(wchar_t);
int wcswidth(const wchar_t *, size_t);

#ifdef __cplusplus
}
#endif

#endif
