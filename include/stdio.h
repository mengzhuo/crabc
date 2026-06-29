#ifndef _STDIO_H
#define _STDIO_H

#include <stddef.h>
#include <stdarg.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct _FILE FILE;
extern FILE *stdin;
extern FILE *stdout;
extern FILE *stderr;

int printf(const char *, ...);
int fprintf(FILE *, const char *, ...);
int dprintf(int, const char *, ...);
int vprintf(const char *, va_list);
int vfprintf(FILE *, const char *, va_list);
int puts(const char *);
int fputs(const char *, FILE *);
int fputc(int, FILE *);
int putchar(int);

int fgetc(FILE *);
int getc(FILE *);
int getchar(void);
char *fgets(char *, int, FILE *);
size_t fread(void *, size_t, size_t, FILE *);
int ungetc(int, FILE *);

#define EOF (-1)

#ifdef __cplusplus
}
#endif

#endif
