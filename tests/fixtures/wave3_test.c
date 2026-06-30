#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

static int fail = 0;

static void check(int cond, const char *msg) {
    if (!cond) {
        fprintf(stderr, "FAIL: %s\n", msg);
        fail = 1;
    }
}

int main(void) {
    {
        char buf[100];
        unsigned char addr[16];
        const char *in = "::ffff:192.168.0.1";
        check(inet_pton(AF_INET6, in, addr) == 1, "inet_pton v4mapped");
        check(inet_ntop(AF_INET6, addr, buf, sizeof(buf)) != NULL, "inet_ntop");
        check(strchr(buf, '.') != NULL, "inet_ntop v4mapped contains '.'");
        check(strcmp(buf, "::ffff:192.168.0.1") == 0, "inet_ntop v4mapped exact");
    }

    {
        char buf[] = { 0 };
        int r = sscanf(buf, "a");
        check(r == EOF, "sscanf EOF literal match");
    }

    {
        void *p1 = malloc(0);
        void *p2 = malloc(0);
        check(p1 != NULL, "malloc(0) non-NULL");
        check(p2 != NULL, "malloc(0) second non-NULL");
        check(p1 != p2, "malloc(0) unique");
        free(p1);
        free(p2);
    }

    {
        char tmp[] = "/tmp/wave3-ftello-XXXXXX";
        int fd = mkstemp(tmp);
        check(fd > 2, "mkstemp");
        check(write(fd, "abcd", 4) == 4, "write abcd");
        check(close(fd) == 0, "close tmp");

        fd = open(tmp, O_WRONLY);
        check(fd > 2, "reopen O_WRONLY");
        FILE *f = fdopen(fd, "a");
        check(f != NULL, "fdopen a");
        check(fwrite("efg", 1, 3, f) == 3, "fwrite efg");
        off_t off = ftello(f);
        check(off == 7, "ftello before flush");
        check(fflush(f) == 0, "fflush");
        off = ftello(f);
        check(off == 7, "ftello after flush");
        check(fclose(f) == 0, "fclose");
        check(unlink(tmp) == 0, "unlink tmp");
    }

    if (fail) return 1;
    printf("wave3 ok\n");
    return 0;
}
