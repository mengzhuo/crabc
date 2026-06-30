#ifndef _PTHREAD_ATFORK_H
#define _PTHREAD_ATFORK_H

int pthread_atfork(void (*prepare)(void), void (*parent)(void), void (*child)(void));

#endif
