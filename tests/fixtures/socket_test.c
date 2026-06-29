#include "sys/socket.h"
#include "unistd.h"
#include "stdio.h"
#include "stdlib.h"

int main(void) {
    int sv[2];
    if (socketpair(AF_UNIX, SOCK_STREAM, 0, sv) != 0) return 1;

    pid_t pid = fork();
    if (pid < 0) return 2;
    if (pid == 0) {
        close(sv[0]);
        const char *msg = "hello";
        if (send(sv[1], msg, 5, 0) != 5) return 3;
        close(sv[1]);
        _exit(0);
    }

    close(sv[1]);
    char buf[6] = {0};
    if (recv(sv[0], buf, 5, 0) != 5) return 4;
    if (buf[0] != 'h' || buf[4] != 'o') return 5;
    close(sv[0]);

    int status;
    if (wait(&status) != pid) return 6;
    if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) return 7;

    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) return 8;
    if (close(fd) != 0) return 9;

    if (htonl(0x12345678) != 0x78563412) return 10;
    if (htons(0x1234) != 0x3412) return 11;

    puts("socket ok");
    return 0;
}
