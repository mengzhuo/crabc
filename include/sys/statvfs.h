#ifndef _SYS_STATVFS_H
#define _SYS_STATVFS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef unsigned long fsblkcnt_t;
typedef unsigned long fsfilcnt_t;

struct statvfs {
	unsigned long f_bsize;
	unsigned long f_frsize;
	fsblkcnt_t f_blocks;
	fsblkcnt_t f_bfree;
	fsblkcnt_t f_bavail;
	fsfilcnt_t f_files;
	fsfilcnt_t f_ffree;
	fsfilcnt_t f_favail;
	unsigned long f_fsid;
	unsigned long f_flag;
	unsigned long f_namemax;
	unsigned int f_type;
	int __reserved[5];
};

int statvfs(const char *, struct statvfs *);
int fstatvfs(int, struct statvfs *);

#define ST_RDONLY     1
#define ST_NOSUID     2
#define ST_NODEV      4
#define ST_NOEXEC     8
#define ST_SYNCHRONOUS 16
#define ST_MANDLOCK   64
#define ST_WRITE      128
#define ST_APPEND     256
#define ST_IMMUTABLE  512
#define ST_NOATIME    1024
#define ST_NODIRATIME 2048
#define ST_RELATIME   4096

#ifdef __cplusplus
}
#endif

#endif
