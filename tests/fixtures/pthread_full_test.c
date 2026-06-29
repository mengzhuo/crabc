#include <pthread.h>
#include <semaphore.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
#include <errno.h>

static int failures = 0;
#define CHECK(expr, msg) do { if (!(expr)) { printf("FAIL: %s\n", msg); failures++; return; } } while(0)

static void test_mutexattr(void) {
    pthread_mutexattr_t ma;
    int t;
    CHECK(pthread_mutexattr_init(&ma) == 0, "mutexattr init");
    CHECK(pthread_mutexattr_gettype(&ma, &t) == 0 && t == PTHREAD_MUTEX_NORMAL, "default type NORMAL");
    CHECK(pthread_mutexattr_settype(&ma, PTHREAD_MUTEX_RECURSIVE) == 0, "settype RECURSIVE");
    CHECK(pthread_mutexattr_gettype(&ma, &t) == 0 && t == PTHREAD_MUTEX_RECURSIVE, "gettype RECURSIVE");
    CHECK(pthread_mutexattr_settype(&ma, PTHREAD_MUTEX_ERRORCHECK) == 0, "settype ERRORCHECK");
    CHECK(pthread_mutexattr_gettype(&ma, &t) == 0 && t == PTHREAD_MUTEX_ERRORCHECK, "gettype ERRORCHECK");
    CHECK(pthread_mutexattr_settype(&ma, 99) != 0, "settype invalid fails");
    CHECK(pthread_mutexattr_setpshared(&ma, PTHREAD_PROCESS_PRIVATE) == 0, "setpshared PRIVATE");
    CHECK(pthread_mutexattr_setpshared(&ma, PTHREAD_PROCESS_SHARED) == 0, "setpshared SHARED");
    int p;
    CHECK(pthread_mutexattr_getpshared(&ma, &p) == 0 && p == PTHREAD_PROCESS_SHARED, "getpshared SHARED");
    CHECK(pthread_mutexattr_destroy(&ma) == 0, "mutexattr destroy");
}

static void test_mutex_types(void) {
    pthread_mutex_t m;
    pthread_mutexattr_t ma;

    pthread_mutexattr_init(&ma);
    CHECK(pthread_mutex_init(&m, &ma) == 0, "init NORMAL");
    CHECK(pthread_mutex_lock(&m) == 0, "lock NORMAL");
    CHECK(pthread_mutex_trylock(&m) != 0, "trylock NORMAL while locked");
    CHECK(pthread_mutex_unlock(&m) == 0, "unlock NORMAL");
    CHECK(pthread_mutex_trylock(&m) == 0, "trylock NORMAL");
    CHECK(pthread_mutex_unlock(&m) == 0, "unlock NORMAL 2");
    CHECK(pthread_mutex_destroy(&m) == 0, "destroy NORMAL");
    pthread_mutexattr_destroy(&ma);

    pthread_mutexattr_init(&ma);
    pthread_mutexattr_settype(&ma, PTHREAD_MUTEX_RECURSIVE);
    CHECK(pthread_mutex_init(&m, &ma) == 0, "init RECURSIVE");
    CHECK(pthread_mutex_lock(&m) == 0, "lock RECURSIVE 1");
    CHECK(pthread_mutex_lock(&m) == 0, "lock RECURSIVE 2");
    CHECK(pthread_mutex_lock(&m) == 0, "lock RECURSIVE 3");
    CHECK(pthread_mutex_unlock(&m) == 0, "unlock RECURSIVE 1");
    CHECK(pthread_mutex_unlock(&m) == 0, "unlock RECURSIVE 2");
    CHECK(pthread_mutex_unlock(&m) == 0, "unlock RECURSIVE 3");
    CHECK(pthread_mutex_destroy(&m) == 0, "destroy RECURSIVE");
    pthread_mutexattr_destroy(&ma);

    pthread_mutexattr_init(&ma);
    pthread_mutexattr_settype(&ma, PTHREAD_MUTEX_ERRORCHECK);
    CHECK(pthread_mutex_init(&m, &ma) == 0, "init ERRORCHECK");
    CHECK(pthread_mutex_lock(&m) == 0, "lock ERRORCHECK");
    CHECK(pthread_mutex_lock(&m) == EDEADLK, "lock ERRORCHECK deadlock");
    CHECK(pthread_mutex_unlock(&m) == 0, "unlock ERRORCHECK");
    CHECK(pthread_mutex_destroy(&m) == 0, "destroy ERRORCHECK");
    pthread_mutexattr_destroy(&ma);
}

static void test_mutex_timedlock(void) {
    pthread_mutex_t m;
    struct timespec ts;
    CHECK(pthread_mutex_init(&m, NULL) == 0, "init for timedlock");
    CHECK(clock_gettime(CLOCK_REALTIME, &ts) == 0, "clock_gettime");
    ts.tv_sec += 1;
    CHECK(pthread_mutex_timedlock(&m, &ts) == 0, "timedlock unlocked");
    CHECK(pthread_mutex_unlock(&m) == 0, "unlock after timedlock");
    CHECK(pthread_mutex_destroy(&m) == 0, "destroy after timedlock");
}

static void test_condattr(void) {
    pthread_condattr_t ca;
    CHECK(pthread_condattr_init(&ca) == 0, "condattr init");
    clockid_t clk;
    CHECK(pthread_condattr_getclock(&ca, &clk) == 0, "condattr getclock default");
    CHECK(pthread_condattr_setclock(&ca, CLOCK_MONOTONIC) == 0, "condattr setclock MONOTONIC");
    CHECK(pthread_condattr_getclock(&ca, &clk) == 0 && clk == CLOCK_MONOTONIC, "condattr getclock MONOTONIC");
    int p;
    CHECK(pthread_condattr_setpshared(&ca, PTHREAD_PROCESS_SHARED) == 0, "condattr setpshared");
    CHECK(pthread_condattr_getpshared(&ca, &p) == 0 && p == PTHREAD_PROCESS_SHARED, "condattr getpshared");
    CHECK(pthread_condattr_destroy(&ca) == 0, "condattr destroy");
}

static pthread_cond_t sig_cond = PTHREAD_COND_INITIALIZER;
static pthread_mutex_t sig_mutex;
static int sig_ready = 0;

static void *sig_waiter(void *arg) {
    (void)arg;
    pthread_mutex_lock(&sig_mutex);
    while (!sig_ready)
        pthread_cond_wait(&sig_cond, &sig_mutex);
    pthread_mutex_unlock(&sig_mutex);
    return NULL;
}

static void test_cond_signal(void) {
    pthread_t t;
    sig_ready = 0;
    pthread_mutex_init(&sig_mutex, NULL);
    CHECK(pthread_create(&t, NULL, sig_waiter, NULL) == 0, "create cond waiter");
    for (volatile int i = 0; i < 100000; i++) {}
    pthread_mutex_lock(&sig_mutex);
    sig_ready = 1;
    pthread_cond_signal(&sig_cond);
    pthread_mutex_unlock(&sig_mutex);
    CHECK(pthread_join(t, NULL) == 0, "join cond waiter");
    pthread_mutex_destroy(&sig_mutex);
}

static void test_rwlock(void) {
    pthread_rwlock_t rw;
    CHECK(pthread_rwlock_init(&rw, NULL) == 0, "rwlock init");
    CHECK(pthread_rwlock_rdlock(&rw) == 0, "rdlock");
    CHECK(pthread_rwlock_unlock(&rw) == 0, "unlock rd");
    CHECK(pthread_rwlock_wrlock(&rw) == 0, "wrlock");
    CHECK(pthread_rwlock_tryrdlock(&rw) != 0, "tryrdlock while wrlocked");
    CHECK(pthread_rwlock_unlock(&rw) == 0, "unlock wr");
    CHECK(pthread_rwlock_tryrdlock(&rw) == 0, "tryrdlock");
    CHECK(pthread_rwlock_unlock(&rw) == 0, "unlock tryrd");
    CHECK(pthread_rwlock_trywrlock(&rw) == 0, "trywrlock");
    CHECK(pthread_rwlock_unlock(&rw) == 0, "unlock trywr");
    CHECK(pthread_rwlock_destroy(&rw) == 0, "rwlock destroy");
}

static void test_rwlockattr(void) {
    pthread_rwlockattr_t ra;
    CHECK(pthread_rwlockattr_init(&ra) == 0, "rwlockattr init");
    int p;
    CHECK(pthread_rwlockattr_setpshared(&ra, PTHREAD_PROCESS_SHARED) == 0, "rwlockattr setpshared");
    CHECK(pthread_rwlockattr_getpshared(&ra, &p) == 0 && p == PTHREAD_PROCESS_SHARED, "rwlockattr getpshared");
    CHECK(pthread_rwlockattr_destroy(&ra) == 0, "rwlockattr destroy");
}

static void test_barrierattr(void) {
    pthread_barrierattr_t ba;
    CHECK(pthread_barrierattr_init(&ba) == 0, "barrierattr init");
    int p;
    CHECK(pthread_barrierattr_setpshared(&ba, PTHREAD_PROCESS_SHARED) == 0, "barrierattr setpshared");
    CHECK(pthread_barrierattr_getpshared(&ba, &p) == 0 && p == PTHREAD_PROCESS_SHARED, "barrierattr getpshared");
    CHECK(pthread_barrierattr_destroy(&ba) == 0, "barrierattr destroy");
}

static pthread_barrier_t bar;
static void *bar_worker(void *a) { (void)a; pthread_barrier_wait(&bar); return NULL; }

static void test_barrier(void) {
    pthread_t ts[3];
    CHECK(pthread_barrier_init(&bar, NULL, 3) == 0, "barrier init");
    for (int i = 0; i < 3; i++)
        CHECK(pthread_create(&ts[i], NULL, bar_worker, NULL) == 0, "create barrier worker");
    for (int i = 0; i < 3; i++)
        CHECK(pthread_join(ts[i], NULL) == 0, "join barrier worker");
    CHECK(pthread_barrier_destroy(&bar) == 0, "barrier destroy");
}

static void test_spinlock(void) {
    pthread_spinlock_t s;
    CHECK(pthread_spin_init(&s, PTHREAD_PROCESS_PRIVATE) == 0, "spin init");
    CHECK(pthread_spin_lock(&s) == 0, "spin lock");
    CHECK(pthread_spin_trylock(&s) != 0, "spin trylock while locked");
    CHECK(pthread_spin_unlock(&s) == 0, "spin unlock");
    CHECK(pthread_spin_trylock(&s) == 0, "spin trylock");
    CHECK(pthread_spin_unlock(&s) == 0, "spin unlock 2");
    CHECK(pthread_spin_destroy(&s) == 0, "spin destroy");
}

static void test_semaphore(void) {
    sem_t s;
    int v;
    CHECK(sem_init(&s, 0, 3) == 0, "sem init");
    CHECK(sem_getvalue(&s, &v) == 0 && v == 3, "sem getvalue 3");
    CHECK(sem_wait(&s) == 0, "sem wait 1");
    CHECK(sem_wait(&s) == 0, "sem wait 2");
    CHECK(sem_getvalue(&s, &v) == 0 && v == 1, "sem getvalue 1");
    CHECK(sem_trywait(&s) == 0, "sem trywait ok");
    CHECK(sem_trywait(&s) != 0, "sem trywait empty");
    CHECK(sem_post(&s) == 0, "sem post");
    CHECK(sem_getvalue(&s, &v) == 0 && v == 1, "sem getvalue after post");
    CHECK(sem_destroy(&s) == 0, "sem destroy");
}

static pthread_once_t once_control = PTHREAD_ONCE_INIT;
static int once_counter = 0;

static void once_init(void) { once_counter++; }
static void *once_worker(void *arg) { (void)arg; pthread_once(&once_control, once_init); return NULL; }

static void test_once(void) {
    pthread_t ts[4];
    once_counter = 0;
    for (int i = 0; i < 4; i++)
        CHECK(pthread_create(&ts[i], NULL, once_worker, NULL) == 0, "create once worker");
    for (int i = 0; i < 4; i++)
        CHECK(pthread_join(ts[i], NULL) == 0, "join once worker");
    CHECK(once_counter == 1, "once_counter == 1");
}

static pthread_key_t tsd_key;

static void *tsd_worker(void *arg) {
    (void)arg;
    pthread_setspecific(tsd_key, (void *)42);
    if (pthread_getspecific(tsd_key) != (void *)42)
        return (void *)1;
    return NULL;
}

static void test_key(void) {
    pthread_t t;
    void *ret;
    CHECK(pthread_key_create(&tsd_key, NULL) == 0, "key create");
    CHECK(pthread_create(&t, NULL, tsd_worker, NULL) == 0, "create tsd worker");
    CHECK(pthread_join(t, &ret) == 0, "join tsd worker");
    CHECK(ret == NULL, "getspecific == 42");
    CHECK(pthread_key_delete(tsd_key) == 0, "key delete");
}

static void test_attr(void) {
    pthread_attr_t a;
    int ds;
    size_t ss, gs;
    CHECK(pthread_attr_init(&a) == 0, "attr init");
    CHECK(pthread_attr_getdetachstate(&a, &ds) == 0 && ds == PTHREAD_CREATE_JOINABLE, "default detach JOINABLE");
    CHECK(pthread_attr_setdetachstate(&a, PTHREAD_CREATE_DETACHED) == 0, "set detach DETACHED");
    CHECK(pthread_attr_getdetachstate(&a, &ds) == 0 && ds == PTHREAD_CREATE_DETACHED, "get detach DETACHED");
    CHECK(pthread_attr_setdetachstate(&a, PTHREAD_CREATE_JOINABLE) == 0, "set detach JOINABLE");
    CHECK(pthread_attr_getstacksize(&a, &ss) == 0 && ss > 0, "default stacksize > 0");
    CHECK(pthread_attr_setstacksize(&a, 65536) == 0, "set stacksize 65536");
    CHECK(pthread_attr_getstacksize(&a, &ss) == 0 && ss == 65536, "get stacksize 65536");
    CHECK(pthread_attr_getguardsize(&a, &gs) == 0, "get guardsize");
    CHECK(pthread_attr_setguardsize(&a, 8192) == 0, "set guardsize");
    CHECK(pthread_attr_getguardsize(&a, &gs) == 0 && gs == 8192, "get guardsize 8192");
    int sc;
    CHECK(pthread_attr_setscope(&a, PTHREAD_SCOPE_SYSTEM) == 0, "setscope SYSTEM");
    CHECK(pthread_attr_getscope(&a, &sc) == 0 && sc == PTHREAD_SCOPE_SYSTEM, "getscope SYSTEM");
    int inh;
    CHECK(pthread_attr_setinheritsched(&a, PTHREAD_EXPLICIT_SCHED) == 0, "setinheritsched EXPLICIT");
    CHECK(pthread_attr_getinheritsched(&a, &inh) == 0 && inh == PTHREAD_EXPLICIT_SCHED, "getinheritsched EXPLICIT");
    int pol;
    CHECK(pthread_attr_setschedpolicy(&a, 0) == 0, "setschedpolicy");
    CHECK(pthread_attr_getschedpolicy(&a, &pol) == 0, "getschedpolicy");
    struct sched_param sp;
    sp.sched_priority = 10;
    CHECK(pthread_attr_setschedparam(&a, &sp) == 0, "setschedparam");
    struct sched_param sp2;
    CHECK(pthread_attr_getschedparam(&a, &sp2) == 0 && sp2.sched_priority == 10, "getschedparam");
    CHECK(pthread_attr_destroy(&a) == 0, "attr destroy");
}

static void *self_worker(void *arg) {
    pthread_t *out = (pthread_t *)arg;
    *out = pthread_self();
    return NULL;
}

static void test_self_equal(void) {
    pthread_t me = pthread_self();
    CHECK(me != 0, "self != 0");
    CHECK(pthread_equal(me, me) != 0, "equal self");
    pthread_t other;
    pthread_t t;
    CHECK(pthread_create(&t, NULL, self_worker, &other) == 0, "create self worker");
    CHECK(pthread_join(t, NULL) == 0, "join self worker");
    CHECK(other != 0, "other self != 0");
    CHECK(pthread_equal(me, other) == 0, "not equal different threads");
}

static void test_cancel(void) {
    int old;
    CHECK(pthread_setcancelstate(PTHREAD_CANCEL_DISABLE, &old) == 0, "setcancelstate DISABLE");
    CHECK(old == PTHREAD_CANCEL_ENABLE, "old state ENABLE");
    CHECK(pthread_setcancelstate(PTHREAD_CANCEL_ENABLE, &old) == 0, "setcancelstate ENABLE");
    CHECK(old == PTHREAD_CANCEL_DISABLE, "old state DISABLE");
    int oldt;
    CHECK(pthread_setcanceltype(PTHREAD_CANCEL_ASYNCHRONOUS, &oldt) == 0, "setcanceltype ASYNC");
    CHECK(oldt == PTHREAD_CANCEL_DEFERRED, "old type DEFERRED");
    CHECK(pthread_setcanceltype(PTHREAD_CANCEL_DEFERRED, &oldt) == 0, "setcanceltype DEFERRED");
}

int main(void) {
    test_mutexattr();
    test_mutex_types();
    test_mutex_timedlock();
    test_condattr();
    test_cond_signal();
    test_rwlock();
    test_rwlockattr();
    test_barrierattr();
    test_barrier();
    test_spinlock();
    test_semaphore();
    test_once();
    test_key();
    test_attr();
    test_self_equal();
    test_cancel();

    if (failures == 0) {
        printf("pthread_full ok\n");
        return 0;
    }
    printf("pthread_full FAIL %d\n", failures);
    return 1;
}
