#ifndef _SYS_SEM_H
#define _SYS_SEM_H

#ifdef __cplusplus
extern "C" {
#endif

#include <sys/ipc.h>
#include <stddef.h>

typedef long time_t;
typedef int pid_t;

#define SEM_UNDO	0x1000
#define GETPID		11
#define GETVAL		12
#define GETALL		13
#define GETNCNT		14
#define GETZCNT		15
#define SETVAL		16
#define SETALL		17

struct semid_ds {
	struct ipc_perm sem_perm;
	time_t sem_otime;
	long __unused1;
	time_t sem_ctime;
	long __unused2;
	unsigned short sem_nsems;
	char __sem_nsems_pad[sizeof(long)-sizeof(short)];
	long __unused3;
	long __unused4;
};

#define _SEM_SEMUN_UNDEFINED 1

#define SEM_STAT (18 | (IPC_STAT & 0x100))
#define SEM_INFO 19
#define SEM_STAT_ANY (20 | (IPC_STAT & 0x100))

struct seminfo {
	int semmap;
	int semmni;
	int semmns;
	int semmnu;
	int semmsl;
	int semopm;
	int semume;
	int semusz;
	int semvmx;
	int semaem;
};

struct sembuf {
	unsigned short sem_num;
	short sem_op;
	short sem_flg;
};

int semctl(int, int, int, ...);
int semget(key_t, int, int);
int semop(int, struct sembuf *, size_t);
int semtimedop(int, struct sembuf *, size_t, const struct timespec *);

#ifdef __cplusplus
}
#endif
#endif
