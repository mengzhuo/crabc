#ifndef _STRING_H
#define _STRING_H

#include <stddef.h>
#include <locale.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifndef NULL
#define NULL ((void*)0)
#endif

void *memcpy(void *, const void *, size_t);
void *memmove(void *, const void *, size_t);
void *memset(void *, int, size_t);
int memcmp(const void *, const void *, size_t);
int bcmp(const void *, const void *, size_t);
void *memchr(const void *, int, size_t);
void *memrchr(const void *, int, size_t);
void *memccpy(void *, const void *, int, size_t);

size_t strlen(const char *);
size_t strnlen(const char *, size_t);
char *strcpy(char *, const char *);
char *strncpy(char *, const char *, size_t);
char *stpcpy(char *, const char *);
char *stpncpy(char *, const char *, size_t);
char *strcat(char *, const char *);
char *strncat(char *, const char *, size_t);
int strcmp(const char *, const char *);
int strncmp(const char *, const char *, size_t);
char *strchr(const char *, int);
char *strrchr(const char *, int);
size_t strspn(const char *, const char *);
size_t strcspn(const char *, const char *);
char *strpbrk(const char *, const char *);
char *strstr(const char *, const char *);
char *strtok(char *, const char *);
char *strtok_r(char *, const char *, char **);
char *strdup(const char *);
char *strndup(const char *, size_t);

int strcoll(const char *, const char *);
size_t strxfrm(char *, const char *, size_t);

size_t strlcpy(char *, const char *, size_t);
size_t strlcat(char *, const char *, size_t);
void *memmem(const void *, size_t, const void *, size_t);

char *strerror(int);
int strerror_r(int, char *, size_t);
int strverscmp(const char *, const char *);
char *strsignal(int);

int strcoll_l(const char *, const char *, locale_t);
size_t strxfrm_l(char *, const char *, size_t, locale_t);
char *strerror_l(int, locale_t);

#ifdef __cplusplus
}
#endif

#endif
