#ifndef _SYS_SELECT_H
#define _SYS_SELECT_H

#include <sys/types.h>
#include <time.h>
#include <signal.h>

#ifdef __cplusplus
extern "C" {
#endif

#define FD_SETSIZE 1024
typedef unsigned long fd_set[FD_SETSIZE / (8 * sizeof(unsigned long))];

#define FD_CLR(d, s) ((s)[(d) / (8 * sizeof(unsigned long))] &= ~(1UL << ((d) % (8 * sizeof(unsigned long)))))
#define FD_SET(d, s) ((s)[(d) / (8 * sizeof(unsigned long))] |=  (1UL << ((d) % (8 * sizeof(unsigned long)))))
#define FD_ISSET(d, s) (((s)[(d) / (8 * sizeof(unsigned long))] >> ((d) % (8 * sizeof(unsigned long)))) & 1)
#define FD_ZERO(s) do { unsigned long *__p = (s); int __i; for (__i = 0; __i < FD_SETSIZE / (8 * sizeof(unsigned long)); __i++) __p[__i] = 0; } while (0)

int select(int, fd_set *, fd_set *, fd_set *, struct timeval *);
int pselect(int, fd_set *, fd_set *, fd_set *, const struct timespec *, const sigset_t *);

#ifdef __cplusplus
}
#endif

#endif
