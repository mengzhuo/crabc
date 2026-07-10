#include <unistd.h>
#include <sys/wait.h>

extern int daemon(int, int);
extern int fork(void);
extern int waitpid(int, int *, int);
extern int getpid(void);
extern int getsid(int);
extern void _exit(int);
extern int setsid(void);

int main(void) {
    int status;
    int child = fork();
    if (child < 0) {
        const char msg[] = "FAIL: fork\n";
        write(1, msg, sizeof(msg) - 1);
        return 1;
    }

    if (child == 0) {
        int old_pid = getpid();
        int old_sid = getsid(0);
        int r = daemon(1, 1);
        if (r != 0) {
            const char msg[] = "FAIL: daemon returned\n";
            write(1, msg, sizeof(msg) - 1);
            _exit(1);
        }
        if (getpid() == old_pid) {
            const char msg[] = "FAIL: pid unchanged\n";
            write(1, msg, sizeof(msg) - 1);
            _exit(1);
        }
        int new_sid = getsid(0);
        if (new_sid == old_sid) {
            const char msg[] = "FAIL: sid unchanged\n";
            write(1, msg, sizeof(msg) - 1);
            _exit(1);
        }
        const char msg[] = "OK\n";
        write(1, msg, sizeof(msg) - 1);
        _exit(0);
    }

    int r = waitpid(child, &status, 0);
    if (r < 0) {
        const char msg[] = "FAIL: waitpid\n";
        write(1, msg, sizeof(msg) - 1);
        return 1;
    }
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
        const char msg[] = "FAIL: child exited\n";
        write(1, msg, sizeof(msg) - 1);
        return 1;
    }
    return 0;
}
