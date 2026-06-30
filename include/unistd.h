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

int pipe(int *);
int pipe2(int *, int);
int dup(int);
int dup2(int, int);
int dup3(int, int, int);
int fcntl(int, int, ...);
int access(const char *, int);
int unlink(const char *);
int rmdir(const char *);
int chdir(const char *);
char *getcwd(char *, size_t);
int gethostname(char *, size_t);
int getpagesize(void);
int truncate(const char *, long long);
int ftruncate(int, long long);
long long lseek(int, long long, int);

int nanosleep(const struct timespec *, struct timespec *);
unsigned int alarm(unsigned int);
int pause(void);
int fsync(int);
int fdatasync(int);
void sync(void);

int symlink(const char *, const char *);
ssize_t readlink(const char *, char *, size_t);
int link(const char *, const char *);
int chmod(const char *, unsigned int);
int fchmod(int, unsigned int);
unsigned int umask(unsigned int);
int isatty(int);
char *ttyname(int);
char *getlogin(void);
int getgroups(int, unsigned int *);
int setuid(unsigned int);
int setgid(unsigned int);
int seteuid(unsigned int);
int setegid(unsigned int);
int setreuid(unsigned int, unsigned int);
int setregid(unsigned int, unsigned int);
pid_t setsid(void);
int setpgid(pid_t, pid_t);
pid_t getpgid(pid_t);
pid_t getsid(pid_t);
pid_t getpgrp(void);
int setpgrp(void);
int mkstemp(char *);
int mkdir(const char *, unsigned int);

#define WIFEXITED(s)   (((s) & 0x7f) == 0)
#define WEXITSTATUS(s) (((s) & 0xff00) >> 8)
#define WIFSIGNALED(s) (((s) & 0x7f) != 0 && (((s) & 0x7f) != 0x7f))
#define WTERMSIG(s)    ((s) & 0x7f)

#ifdef __cplusplus
}
#endif

#endif
