#ifndef PTHREAD_H
#define PTHREAD_H

#include <stddef.h>

typedef unsigned long pthread_t;

typedef struct {
    int lock;
} pthread_mutex_t;

typedef struct {
} pthread_attr_t;

typedef struct {
} pthread_mutexattr_t;

int pthread_create(pthread_t *thread, const pthread_attr_t *attr,
                   void *(*start_routine)(void *), void *arg);
int pthread_join(pthread_t thread, void **retval);
void pthread_exit(void *retval) __attribute__((__noreturn__));
pthread_t pthread_self(void);
int pthread_equal(pthread_t t1, pthread_t t2);

int pthread_mutex_init(pthread_mutex_t *mutex,
                       const pthread_mutexattr_t *attr);
int pthread_mutex_destroy(pthread_mutex_t *mutex);
int pthread_mutex_lock(pthread_mutex_t *mutex);
int pthread_mutex_unlock(pthread_mutex_t *mutex);

#endif
