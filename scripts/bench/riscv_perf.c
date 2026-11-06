/* Reduced benchmark for riscv64 qemu runs: keeps the loops short enough for
 * TCG while still comparing the hot libc paths. Output is picoseconds/op so
 * it does not depend on floating-point formatting. */
#define _GNU_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

static volatile uint64_t sink;

static uint64_t ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ull + ts.tv_nsec;
}

static void report(const char *name, uint64_t dt, uint64_t ops) {
    printf("%s\t%llu\n", name, (unsigned long long)(dt * 1000 / ops));
    fflush(stdout);
}

int main(void) {
    unsigned char *src = malloc(8192), *dst = malloc(8192);
    memset(src, 0xA5, 8192);
    memset(dst, 0x5A, 8192);
    uint64_t t;

    t = ns();
    for (int i = 0; i < 20000; i++) { memcpy(dst, src, 4096); sink += dst[i & 63]; }
    report("memcpy_4k", ns() - t, 20000);

    t = ns();
    for (int i = 0; i < 20000; i++) { memset(dst, i, 4096); sink += dst[i & 63]; }
    report("memset_4k", ns() - t, 20000);

    /* Compare equal data up to the middle, as in the x86 benchmark; a
     * first-byte difference only exercises the scalar early-out. */
    memset(src, 0x5A, 4096);
    memset(dst, 0x5A, 4096);
    dst[2048] ^= 1;
    t = ns();
    for (int i = 0; i < 20000; i++) { sink += (uint64_t)memcmp(src, dst, 4096); }
    report("memcmp_4k", ns() - t, 20000);

    memset(src, 'x', 4096);
    src[4096] = 0;
    t = ns();
    for (int i = 0; i < 50000; i++) { sink += strlen((char *)src); }
    report("strlen_4k", ns() - t, 50000);

    char a[32], b[32];
    strcpy(a, "the quick brown fox");
    strcpy(b, "the quick brown fix");
    t = ns();
    for (int i = 0; i < 200000; i++) { sink += (uint64_t)strcmp(a, b); }
    report("strcmp_short", ns() - t, 200000);

    t = ns();
    for (int i = 0; i < 50000; i++) { void *p = malloc(16); sink += (uintptr_t)p; free(p); }
    report("malloc_free_16", ns() - t, 50000);

    t = ns();
    for (int i = 0; i < 20000; i++) { void *p = calloc(1, 1024); sink += (uintptr_t)p; free(p); }
    report("calloc_1k", ns() - t, 20000);

    t = ns();
    for (int i = 0; i < 200000; i++) {
        struct timespec ts;
        clock_gettime(CLOCK_MONOTONIC, &ts);
        sink += (uint64_t)ts.tv_nsec;
    }
    report("clock_gettime", ns() - t, 200000);

    free(src);
    free(dst);
    return 0;
}
