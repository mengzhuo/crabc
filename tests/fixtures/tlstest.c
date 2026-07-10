__thread int x = 42;

#if defined(__x86_64__)

static void my_write(int fd, const void *buf, unsigned long count) {
    __asm__ volatile (
        "syscall"
        :
        : "a"(1), "D"(fd), "S"(buf), "d"(count)
        : "rcx", "r11", "memory"
    );
}

static void my_exit(int code) {
    __asm__ volatile (
        "syscall"
        :
        : "a"(60), "D"(code)
        : "memory"
    );
    __builtin_unreachable();
}

#elif defined(__aarch64__)

static void my_write(int fd, const void *buf, unsigned long count) {
    register long x8 __asm__("x8") = 64;
    register long x0 __asm__("x0") = fd;
    register long x1 __asm__("x1") = (long)buf;
    register long x2 __asm__("x2") = count;
    __asm__ volatile (
        "svc #0"
        :
        : "r"(x8), "r"(x0), "r"(x1), "r"(x2)
        : "memory"
    );
}

static void my_exit(int code) {
    register long x8 __asm__("x8") = 93;
    register long x0 __asm__("x0") = code;
    __asm__ volatile (
        "svc #0"
        :
        : "r"(x8), "r"(x0)
        : "memory"
    );
    __builtin_unreachable();
}

#else
#error "unsupported architecture"
#endif

void _start(void) {
    if (x == 42) {
        my_write(1, "ok\n", 3);
    }
    my_exit(0);
}
