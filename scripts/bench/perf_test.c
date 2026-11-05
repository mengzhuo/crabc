#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <math.h>
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <time.h>
#include <unistd.h>

static volatile uint64_t sink;

static uint64_t now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ull + (uint64_t)ts.tv_nsec;
}

static void report(const char *name, uint64_t ns, uint64_t ops) {
    printf("%s\t%.3f\n", name, (double)ns / (double)ops);
    fflush(stdout);
}

static void bench_memcpy(size_t n, uint64_t ops, const char *name) {
    unsigned char *src = malloc(n + 64);
    unsigned char *dst = malloc(n + 64);
    memset(src, 0xA5, n + 64);
    uint64_t t0 = now_ns();
    for (uint64_t i = 0; i < ops; i++) {
        memcpy(dst, src, n);
        sink += dst[i % n];
    }
    uint64_t dt = now_ns() - t0;
    report(name, dt, ops);
    free(src);
    free(dst);
}

static void bench_memset(size_t n, uint64_t ops, const char *name) {
    unsigned char *p = malloc(n);
    uint64_t t0 = now_ns();
    for (uint64_t i = 0; i < ops; i++) {
        memset(p, (int)i, n);
        sink += p[i % n];
    }
    uint64_t dt = now_ns() - t0;
    report(name, dt, ops);
    free(p);
}

static void bench_memcmp(size_t n, uint64_t ops) {
    unsigned char *a = malloc(n);
    unsigned char *b = malloc(n);
    memset(a, 0x5A, n);
    memset(b, 0x5A, n);
    b[n / 2] ^= 1;
    uint64_t t0 = now_ns();
    for (uint64_t i = 0; i < ops; i++) {
        sink += (uint64_t)memcmp(a, b, n);
    }
    uint64_t dt = now_ns() - t0;
    report("memcmp_4k", dt, ops);
    free(a);
    free(b);
}

static void bench_strings(void) {
    char *s = malloc(4097);
    memset(s, 'x', 4096);
    s[4096] = 0;

    uint64_t t0 = now_ns();
    for (uint64_t i = 0; i < 5000000; i++) sink += strlen(s);
    report("strlen_4k", now_ns() - t0, 5000000);

    char a[32], b[32];
    strcpy(a, "the quick brown fox");
    strcpy(b, "the quick brown fix");
    t0 = now_ns();
    for (uint64_t i = 0; i < 20000000; i++) sink += (uint64_t)strcmp(a, b);
    report("strcmp_short", now_ns() - t0, 20000000);

    memcpy(s, "abc needle xyz", 15);
    t0 = now_ns();
    for (uint64_t i = 0; i < 5000000; i++) sink += (uint64_t)(strstr(s, "needle") != 0);
    report("strstr", now_ns() - t0, 5000000);
    free(s);
}

static void bench_format(void) {
    char buf[128];
    uint64_t t0 = now_ns();
    for (uint64_t i = 0; i < 2000000; i++) {
        int n = snprintf(buf, sizeof buf, "%llu %s %.3f", (unsigned long long)i, "hello", 1.234);
        sink += (uint64_t)n;
    }
    report("snprintf", now_ns() - t0, 2000000);

    FILE *f = fopen("/dev/null", "w");
    t0 = now_ns();
    for (uint64_t i = 0; i < 500000; i++) {
        fprintf(f, "%llu %s %.3f\n", (unsigned long long)i, "hello", 1.234);
    }
    fflush(f);
    report("fprintf_devnull", now_ns() - t0, 500000);
    fclose(f);
}

static void bench_malloc(void) {
    uint64_t t0 = now_ns();
    for (uint64_t i = 0; i < 200000; i++) {
        void *p = malloc(16);
        sink += (uintptr_t)p;
        free(p);
    }
    report("malloc_free_16", now_ns() - t0, 200000);

    t0 = now_ns();
    for (uint64_t i = 0; i < 200000; i++) {
        void *p = malloc(256);
        sink += (uintptr_t)p;
        free(p);
    }
    report("malloc_free_256", now_ns() - t0, 200000);

    t0 = now_ns();
    for (uint64_t i = 0; i < 100000; i++) {
        void *p = calloc(1, 1024);
        sink += (uintptr_t)p;
        free(p);
    }
    report("calloc_1k", now_ns() - t0, 100000);

    t0 = now_ns();
    void *p = malloc(64);
    for (uint64_t i = 0; i < 100000; i++) {
        p = realloc(p, 64 + (i & 255));
        sink += (uintptr_t)p;
    }
    report("realloc_grow", now_ns() - t0, 100000);
    free(p);
}

static void bench_math(void) {
    volatile double vx = 0.0;
    uint64_t t0 = now_ns();
    for (uint64_t i = 0; i < 5000000; i++) {
        double x = (double)(int)(i & 1023) / 1024.0 - 0.5;
        vx = sin(x);
        sink += (uint64_t)(vx * 1e9);
    }
    report("sin", now_ns() - t0, 5000000);

    t0 = now_ns();
    for (uint64_t i = 0; i < 5000000; i++) {
        double x = (double)(int)(i & 1023) / 1024.0 - 0.5;
        vx = exp(x);
        sink += (uint64_t)(vx * 1e6);
    }
    report("exp", now_ns() - t0, 5000000);

    t0 = now_ns();
    for (uint64_t i = 0; i < 5000000; i++) {
        double x = 1.0 + (double)(i & 1023) / 1024.0;
        vx = log(x);
        sink += (uint64_t)(vx * 1e9);
    }
    report("log", now_ns() - t0, 5000000);

    t0 = now_ns();
    for (uint64_t i = 0; i < 2000000; i++) {
        double x = 1.0 + (double)(i & 1023) / 1024.0;
        vx = pow(x, 0.5);
        sink += (uint64_t)(vx * 1e9);
    }
    report("pow", now_ns() - t0, 2000000);

    t0 = now_ns();
    for (uint64_t i = 0; i < 10000000; i++) {
        vx = sqrt((double)(i & 0xFFFFF));
        sink += (uint64_t)vx;
    }
    report("sqrt", now_ns() - t0, 10000000);
}

static void bench_syscalls(void) {
    uint64_t t0 = now_ns();
    for (uint64_t i = 0; i < 5000000; i++) sink += (uint64_t)getpid();
    report("getpid", now_ns() - t0, 5000000);

    t0 = now_ns();
    for (uint64_t i = 0; i < 5000000; i++) {
        struct timespec ts;
        clock_gettime(CLOCK_MONOTONIC, &ts);
        sink += (uint64_t)ts.tv_nsec;
    }
    report("clock_gettime", now_ns() - t0, 5000000);

    t0 = now_ns();
    for (uint64_t i = 0; i < 200000; i++) {
        int fd = open("/dev/null", O_RDONLY);
        sink += (uint64_t)fd;
        close(fd);
    }
    report("open_close", now_ns() - t0, 200000);

    struct stat st;
    t0 = now_ns();
    for (uint64_t i = 0; i < 200000; i++) sink += (uint64_t)stat("/dev/null", &st);
    report("stat", now_ns() - t0, 200000);

    t0 = now_ns();
    for (uint64_t i = 0; i < 10000000; i++) sink += (uint64_t)(getenv("PATH") != 0);
    report("getenv", now_ns() - t0, 10000000);
}

static pthread_mutex_t mu = PTHREAD_MUTEX_INITIALIZER;
static volatile int thread_runs;

static void *thread_noop(void *arg) {
    (void)arg;
    __sync_fetch_and_add(&thread_runs, 1);
    return 0;
}

static void bench_threads(void) {
    uint64_t t0 = now_ns();
    for (uint64_t i = 0; i < 10000000; i++) {
        pthread_mutex_lock(&mu);
        pthread_mutex_unlock(&mu);
    }
    report("mutex_lock_unlock", now_ns() - t0, 10000000);

    t0 = now_ns();
    for (uint64_t i = 0; i < 2000; i++) {
        pthread_t th;
        if (pthread_create(&th, 0, thread_noop, 0) == 0) pthread_join(th, 0);
    }
    report("pthread_create_join", now_ns() - t0, 2000);
    printf("thread_runs\t%d\n", thread_runs);
    fflush(stdout);
}

int main(void) {
    bench_memcpy(64, 20000000, "memcpy_64");
    bench_memcpy(4096, 1000000, "memcpy_4k");
    bench_memset(4096, 1000000, "memset_4k");
    bench_memcmp(4096, 1000000);
    bench_strings();
    bench_format();
    bench_malloc();
    bench_math();
    bench_syscalls();
    bench_threads();
    printf("sink\t%llu\n", (unsigned long long)sink);
    return 0;
}
