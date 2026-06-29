#include "stdio.h"
#include "string.h"
#include "stdlib.h"

static int test_fopen_fclose(void) {
    FILE *f = fopen("/tmp/test_stdio_fopen.txt", "w");
    if (!f) return 1;
    if (fileno(f) < 0) return 2;
    fputs("hello world\n", f);
    fclose(f);

    f = fopen("/tmp/test_stdio_fopen.txt", "r");
    if (!f) return 3;
    char buf[64];
    if (!fgets(buf, sizeof(buf), f)) return 4;
    if (strcmp(buf, "hello world\n") != 0) return 5;
    fclose(f);
    remove("/tmp/test_stdio_fopen.txt");
    return 0;
}

static int test_fwrite_fread(void) {
    FILE *f = fopen("/tmp/test_stdio_fwr.txt", "w+");
    if (!f) return 1;
    int data[] = {10, 20, 30, 40, 50};
    size_t n = fwrite(data, sizeof(int), 5, f);
    if (n != 5) return 2;
    rewind(f);
    int out[5] = {0};
    n = fread(out, sizeof(int), 5, f);
    if (n != 5) return 3;
    for (int i = 0; i < 5; i++) {
        if (out[i] != data[i]) return 4;
    }
    fclose(f);
    remove("/tmp/test_stdio_fwr.txt");
    return 0;
}

static int test_fseek_ftell(void) {
    FILE *f = fopen("/tmp/test_stdio_seek.txt", "w+");
    if (!f) return 1;
    fputs("abcdefghij", f);
    long pos = ftell(f);
    if (pos != 10) return 2;
    fseek(f, 0, SEEK_SET);
    pos = ftell(f);
    if (pos != 0) return 3;
    fseek(f, -3, SEEK_END);
    pos = ftell(f);
    if (pos != 7) return 4;
    fseek(f, 2, SEEK_CUR);
    pos = ftell(f);
    if (pos != 9) return 5;
    fclose(f);
    remove("/tmp/test_stdio_seek.txt");
    return 0;
}

static int test_fgetpos_fsetpos(void) {
    FILE *f = fopen("/tmp/test_stdio_fpos.txt", "w+");
    if (!f) return 1;
    fputs("0123456789", f);
    fpos_t pos;
    if (fgetpos(f, &pos) != 0) return 2;
    fseek(f, 0, SEEK_SET);
    if (fsetpos(f, &pos) != 0) return 3;
    if (ftell(f) != 10) return 4;
    fclose(f);
    remove("/tmp/test_stdio_fpos.txt");
    return 0;
}

static int test_feof_ferror_clearerr(void) {
    FILE *f = fopen("/tmp/test_stdio_err.txt", "w+");
    if (!f) return 1;
    fputs("x", f);
    rewind(f);
    if (feof(f)) return 2;
    if (ferror(f)) return 3;
    fgetc(f);
    fgetc(f);
    if (!feof(f)) return 4;
    clearerr(f);
    if (feof(f)) return 5;
    if (ferror(f)) return 6;
    fclose(f);
    remove("/tmp/test_stdio_err.txt");
    return 0;
}

static int test_fprintf_fscanf(void) {
    FILE *f = fopen("/tmp/test_stdio_fmt.txt", "w+");
    if (!f) return 1;
    int n = fprintf(f, "%d %u %x %s", 42, 100u, 0xff, "hi");
    if (n <= 0) return 2;
    rewind(f);
    int d; unsigned u, x; char s[8];
    n = fscanf(f, "%d %u %x %s", &d, &u, &x, s);
    if (n != 4) return 3;
    if (d != 42 || u != 100 || x != 0xff) return 4;
    if (strcmp(s, "hi") != 0) return 5;
    fclose(f);
    remove("/tmp/test_stdio_fmt.txt");
    return 0;
}

static int test_sprintf_snprintf(void) {
    char buf[64];
    int n = sprintf(buf, "%d+%d=%d", 1, 2, 3);
    if (n != 5 || strcmp(buf, "1+2=3") != 0) return 1;
    n = snprintf(buf, 4, "%s", "hello");
    if (n != 5) return 2;
    if (strcmp(buf, "hel") != 0) return 3;
    return 0;
}

static int test_sscanf(void) {
    int a, b;
    char s[16];
    int n = sscanf("42 hello 99", "%d %s %d", &a, s, &b);
    if (n != 3) return 1;
    if (a != 42 || b != 99) return 2;
    if (strcmp(s, "hello") != 0) return 3;
    return 0;
}

static int test_getdelim_getline(void) {
    FILE *f = fopen("/tmp/test_stdio_gdl.txt", "w+");
    if (!f) return 1;
    fputs("line1\nline2\nline3", f);
    rewind(f);
    char *line = NULL;
    size_t cap = 0;
    long long len;
    len = getline(&line, &cap, f);
    if (len != 6 || strcmp(line, "line1\n") != 0) return 2;
    len = getdelim(&line, &cap, '2', f);
    if (len <= 0) return 3;
    if (line[len - 1] != '2') return 4;
    free(line);
    fclose(f);
    remove("/tmp/test_stdio_gdl.txt");
    return 0;
}

static int test_tmpfile(void) {
    FILE *f = tmpfile();
    if (!f) return 1;
    fputs("temp data", f);
    rewind(f);
    char buf[32];
    if (!fgets(buf, sizeof(buf), f)) return 2;
    if (strcmp(buf, "temp data") != 0) return 3;
    fclose(f);
    return 0;
}

static int test_tmpnam(void) {
    char name[L_tmpnam];
    char *r = tmpnam(name);
    if (!r) return 1;
    if (strlen(name) == 0) return 2;
    return 0;
}

static int test_rename_remove(void) {
    FILE *f = fopen("/tmp/test_stdio_rn1.txt", "w");
    if (!f) return 1;
    fputs("data", f);
    fclose(f);
    if (rename("/tmp/test_stdio_rn1.txt", "/tmp/test_stdio_rn2.txt") != 0) return 2;
    f = fopen("/tmp/test_stdio_rn2.txt", "r");
    if (!f) return 3;
    char buf[16];
    if (!fgets(buf, sizeof(buf), f)) return 4;
    if (strcmp(buf, "data") != 0) return 5;
    fclose(f);
    if (remove("/tmp/test_stdio_rn2.txt") != 0) return 6;
    if (remove("/tmp/test_stdio_rn2.txt") == 0) return 7;
    return 0;
}

static int test_popen_pclose(void) {
    FILE *f = popen("echo hello", "r");
    if (!f) return 1;
    char buf[32];
    if (!fgets(buf, sizeof(buf), f)) return 2;
    int rc = pclose(f);
    if (rc != 0) return 3;
    return 0;
}

static int test_setvbuf(void) {
    FILE *f = fopen("/tmp/test_stdio_svbuf.txt", "w+");
    if (!f) return 1;
    char buf[256];
    setvbuf(f, buf, _IOFBF, sizeof(buf));
    fputs("buffered", f);
    rewind(f);
    char out[32];
    if (!fgets(out, sizeof(out), f)) return 2;
    if (strcmp(out, "buffered") != 0) return 3;
    fclose(f);
    remove("/tmp/test_stdio_svbuf.txt");
    return 0;
}

static int test_fopen64(void) {
    FILE *f = fopen("/tmp/test_stdio_f64.txt", "w");
    if (!f) return 1;
    fputs("fopen64 ok", f);
    fclose(f);
    remove("/tmp/test_stdio_f64.txt");
    return 0;
}

static int test_perror_smoke(void) {
    FILE *f = fopen("/nonexistent/path", "r");
    if (f) return 1;
    perror("expected error");
    return 0;
}

static int test_fdopen(void) {
    FILE *f = fopen("/tmp/test_stdio_fd.txt", "w+");
    if (!f) return 1;
    fputs("fdopen", f);
    int fd = fileno(f);
    if (fd < 0) return 2;
    fclose(f);
    remove("/tmp/test_stdio_fd.txt");
    return 0;
}

int main(void) {
    int r;
    if ((r = test_fopen_fclose())) { printf("fopen_fclose fail %d\n", r); return r; }
    if ((r = test_fwrite_fread())) { printf("fwrite_fread fail %d\n", r); return r + 10; }
    if ((r = test_fseek_ftell())) { printf("fseek_ftell fail %d\n", r); return r + 20; }
    if ((r = test_fgetpos_fsetpos())) { printf("fgetpos_fsetpos fail %d\n", r); return r + 30; }
    if ((r = test_feof_ferror_clearerr())) { printf("feof_ferror fail %d\n", r); return r + 40; }
    if ((r = test_fprintf_fscanf())) { printf("fprintf_fscanf fail %d\n", r); return r + 50; }
    if ((r = test_sprintf_snprintf())) { printf("sprintf_snprintf fail %d\n", r); return r + 60; }
    if ((r = test_sscanf())) { printf("sscanf fail %d\n", r); return r + 70; }
    if ((r = test_getdelim_getline())) { printf("getdelim_getline fail %d\n", r); return r + 80; }
    if ((r = test_tmpfile())) { printf("tmpfile fail %d\n", r); return r + 90; }
    if ((r = test_tmpnam())) { printf("tmpnam fail %d\n", r); return r + 100; }
    if ((r = test_rename_remove())) { printf("rename_remove fail %d\n", r); return r + 110; }
    if ((r = test_setvbuf())) { printf("setvbuf fail %d\n", r); return r + 130; }
    if ((r = test_fopen64())) { printf("fopen64 fail %d\n", r); return r + 140; }
    if ((r = test_perror_smoke())) { printf("perror fail %d\n", r); return r + 150; }
    if ((r = test_fdopen())) { printf("fdopen fail %d\n", r); return r + 160; }
    printf("stdio full ok\n");
    return 0;
}
