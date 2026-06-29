#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int cmp_int(const void *a, const void *b) {
    return *(const int *)a - *(const int *)b;
}

static void test_qsort_bsearch(void) {
    int arr[] = {5, 2, 8, 1, 9, 3, 7, 4, 6};
    int n = sizeof(arr)/sizeof(arr[0]);
    qsort(arr, n, sizeof(int), cmp_int);
    for (int i = 0; i < n; i++)
        if (arr[i] != i + 1) { puts("qsort fail"); exit(1); }

    int key = 7;
    int *found = bsearch(&key, arr, n, sizeof(int), cmp_int);
    if (!found || *found != 7) { puts("bsearch fail"); exit(1); }

    key = 10;
    found = bsearch(&key, arr, n, sizeof(int), cmp_int);
    if (found) { puts("bsearch miss fail"); exit(1); }
}

static void test_div(void) {
    div_t d = div(17, 5);
    if (d.quot != 3 || d.rem != 2) { puts("div fail"); exit(1); }

    ldiv_t ld = ldiv(100000L, 7L);
    if (ld.quot != 14285L || ld.rem != 5L) { puts("ldiv fail"); exit(1); }

    lldiv_t lld = lldiv(1000000000000LL, 3LL);
    if (lld.quot != 333333333333LL || lld.rem != 1LL) { puts("lldiv fail"); exit(1); }
}

static int atexit_called = 0;
static void atexit_handler(void) { atexit_called++; }

static void test_atexit(void) {
    if (atexit(atexit_handler) != 0) { puts("atexit register fail"); exit(1); }
}

static void test_env(void) {
    if (setenv("TEST_VAR", "hello", 1) != 0) { puts("setenv fail"); exit(1); }
    char *v = getenv("TEST_VAR");
    if (!v || strcmp(v, "hello") != 0) { puts("getenv fail"); exit(1); }

    if (setenv("TEST_VAR", "world", 0) != 0) { puts("setenv nooverwrite fail"); exit(1); }
    v = getenv("TEST_VAR");
    if (!v || strcmp(v, "hello") != 0) { puts("setenv nooverwrite check fail"); exit(1); }

    if (setenv("TEST_VAR", "world", 1) != 0) { puts("setenv overwrite fail"); exit(1); }
    v = getenv("TEST_VAR");
    if (!v || strcmp(v, "world") != 0) { puts("setenv overwrite check fail"); exit(1); }

    if (unsetenv("TEST_VAR") != 0) { puts("unsetenv fail"); exit(1); }
    if (getenv("TEST_VAR") != NULL) { puts("unsetenv check fail"); exit(1); }

    if (setenv("", "x", 1) != -1) { puts("setenv empty should fail"); exit(1); }
    if (setenv("=bad", "x", 1) != -1) { puts("setenv =bad should fail"); exit(1); }
}

static void test_multibyte(void) {
    /* mblen */
    if (mblen("", 1) != 0) { puts("mblen empty fail"); exit(1); }
    if (mblen("A", 1) != 1) { puts("mblen ASCII fail"); exit(1); }

    /* mbtowc / wctomb */
    int wc;
    int r = mbtowc(&wc, "\xC3\xA9", 2);
    if (r != 2 || wc != 0xE9) { puts("mbtowc fail"); exit(1); }

    char buf[4];
    r = wctomb(buf, 0xE9);
    if (r != 2 || buf[0] != (char)0xC3 || buf[1] != (char)0xA9) { puts("wctomb fail"); exit(1); }

    /* mbstowcs / wcstombs */
    int wcs[16];
    const char *utf8 = "H\xC3\xA9llo";
    r = mbstowcs(wcs, utf8, 16);
    if (r != 5) { puts("mbstowcs fail"); exit(1); }
    if (wcs[0] != 'H' || wcs[1] != 0xE9 || wcs[2] != 'l') { puts("mbstowcs val fail"); exit(1); }

    char mbs[16];
    r = wcstombs(mbs, wcs, 16);
    if (r != 6 || strcmp(mbs, utf8) != 0) { puts("wcstombs fail"); exit(1); }

    /* 3-byte: U+4E00 = 一 */
    r = mbtowc(&wc, "\xE4\xB8\x80", 3);
    if (r != 3 || wc != 0x4E00) { puts("mbtowc 3byte fail"); exit(1); }

    r = wctomb(buf, 0x4E00);
    if (r != 3) { puts("wctomb 3byte fail"); exit(1); }

    /* 4-byte: U+1F600 = 😀 */
    r = mbtowc(&wc, "\xF0\x9F\x98\x80", 4);
    if (r != 4 || wc != 0x1F600) { puts("mbtowc 4byte fail"); exit(1); }

    r = wctomb(buf, 0x1F600);
    if (r != 4) { puts("wctomb 4byte fail"); exit(1); }

    /* invalid sequence */
    r = mbtowc(&wc, "\xFF", 1);
    if (r != -1) { puts("mbtowc invalid should fail"); exit(1); }
}

static void test_system(void) {
    int r = system("exit 42");
    /* system returns status in waitpid format; exit 42 -> WEXITSTATUS = 42 */
    if (((r >> 8) & 0xFF) != 42) { puts("system fail"); exit(1); }
}

int main(void) {
    /* original tests */
    if (atoi("123") != 123) return 1;
    if (atoi("-42") != -42) return 2;
    if (atol("9999999999") != 9999999999L) return 3;

    char *end;
    if (strtol("0x1a", &end, 0) != 26 || *end != 0) return 4;
    if (strtol("077", &end, 0) != 63 || *end != 0) return 5;
    if (strtol("abc", &end, 16) != 0xabc || *end != 0) return 6;
    if (strtol("123abc", &end, 10) != 123 || *end != 'a') return 7;

    if (strtoul("-1", NULL, 10) != (unsigned long)-1) return 8;
    if (strtoul("0xFF", NULL, 0) != 255) return 9;

    if (atoll("-9223372036854775807") != -9223372036854775807LL) return 10;

    if (abs(-7) != 7) return 11;
    if (labs(-7000000000L) != 7000000000L) return 12;
    if (llabs(-9000000000000000000LL) != 9000000000000000000LL) return 13;

    srand(1);
    int r1 = rand();
    int r2 = rand();
    if (r1 == r2) return 14;
    if (r1 < 0 || r1 > RAND_MAX) return 15;
    srand(1);
    if (rand() != r1) return 16;

    /* new tests */
    test_qsort_bsearch();
    test_div();
    test_atexit();
    test_env();
    test_multibyte();
    test_system();

    puts("stdlib ok");
    return 0;
}
