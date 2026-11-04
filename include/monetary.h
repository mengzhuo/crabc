#ifndef _MONETARY_H
#define _MONETARY_H

#include <stddef.h>
#include <sys/types.h>
#include <locale.h>

#ifdef __cplusplus
extern "C" {
#endif

ssize_t strfmon(char *__restrict, size_t, const char *__restrict, ...);
ssize_t strfmon_l(char *__restrict, size_t, locale_t, const char *__restrict, ...);

#ifdef __cplusplus
}
#endif

#endif
