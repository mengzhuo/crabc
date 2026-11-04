#ifndef _RESOLV_H
#define _RESOLV_H

#ifdef __cplusplus
extern "C" {
#endif

int res_init(void);
int res_query(const char *, int, int, unsigned char *, int);
int res_querydomain(const char *, const char *, int, int, unsigned char *, int);
int res_mkquery(int, const char *, int, int, const unsigned char *, int,
                const unsigned char *, unsigned char *, int);
int res_send(const unsigned char *, int, unsigned char *, int);
int res_search(const char *, int, int, unsigned char *, int);

#ifdef __cplusplus
}
#endif

#endif
