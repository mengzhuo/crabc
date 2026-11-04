#ifndef _SYS_FILE_H
#define _SYS_FILE_H

#define F_LOCK  1
#define F_TLOCK  2
#define F_ULOCK  3
#define F_TEST  4

#define LOCK_SH  1
#define LOCK_EX  2
#define LOCK_NB  4
#define LOCK_UN  8

int flock(int, int);

#endif