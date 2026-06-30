#ifndef _SYS_MSG_H
#define _SYS_MSG_H

#ifdef __cplusplus
extern "C" {
#endif

#include <sys/ipc.h>
#include <stddef.h>
#include <sys/types.h>

typedef long ssize_t;
typedef unsigned long msgqnum_t;
typedef unsigned long msglen_t;
typedef long time_t;
typedef int pid_t;

struct msqid_ds {
	struct ipc_perm msg_perm;
	time_t msg_stime;
	time_t msg_rtime;
	time_t msg_ctime;
	unsigned long msg_cbytes;
	msgqnum_t msg_qnum;
	msglen_t msg_qbytes;
	pid_t msg_lspid;
	pid_t msg_lrpid;
	unsigned long __unused[2];
};

#define MSG_NOERROR 010000
#define MSG_EXCEPT  020000

#define MSG_STAT (11 | (IPC_STAT & 0x100))
#define MSG_INFO 12
#define MSG_STAT_ANY (13 | (IPC_STAT & 0x100))

struct msginfo {
	int msgpool, msgmap, msgmax, msgmnb, msgmni, msgssz, msgtql;
	unsigned short msgseg;
};

int msgctl(int, int, struct msqid_ds *);
int msgget(key_t, int);
ssize_t msgrcv(int, void *, size_t, long, int);
int msgsnd(int, const void *, size_t, int);

struct msgbuf {
	long mtype;
	char mtext[1];
};

#ifdef __cplusplus
}
#endif

#endif
