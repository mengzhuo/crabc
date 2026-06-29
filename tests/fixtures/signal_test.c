#include "signal.h"
#include "stdio.h"

static volatile int got = 0;

static void handler(int sig) {
    got = sig;
}

int main(void) {
    if (signal(SIGINT, handler) == SIG_ERR) return 1;
    if (raise(SIGINT) != 0) return 2;
    if (got != SIGINT) return 3;

    if (signal(SIGINT, SIG_DFL) == SIG_ERR) return 4;

    sigset_t set;
    if (sigemptyset(&set) != 0) return 5;
    if (sigismember(&set, SIGINT) != 0) return 6;
    if (sigaddset(&set, SIGINT) != 0) return 7;
    if (sigismember(&set, SIGINT) != 1) return 8;
    if (sigfillset(&set) != 0) return 9;
    if (sigismember(&set, SIGTERM) != 1) return 10;
    if (sigdelset(&set, SIGINT) != 0) return 11;
    if (sigismember(&set, SIGINT) != 0) return 12;

    if (getpid() <= 0) return 13;

    puts("signal ok");
    return 0;
}
