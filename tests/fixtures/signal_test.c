#include "signal.h"
#include "stdio.h"

static void *mmap_anon(unsigned long size) {
    register long r10 __asm__("r10") = 0x22; /* MAP_PRIVATE|MAP_ANONYMOUS */
    register long r8 __asm__("r8") = -1;
    register long r9 __asm__("r9") = 0;
    long ret;
    __asm__ volatile(
        "syscall"
        : "=a"(ret)
        : "a"(9), "D"(0), "S"(size), "d"(3),
          "r"(r10), "r"(r8), "r"(r9)
        : "rcx", "r11", "memory"
    );
    return (void *)ret;
}

static volatile int got_signal = 0;
static volatile int on_altstack = 0;

static void usr1_handler(int sig) {
    got_signal = sig;
}

static void usr1_handler_alt(int sig) {
    got_signal = sig;
    stack_t ss;
    if (sigaltstack(NULL, &ss) == 0 && (ss.ss_flags & SS_ONSTACK))
        on_altstack = 1;
}

int main(void) {
    sigset_t set, oldmask, pending, fullset;
    struct sigaction sa, old_sa;
    int sig;

    sigfillset(&fullset);
    if (!sigismember(&fullset, SIGHUP)) return 1;
    if (!sigismember(&fullset, SIGTERM)) return 2;
    /* musl: signals 32/33 excluded from fillset */
    if (sigismember(&fullset, 32)) return 3;

    sigemptyset(&set);
    if (sigaddset(&set, 0) != -1) return 4;
    if (sigaddset(&set, 65) != -1) return 5;
    if (sigdelset(&set, 0) != -1) return 6;
    if (sigdelset(&set, 65) != -1) return 7;
    if (sigismember(&set, 0) != 0) return 8;
    if (sigismember(&set, 65) != 0) return 9;

    sa.sa_handler = usr1_handler;
    sa.sa_flags = 0;
    sigemptyset(&sa.sa_mask);
    sigaction(SIGUSR1, &sa, &old_sa);

    sigemptyset(&set);
    sigaddset(&set, SIGUSR1);
    sigprocmask(SIG_BLOCK, &set, &oldmask);

    raise(SIGUSR1);
    sigpending(&pending);
    if (!sigismember(&pending, SIGUSR1)) return 10;

    {
        sigset_t empty;
        sigemptyset(&empty);
        got_signal = 0;
        sigsuspend(&empty);
    }
    if (got_signal != SIGUSR1) return 11;

    raise(SIGUSR1);
    {
        siginfo_t info;
        struct timespec ts = {1, 0};
        if (sigtimedwait(&set, &info, &ts) != SIGUSR1) return 12;
        if (info.si_signo != SIGUSR1) return 13;
    }

    raise(SIGUSR1);
    {
        siginfo_t info;
        if (sigwaitinfo(&set, &info) != SIGUSR1) return 14;
        if (info.si_signo != SIGUSR1) return 15;
    }

    raise(SIGUSR1);
    sig = 0;
    if (sigwait(&set, &sig) != 0) return 16;
    if (sig != SIGUSR1) return 17;

    sigprocmask(SIG_SETMASK, &oldmask, NULL);

    {
        stack_t altstack, oldstack;
        altstack.ss_sp = mmap_anon(SIGSTKSZ);
        altstack.ss_size = SIGSTKSZ;
        altstack.ss_flags = 0;
        if (sigaltstack(&altstack, &oldstack) != 0) return 18;

        sa.sa_handler = usr1_handler_alt;
        sa.sa_flags = SA_ONSTACK;
        sigemptyset(&sa.sa_mask);
        sigaction(SIGUSR1, &sa, NULL);

        got_signal = 0;
        on_altstack = 0;
        raise(SIGUSR1);
        if (got_signal != SIGUSR1) return 19;
        if (!on_altstack) return 20;

        sigaltstack(&oldstack, NULL);
    }

    sa.sa_handler = usr1_handler;
    sa.sa_flags = 0;
    sigemptyset(&sa.sa_mask);
    sigaction(SIGUSR1, &sa, NULL);
    got_signal = 0;
    tgkill(getpid(), getpid(), SIGUSR1);
    if (got_signal != SIGUSR1) return 21;

    sigaction(SIGUSR1, &old_sa, NULL);

    puts("signal ok");
    return 0;
}
