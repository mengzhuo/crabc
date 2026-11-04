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

    /* membarrier may be restricted by the kernel; just exercise the ABI. */
    membarrier(0, 0, 0);

    if (failures == 0) puts("compat ok");
    return failures != 0;
}
