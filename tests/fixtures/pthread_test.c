#include <pthread.h>
#include <stdio.h>

static int counter = 0;
static pthread_mutex_t mutex;

static void *worker(void *arg) {
    (void)arg;
    for (int i = 0; i < 10000; i++) {
        pthread_mutex_lock(&mutex);
        counter++;
        pthread_mutex_unlock(&mutex);
    }
    return NULL;
}

int main(void) {
    pthread_t t1, t2;

    pthread_mutex_init(&mutex, NULL);
    pthread_create(&t1, NULL, worker, NULL);
    pthread_create(&t2, NULL, worker, NULL);
    pthread_join(t1, NULL);
    pthread_join(t2, NULL);

    if (counter == 20000) {
        printf("pthread ok\n");
        return 0;
    }
    printf("counter=%d\n", counter);
    return 1;
}
