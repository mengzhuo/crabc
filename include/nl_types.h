#ifndef _NL_TYPES_H
#define _NL_TYPES_H

#include <sys/types.h>

typedef void *nl_catd;
typedef int nl_item;

#define NL_SETD 1
#define NL_CAT_LOCALE 1

nl_catd catopen(const char *name, int oflag);
int catclose(nl_catd catd);
char *catgets(nl_catd catd, int set_id, int msg_id, const char *s);

#endif
