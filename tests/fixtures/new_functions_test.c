#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <setjmp.h>
#include <signal.h>
#include <search.h>
#include <fnmatch.h>
#include <mntent.h>
#include <math.h>
#include <sys/stat.h>
#include <sys/resource.h>
#include <unistd.h>
#include <errno.h>
#include <time.h>

static int cmp(const void *a, const void *b) {
    return strcmp((const char *)a, (const char *)b);
}

static void test_hsearch(void) {
    if (!hcreate(16)) { puts("hcreate fail"); exit(1); }
    ENTRY e, *ep;
    e.key = "hello"; e.data = (void*)42;
    ep = hsearch(e, ENTER);
    if (!ep || strcmp(ep->key, "hello") != 0 || (long)ep->data != 42) {
        puts("hsearch ENTER fail"); exit(1);
    }
    ep = hsearch((ENTRY){"hello", 0}, FIND);
    if (!ep || (long)ep->data != 42) { puts("hsearch FIND fail"); exit(1); }
    ep = hsearch((ENTRY){"nokey", 0}, FIND);
    if (ep) { puts("hsearch FIND miss fail"); exit(1); }
    hdestroy();
}

static void test_insque(void) {
    struct q { struct q *next, *prev; int val; };
    struct q a = {0,0,1}, b = {0,0,2}, c = {0,0,3};
    insque(&a, 0);
    insque(&b, &a);
    insque(&c, &a);
    if (a.next != &c || c.prev != &a || c.next != &b || b.prev != &c) {
        puts("insque fail"); exit(1);
    }
    remque(&c);
    if (a.next != &b || b.prev != &a) { puts("remque fail"); exit(1); }
}

static int icmp(const void *a, const void *b) {
    return *(const int*)a - *(const int*)b;
}

static void test_lsearch(void) {
    int arr[16] = {1,2,3};
    size_t n = 3;
    int key = 2;
    int *found = lfind(&key, arr, &n, sizeof(int), icmp);
    if (!found || *found != 2) { puts("lfind fail"); exit(1); }
    key = 5;
    found = lsearch(&key, arr, &n, sizeof(int), icmp);
    if (!found || *found != 5 || n != 4) { puts("lsearch fail"); exit(1); }
    key = 5;
    found = lfind(&key, arr, &n, sizeof(int), icmp);
    if (!found || *found != 5) { puts("lfind after lsearch fail"); exit(1); }
}

static int tree_keys_visited = 0;
static void tree_action(const void *node, VISIT which, int depth) {
    (void)node; (void)depth;
    if (which == leaf || which == postorder) tree_keys_visited++;
}

static void test_tsearch(void) {
    void *root = 0;
    const char *keys[] = {"delta", "alpha", "gamma", "beta"};
    for (int i = 0; i < 4; i++) {
        void *n = tsearch(keys[i], &root, cmp);
        if (!n) { puts("tsearch fail"); exit(1); }
    }
    // find existing
    void *f = tfind("gamma", &root, cmp);
    if (!f) { puts("tfind fail"); exit(1); }
    // find missing
    f = tfind("omega", &root, cmp);
    if (f) { puts("tfind miss fail"); exit(1); }
    // walk
    tree_keys_visited = 0;
    twalk(root, tree_action);
    if (tree_keys_visited != 4) { puts("twalk fail"); exit(1); }
    // delete
    void *p = tdelete("alpha", &root, cmp);
    (void)p;
    f = tfind("alpha", &root, cmp);
    if (f) { puts("tdelete fail"); exit(1); }
    tree_keys_visited = 0;
    twalk(root, tree_action);
    if (tree_keys_visited != 3) { puts("twalk after tdelete fail"); exit(1); }
    tdestroy(root, 0);
}

static void test_fnmatch(void) {
    if (fnmatch("*.c", "foo.c", 0) != 0) { puts("fnmatch *.c fail"); exit(1); }
    if (fnmatch("*.c", "foo.h", 0) == 0) { puts("fnmatch *.c neg fail"); exit(1); }
    if (fnmatch("f?o", "foo", 0) != 0) { puts("fnmatch ? fail"); exit(1); }
    if (fnmatch("f?o", "fo", 0) == 0) { puts("fnmatch ? short fail"); exit(1); }
    if (fnmatch("[abc]", "b", 0) != 0) { puts("fnmatch [abc] fail"); exit(1); }
    if (fnmatch("[abc]", "d", 0) == 0) { puts("fnmatch [abc] neg fail"); exit(1); }
    if (fnmatch("[a-z]", "m", 0) != 0) { puts("fnmatch [a-z] fail"); exit(1); }
    if (fnmatch("[!a-z]", "M", 0) != 0) { puts("fnmatch [!a-z] fail"); exit(1); }
    if (fnmatch("*.tar.gz", "foo.tar.gz", 0) != 0) { puts("fnmatch multi-star fail"); exit(1); }
    if (fnmatch("a*b*c", "abc", 0) != 0) { puts("fnmatch a*b*c fail"); exit(1); }
    if (fnmatch("a*b*c", "aXbYc", 0) != 0) { puts("fnmatch a*b*c mid fail"); exit(1); }
    if (fnmatch("\\*", "*", 0) != 0) { puts("fnmatch escape fail"); exit(1); }
    if (fnmatch("\\*", "*", FNM_NOESCAPE) == 0) { puts("fnmatch noescape fail"); exit(1); }
    if (fnmatch("*.c", ".c", FNM_PERIOD) == 0) { puts("fnmatch period fail"); exit(1); }
    if (fnmatch(".*", ".c", FNM_PERIOD) != 0) { puts("fnmatch .* period fail"); exit(1); }
    if (fnmatch("foo/bar", "foo/bar", FNM_PATHNAME) != 0) { puts("fnmatch pathname literal fail"); exit(1); }
    if (fnmatch("*/bar", "foo/bar", FNM_PATHNAME) != 0) { puts("fnmatch pathname star fail"); exit(1); }
    if (fnmatch("f*o", "foo", 0) != 0) { puts("fnmatch star mid fail"); exit(1); }
    if (fnmatch("f*o", "foo/bar", 0) == 0) { puts("fnmatch star tail mismatch fail"); exit(1); }
    // casefold
    if (fnmatch("*.C", "foo.c", FNM_CASEFOLD) != 0) { puts("fnmatch casefold fail"); exit(1); }
    // bracket with casefold
    if (fnmatch("[A-Z]", "a", FNM_CASEFOLD) != 0) { puts("fnmatch bracket casefold fail"); exit(1); }
}

static void test_mntent(void) {
    // Test with a fake /proc/mounts via fmemopen
    char fstab[] = "/dev/sda1 / ext4 rw,relatime 0 1\n"
                   "//server/share /mnt cifs ro,user 0 0\n";
    FILE *f = fmemopen(fstab, sizeof fstab - 1, "r");
    if (!f) { puts("fmemopen fail"); exit(1); }
    struct mntent *m;
    m = getmntent(f);
    if (!m) { puts("getmntent 1 fail"); exit(1); }
    if (strcmp(m->mnt_fsname, "/dev/sda1") != 0) { puts("mnt_fsname 1 fail"); exit(1); }
    if (strcmp(m->mnt_dir, "/") != 0) { puts("mnt_dir 1 fail"); exit(1); }
    if (strcmp(m->mnt_type, "ext4") != 0) { puts("mnt_type 1 fail"); exit(1); }
    if (strcmp(m->mnt_opts, "rw,relatime") != 0) { puts("mnt_opts 1 fail"); exit(1); }
    if (m->mnt_freq != 0 || m->mnt_passno != 1) { puts("mnt freq/passno 1 fail"); exit(1); }
    if (!hasmntopt(m, "rw")) { puts("hasmntopt rw fail"); exit(1); }
    if (!hasmntopt(m, "relatime")) { puts("hasmntopt relatime fail"); exit(1); }
    if (hasmntopt(m, "ro")) { puts("hasmntopt ro should fail"); exit(1); }
    m = getmntent(f);
    if (!m) { puts("getmntent 2 fail"); exit(1); }
    if (strcmp(m->mnt_dir, "/mnt") != 0) { puts("mnt_dir 2 fail"); exit(1); }
    if (!hasmntopt(m, "ro")) { puts("hasmntopt ro fail"); exit(1); }
    if (!hasmntopt(m, "user")) { puts("hasmntopt user fail"); exit(1); }
    m = getmntent(f);
    if (m) { puts("getmntent eof fail"); exit(1); }
    endmntent(f);

    // test getmntent_r
    char fstab2[] = "/dev/sdb1 /data xfs defaults 0 2\n";
    f = fmemopen(fstab2, sizeof fstab2 - 1, "r");
    if (!f) { puts("fmemopen 2 fail"); exit(1); }
    struct mntent mnt;
    char linebuf[256];
    m = getmntent_r(f, &mnt, linebuf, sizeof linebuf);
    if (!m) { puts("getmntent_r fail"); exit(1); }
    if (strcmp(mnt.mnt_type, "xfs") != 0) { puts("getmntent_r type fail"); exit(1); }
    endmntent(f);
}

static void test_strlcpy_strlcat(void) {
    char buf[32];
    size_t r;
    r = strlcpy(buf, "hello", sizeof(buf));
    if (r != 5 || strcmp(buf, "hello") != 0) { puts("strlcpy fail"); exit(1); }
    r = strlcpy(buf, "longer string that exceeds", 8);
    if (r != 26 || strcmp(buf, "longer ") != 0) { puts("strlcpy trunc fail"); exit(1); }
    r = strlcpy(buf, "", sizeof(buf));
    if (r != 0 || buf[0] != 0) { puts("strlcpy empty fail"); exit(1); }
    strcpy(buf, "hi");
    r = strlcat(buf, " there", sizeof(buf));
    if (r != 8 || strcmp(buf, "hi there") != 0) { puts("strlcat fail"); exit(1); }
    strcpy(buf, "abc");
    r = strlcat(buf, "def", 5);
    if (r != 6 || strcmp(buf, "abcd") != 0) { puts("strlcat trunc fail"); exit(1); }
}

static void test_memmem(void) {
    char haystack[] = "hello world foo bar";
    char *r;
    r = (char*)memmem(haystack, strlen(haystack), "world", 5);
    if (!r || strcmp(r, "world foo bar") != 0) { puts("memmem fail"); exit(1); }
    r = (char*)memmem(haystack, strlen(haystack), "xyz", 3);
    if (r) { puts("memmem miss fail"); exit(1); }
    r = (char*)memmem(haystack, strlen(haystack), "", 0);
    if (r != haystack) { puts("memmem empty needle fail"); exit(1); }
    r = (char*)memmem(haystack, 3, "hello", 5);
    if (r) { puts("memmem short haystack fail"); exit(1); }
}

static void test_random_funcs(void) {
    srandom(42);
    long a = random();
    long b = random();
    if (a == b) { puts("random same consec fail"); exit(1); }
    srandom(42);
    long c = random();
    if (a != c) { puts("random deterministic fail"); exit(1); }
    char state[256];
    char *old = initstate(123, state, sizeof(state));
    if (!old) { puts("initstate fail"); exit(1); }
    long d = random();
    (void)d;
    char *old2 = setstate(old);
    if (!old2) { puts("setstate fail"); exit(1); }
}

static void test_stat_funcs(void) {
    struct stat st;
    if (stat(".", &st) != 0) { puts("stat . fail"); exit(1); }
    if (!S_ISDIR(st.st_mode)) { puts("stat S_ISDIR fail"); exit(1); }
    if (st.st_nlink == 0) { puts("stat nlink fail"); exit(1); }
    if (stat("/dev/null", &st) != 0) { puts("stat /dev/null fail"); exit(1); }
    if (!S_ISCHR(st.st_mode)) { puts("stat S_ISCHR fail"); exit(1); }
    FILE *f = tmpfile();
    if (f) {
        fputs("test", f);
        fflush(f);
        if (fstat(fileno(f), &st) != 0) { puts("fstat fail"); exit(1); }
        if (st.st_size != 4) { puts("fstat size fail"); exit(1); }
        fclose(f);
    }
}

static void test_utimensat_futimens(void) {
    char tmp[] = "/tmp/utime_test_XXXXXX";
    int fd = mkstemp(tmp);
    if (fd < 0) { puts("mkstemp fail"); exit(1); }
    write(fd, "x", 1);
    struct timespec ts[2] = {{1000000, 0}, {2000000, 0}};
    if (futimens(fd, ts) != 0) { puts("futimens fail"); exit(1); }
    struct stat st;
    if (fstat(fd, &st) != 0) { puts("fstat after futimens fail"); exit(1); }
    if (st.st_atim.tv_sec != 1000000) { puts("futimens atime fail"); exit(1); }
    if (st.st_mtim.tv_sec != 2000000) { puts("futimens mtime fail"); exit(1); }
    close(fd);
    unlink(tmp);
}

static void test_rlimit(void) {
    struct rlimit rl;
    if (getrlimit(RLIMIT_NOFILE, &rl) != 0) { puts("getrlimit fail"); exit(1); }
    if (rl.rlim_cur == 0) { puts("getrlimit rlim_cur=0 fail"); exit(1); }
}

static void test_lrint_funcs(void) {
    long r;
    r = lrint(1.6);
    if (r != 2) { puts("lrint(1.6) fail"); exit(1); }
    r = lrint(-1.6);
    if (r != -2) { puts("lrint(-1.6) fail"); exit(1); }
    r = lrintf(2.5f);
    if (r != 2) { puts("lrintf(2.5f) fail"); exit(1); }
    long long lr;
    lr = llrint(3.7);
    if (lr != 4) { puts("llrint(3.7) fail"); exit(1); }
}

static void test_sigsetjmp(void) {
    sigjmp_buf sjb;
    sigset_t set, oldset;
    sigemptyset(&set);
    sigaddset(&set, SIGUSR1);
    sigprocmask(SIG_UNBLOCK, &set, &oldset);

    if (!sigsetjmp(sjb, 1)) {
        sigemptyset(&set);
        sigaddset(&set, SIGUSR1);
        sigprocmask(SIG_BLOCK, &set, 0);
        siglongjmp(sjb, 1);
    }
    sigprocmask(SIG_SETMASK, &oldset, &set);
    if (sigismember(&set, SIGUSR1)) {
        puts("sigsetjmp savemask=1: mask not restored");
        exit(1);
    }

    if (!sigsetjmp(sjb, 0)) {
        sigemptyset(&set);
        sigaddset(&set, SIGUSR1);
        sigprocmask(SIG_BLOCK, &set, 0);
        siglongjmp(sjb, 1);
    }
    sigprocmask(SIG_SETMASK, &oldset, &set);
    if (!sigismember(&set, SIGUSR1)) {
        puts("sigsetjmp savemask=0: mask incorrectly restored");
        exit(1);
    }
    sigprocmask(SIG_SETMASK, &oldset, 0);
}

int main(void) {
    test_hsearch();
    test_insque();
    test_lsearch();
    test_tsearch();
    test_fnmatch();
    test_mntent();
    test_strlcpy_strlcat();
    test_memmem();
    test_random_funcs();
    test_stat_funcs();
    test_utimensat_futimens();
    test_rlimit();
    test_lrint_funcs();
    test_sigsetjmp();
    puts("new_functions ok");
    return 0;
}
