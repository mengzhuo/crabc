#ifndef _NETINET_IN_H
#define _NETINET_IN_H

#include <stdint.h>
#include <sys/socket.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint16_t in_port_t;
typedef uint32_t in_addr_t;

#ifndef __crabc_in_addr_struct
#define __crabc_in_addr_struct
struct in_addr { in_addr_t s_addr; };
#endif

struct sockaddr_in {
    sa_family_t sin_family;
    in_port_t sin_port;
    struct in_addr sin_addr;
    unsigned char sin_zero[8];
};

struct in6_addr { unsigned char s6_addr[16]; };

struct sockaddr_in6 {
    sa_family_t sin6_family;
    in_port_t sin6_port;
    uint32_t sin6_flowinfo;
    struct in6_addr sin6_addr;
    uint32_t sin6_scope_id;
};

struct ipv6_mreq {
    struct in6_addr ipv6mr_multiaddr;
    unsigned ipv6mr_interface;
};

extern const struct in6_addr in6addr_any;
extern const struct in6_addr in6addr_loopback;

#define IN6ADDR_ANY_INIT { { 0 } }
#define IN6ADDR_LOOPBACK_INIT { { 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1 } }

#define IPPROTO_IP   0
#define IPPROTO_ICMP 1
#define IPPROTO_TCP  6
#define IPPROTO_UDP  17
#define IPPROTO_IPV6 41
#define IPPROTO_RAW  255

#define INET_ADDRSTRLEN  16
#define INET6_ADDRSTRLEN 46

#define IPV6_JOIN_GROUP       20
#define IPV6_LEAVE_GROUP      21
#define IPV6_MULTICAST_HOPS   18
#define IPV6_MULTICAST_IF     17
#define IPV6_MULTICAST_LOOP   19
#define IPV6_UNICAST_HOPS     16
#define IPV6_V6ONLY           26

#define IN6_IS_ADDR_UNSPECIFIED(a) \
    ((a)->s6_addr[0] == 0 && (a)->s6_addr[1] == 0 && (a)->s6_addr[2] == 0 && (a)->s6_addr[3] == 0 && \
     (a)->s6_addr[4] == 0 && (a)->s6_addr[5] == 0 && (a)->s6_addr[6] == 0 && (a)->s6_addr[7] == 0 && \
     (a)->s6_addr[8] == 0 && (a)->s6_addr[9] == 0 && (a)->s6_addr[10] == 0 && (a)->s6_addr[11] == 0 && \
     (a)->s6_addr[12] == 0 && (a)->s6_addr[13] == 0 && (a)->s6_addr[14] == 0 && (a)->s6_addr[15] == 0)
#define IN6_IS_ADDR_LOOPBACK(a) \
    ((a)->s6_addr[0] == 0 && (a)->s6_addr[1] == 0 && (a)->s6_addr[2] == 0 && (a)->s6_addr[3] == 0 && \
     (a)->s6_addr[4] == 0 && (a)->s6_addr[5] == 0 && (a)->s6_addr[6] == 0 && (a)->s6_addr[7] == 0 && \
     (a)->s6_addr[8] == 0 && (a)->s6_addr[9] == 0 && (a)->s6_addr[10] == 0 && (a)->s6_addr[11] == 0 && \
     (a)->s6_addr[12] == 0 && (a)->s6_addr[13] == 0 && (a)->s6_addr[14] == 0 && (a)->s6_addr[15] == 1)
#define IN6_IS_ADDR_MULTICAST(a) ((a)->s6_addr[0] == 0xff)
#define IN6_IS_ADDR_LINKLOCAL(a) ((a)->s6_addr[0] == 0xfe && ((a)->s6_addr[1] & 0xc0) == 0x80)
#define IN6_IS_ADDR_SITELOCAL(a) ((a)->s6_addr[0] == 0xfe && ((a)->s6_addr[1] & 0xc0) == 0xc0)
#define IN6_IS_ADDR_V4MAPPED(a) \
    ((a)->s6_addr[0] == 0 && (a)->s6_addr[1] == 0 && (a)->s6_addr[2] == 0 && (a)->s6_addr[3] == 0 && \
     (a)->s6_addr[4] == 0 && (a)->s6_addr[5] == 0 && (a)->s6_addr[6] == 0 && (a)->s6_addr[7] == 0 && \
     (a)->s6_addr[8] == 0 && (a)->s6_addr[9] == 0 && (a)->s6_addr[10] == 0xff && (a)->s6_addr[11] == 0xff)
#define IN6_IS_ADDR_V4COMPAT(a) \
    ((a)->s6_addr[0] == 0 && (a)->s6_addr[1] == 0 && (a)->s6_addr[2] == 0 && (a)->s6_addr[3] == 0 && \
     (a)->s6_addr[4] == 0 && (a)->s6_addr[5] == 0 && (a)->s6_addr[6] == 0 && (a)->s6_addr[7] == 0 && \
     (a)->s6_addr[8] == 0 && (a)->s6_addr[9] == 0 && (a)->s6_addr[10] == 0 && (a)->s6_addr[11] == 0)
#define IN6_IS_ADDR_MC_NODELOCAL(a) ((a)->s6_addr[0] == 0xff && ((a)->s6_addr[1] & 0x0f) == 1)
#define IN6_IS_ADDR_MC_LINKLOCAL(a) ((a)->s6_addr[0] == 0xff && ((a)->s6_addr[1] & 0x0f) == 2)
#define IN6_IS_ADDR_MC_SITELOCAL(a) ((a)->s6_addr[0] == 0xff && ((a)->s6_addr[1] & 0x0f) == 5)
#define IN6_IS_ADDR_MC_ORGLOCAL(a)  ((a)->s6_addr[0] == 0xff && ((a)->s6_addr[1] & 0x0f) == 8)
#define IN6_IS_ADDR_MC_GLOBAL(a)    ((a)->s6_addr[0] == 0xff && ((a)->s6_addr[1] & 0x0f) == 14)

#define INADDR_ANY       ((in_addr_t)0x00000000)
#define INADDR_LOOPBACK  ((in_addr_t)0x7f000001)
#define INADDR_BROADCAST ((in_addr_t)0xffffffff)
#define INADDR_NONE      ((in_addr_t)0xffffffff)

#ifdef __cplusplus
}
#endif

#endif
