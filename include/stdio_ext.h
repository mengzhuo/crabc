#ifndef _STDIO_EXT_H
#define _STDIO_EXT_H

#include <stdio.h>

#ifdef __cplusplus
extern "C" {
#endif

#define FSETLOCKING_QUERY   0
#define FSETLOCKING_INTERNAL 1
#define FSETLOCKING_BYCALLER 2

size_t __fbufsize(FILE *);
size_t __fpending(FILE *);
int __flbf(FILE *);
int __freadable(FILE *);
int __fwritable(FILE *);
int __freading(FILE *);
int __fwriting(FILE *);
size_t __freadahead(FILE *);
const char *__freadptr(FILE *, size_t *);
void __freadptrinc(FILE *, size_t);
void __fseterr(FILE *);
void __fpurge(FILE *);
int __fsetlocking(FILE *, int);
void _flushlbf(void);

#ifdef __cplusplus
}
#endif

#endif
