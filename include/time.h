#ifndef _TIME_H
#define _TIME_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <sys/types.h>
#include <locale.h>

#ifndef NULL
#define NULL ((void*)0)
#endif

#ifndef __crabc_struct_timespec_defined
#ifndef __DEFINED_struct_timespec
#define __DEFINED_struct_timespec
#define __crabc_struct_timespec_defined
struct timespec {
    time_t tv_sec;
    long tv_nsec;
};
#endif
#endif

#ifndef __crabc_struct_timeval_defined
#define __crabc_struct_timeval_defined
#ifndef _TIMEVAL_DEFINED
#define _TIMEVAL_DEFINED
#endif
struct timeval {
    time_t tv_sec;
    suseconds_t tv_usec;
};
#endif

#define CLOCKS_PER_SEC 1000000L

#define TIME_UTC 1

#define CLOCK_REALTIME           0
#define CLOCK_MONOTONIC          1
#define CLOCK_PROCESS_CPUTIME_ID 2
#define CLOCK_THREAD_CPUTIME_ID  3
#define CLOCK_MONOTONIC_RAW      4
#define CLOCK_REALTIME_COARSE    5
#define CLOCK_MONOTONIC_COARSE   6
#define CLOCK_BOOTTIME           7
#define CLOCK_REALTIME_ALARM     8
#define CLOCK_BOOTTIME_ALARM     9
#define CLOCK_SGI_CYCLE         10
#define CLOCK_TAI               11

#define TIMER_ABSTIME 1

struct tm {
    int tm_sec;
    int tm_min;
    int tm_hour;
    int tm_mday;
    int tm_mon;
    int tm_year;
    int tm_wday;
    int tm_yday;
    int tm_isdst;
    long tm_gmtoff;
    const char *tm_zone;
};

struct itimerspec {
    struct timespec it_interval;
    struct timespec it_value;
};

struct sigevent;

clock_t clock(void);
time_t time(time_t *);
double difftime(time_t, time_t);
time_t mktime(struct tm *);
size_t strftime(char *, size_t, const char *, const struct tm *);
struct tm *gmtime(const time_t *);
struct tm *localtime(const time_t *);
char *asctime(const struct tm *);
char *ctime(const time_t *);
int timespec_get(struct timespec *, int);

size_t strftime_l(char *, size_t, const char *, const struct tm *, locale_t);

struct tm *gmtime_r(const time_t *, struct tm *);
struct tm *localtime_r(const time_t *, struct tm *);
char *asctime_r(const struct tm *, char *);
char *ctime_r(const time_t *, char *);

int nanosleep(const struct timespec *, struct timespec *);
int clock_getres(clockid_t, struct timespec *);
int clock_gettime(clockid_t, struct timespec *);
int clock_settime(clockid_t, const struct timespec *);
int clock_nanosleep(clockid_t, int, const struct timespec *, struct timespec *);
int clock_getcpuclockid(pid_t, clockid_t *);

int timer_create(clockid_t, struct sigevent *, timer_t *);
int timer_delete(timer_t);
int timer_settime(timer_t, int, const struct itimerspec *, struct itimerspec *);
int timer_gettime(timer_t, struct itimerspec *);
int timer_getoverrun(timer_t);

void tzset(void);
extern char *tzname[2];

char *strptime(const char *, const char *, struct tm *);
extern int daylight;
extern long timezone;
extern int getdate_err;
struct tm *getdate(const char *);

int stime(const time_t *);
time_t timegm(struct tm *);

int gettimeofday(struct timeval *, void *);

#ifdef __cplusplus
}
#endif

#endif
