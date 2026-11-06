#ifndef _SIGNAL_H
#define _SIGNAL_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <sys/types.h>
#include <time.h>

#define SIGHUP    1
#define SIGINT    2
#define SIGQUIT   3
#define SIGILL    4
#define SIGTRAP   5
#define SIGABRT   6
#define SIGIOT    SIGABRT
#define SIGBUS    7
#define SIGFPE    8
#define SIGKILL   9
#define SIGUSR1  10
#define SIGSEGV  11
#define SIGUSR2  12
#define SIGPIPE  13
#define SIGALRM  14
#define SIGTERM  15
#define SIGSTKFLT 16
#define SIGCHLD  17
#define SIGCONT  18
#define SIGSTOP  19
#define SIGTSTP  20
#define SIGTTIN  21
#define SIGTTOU  22
#define SIGURG   23
#define SIGXCPU  24
#define SIGXFSZ  25
#define SIGVTALRM 26
#define SIGPROF  27
#define SIGWINCH 28
#define SIGIO    29
#define SIGPOLL  29
#define SIGPWR   30
#define SIGSYS   31
#define SIGUNUSED SIGSYS

#define _NSIG    65
#define NSIG     _NSIG

#define SIG_BLOCK   0
#define SIG_UNBLOCK 1
#define SIG_SETMASK 2

#define SIG_ERR  ((void (*)(int))-1)
#define SIG_DFL  ((void (*)(int)) 0)
#define SIG_IGN  ((void (*)(int)) 1)
#define SIG_HOLD ((void (*)(int)) 2)

#define SA_NOCLDSTOP  1
#define SA_NOCLDWAIT  2
#define SA_SIGINFO    4
#define SA_ONSTACK    0x08000000
#define SA_RESTART    0x10000000
#define SA_NODEFER    0x40000000
#define SA_RESETHAND  0x80000000
#define SA_RESTORER   0x04000000

#define SS_ONSTACK    1
#define SS_DISABLE    2
#define SS_AUTODISARM (1U << 31)
#define SS_FLAG_BITS  SS_AUTODISARM

#define MINSIGSTKSZ   2048
#define SIGSTKSZ      8192

#define SI_ASYNCNL (-60)
#define SI_TKILL   (-6)
#define SI_SIGIO   (-5)
#define SI_ASYNCIO (-4)
#define SI_MESGQ   (-3)
#define SI_TIMER   (-2)
#define SI_QUEUE   (-1)
#define SI_USER     0
#define SI_KERNEL   128

#define FPE_INTDIV 1
#define FPE_INTOVF 2
#define FPE_FLTDIV 3
#define FPE_FLTOVF 4
#define FPE_FLTUND 5
#define FPE_FLTRES 6
#define FPE_FLTINV 7
#define FPE_FLTSUB 8

#define ILL_ILLOPC 1
#define ILL_ILLOPN 2
#define ILL_ILLADR 3
#define ILL_ILLTRP 4
#define ILL_PRVOPC 5
#define ILL_PRVREG 6
#define ILL_COPROC 7
#define ILL_BADSTK 8

#define SEGV_MAPERR 1
#define SEGV_ACCERR 2
#define SEGV_BNDERR 3
#define SEGV_PKUERR 4

#define BUS_ADRALN 1
#define BUS_ADRERR 2
#define BUS_OBJERR 3
#define BUS_MCEERR_AR 4
#define BUS_MCEERR_AO 5

#define CLD_EXITED    1
#define CLD_KILLED    2
#define CLD_DUMPED    3
#define CLD_TRAPPED   4
#define CLD_STOPPED   5
#define CLD_CONTINUED 6

#define TRAP_BRKPT 1
#define TRAP_TRACE 2

#ifndef __crabc_sigset_t_defined
#define __crabc_sigset_t_defined
#ifndef __DEFINED_sigset_t
#define __DEFINED_sigset_t
#ifndef _SIGSET_T_DEFINED
#define _SIGSET_T_DEFINED
#endif
typedef struct __sigset_t { unsigned long __bits[128/sizeof(unsigned long)]; } sigset_t;
#endif
#endif

typedef int sig_atomic_t;

union sigval {
    int sival_int;
    void *sival_ptr;
};

typedef struct {
    int si_signo, si_errno, si_code;
    union {
        char __pad[128 - 2*sizeof(int) - sizeof(long)];
        struct {
            union {
                struct {
                    pid_t si_pid;
                    uid_t si_uid;
                } __piduid;
                struct {
                    int si_timerid;
                    int si_overrun;
                } __timer;
            } __first;
            union {
                union sigval si_value;
                struct {
                    int si_status;
                    clock_t si_utime, si_stime;
                } __sigchld;
            } __second;
        } __si_common;
        struct {
            void *si_addr;
            short si_addr_lsb;
            union {
                struct {
                    void *si_lower;
                    void *si_upper;
                } __addr_bnd;
                unsigned int si_pkey;
            } __first;
        } __sigfault;
        struct {
            long si_band;
            int si_fd;
        } __sigpoll;
        struct {
            void *si_call_addr;
            int si_syscall;
            unsigned int si_arch;
        } __sigsys;
    } __si_fields;
} siginfo_t;

#define si_pid     __si_fields.__si_common.__first.__piduid.si_pid
#define si_uid     __si_fields.__si_common.__first.__piduid.si_uid
#define si_status  __si_fields.__si_common.__second.__sigchld.si_status
#define si_utime   __si_fields.__si_common.__second.__sigchld.si_utime
#define si_stime   __si_fields.__si_common.__second.__sigchld.si_stime
#define si_value   __si_fields.__si_common.__second.si_value
#define si_addr    __si_fields.__sigfault.si_addr
#define si_addr_lsb __si_fields.__sigfault.si_addr_lsb
#define si_lower   __si_fields.__sigfault.__first.__addr_bnd.si_lower
#define si_upper   __si_fields.__sigfault.__first.__addr_bnd.si_upper
#define si_pkey    __si_fields.__sigfault.__first.si_pkey
#define si_band    __si_fields.__sigpoll.si_band
#define si_fd      __si_fields.__sigpoll.si_fd
#define si_timerid __si_fields.__si_common.__first.__timer.si_timerid
#define si_overrun __si_fields.__si_common.__first.__timer.si_overrun
#define si_ptr     si_value.sival_ptr
#define si_int     si_value.sival_int

struct sigaction {
    union {
        void (*sa_handler)(int);
        void (*sa_sigaction)(int, siginfo_t *, void *);
    } __sa_handler;
    sigset_t sa_mask;
    int sa_flags;
    void (*sa_restorer)(void);
};

#define sa_handler   __sa_handler.sa_handler
#define sa_sigaction __sa_handler.sa_sigaction

struct sigevent {
    union sigval sigev_value;
    int sigev_signo;
    int sigev_notify;
    union {
        char __pad[64 - 2*sizeof(int) - sizeof(union sigval)];
        pid_t sigev_notify_thread_id;
        struct {
            void (*sigev_notify_function)(union sigval);
            pthread_attr_t *sigev_notify_attributes;
        } __sev_thread;
    } __sev_fields;
};

#define sigev_notify_thread_id __sev_fields.sigev_notify_thread_id
#define sigev_notify_function __sev_fields.__sev_thread.sigev_notify_function
#define sigev_notify_attributes __sev_fields.__sev_thread.sigev_notify_attributes

#define SIGEV_SIGNAL    0
#define SIGEV_NONE      1
#define SIGEV_THREAD    2
#define SIGEV_THREAD_ID 4

struct sigaltstack {
    void *ss_sp;
    int ss_flags;
    size_t ss_size;
};

typedef struct sigaltstack stack_t;

typedef struct {
    unsigned long __space[32];
} mcontext_t;

typedef struct __ucontext {
    unsigned long uc_flags;
    struct __ucontext *uc_link;
    stack_t uc_stack;
    mcontext_t uc_mcontext;
    sigset_t uc_sigmask;
    unsigned long __fpregs_mem[64];
} ucontext_t;

int __libc_current_sigrtmin(void);
int __libc_current_sigrtmax(void);

#define SIGRTMIN (__libc_current_sigrtmin())
#define SIGRTMAX (__libc_current_sigrtmax())

int sigaction(int, const struct sigaction *, struct sigaction *);
void (*signal(int, void (*)(int)))(int);
int raise(int);
int kill(pid_t, int);
int killpg(pid_t, int);
int tgkill(int, int, int);
int sigemptyset(sigset_t *);
int sigfillset(sigset_t *);
int sigaddset(sigset_t *, int);
int sigdelset(sigset_t *, int);
int sigismember(const sigset_t *, int);
int sigprocmask(int, const sigset_t *, sigset_t *);
int sigpending(sigset_t *);
int sigsuspend(const sigset_t *);
int sigtimedwait(const sigset_t *, siginfo_t *, const struct timespec *);
int sigwaitinfo(const sigset_t *, siginfo_t *);
int sigwait(const sigset_t *, int *);
int sigaltstack(const stack_t *, stack_t *);
int sigqueue(pid_t, int, union sigval);
void psiginfo(const siginfo_t *, const char *);
void psignal(int, const char *);
int sighold(int);
int sigignore(int);
int siginterrupt(int, int);
int sigpause(int);
int sigrelse(int);
void (*sigset(int, void (*)(int)))(int);
int pthread_sigmask(int, const sigset_t *, sigset_t *);
int pthread_kill(pthread_t, int);

int getpid(void);

#ifdef __cplusplus
}
#endif

#endif
