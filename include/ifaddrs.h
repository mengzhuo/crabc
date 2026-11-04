#ifndef _IFADDRS_H
#define _IFADDRS_H

#include <sys/types.h>
#include <sys/socket.h>

#ifdef __cplusplus
extern "C" {
#endif

struct ifaddrs {
    struct ifaddrs *ifa_next;
    char *ifa_name;
    unsigned int ifa_flags;
    struct sockaddr *ifa_addr;
    struct sockaddr *ifa_netmask;
    union {
        struct sockaddr *ifu_broadaddr;
        struct sockaddr *ifu_dstaddr;
    } ifa_ifu;
    void *ifa_data;
};

#define ifa_broadaddr ifa_ifu.ifu_broadaddr
#define ifa_dstaddr   ifa_ifu.ifu_dstaddr

int getifaddrs(struct ifaddrs **);
void freeifaddrs(struct ifaddrs *);

#ifdef __cplusplus
}
#endif

#endif
