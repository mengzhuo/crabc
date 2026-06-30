static int my_strlen(const char *s) {
    int n = 0;
    while (s[n]) n++;
    return n;
}

static void my_write(int fd, const void *buf, unsigned long count) {
    __asm__ volatile (
        "syscall"
        :
        : "a"(1), "D"(fd), "S"(buf), "d"(count)
        : "rcx", "r11", "memory"
    );
}

static void my_puts(const char *s) {
    my_write(1, s, my_strlen(s));
}

static void my_putn(long n) {
    char buf[20];
    int i = 0;
    if (n == 0) { my_write(1, "0", 1); return; }
    if (n < 0) { my_write(1, "-", 1); n = -n; }
    while (n > 0) { buf[i++] = '0' + (n % 10); n /= 10; }
    while (i > 0) { my_write(1, &buf[--i], 1); }
}

extern int daemon(int, int);
extern int fork(void);
extern int waitpid(int, int *, int);
extern int getpid(void);
extern int getsid(int);
extern void _exit(int);
extern int setsid(void);

static int my_waitpid(int pid, int *status, int options) {
    /* ponytail: inline waitpid via syscall (rax=61, rdi=pid, rsi=status, rdx=options, r10=0) */
    long ret;
    register long r10 __asm__("r10") = 0;
    __asm__ volatile (
        "syscall"
        : "=a"(ret)
        : "a"(61l), "D"((long)pid), "S"(status), "d"((long)options), "r"(r10)
        : "rcx", "r11", "memory"
    );
    return (int)ret;
}

int main(void) {
    int status;
    int child = fork();
    if (child < 0) {
        my_puts("FAIL: fork\n");
        return 1;
    }

    if (child == 0) {
        int old_pid = getpid();
        int old_sid = getsid(0);
        int r = daemon(1, 1);
        if (r != 0) {
            my_puts("FAIL: daemon returned ");
            my_putn(r);
            my_puts("\n");
            _exit(1);
        }
        if (getpid() == old_pid) {
            my_puts("FAIL: pid unchanged\n");
            _exit(1);
        }
        int new_sid = getsid(0);
        if (new_sid == old_sid) {
            my_puts("FAIL: sid unchanged\n");
            _exit(1);
        }
        my_puts("OK\n");
        _exit(0);
    }

    int r = my_waitpid(child, &status, 0);
    if (r < 0) {
        my_puts("FAIL: waitpid\n");
        return 1;
    }
    if (status != 0) {
        my_puts("FAIL: child exited ");
        my_putn(status);
        my_puts("\n");
        return 1;
    }
    return 0;
}
