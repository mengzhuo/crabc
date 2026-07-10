#include <pthread.h>
#include <stdio.h>

extern int get_libvar(void);
extern void set_libvar(int v);

__thread int mainvar = 42;

static void *worker(void *arg) {
    (void)arg;
    set_libvar(123);
    return NULL;
}

int main(void) {
    fprintf(stderr, "mainvar addr=%p value=%d\n", &mainvar, mainvar);
    fflush(stderr);
    if (mainvar != 42) return 1;
    if (get_libvar() != 99) return 2;

    pthread_t t;
    pthread_create(&t, NULL, worker, NULL);
    pthread_join(t, NULL);

    if (get_libvar() != 99) return 3;
    if (mainvar != 42) return 4;

    printf("dso tls ok\n");
    return 0;
}
