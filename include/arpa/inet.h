#ifndef _ARPA_INET_H
#define _ARPA_INET_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint32_t in_addr_t;

int inet_pton(int, const char *, void *);
const char *inet_ntop(int, const void *, char *, unsigned int);
in_addr_t inet_addr(const char *);
char *inet_ntoa(uint32_t);
int inet_aton(const char *, uint32_t *);
in_addr_t inet_network(const char *);
uint32_t inet_makeaddr(int, int);
int inet_lnaof(uint32_t);
int inet_netof(uint32_t);

#ifdef __cplusplus
}
#endif

#endif
