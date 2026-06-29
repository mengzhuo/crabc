#include "unistd.h"
#include "stdio.h"
#include "stdlib.h"

int main(void) {
    int status = -1;
    pid_t pid = fork();
    if (pid < 0) return 1;
    if (pid == 0) {
        _exit(42);
    }
    pid_t w = waitpid(pid, &status, 0);
    if (w != pid) return 2;
    if (!WIFEXITED(status)) return 3;
    if (WEXITSTATUS(status) != 42) return 4;

    pid = fork();
    if (pid < 0) return 5;
    if (pid == 0) {
        char *const argv[] = {(char*)"/bin/true", NULL};
        char *const envp[] = {NULL};
        execve("/bin/true", argv, envp);
        _exit(99);
    }
    w = wait(&status);
    if (w != pid) return 6;
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) return 7;

    if (getpid() <= 0) return 8;
    if (getppid() <= 0) return 9;
    if (getuid() == (uid_t)-1) return 10;
    if (getgid() == (gid_t)-1) return 11;

    puts("process ok");
    return 0;
}
