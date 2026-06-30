#ifndef _FCNTL_H
#define _FCNTL_H

#ifdef __cplusplus
extern "C" {
#endif

int open(const char *, int, ...);
int creat(const char *, unsigned int);
int fcntl(int, int, ...);

#define O_RDONLY   0
#define O_WRONLY   1
#define O_RDWR     2
#define O_CREAT    64
#define O_EXCL     128
#define O_NOCTTY   256
#define O_TRUNC    512
#define O_APPEND   1024
#define O_NONBLOCK 2048
#define O_CLOEXEC  0x80000

#define F_DUPFD  0
#define F_GETFD  1
#define F_SETFD  2
#define F_GETFL  3
#define F_SETFL  4

#define FD_CLOEXEC 1

#ifdef __cplusplus
}
#endif

#endif
