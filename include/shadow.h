#ifndef _SHADOW_H
#define _SHADOW_H

#include <sys/types.h>
#include <stdio.h>

#ifdef __cplusplus
extern "C" {
#endif

struct spwd {
    char *sp_namp;
    char *sp_pwdp;
    long sp_lstchg;
    long sp_min;
    long sp_max;
    long sp_warn;
    long sp_inact;
    long sp_expire;
    unsigned long sp_flag;
};

void setspent(void);
void endspent(void);
struct spwd *getspent(void);
struct spwd *getspnam(const char *);
struct spwd *fgetspent(FILE *);
int putspent(const struct spwd *, FILE *);
int getspnam_r(const char *, struct spwd *, char *, size_t, struct spwd **);
int lckpwdf(void);
int ulckpwdf(void);

#ifdef __cplusplus
}
#endif

#endif
