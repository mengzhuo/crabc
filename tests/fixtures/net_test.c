#include <arpa/inet.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <sys/socket.h>
#include <sys/wait.h>

#ifndef F_SETLK
#define F_SETLK 6
#define F_GETLK 5
#define F_UNLCK 2
#define F_WRLCK 1
struct flock {
    short l_type;
    short l_whence;
    long l_start;
    long l_len;
    int l_pid;
};
#endif

struct in_addr_local { uint32_t s_addr; };
struct sockaddr_in_local {
    uint16_t sin_family;
    uint16_t sin_port;
    struct in_addr_local sin_addr;
    char pad[8];
};

static int failures = 0;
#define CHECK(expr, msg) do { if (!(expr)) { printf("FAIL: %s\n", msg); failures++; } } while(0)

static void test_inet(void) {
    struct in_addr_local a;
    int r;

    a.s_addr = inet_addr("127.0.0.1");
    CHECK(a.s_addr == htonl(0x7f000001), "inet_addr 127.0.0.1");

    r = inet_aton("10.0.128.31", &a.s_addr);
    CHECK(r == 1, "inet_aton parses");
    CHECK(a.s_addr == htonl(0x0a00801f), "inet_aton value");

    r = inet_pton(AF_INET, "255.255.255.255", &a.s_addr);
    CHECK(r == 1, "inet_pton v4");
    CHECK(a.s_addr == htonl(0xffffffff), "inet_pton v4 value");

    char buf[64];
    uint32_t loopback = htonl(0x7f000001);
    const char *p = inet_ntop(AF_INET, &loopback, buf, sizeof(buf));
    CHECK(p && strcmp(buf, "127.0.0.1") == 0, "inet_ntop v4");

    a.s_addr = htonl(0x0a00801f);
    p = inet_ntoa(a.s_addr);
    CHECK(p && strcmp(p, "10.0.128.31") == 0, "inet_ntoa");
}

static void test_fcntl_lock(void) {
    char path[] = "/tmp/crabc_fcntl_lock_test_XXXXXX";
    int fd = mkstemp(path);
    CHECK(fd >= 0, "mkstemp for lock test");
    if (fd < 0) return;
    write(fd, "x", 1);

    struct flock fl = { .l_type = F_WRLCK, .l_whence = 0, .l_start = 0, .l_len = 1 };
    int r = fcntl(fd, F_SETLK, &fl);
    CHECK(r == 0, "F_SETLK lock");

    pid_t pid = fork();
    if (pid == 0) {
        struct flock fl2 = { .l_type = F_WRLCK, .l_whence = 0, .l_start = 0, .l_len = 1 };
        int rr = fcntl(fd, F_SETLK, &fl2);
        _exit(rr == 0 ? 1 : 0);
    }
    int status;
    waitpid(pid, &status, 0);
    CHECK(WIFEXITED(status) && WEXITSTATUS(status) == 0, "child detected held lock");

    fl.l_type = F_UNLCK;
    fcntl(fd, F_SETLK, &fl);
    close(fd);
    unlink(path);
}

static void test_socket(void) {
    int s = socket(AF_INET, SOCK_STREAM, 0);
    CHECK(s >= 0, "socket create");
    if (s < 0) return;

    int yes = 1;
    setsockopt(s, SOL_SOCKET, SO_REUSEADDR, &yes, sizeof(yes));

    struct sockaddr_in_local sa = {0};
    sa.sin_family = AF_INET;
    sa.sin_addr.s_addr = htonl(0x7f000001);
    int r = bind(s, (struct sockaddr *)&sa, sizeof(sa));
    CHECK(r == 0, "bind loopback");

    r = listen(s, 1);
    CHECK(r == 0, "listen");

    socklen_t len = sizeof(sa);
    r = getsockname(s, (struct sockaddr *)&sa, &len);
    CHECK(r == 0, "getsockname");

    int c = socket(AF_INET, SOCK_STREAM, 0);
    CHECK(c >= 0, "client socket");

    r = connect(c, (struct sockaddr *)&sa, sizeof(sa));
    CHECK(r == 0, "connect");

    int a = accept(s, NULL, NULL);
    CHECK(a >= 0, "accept");

    const char *msg = "hello";
    send(c, msg, 5, 0);
    char rcv[8] = {0};
    ssize_t n = recv(a, rcv, sizeof(rcv), 0);
    CHECK(n == 5 && memcmp(rcv, "hello", 5) == 0, "send/recv");

    close(a);
    close(c);
    close(s);
}

int main(void) {
    test_inet();
    test_fcntl_lock();
    test_socket();
    if (failures == 0) {
        printf("net ok\n");
        return 0;
    }
    printf("net FAIL %d\n", failures);
    return 1;
}
