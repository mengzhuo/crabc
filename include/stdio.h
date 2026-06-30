#ifndef _STDIO_H
#define _STDIO_H

#include <stddef.h>
#include <stdarg.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct _FILE FILE;
typedef long long fpos_t;

extern FILE *stdin;
extern FILE *stdout;
extern FILE *stderr;

#define NULL ((void*)0)
#define EOF (-1)

#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

#define _IOFBF 0
#define _IOLBF 1
#define _IONBF 2

#define BUFSIZ 1024
#define FILENAME_MAX 4096
#define FOPEN_MAX 1000
#define TMP_MAX 10000
#define L_tmpnam 20

/* File access */
FILE *fopen(const char *, const char *);
FILE *fdopen(int, const char *);
FILE *freopen(const char *, const char *, FILE *);
int fclose(FILE *);

/* Buffering */
int setvbuf(FILE *, char *, int, size_t);
void setbuf(FILE *, char *);
void setbuffer(FILE *, char *, size_t);
int fflush(FILE *);

/* Wide orientation */
int fwide(FILE *, int);

/* Direct I/O */
size_t fread(void *, size_t, size_t, FILE *);
size_t fwrite(const void *, size_t, size_t, FILE *);

/* File positioning */
int fseek(FILE *, long, int);
long ftell(FILE *);
void rewind(FILE *);
int fseeko(FILE *, long long, int);
long long ftello(FILE *);
int fgetpos(FILE *, fpos_t *);
int fsetpos(FILE *, const fpos_t *);

/* Error handling */
int feof(FILE *);
int ferror(FILE *);
void clearerr(FILE *);
int fileno(FILE *);

/* Character I/O */
int fgetc(FILE *);
int getc(FILE *);
int getchar(void);
int fputc(int, FILE *);
int putc(int, FILE *);
int putchar(int);
int ungetc(int, FILE *);

/* Line I/O */
char *fgets(char *, int, FILE *);
int fputs(const char *, FILE *);
int puts(const char *);

/* Formatted output */
int printf(const char *, ...);
int fprintf(FILE *, const char *, ...);
int sprintf(char *, const char *, ...);
int snprintf(char *, size_t, const char *, ...);
int dprintf(int, const char *, ...);
int vprintf(const char *, va_list);
int vfprintf(FILE *, const char *, va_list);
int vsprintf(char *, const char *, va_list);
int vsnprintf(char *, size_t, const char *, va_list);
int vdprintf(int, const char *, va_list);

/* Formatted input */
int scanf(const char *, ...);
int fscanf(FILE *, const char *, ...);
int sscanf(const char *, const char *, ...);
int vscanf(const char *, va_list);
int vfscanf(FILE *, const char *, va_list);
int vsscanf(const char *, const char *, va_list);

/* Line input */
long long getdelim(char **, size_t *, int, FILE *);
long long getline(char **, size_t *, FILE *);

/* File operations */
int remove(const char *);
int rename(const char *, const char *);

/* Temp files */
char *tmpnam(char *);
FILE *tmpfile(void);

/* Pipes */
FILE *popen(const char *, const char *);
int pclose(FILE *);

/* Error */
void perror(const char *);

/* Memory streams */
FILE *open_memstream(char **, size_t *);
FILE *fmemopen(void *, size_t, const char *);

/* mkstemp */
int mkstemp(char *);

#ifdef __cplusplus
}
#endif

#endif
