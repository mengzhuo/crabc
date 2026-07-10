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
    fprintf(stderr, "main: start\n");
    fflush(stderr);
    fprintf(stderr, "mainvar addr=%p value=%d\n", &mainvar, mainvar);
    fflush(stderr);
    if (mainvar != 42) return 1;
    fprintf(stderr, "main: get_libvar()...\n");
    fflush(stderr);
    int v = get_libvar();
    fprintf(stderr, "main: get_libvar returned %d\n", v);
    fflush(stderr);
    if (v != 99) return 2;

    fprintf(stderr, "main: pthread_create...\n");
    fflush(stderr);
    pthread_t t;
    int r = pthread_create(&t, NULL, worker, NULL);
    fprintf(stderr, "main: pthread_create returned %d\n", r);
    fflush(stderr);
    if (r != 0) return 5;
    fprintf(stderr, "main: pthread_join...\n");
    fflush(stderr);
    pthread_join(t, NULL);
    fprintf(stderr, "main: pthread_join done\n");
    fflush(stderr);

    fprintf(stderr, "main: get_libvar after thread...\n");
    fflush(stderr);
    v = get_libvar();
    fprintf(stderr, "main: get_libvar after thread returned %d\n", v);
    fflush(stderr);
    if (v != 99) return 3;
    fprintf(stderr, "main: mainvar check after thread...\n");
    fflush(stderr);
    if (mainvar != 42) return 4;

    printf("dso tls ok\n");
    return 0;
}
