#ifndef _SCHED_H
#define _SCHED_H

#include <sys/types.h>
#include <time.h>

#ifdef __cplusplus
extern "C" {
#endif

struct sched_param {
    int sched_priority;
};

#define SCHED_OTHER 0
#define SCHED_FIFO  1
#define SCHED_RR    2
#define SCHED_BATCH 3
#define SCHED_IDLE  5

int sched_yield(void);
int sched_get_priority_max(int);
int sched_get_priority_min(int);
int sched_getparam(pid_t, struct sched_param *);
int sched_setparam(pid_t, const struct sched_param *);
int sched_getscheduler(pid_t);
int sched_setscheduler(pid_t, int, const struct sched_param *);
int sched_rr_get_interval(pid_t, struct timespec *);
int sched_getaffinity(pid_t, size_t, void *);
int sched_setaffinity(pid_t, size_t, const void *);
int sched_getcpu(void);

#ifdef __cplusplus
}
#endif

#endif
