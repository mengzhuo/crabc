#ifndef _NET_IF_H
#define _NET_IF_H

#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

#define IF_NAMESIZE 16
#define IFNAMSIZ    16

#define IFF_UP          0x1
#define IFF_BROADCAST   0x2
#define IFF_DEBUG       0x4
#define IFF_LOOPBACK    0x8
#define IFF_POINTOPOINT 0x10
#define IFF_NOTRAILERS  0x20
#define IFF_RUNNING     0x40
#define IFF_NOARP       0x80
#define IFF_PROMISC     0x100
#define IFF_ALLMULTI    0x200
#define IFF_MASTER      0x400
#define IFF_SLAVE       0x800
#define IFF_MULTICAST   0x1000
#define IFF_PORTSEL     0x2000
#define IFF_AUTOMEDIA   0x4000
#define IFF_DYNAMIC     0x8000

struct if_nameindex {
    unsigned if_index;
    char *if_name;
};

unsigned if_nametoindex(const char *);
char *if_indextoname(unsigned, char *);
struct if_nameindex *if_nameindex(void);
void if_freenameindex(struct if_nameindex *);

#ifdef __cplusplus
}
#endif

#endif
