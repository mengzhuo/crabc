#include <unistd.h>

static int my_strlen(const char *s) {
    int n = 0;
    while (s[n]) n++;
    return n;
}

static void my_puts(const char *s) {
    write(1, s, my_strlen(s));
}

static void my_putn(long n) {
    char buf[20];
    int i = 0;
    if (n == 0) { write(1, "0", 1); return; }
    if (n < 0) { write(1, "-", 1); n = -n; }
    while (n > 0) { buf[i++] = '0' + (n % 10); n /= 10; }
    while (i > 0) { write(1, &buf[--i], 1); }
}

extern char *getenv(const char *);

int main(int argc, char **argv) {
    my_puts("argc=");
    my_putn(argc);
    my_puts(" argv0=");
    my_puts(argv[0]);
    my_puts(" env=");
    const char *v = getenv("TEST_STARTUP_VAR");
    my_puts(v ? v : "NONE");
    my_puts("\n");
    return 0;
}
