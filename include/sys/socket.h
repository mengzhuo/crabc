#ifndef _SYS_SOCKET_H
#define _SYS_SOCKET_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef long ssize_t;
typedef unsigned short sa_family_t;
typedef unsigned int socklen_t;

struct sockaddr {
    sa_family_t sa_family;
    char sa_data[14];
};

#define AF_UNIX   1
#define AF_INET   2
#define AF_INET6 10

#define SOCK_STREAM 1
#define SOCK_DGRAM  2

#define SHUT_RD   0
#define SHUT_WR   1
#define SHUT_RDWR 2

#define SOL_SOCKET    1
#define SO_REUSEADDR  2

int socket(int, int, int);
int socketpair(int, int, int, int[2]);
int bind(int, const struct sockaddr *, socklen_t);
int listen(int, int);
int accept(int, struct sockaddr *, socklen_t *);
int connect(int, const struct sockaddr *, socklen_t);
ssize_t send(int, const void *, size_t, int);
ssize_t recv(int, void *, size_t, int);
ssize_t sendto(int, const void *, size_t, int, const struct sockaddr *, socklen_t);
ssize_t recvfrom(int, void *, size_t, int, struct sockaddr *, socklen_t *);
int shutdown(int, int);
int setsockopt(int, int, int, const void *, socklen_t);
int getsockname(int, struct sockaddr *, socklen_t *);

unsigned int htonl(unsigned int);
unsigned int ntohl(unsigned int);
unsigned short htons(unsigned short);
unsigned short ntohs(unsigned short);

#ifdef __cplusplus
}
#endif

#endif
