#ifndef _NL_TYPES_H
#define _NL_TYPES_H

typedef void *nl_catd;

#define NL_SETD 1

nl_catd catopen(const char *name, int oflag);
int catclose(nl_catd catd);
char *catgets(nl_catd catd, int set_id, int msg_id, const char *s);

#endif
