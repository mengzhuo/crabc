#define _GNU_SOURCE
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static unsigned long long now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (unsigned long long)ts.tv_sec * 1000000000ull + (unsigned long long)ts.tv_nsec;
}

int main(int argc, char **argv) {
    if (argc < 3) return 2;
    int n = atoi(argv[2]);
    unsigned long long t0 = now_ns();
    for (int i = 0; i < n; i++) {
        pid_t pid = fork();
        if (pid == 0) {
            int fd = open("/dev/null", O_WRONLY);
            if (fd >= 0) { dup2(fd, 1); dup2(fd, 2); }
            execl(argv[1], argv[1], (char *)0);
            _exit(127);
        }
        int status;
        waitpid(pid, &status, 0);
    }
    unsigned long long dt = now_ns() - t0;
    printf("%.3f\n", (double)dt / (double)n);
    return 0;
}
