#ifndef _SYS_WAIT_H
#define _SYS_WAIT_H

#include <sys/types.h>
#include <signal.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifndef WNOHANG
#define WNOHANG   1
#define WUNTRACED 2
#define WSTOPPED  2
#define WEXITED   4
#define WCONTINUED 8
#define WNOWAIT   0x01000000
#endif

#ifndef WEXITSTATUS
#define WEXITSTATUS(s) (((s) & 0xff00) >> 8)
#define WTERMSIG(s)    ((s) & 0x7f)
#define WSTOPSIG(s)    WEXITSTATUS(s)
#define WIFEXITED(s)   (WTERMSIG(s) == 0)
#define WIFSIGNALED(s) (((signed char)(((s) & 0x7f) + 1) >> 1) > 0)
#define WIFSTOPPED(s)  (((s) & 0xff) == 0x7f)
#define WIFCONTINUED(s) ((s) == 0xffff)
#endif

typedef enum {
    P_ALL = 0,
    P_PID = 1,
    P_PGID = 2,
    P_PIDFD = 3,
} idtype_t;

pid_t wait(int *);
pid_t waitpid(pid_t, int *, int);
int waitid(idtype_t, id_t, siginfo_t *, int);

#ifdef __cplusplus
}
#endif

#endif
