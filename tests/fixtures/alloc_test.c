#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

#define THREADS 4
#define ITERS 50000

static void *worker(void *arg) {
    unsigned seed = (unsigned)(uintptr_t)arg;
    for (int i = 0; i < ITERS; i++) {
        size_t n = (size_t)((seed = seed * 1103515245u + 12345u) % 2048) + 1;
        unsigned char *p = malloc(n);
        if (!p) return (void *)1;
        memset(p, i & 0xff, n);
        if (p[0] != (unsigned char)(i & 0xff)) return (void *)2;
        if ((i & 3) == 0) {
            size_t nn = n + (seed % 512);
            p = realloc(p, nn);
            if (!p) return (void *)3;
            memset(p + n, 0x5a, nn - n);
        }
        free(p);
    }
    return 0;
}

int main(void) {
    enum { N = 5000 };
    static void *p[N];
    static size_t sz[N];

    for (int i = 0; i < N; i++) {
        sz[i] = (size_t)((i * 37) % 4096) + 1;
        p[i] = malloc(sz[i]);
        if (!p[i]) return 10;
        if (malloc_usable_size(p[i]) < sz[i]) return 11;
        memset(p[i], i & 0xff, sz[i]);
    }
    for (int i = 0; i < N; i++) {
        unsigned char *q = p[i];
        for (size_t j = 0; j < sz[i]; j++) {
            if (q[j] != (unsigned char)(i & 0xff)) return 12;
        }
        if (i & 1) {
            p[i] = realloc(p[i], sz[i] + 100);
            if (!p[i]) return 13;
            memset((char *)p[i] + sz[i], 0xaa, 100);
        }
    }
    for (int i = 0; i < N; i++) free(p[i]);

    for (int i = 0; i < 500; i++) {
        void *q = aligned_alloc(256, 1024);
        if (!q || ((uintptr_t)q & 255)) return 14;
        memset(q, 0x5a, 1024);
        free(q);
    }
    for (int i = 0; i < 500; i++) {
        void *q = NULL;
        if (posix_memalign(&q, 4096, 100)) return 15;
        if ((uintptr_t)q & 4095) return 16;
        free(q);
    }
    {
        void *q = valloc(300);
        if (!q || ((uintptr_t)q & 4095)) return 17;
        free(q);
    }

    pthread_t t[THREADS];
    for (long i = 0; i < THREADS; i++) {
        if (pthread_create(&t[i], 0, worker, (void *)(i + 1))) return 18;
    }
    for (int i = 0; i < THREADS; i++) {
        void *r;
        if (pthread_join(t[i], &r) || r) return 19;
    }

    void *fork_ptr = malloc(1234);
    if (!fork_ptr) return 20;
    memset(fork_ptr, 0x33, 1234);
    pid_t pid = fork();
    if (pid == 0) {
        void *q = malloc(777);
        if (!q) _exit(21);
        memset(q, 0x44, 777);
        free(q);
        free(fork_ptr);
        _exit(0);
    }
    int st;
    if (pid < 0 || waitpid(pid, &st, 0) != pid) return 22;
    if (!WIFEXITED(st) || WEXITSTATUS(st)) return 23;
    free(fork_ptr);

    puts("allocator ok");
    return 0;
}
