#ifndef _PWD_H
#define _PWD_H

#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

struct passwd {
    char *pw_name;
    char *pw_passwd;
    uid_t pw_uid;
    gid_t pw_gid;
    char *pw_gecos;
    char *pw_dir;
    char *pw_shell;
};

struct passwd *getpwnam(const char *);
struct passwd *getpwuid(uid_t);
int getpwnam_r(const char *, struct passwd *, char *, size_t, struct passwd **);
int getpwuid_r(uid_t, struct passwd *, char *, size_t, struct passwd **);
void setpwent(void);
void endpwent(void);
struct passwd *getpwent(void);
struct passwd *fgetpwent(void *);
int putpwent(const struct passwd *, void *);

#ifdef __cplusplus
}
#endif

#endif
