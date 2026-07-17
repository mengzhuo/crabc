#ifndef _WCHAR_H
#define _WCHAR_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdarg.h>
#include <stdio.h>

#ifndef NULL
#define NULL ((void*)0)
#endif

#ifndef WEOF
#define WEOF 0xffffffffU
#endif

#ifndef __cplusplus
#if defined(__aarch64__)
typedef unsigned int wchar_t;
#elif defined(__riscv)
typedef int wchar_t;
#else
typedef int wchar_t;
#endif
#endif
typedef unsigned int wint_t;
typedef unsigned int mbstate_t;

/* Wide string functions */
size_t wcslen(const wchar_t *);
wchar_t *wcscpy(wchar_t *, const wchar_t *);
wchar_t *wcsncpy(wchar_t *, const wchar_t *, size_t);
wchar_t *wcscat(wchar_t *, const wchar_t *);
wchar_t *wcsncat(wchar_t *, const wchar_t *, size_t);
int wcscmp(const wchar_t *, const wchar_t *);
int wcsncmp(const wchar_t *, const wchar_t *, size_t);
wchar_t *wcschr(const wchar_t *, wchar_t);
wchar_t *wcsrchr(const wchar_t *, wchar_t);
wchar_t *wcsstr(const wchar_t *, const wchar_t *);
size_t wcscspn(const wchar_t *, const wchar_t *);
size_t wcsspn(const wchar_t *, const wchar_t *);
wchar_t *wcspbrk(const wchar_t *, const wchar_t *);
wchar_t *wcsdup(const wchar_t *);
size_t wcsnlen(const wchar_t *, size_t);
size_t wcsxfrm(wchar_t *, const wchar_t *, size_t);

/* Multibyte/wide conversions */
wint_t btowc(int);
int wctob(wint_t);
int mbsinit(const mbstate_t *);
size_t mbrtowc(wchar_t *, const char *, size_t, mbstate_t *);
size_t wcrtomb(char *, wchar_t, mbstate_t *);
size_t mbrlen(const char *, size_t, mbstate_t *);
size_t mbsrtowcs(wchar_t *, const char **, size_t, mbstate_t *);
size_t wcsrtombs(char *, const wchar_t **, size_t, mbstate_t *);
size_t mbstowcs(wchar_t *, const char *, size_t);
size_t wcstombs(char *, const wchar_t *, size_t);

/* Wide number conversions */
long wcstol(const wchar_t *, wchar_t **, int);
unsigned long wcstoul(const wchar_t *, wchar_t **, int);
long long wcstoll(const wchar_t *, wchar_t **, int);
unsigned long long wcstoull(const wchar_t *, wchar_t **, int);
double wcstod(const wchar_t *, wchar_t **);
float wcstof(const wchar_t *, wchar_t **);
long double wcstold(const wchar_t *, wchar_t **);
long long wcstoimax(const wchar_t *, wchar_t **, int);
unsigned long long wcstoumax(const wchar_t *, wchar_t **, int);

/* Wide stdio */
wint_t fgetwc(FILE *);
wint_t getwchar(void);
wint_t fputwc(wchar_t, FILE *);
wint_t putwchar(wchar_t);
int fputws(const wchar_t *, FILE *);
wint_t ungetwc(wint_t, FILE *);

/* Wide printf */
int swprintf(wchar_t *, size_t, const wchar_t *, ...);
int vswprintf(wchar_t *, size_t, const wchar_t *, va_list);
int fwprintf(FILE *, const wchar_t *, ...);
int vfwprintf(FILE *, const wchar_t *, va_list);

/* Wide scanf */
int wscanf(const wchar_t *, ...);
int fwscanf(FILE *, const wchar_t *, ...);
int swscanf(const wchar_t *, const wchar_t *, ...);
int vwscanf(const wchar_t *, va_list);
int vfwscanf(FILE *, const wchar_t *, va_list);
int vswscanf(const wchar_t *, const wchar_t *, va_list);

#ifdef __cplusplus
}
#endif

#endif
