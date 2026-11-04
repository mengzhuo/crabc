#ifndef _DLFCN_H
#define _DLFCN_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define RTLD_LAZY   1
#define RTLD_NOW    2
#define RTLD_GLOBAL 0x100
#define RTLD_LOCAL  0
#define RTLD_NODELETE 0x1000
#define RTLD_NOLOAD   0x4
#define RTLD_DEEPBIND 0x8

#define RTLD_NEXT      ((void *)-1)
#define RTLD_DEFAULT   ((void *)0)

#define RTLD_DI_LINKMAP 2
#define RTLD_DI_ORIGIN  1

typedef struct {
    const char *dli_fname;
    void *dli_fbase;
    const char *dli_sname;
    void *dli_saddr;
} Dl_info;

void *dlopen(const char *, int);
int dlclose(void *);
void *dlsym(void *, const char *);
char *dlerror(void);
int dladdr(const void *, Dl_info *);
int dlinfo(void *, int, void *);

struct dl_phdr_info {
    unsigned long long dlpi_addr;
    const char *dlpi_name;
    const void *dlpi_phdr;
    unsigned short dlpi_phnum;
    unsigned long long dlpi_adds;
    unsigned long long dlpi_subs;
    size_t dlpi_tls_modid;
    void *dlpi_tls_data;
};

int dl_iterate_phdr(int (*)(struct dl_phdr_info *, size_t, void *), void *);

#ifdef __cplusplus
}
#endif

#endif
