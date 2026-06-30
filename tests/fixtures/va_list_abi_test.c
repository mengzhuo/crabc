#include <stdarg.h>
#include <stdio.h>
#include <string.h>

static int my_vsnprintf(char *buf, size_t n, const char *fmt, va_list ap)
{
    return vsnprintf(buf, n, fmt, ap);
}

static int my_snprintf(char *buf, size_t n, const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    int r = my_vsnprintf(buf, n, fmt, ap);
    va_end(ap);
    return r;
}

int main(void)
{
    char buf[256];
    int r;

    r = my_snprintf(buf, sizeof(buf), "hello %s %d", "world", 42);
    if (r < 0 || strcmp(buf, "hello world 42") != 0) {
        printf("FAIL snprintf-style va_list: got '%s' r=%d\n", buf, r);
        return 1;
    }

    r = my_snprintf(buf, sizeof(buf), "%c %lx %p", 'Z', 0xdeadbeefUL, (void *)0x1234);
    if (r < 0) {
        printf("FAIL mixed va_list: r=%d\n", r);
        return 2;
    }

    char out[16];
    r = my_snprintf(out, sizeof(out), "this is a very long string indeed");
    if (r < 0 || (size_t)r <= sizeof(out)) {
        printf("FAIL truncation: r=%d\n", r);
        return 3;
    }

    printf("OK\n");
    return 0;
}
