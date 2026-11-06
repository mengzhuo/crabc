#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

static int fails;
#define CHECK(c, ...) do { if (!(c)) { printf(__VA_ARGS__); puts(""); fails++; } } while (0)

static long PG;

static unsigned char *map_guarded(int pages) {
    unsigned char *m = mmap(0, (size_t)(pages + 1) * PG, PROT_READ | PROT_WRITE,
                            MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (m == MAP_FAILED) { perror("mmap"); exit(2); }
    mprotect(m + (size_t)pages * PG, PG, PROT_NONE);
    return m;
}

static void test_strlen(void) {
    int pages = 8;
    unsigned char *m = map_guarded(pages);
    unsigned char *end = m + (size_t)pages * PG;
    size_t lens[] = {0, 1, 2, 7, 8, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 1024, 4095};
    for (unsigned i = 0; i < sizeof lens / sizeof *lens; i++) {
        size_t n = lens[i];
        for (int align = 0; align < 40; align++) {
            unsigned char *s = end - n - 1 - align;
            memset(s, 'x', n);
            s[n] = 0;
            CHECK(strlen((char *)s) == n, "strlen n=%zu align=%d got=%zu", n, align, strlen((char *)s));
        }
    }
    munmap(m, (size_t)(pages + 1) * PG);
}

static void test_memcpy(void) {
    int pages = 8;
    unsigned char *m = map_guarded(pages);
    unsigned char *end = m + (size_t)pages * PG;
    unsigned char *ref = malloc(8192);
    for (size_t n = 0; n <= 300; n++) {
        for (int off = 0; off < 40; off++) {
            unsigned char *dst = end - n - 1 - off;
            unsigned char *src = m + 64 + off;
            for (size_t k = 0; k < n; k++) src[k] = (unsigned char)(k * 7 + off);
            memcpy(ref, src, n);
            memset(dst, 0xcc, n);
            memcpy(dst, src, n);
            CHECK(memcmp(dst, ref, n) == 0, "memcpy n=%zu off=%d", n, off);
        }
    }
    for (size_t n = 1; n <= 8192; n = n * 2 + 1) {
        unsigned char *a = m, *b = m + 4096;
        for (size_t k = 0; k < n; k++) a[k] = (unsigned char)k;
        memcpy(b, a, n);
        CHECK(memcmp(a, b, n) == 0, "memcpy large n=%zu", n);
    }
    free(ref);
    munmap(m, (size_t)(pages + 1) * PG);
}

static void test_memset(void) {
    int pages = 8;
    unsigned char *m = map_guarded(pages);
    unsigned char *end = m + (size_t)pages * PG;
    for (size_t n = 0; n <= 300; n++) {
        for (int off = 0; off < 40; off++) {
            unsigned char *p = end - n - 1 - off;
            memset(p, 0x5a, n);
            for (size_t k = 0; k < n; k++) {
                if (p[k] != 0x5a) {
                    printf("memset n=%zu off=%d k=%zu\n", n, off, k);
                    fails++;
                    break;
                }
            }
        }
    }
    munmap(m, (size_t)(pages + 1) * PG);
}

static void test_memmove(void) {
    size_t N = 512;
    unsigned char *buf = malloc(N * 2);
    unsigned char *exp = malloc(N * 2);
    for (int delta = -64; delta <= 64; delta++) {
        for (size_t n = 0; n <= 200; n++) {
            for (size_t k = 0; k < N * 2; k++) buf[k] = (unsigned char)(k * 13 + 1);
            memcpy(exp, buf, N * 2);
            size_t so = 200, dof = (size_t)((long)so + delta);
            if (dof > so && dof < so + n) {
                for (size_t i = n; i > 0; i--) exp[dof + i - 1] = exp[so + i - 1];
            } else {
                for (size_t i = 0; i < n; i++) exp[dof + i] = exp[so + i];
            }
            memmove(buf + dof, buf + so, n);
            CHECK(memcmp(buf, exp, N * 2) == 0, "memmove delta=%d n=%zu", delta, n);
        }
    }
    free(buf);
    free(exp);
}

static void test_memcmp(void) {
    unsigned char a[305], b[305];
    for (size_t n = 0; n <= 305; n++) {
        for (size_t k = 0; k < n; k++) { a[k] = (unsigned char)k; b[k] = (unsigned char)k; }
        CHECK(memcmp(a, b, n) == 0, "memcmp eq n=%zu", n);
        for (size_t pos = 0; pos < n; pos += 7) {
            b[pos] ^= 0x40;
            int r = memcmp(a, b, n);
            CHECK(r != 0 && ((r < 0) == (a[pos] < b[pos])), "memcmp diff n=%zu pos=%zu", n, pos);
            b[pos] ^= 0x40;
        }
    }
}

static void test_strcmp(void) {
    char a[64], b[64];
    for (int n = 0; n < 32; n++) {
        memset(a, 'a', n); memset(b, 'a', n);
        a[n] = b[n] = 0;
        CHECK(strcmp(a, b) == 0, "strcmp eq n=%d", n);
        for (int pos = 0; pos < n; pos++) {
            a[pos] = 'b';
            CHECK(strcmp(a, b) > 0, "strcmp diff n=%d pos=%d", n, pos);
            a[pos] = 'a';
        }
    }
}

static void test_strcmp_page(void) {
    int pages = 8;
    unsigned char *m = map_guarded(pages);
    unsigned char *end = m + (size_t)pages * PG;
    for (int align = 0; align < 16; align++) {
        unsigned char *a = end - 21 - align;
        unsigned char *b = a - 32;
        memset(a, 'q', 20);
        a[20] = 0;
        memset(b, 'q', 20);
        b[20] = 0;
        CHECK(strcmp((char *)a, (char *)b) == 0, "strcmp page eq align=%d", align);
        b[17] = 'r';
        CHECK(strcmp((char *)a, (char *)b) != 0, "strcmp page diff align=%d", align);
    }
    munmap(m, (size_t)(pages + 1) * PG);
}

int main(void) {
    PG = sysconf(_SC_PAGESIZE);
    test_strlen();
    test_memcpy();
    test_memset();
    test_memmove();
    test_memcmp();
    test_strcmp();
    test_strcmp_page();
    if (fails) { printf("FAIL %d\n", fails); return 1; }
    puts("string fast ok");
    return 0;
}
