#ifndef _UNISTD_H
#define _UNISTD_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef long ssize_t;
typedef int pid_t;
typedef unsigned int uid_t;
typedef unsigned int gid_t;

int fork(void);
int execve(const char *, char *const [], char *const []);
int wait(int *);
int waitpid(int, int *, int);
pid_t getpid(void);
pid_t getppid(void);
uid_t getuid(void);
gid_t getgid(void);
uid_t geteuid(void);
gid_t getegid(void);

int close(int);
ssize_t read(int, void *, size_t);
ssize_t write(int, const void *, size_t);
void _exit(int) __attribute__((noreturn));

unsigned int sleep(unsigned int);
int usleep(unsigned int);

#define WIFEXITED(s)   (((s) & 0x7f) == 0)
#define WEXITSTATUS(s) (((s) & 0xff00) >> 8)
#define WIFSIGNALED(s) (((s) & 0x7f) != 0 && (((s) & 0x7f) != 0x7f))
#define WTERMSIG(s)    ((s) & 0x7f)

#ifdef __cplusplus
}
#endif

#endif
