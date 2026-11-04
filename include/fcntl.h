#ifndef _FCNTL_H
#define _FCNTL_H

#ifdef __cplusplus
extern "C" {
#endif

#include <sys/types.h>

struct flock {
    short l_type;
    short l_whence;
    off_t l_start;
    off_t l_len;
    pid_t l_pid;
};

int open(const char *, int, ...);
int openat(int, const char *, int, ...);
int creat(const char *, mode_t);
int fcntl(int, int, ...);

int posix_fadvise(int, off_t, off_t, int);
int posix_fallocate(int, off_t, off_t);

#define O_CREAT        0100
#define O_EXCL         0200
#define O_NOCTTY       0400
#define O_TRUNC       01000
#define O_APPEND      02000
#define O_NONBLOCK    04000
#define O_DSYNC      010000
#define O_SYNC     04010000
#define O_RSYNC    04010000
#define O_DIRECTORY 0200000
#define O_NOFOLLOW  0400000
#define O_CLOEXEC  02000000

#define O_ASYNC      020000
#define O_DIRECT     040000
#define O_LARGEFILE 0100000
#define O_NOATIME  01000000
#define O_PATH    010000000
#define O_TMPFILE 020200000
#define O_NDELAY O_NONBLOCK

#define O_SEARCH   O_PATH
#define O_EXEC     O_PATH
#define O_TTY_INIT 0

#define O_ACCMODE (03|O_SEARCH)
#define O_RDONLY  00
#define O_WRONLY  01
#define O_RDWR    02

#define F_DUPFD  0
#define F_GETFD  1
#define F_SETFD  2
#define F_GETFL  3
#define F_SETFL  4
#define F_GETLK  5
#define F_SETLK  6
#define F_SETLKW 7

#define F_SETOWN 8
#define F_GETOWN 9
#define F_SETSIG 10
#define F_GETSIG 11

#define F_DUPFD_CLOEXEC 1030
#define F_GETLK64  5
#define F_SETLK64  6
#define F_SETLKW64 7

#define FD_CLOEXEC 1

#define F_RDLCK 0
#define F_WRLCK 1
#define F_UNLCK 2

#define AT_FDCWD (-100)
#define AT_SYMLINK_NOFOLLOW 0x100
#define AT_REMOVEDIR 0x200
#define AT_SYMLINK_FOLLOW 0x400
#define AT_EACCESS 0x200

#define POSIX_FADV_NORMAL     0
#define POSIX_FADV_RANDOM     1
#define POSIX_FADV_SEQUENTIAL 2
#define POSIX_FADV_WILLNEED   3
#define POSIX_FADV_DONTNEED   4
#define POSIX_FADV_NOREUSE    5

#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2
#define SEEK_DATA 3
#define SEEK_HOLE 4

#ifndef S_IRUSR
#define S_ISUID 04000
#define S_ISGID 02000
#define S_ISVTX 01000
#define S_IRUSR 0400
#define S_IWUSR 0200
#define S_IXUSR 0100
#define S_IRWXU 0700
#define S_IRGRP 0040
#define S_IWGRP 0020
#define S_IXGRP 0010
#define S_IRWXG 0070
#define S_IROTH 0004
#define S_IWOTH 0002
#define S_IXOTH 0001
#define S_IRWXO 0007
#endif

#define F_ULOCK 0
#define F_LOCK  1
#define F_TLOCK 2
#define F_TEST  3

int lockf(int, int, off_t);

#if defined(_GNU_SOURCE)
#define SYNC_FILE_RANGE_WAIT_BEFORE 1
#define SYNC_FILE_RANGE_WRITE       2
#define SYNC_FILE_RANGE_WAIT_AFTER  4
int sync_file_range(int, off_t, off_t, unsigned);
#endif

#ifdef __cplusplus
}
#endif

#endif
