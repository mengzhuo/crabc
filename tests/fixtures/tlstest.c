__thread int x = 42;

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

void _start(void) {
    if (x == 42) {
        my_write(1, "ok\n", 3);
    }
    my_exit(0);
}
