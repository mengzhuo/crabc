#ifndef _NETDB_H
#define _NETDB_H

#include <stddef.h>
#include <stdint.h>
#include <sys/socket.h>
#include <netinet/in.h>

#ifdef __cplusplus
extern "C" {
#endif

struct addrinfo {
    int ai_flags;
    int ai_family;
    int ai_socktype;
    int ai_protocol;
    socklen_t ai_addrlen;
    struct sockaddr *ai_addr;
    char *ai_canonname;
    struct addrinfo *ai_next;
};

#define AI_PASSIVE      0x01
#define AI_CANONNAME    0x02
#define AI_NUMERICHOST  0x04
#define AI_V4MAPPED     0x08
#define AI_ALL          0x10
#define AI_ADDRCONFIG   0x20
#define AI_NUMERICSERV  0x400

#define NI_NUMERICHOST  0x01
#define NI_NUMERICSERV  0x02
#define NI_NOFQDN       0x04
#define NI_NAMEREQD     0x08
#define NI_DGRAM        0x10
#define NI_NUMERICSCOPE 0x100

#define EAI_BADFLAGS   -1
#define EAI_NONAME     -2
#define EAI_AGAIN      -3
#define EAI_FAIL       -4
#define EAI_NODATA     -5
#define EAI_FAMILY     -6
#define EAI_SOCKTYPE   -7
#define EAI_SERVICE    -8
#define EAI_MEMORY     -10
#define EAI_SYSTEM     -11
#define EAI_OVERFLOW   -12

#define IPPORT_RESERVED 1024

#define HOST_NOT_FOUND 1
#define TRY_AGAIN      2
#define NO_RECOVERY    3
#define NO_DATA        4
#define NO_ADDRESS     NO_DATA

struct netent {
    char *n_name;
    char **n_aliases;
    int n_addrtype;
    uint32_t n_net;
};

struct hostent {
    char *h_name;
    char **h_aliases;
    int h_addrtype;
    int h_length;
    char **h_addr_list;
};
#define h_addr h_addr_list[0]

struct servent {
    char *s_name;
    char **s_aliases;
    int s_port;
    char *s_proto;
};

struct protoent {
    char *p_name;
    char **p_aliases;
    int p_proto;
};

int *__h_errno_location(void);
#define h_errno (*__h_errno_location())

int getaddrinfo(const char *, const char *, const struct addrinfo *, struct addrinfo **);
void freeaddrinfo(struct addrinfo *);
int getnameinfo(const struct sockaddr *, socklen_t, char *, socklen_t, char *, socklen_t, int);
const char *gai_strerror(int);

struct hostent *gethostbyname(const char *);
struct hostent *gethostbyname2(const char *, int);
struct hostent *gethostbyaddr(const void *, socklen_t, int);
void sethostent(int);
void endhostent(void);
struct hostent *gethostent(void);

struct servent *getservbyname(const char *, const char *);
struct servent *getservbyport(int, const char *);
void setservent(int);
void endservent(void);
struct servent *getservent(void);

struct protoent *getprotobyname(const char *);
struct protoent *getprotobynumber(int);
void setprotoent(int);
void endprotoent(void);
struct protoent *getprotoent(void);

struct netent *getnetbyname(const char *);
struct netent *getnetbyaddr(uint32_t, int);
void setnetent(int);
void endnetent(void);
struct netent *getnetent(void);

#ifdef __cplusplus
}
#endif

#endif
