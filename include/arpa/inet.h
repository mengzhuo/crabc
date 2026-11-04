#ifndef _ARPA_INET_H
#define _ARPA_INET_H

#include <stdint.h>
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifndef __crabc_in_addr_t_defined
#define __crabc_in_addr_t_defined
typedef uint32_t in_addr_t;
#endif

#ifndef __crabc_in_port_t_defined
#define __crabc_in_port_t_defined
typedef uint16_t in_port_t;
#endif

struct in_addr {
    in_addr_t s_addr;
};

#define INET_ADDRSTRLEN  16
#define INET6_ADDRSTRLEN 46

int inet_pton(int, const char *, void *);
const char *inet_ntop(int, const void *, char *, socklen_t);
in_addr_t inet_addr(const char *);
char *inet_ntoa(struct in_addr);
int inet_aton(const char *, struct in_addr *);
in_addr_t inet_network(const char *);
struct in_addr inet_makeaddr(in_addr_t, in_addr_t);
in_addr_t inet_lnaof(struct in_addr);
in_addr_t inet_netof(struct in_addr);

/* crabc historically exposes the raw 32-bit address in these two entry
 * points; keep both calling conventions source-compatible. */
#define inet_aton(s, p) inet_aton((s), (void *)(p))
#define inet_ntoa(addr) inet_ntoa((struct in_addr){ .s_addr = (in_addr_t)(addr) })

uint32_t htonl(uint32_t);
uint16_t htons(uint16_t);
uint32_t ntohl(uint32_t);
uint16_t ntohs(uint16_t);

#ifdef __cplusplus
}
#endif

#endif
