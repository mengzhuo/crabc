#ifndef _SYS_TYPES_H
#define _SYS_TYPES_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#ifndef __crabc_ssize_t_defined
#define __crabc_ssize_t_defined
#ifndef __DEFINED_ssize_t
#define __DEFINED_ssize_t
typedef long ssize_t;
#endif
#endif

#ifndef __crabc_off_t_defined
#define __crabc_off_t_defined
#ifndef __DEFINED_off_t
#define __DEFINED_off_t
typedef long off_t;
#endif
#endif

#ifndef __crabc_pid_t_defined
#define __crabc_pid_t_defined
#ifndef __DEFINED_pid_t
#define __DEFINED_pid_t
typedef int pid_t;
#endif
#endif

#ifndef __crabc_id_t_defined
#define __crabc_id_t_defined
#ifndef __DEFINED_id_t
#define __DEFINED_id_t
typedef unsigned int id_t;
#endif
#endif

#ifndef __crabc_uid_t_defined
#define __crabc_uid_t_defined
#ifndef __DEFINED_uid_t
#define __DEFINED_uid_t
typedef unsigned int uid_t;
#endif
#endif

#ifndef __crabc_gid_t_defined
#define __crabc_gid_t_defined
#ifndef __DEFINED_gid_t
#define __DEFINED_gid_t
typedef unsigned int gid_t;
#endif
#endif

#ifndef __crabc_mode_t_defined
#define __crabc_mode_t_defined
#ifndef __DEFINED_mode_t
#define __DEFINED_mode_t
typedef unsigned int mode_t;
#endif
#endif

#ifndef __crabc_dev_t_defined
#define __crabc_dev_t_defined
#ifndef __DEFINED_dev_t
#define __DEFINED_dev_t
typedef unsigned long long dev_t;
#endif
#endif

#ifndef __crabc_ino_t_defined
#define __crabc_ino_t_defined
#ifndef __DEFINED_ino_t
#define __DEFINED_ino_t
typedef unsigned long long ino_t;
#endif
#endif

#ifndef __crabc_nlink_t_defined
#define __crabc_nlink_t_defined
#ifndef __DEFINED_nlink_t
#define __DEFINED_nlink_t
#if defined(__aarch64__) || defined(__riscv)
typedef unsigned int nlink_t;
#else
typedef unsigned long nlink_t;
#endif
#endif
#endif

#ifndef __crabc_blksize_t_defined
#define __crabc_blksize_t_defined
#ifndef __DEFINED_blksize_t
#define __DEFINED_blksize_t
#if defined(__aarch64__) || defined(__riscv)
typedef int blksize_t;
#else
typedef long blksize_t;
#endif
#endif
#endif

#ifndef __crabc_blkcnt_t_defined
#define __crabc_blkcnt_t_defined
#ifndef __DEFINED_blkcnt_t
#define __DEFINED_blkcnt_t
typedef long blkcnt_t;
#endif
#endif

#ifndef __crabc_fsblkcnt_t_defined
#define __crabc_fsblkcnt_t_defined
#ifndef __DEFINED_fsblkcnt_t
#define __DEFINED_fsblkcnt_t
typedef unsigned long long fsblkcnt_t;
#endif
#endif

#ifndef __crabc_fsfilcnt_t_defined
#define __crabc_fsfilcnt_t_defined
#ifndef __DEFINED_fsfilcnt_t
#define __DEFINED_fsfilcnt_t
typedef unsigned long long fsfilcnt_t;
#endif
#endif

#ifndef __crabc_key_t_defined
#define __crabc_key_t_defined
#ifndef __DEFINED_key_t
#define __DEFINED_key_t
typedef int key_t;
#endif
#endif

#ifndef __crabc_suseconds_t_defined
#define __crabc_suseconds_t_defined
#ifndef __DEFINED_suseconds_t
#define __DEFINED_suseconds_t
typedef long suseconds_t;
#endif
#endif

#ifndef __crabc_useconds_t_defined
#define __crabc_useconds_t_defined
#ifndef __DEFINED_useconds_t
#define __DEFINED_useconds_t
typedef unsigned int useconds_t;
#endif
#endif

#ifndef __crabc_time_t_defined
#define __crabc_time_t_defined
#ifndef __DEFINED_time_t
#define __DEFINED_time_t
typedef long time_t;
#endif
#endif

#ifndef __crabc_clock_t_defined
#define __crabc_clock_t_defined
#ifndef __DEFINED_clock_t
#define __DEFINED_clock_t
typedef long clock_t;
#endif
#endif

#ifndef __crabc_clockid_t_defined
#define __crabc_clockid_t_defined
#ifndef __DEFINED_clockid_t
#define __DEFINED_clockid_t
typedef int clockid_t;
#endif
#endif

#ifndef __crabc_timer_t_defined
#define __crabc_timer_t_defined
#ifndef __DEFINED_timer_t
#define __DEFINED_timer_t
typedef void *timer_t;
#endif
#endif

#ifndef __crabc_in_addr_t_defined
#define __crabc_in_addr_t_defined
typedef unsigned int in_addr_t;
#endif

#ifndef __crabc_in_port_t_defined
#define __crabc_in_port_t_defined
typedef unsigned short in_port_t;
#endif

#ifndef __crabc_sa_family_t_defined
#define __crabc_sa_family_t_defined
#ifndef __DEFINED_sa_family_t
#define __DEFINED_sa_family_t
typedef unsigned short sa_family_t;
#endif
#endif

#ifndef __crabc_socklen_t_defined
#define __crabc_socklen_t_defined
#ifndef __DEFINED_socklen_t
#define __DEFINED_socklen_t
typedef unsigned int socklen_t;
#endif
#endif

/* pthread types */
#ifndef __crabc_pthread_t_defined
#define __crabc_pthread_t_defined
#ifndef __DEFINED_pthread_t
#define __DEFINED_pthread_t
typedef unsigned long pthread_t;
#endif
#endif

#ifndef __crabc_pthread_once_t_defined
#define __crabc_pthread_once_t_defined
#ifndef __DEFINED_pthread_once_t
#define __DEFINED_pthread_once_t
typedef int pthread_once_t;
#endif
#endif

#ifndef __crabc_pthread_key_t_defined
#define __crabc_pthread_key_t_defined
#ifndef __DEFINED_pthread_key_t
#define __DEFINED_pthread_key_t
typedef unsigned int pthread_key_t;
#endif
#endif

#ifndef __crabc_pthread_spinlock_t_defined
#define __crabc_pthread_spinlock_t_defined
#ifndef __DEFINED_pthread_spinlock_t
#define __DEFINED_pthread_spinlock_t
typedef int pthread_spinlock_t;
#endif
#endif

#ifndef __crabc_pthread_mutexattr_t_defined
#define __crabc_pthread_mutexattr_t_defined
#ifndef __DEFINED_pthread_mutexattr_t
#define __DEFINED_pthread_mutexattr_t
typedef struct { unsigned __attr; } pthread_mutexattr_t;
#endif
#endif

#ifndef __crabc_pthread_condattr_t_defined
#define __crabc_pthread_condattr_t_defined
#ifndef __DEFINED_pthread_condattr_t
#define __DEFINED_pthread_condattr_t
typedef struct { unsigned __attr; } pthread_condattr_t;
#endif
#endif

#ifndef __crabc_pthread_barrierattr_t_defined
#define __crabc_pthread_barrierattr_t_defined
#ifndef __DEFINED_pthread_barrierattr_t
#define __DEFINED_pthread_barrierattr_t
typedef struct { unsigned __attr; } pthread_barrierattr_t;
#endif
#endif

#ifndef __crabc_pthread_rwlockattr_t_defined
#define __crabc_pthread_rwlockattr_t_defined
#ifndef __DEFINED_pthread_rwlockattr_t
#define __DEFINED_pthread_rwlockattr_t
typedef struct { unsigned __attr[2]; } pthread_rwlockattr_t;
#endif
#endif

#ifndef __crabc_pthread_attr_t_defined
#define __crabc_pthread_attr_t_defined
#ifndef __DEFINED_pthread_attr_t
#define __DEFINED_pthread_attr_t
typedef struct { int __i[14]; } pthread_attr_t;
#endif
#endif

#ifndef __crabc_pthread_mutex_t_defined
#define __crabc_pthread_mutex_t_defined
#ifndef __DEFINED_pthread_mutex_t
#define __DEFINED_pthread_mutex_t
typedef struct { int __i[10]; } pthread_mutex_t;
#endif
#endif

#ifndef __crabc_pthread_cond_t_defined
#define __crabc_pthread_cond_t_defined
#ifndef __DEFINED_pthread_cond_t
#define __DEFINED_pthread_cond_t
typedef struct { int __i[12]; } pthread_cond_t;
#endif
#endif

#ifndef __crabc_pthread_rwlock_t_defined
#define __crabc_pthread_rwlock_t_defined
#ifndef __DEFINED_pthread_rwlock_t
#define __DEFINED_pthread_rwlock_t
typedef struct { int __i[14]; } pthread_rwlock_t;
#endif
#endif

#ifndef __crabc_pthread_barrier_t_defined
#define __crabc_pthread_barrier_t_defined
#ifndef __DEFINED_pthread_barrier_t
#define __DEFINED_pthread_barrier_t
typedef struct { int __i[8]; } pthread_barrier_t;
#endif
#endif

#ifndef NULL
#define NULL ((void*)0)
#endif

#ifdef __cplusplus
}
#endif

#endif
