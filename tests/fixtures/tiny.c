// Tiny PIE program: raw syscalls, no libc dependency.
// Prints "Hello\n" via SYS_write, exits 0 via SYS_exit.

static void my_write(int fd, const void *buf, unsigned long count) {
    // SYS_write = 1 on x86_64
    __asm__ volatile (
        "syscall"
        :
        : "a"(1), "D"(fd), "S"(buf), "d"(count)
        : "rcx", "r11", "memory"
    );
}

static void my_exit(int code) {
    // SYS_exit = 60 on x86_64
    __asm__ volatile (
        "syscall"
        :
        : "a"(60), "D"(code)
        : "memory"
    );
    __builtin_unreachable();
}

void _start(void) {
    my_write(1, "Hello\n", 6);
    my_exit(0);
}
