#ifndef _STDLIB_H
#define _STDLIB_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

int atoi(const char *);
long atol(const char *);
long long atoll(const char *);
long strtol(const char *, char **, int);
unsigned long strtoul(const char *, char **, int);
long long strtoll(const char *, char **, int);
unsigned long long strtoull(const char *, char **, int);

int abs(int);
long labs(long);
long long llabs(long long);

void srand(unsigned);
int rand(void);

#define RAND_MAX 0x7fff

#ifdef __cplusplus
}
#endif

#endif
