#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdlib.h>
#include <errno.h>
#include <pthread.h>
#include <sys/statfs.h>
#include <sys/sysinfo.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <sys/time.h>
#include <sys/cachectl.h>
#include <resolv.h>
#include <utmp.h>
#include <netdb.h>
#include <pwd.h>
#include <grp.h>
#include <glob.h>
#include <ftw.h>
#include <wordexp.h>
#include <monetary.h>
#include <wchar.h>
#include <time.h>
#include <dlfcn.h>
#include <crypt.h>
#include <limits.h>
#include <aio.h>
#include <getopt.h>

struct memcookie { char data[64]; size_t len, pos; };
static ssize_t mc_read(void *c, char *b, size_t n) {
    struct memcookie *m = c;
    size_t left = m->len - m->pos;
    if (n > left) n = left;
    memcpy(b, m->data + m->pos, n);
    m->pos += n;
    return n;
}
static ssize_t mc_write(void *c, const char *b, size_t n) {
    struct memcookie *m = c;
    if (m->pos + n > sizeof(m->data)) return -1;
    memcpy(m->data + m->pos, b, n);
    m->pos += n;
    if (m->pos > m->len) m->len = m->pos;
    return n;
}
static int mc_seek(void *c, off_t *o, int w) {
    struct memcookie *m = c;
    off_t base = w == SEEK_SET ? 0 : w == SEEK_CUR ? (off_t)m->pos : (off_t)m->len;
    if (base + *o < 0) return -1;
    m->pos = base + *o;
    *o = m->pos;
    return 0;
}
static int mc_close(void *c) { (void)c; return 0; }

static int nftw_visits;
static int nftw_cb(const char *path, const struct stat *st, int type, struct FTW *ftw) {
    (void)path; (void)st; (void)type; (void)ftw;
    nftw_visits++;
    return 0;
}

static int phdr_objects;
static int phdr_cb(struct dl_phdr_info *info, size_t sz, void *data) {
    (void)info; (void)sz; (void)data;
    phdr_objects++;
    return 0;
}

static void *worker(void *arg) {
    (void)arg;
    return (void *)42;
}

static int failures;

#define CHECK(cond, msg) do { if (!(cond)) { printf("FAIL: %s\n", msg); failures++; } } while (0)

int main(void) {
    struct statfs s;
    CHECK(statfs("/", &s) == 0, "statfs");
    int dfd = open("/", O_RDONLY | O_DIRECTORY);
    CHECK(dfd >= 0, "open /");
    if (dfd >= 0) {
        CHECK(fstatfs(dfd, &s) == 0, "fstatfs");
        char dbuf[1024];
        CHECK(posix_getdents(dfd, dbuf, sizeof(dbuf), 0) > 0, "posix_getdents");
        close(dfd);
    }

    struct sysinfo si;
    CHECK(sysinfo(&si) == 0, "sysinfo");

    char *p = mmap(0, 4096, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    CHECK(p != MAP_FAILED, "mmap");
    if (p != MAP_FAILED) {
        char *q = mremap(p, 4096, 8192, 1);
        if (q == MAP_FAILED) {
            munmap(p, 4096);
        } else {
            munmap(q, 8192);
        }
        CHECK(q != MAP_FAILED, "mremap");
    }

    pthread_t t;
    if (pthread_create(&t, 0, worker, 0) == 0) {
        void *r = 0;
        int rc = pthread_timedjoin_np(t, &r, 0);
        CHECK(rc == 0 && r == (void *)42, "pthread_timedjoin_np");
    } else {
        puts("FAIL: pthread_create");
        failures++;
    }

    int pty = open("/dev/ptmx", O_RDWR);
    if (pty >= 0) {
        char name[64];
        CHECK(ptsname_r(pty, name, sizeof(name)) == 0, "ptsname_r");
        /* TIOCGPTN, not TIOCGWINSZ: the name must be /dev/pts/<n>, not the
           window size interpreted as a pty number. */
        CHECK(strncmp(name, "/dev/pts/", 9) == 0 && name[9] != 0, "ptsname_r name");
        close(pty);
    }

    char tmpl[] = "/tmp/crabc-compat-XXXXXX";
    int fd = mkostemps(tmpl, 0, O_CLOEXEC);
    CHECK(fd >= 0, "mkostemps");
    if (fd >= 0) { close(fd); unlink(tmpl); }

    struct timeval tv[2] = {{1111111, 0}, {2222222, 0}};
    char path[] = "/tmp/crabc-futimes-XXXXXX";
    fd = mkstemp(path);
    CHECK(fd >= 0, "mkstemp");
    if (fd >= 0) {
        close(fd);
        CHECK(futimesat(AT_FDCWD, path, tv) == 0, "futimesat");
        struct stat st;
        CHECK(stat(path, &st) == 0 && st.st_atim.tv_sec == 1111111, "futimesat value");
        unlink(path);
    }

    CHECK(riscv_flush_icache(0, 0, 0) == 0, "riscv_flush_icache");
    unsigned char dnsq[280];
    CHECK(res_mkquery(0, "example.com", 1, 1, 0, 0, 0, dnsq, sizeof(dnsq)) > 12, "res_mkquery");
    CHECK(res_init() == 0, "res_init");

    struct addrinfo hints, *ai = 0;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_INET;
    hints.ai_socktype = SOCK_STREAM;
    CHECK(getaddrinfo("127.0.0.1", "80", &hints, &ai) == 0 && ai != 0, "getaddrinfo");
    if (ai) freeaddrinfo(ai);
    struct hostent *he = gethostbyname("127.0.0.1");
    CHECK(he != 0 && he->h_length == 4, "gethostbyname");
    struct passwd *pw = getpwnam("root");
    CHECK(pw != 0 && pw->pw_uid == 0, "getpwnam");
    struct group *gr = getgrnam("root");
    CHECK(gr != 0 && gr->gr_gid == 0, "getgrnam");

    char utmp_path[] = "/tmp/crabc-utmp-XXXXXX";
    fd = mkstemp(utmp_path);
    CHECK(fd >= 0, "utmp temp");
    if (fd >= 0) {
        close(fd);
        CHECK(utmpname(utmp_path) == 0, "utmpname");
        setutent();
        CHECK(getutent() == 0, "empty utmp");
        struct utmpx u;
        memset(&u, 0, sizeof(u));
        u.ut_type = USER_PROCESS;
        u.ut_pid = getpid();
        strcpy(u.ut_line, "pts/0");
        strcpy(u.ut_id, "ts0");
        strcpy(u.ut_user, "tester");
        CHECK(pututline(&u) != 0, "pututline");
        setutent();
        struct utmpx *up = getutent();
        CHECK(up && strcmp(up->ut_user, "tester") == 0, "getutent");
        setutent();
        CHECK(getutline(&u) != 0, "getutline");
        setutent();
        CHECK(getutid(&u) != 0, "getutid");
        endutent();
        unlink(utmp_path);
    }

    /* path resolution */
    char rp[PATH_MAX];
    CHECK(realpath("/tmp/../tmp", rp) != 0 && strcmp(rp, "/tmp") == 0, "realpath");

    /* glob */
    glob_t gl;
    memset(&gl, 0, sizeof(gl));
    CHECK(glob("/tmp/*", 0, 0, &gl) == 0 && gl.gl_pathc > 0, "glob");
    globfree(&gl);

    /* nftw over a small tree */
    char tree[] = "/tmp/crabc-ftw-XXXXXX";
    if (mkdtemp(tree)) {
        char fp[128];
        snprintf(fp, sizeof(fp), "%s/a", tree);
        FILE *ff = fopen(fp, "w");
        if (ff) fclose(ff);
        nftw_visits = 0;
        CHECK(nftw(tree, nftw_cb, 8, FTW_PHYS) == 0 && nftw_visits >= 2, "nftw");
        unlink(fp);
        rmdir(tree);
    }

    /* wcsftime and %ls */
    wchar_t wbuf[64];
    struct tm tmv;
    memset(&tmv, 0, sizeof(tmv));
    tmv.tm_year = 70; tmv.tm_mday = 1;
    CHECK(wcsftime(wbuf, 64, L"%Y-%m-%d", &tmv) == 10 && wcscmp(wbuf, L"1970-01-01") == 0, "wcsftime");
    char mbuf[256];
    snprintf(mbuf, sizeof(mbuf), "%ls", wbuf);
    CHECK(strcmp(mbuf, "1970-01-01") == 0, "printf %ls");

    /* strfmon */
    char money[64];
    CHECK(strfmon(money, sizeof(money), "%n", 1234.5) > 0 && strcmp(money, "1234.50") == 0, "strfmon");

    /* traditional DES crypt */
    CHECK(strcmp(crypt("password", "ab"), "abJnggxhB/yWI") == 0, "crypt DES");
    CHECK(strcmp(crypt("password", "_J9..rasm"), "_J9..rasmEedsvB6g8/6") == 0, "crypt extended DES");

    /* wordexp */
    wordexp_t we;
    memset(&we, 0, sizeof(we));
    setenv("CRABC_WE", "value", 1);
    CHECK(wordexp("a 'b c' \"d $CRABC_WE\"", &we, 0) == 0 && we.we_wordc == 3, "wordexp");
    if (we.we_wordc == 3) {
        CHECK(strcmp(we.we_wordv[1], "b c") == 0 && strcmp(we.we_wordv[2], "d value") == 0, "wordexp values");
    }
    wordfree(&we);

    /* fopencookie over a memory buffer */
    {
        struct memcookie mc;
        memset(&mc, 0, sizeof(mc));
        cookie_io_functions_t io = { mc_read, mc_write, mc_seek, mc_close };
        FILE *cf = fopencookie(&mc, "w+", io);
        CHECK(cf != 0, "fopencookie");
        if (cf) {
            CHECK(fwrite("cookie", 1, 6, cf) == 6, "cookie write");
            CHECK(fseek(cf, 0, SEEK_SET) == 0, "cookie seek");
            char cb[8] = {0};
            CHECK(fread(cb, 1, 6, cf) == 6 && strcmp(cb, "cookie") == 0, "cookie read");
            fclose(cf);
        }
    }

    /* open_wmemstream */
    {
        wchar_t *ws = 0;
        size_t wl = 0;
        FILE *wf = open_wmemstream(&ws, &wl);
        CHECK(wf != 0, "open_wmemstream");
        if (wf) {
            fputws(L"h\u00e9llo", wf);
            fclose(wf);
            CHECK(wl == 5 && ws && ws[1] == 0xe9, "wmemstream contents");
            free(ws);
        }
    }

    /* AIO */
    {
        char apath[] = "/tmp/crabc-aio-XXXXXX";
        int afd = mkstemp(apath);
        CHECK(afd >= 0, "aio temp");
        if (afd >= 0) {
            struct aiocb cb;
            memset(&cb, 0, sizeof(cb));
            cb.aio_fildes = afd;
            cb.aio_buf = (void *)"aio!";
            cb.aio_nbytes = 4;
            CHECK(aio_write(&cb) == 0 && aio_error(&cb) == 0 && aio_return(&cb) == 4, "aio_write");
            char rbuf[8] = {0};
            memset(&cb, 0, sizeof(cb));
            cb.aio_fildes = afd;
            cb.aio_buf = rbuf;
            cb.aio_nbytes = 4;
            CHECK(aio_read(&cb) == 0 && aio_error(&cb) == 0 && aio_return(&cb) == 4 && strcmp(rbuf, "aio!") == 0, "aio_read");
            close(afd);
            unlink(apath);
        }
    }

    /* pthread_getattr_np */
    {
        pthread_attr_t pa;
        CHECK(pthread_getattr_np(pthread_self(), &pa) == 0, "pthread_getattr_np");
        size_t ssz = 0;
        CHECK(pthread_attr_getstacksize(&pa, &ssz) == 0 && ssz > 0, "pthread_attr_getstacksize");
    }

    /* dladdr / dl_iterate_phdr */
    {
        Dl_info di;
        CHECK(dladdr((void *)main, &di) != 0 && di.dli_fname != 0, "dladdr");
        phdr_objects = 0;
        CHECK(dl_iterate_phdr(phdr_cb, 0) == 0 && phdr_objects > 0, "dl_iterate_phdr");
    }

    /* getopt_long */
    {
        static struct option opts[] = {
            {"alpha", no_argument, 0, 'a'},
            {"beta", required_argument, 0, 'b'},
            {0, 0, 0, 0}
        };
        char *av[] = { (char *)"p", (char *)"-abVAL", (char *)"--alpha", 0 };
        int ac = 3;
        int got_a = 0, got_b = 0;
        const char *bval = 0;
        int oc;
        optind = 1; opterr = 0;
        while ((oc = getopt_long(ac, av, "ab:", opts, 0)) != -1) {
            if (oc == 'a') got_a++;
            else if (oc == 'b') { got_b++; bval = optarg; }
        }
        CHECK(got_a == 2 && got_b == 1 && bval && strcmp(bval, "VAL") == 0, "getopt_long");
    }

    /* membarrier may be restricted by the kernel; just exercise the ABI. */
    membarrier(0, 0, 0);

    if (failures == 0) puts("compat ok");
    return failures != 0;
}
