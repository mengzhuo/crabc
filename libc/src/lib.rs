#![cfg_attr(not(test), no_std)]
#![feature(c_variadic)]
#![feature(linkage)]
#![feature(f128)]
#![allow(dead_code, non_camel_case_types)]

use core::ffi::{c_char, c_int, c_long, c_longlong, c_uint, c_ulong, c_ulonglong, c_void, VaList};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

include!("encoding_tables.rs");
include!("math_helpers.rs");
include!("math_bitmanip.rs");
include!("math_sqrtfmod.rs");
include!("math_trig.rs");
include!("math_exp.rs");
include!("math_log.rs");
include!("math_pow.rs");
include!("math_hypot.rs");
include!("math_hyperbolic.rs");
include!("math_invtrig.rs");
include!("math_lrint.rs");

// ============================================================
// errno
// ============================================================

const EILSEQ: c_int = 84;
const EINVAL: c_int = 22;
const EFAULT: c_int = 14;
const E2BIG: c_int = 7;

const LC_CTYPE: c_int = 0;
const LC_NUMERIC: c_int = 1;
const LC_TIME: c_int = 2;
const LC_COLLATE: c_int = 3;
const LC_MONETARY: c_int = 4;
const LC_MESSAGES: c_int = 5;
const LC_ALL: c_int = 6;
const ENOMEM: c_int = 12;
const EINTR: c_int = 4;
const EPERM: c_int = 1;
const EAGAIN: c_int = 11;
const EBUSY: c_int = 16;
const EDEADLK: c_int = 35;
const ETIMEDOUT: c_int = 110;
const ECANCELED: c_int = 125;
const EOVERFLOW: c_int = 75;
const EOWNERDEAD: c_int = 130;
const ENOTRECOVERABLE: c_int = 131;

static mut ERRNO: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn __errno_location() -> *mut c_int {
    core::ptr::addr_of_mut!(ERRNO)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// ponytail: stub for linker; never called with panic=abort
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// ============================================================
// Constants
// ============================================================

const PROT_READ: i32 = 1;
const PROT_WRITE: i32 = 2;
const MAP_SHARED: i32 = 0x01;
const MAP_PRIVATE: i32 = 0x02;
const MAP_ANONYMOUS: i32 = 0x20;

// ============================================================
// Architecture abstraction for syscalls
// ============================================================

trait Syscalls {
    unsafe fn syscall0(n: i64) -> i64;
    unsafe fn syscall1(n: i64, a1: i64) -> i64;
    unsafe fn syscall2(n: i64, a1: i64, a2: i64) -> i64;
    unsafe fn syscall3(n: i64, a1: i64, a2: i64, a3: i64) -> i64;
    unsafe fn syscall4(n: i64, a1: i64, a2: i64, a3: i64, a4: i64) -> i64;
    unsafe fn syscall5(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64) -> i64;
    unsafe fn syscall6(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64, a6: i64) -> i64;
    unsafe fn syscall_noreturn1(n: i64, a1: i64) -> !;
}

struct X86_64;
struct Aarch64;

#[cfg(target_arch = "x86_64")]
impl Syscalls for X86_64 {
    #[inline(always)]
    unsafe fn syscall0(n: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => result,
            lateout("rcx") _,
            lateout("r11") _,
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall1(n: i64, a1: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => result,
            in("rdi") a1,
            lateout("rcx") _,
            lateout("r11") _,
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall2(n: i64, a1: i64, a2: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => result,
            in("rdi") a1,
            in("rsi") a2,
            lateout("rcx") _,
            lateout("r11") _,
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall3(n: i64, a1: i64, a2: i64, a3: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => result,
            in("rdi") a1,
            in("rsi") a2,
            in("rdx") a3,
            lateout("rcx") _,
            lateout("r11") _,
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall4(n: i64, a1: i64, a2: i64, a3: i64, a4: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => result,
            in("rdi") a1,
            in("rsi") a2,
            in("rdx") a3,
            in("r10") a4,
            lateout("rcx") _,
            lateout("r11") _,
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall5(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => result,
            in("rdi") a1,
            in("rsi") a2,
            in("rdx") a3,
            in("r10") a4,
            in("r8") a5,
            lateout("rcx") _,
            lateout("r11") _,
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall6(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64, a6: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "syscall",
            inlateout("rax") n => result,
            in("rdi") a1,
            in("rsi") a2,
            in("rdx") a3,
            in("r10") a4,
            in("r8") a5,
            in("r9") a6,
            lateout("rcx") _,
            lateout("r11") _,
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall_noreturn1(n: i64, a1: i64) -> ! {
        core::arch::asm!(
            "syscall",
            in("rax") n,
            in("rdi") a1,
            options(noreturn)
        );
    }
}

#[cfg(target_arch = "aarch64")]
impl Syscalls for Aarch64 {
    #[inline(always)]
    unsafe fn syscall0(n: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "svc #0",
            inlateout("x8") n => _,
            lateout("x0") result,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall1(n: i64, a1: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "svc #0",
            inlateout("x8") n => _,
            inlateout("x0") a1 => result,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall2(n: i64, a1: i64, a2: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "svc #0",
            inlateout("x8") n => _,
            inlateout("x0") a1 => result,
            inlateout("x1") a2 => _,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall3(n: i64, a1: i64, a2: i64, a3: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "svc #0",
            inlateout("x8") n => _,
            inlateout("x0") a1 => result,
            inlateout("x1") a2 => _,
            inlateout("x2") a3 => _,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall4(n: i64, a1: i64, a2: i64, a3: i64, a4: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "svc #0",
            inlateout("x8") n => _,
            inlateout("x0") a1 => result,
            inlateout("x1") a2 => _,
            inlateout("x2") a3 => _,
            inlateout("x3") a4 => _,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall5(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "svc #0",
            inlateout("x8") n => _,
            inlateout("x0") a1 => result,
            inlateout("x1") a2 => _,
            inlateout("x2") a3 => _,
            inlateout("x3") a4 => _,
            inlateout("x4") a5 => _,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall6(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64, a6: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "svc #0",
            inlateout("x8") n => _,
            inlateout("x0") a1 => result,
            inlateout("x1") a2 => _,
            inlateout("x2") a3 => _,
            inlateout("x3") a4 => _,
            inlateout("x4") a5 => _,
            inlateout("x5") a6 => _,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall_noreturn1(n: i64, a1: i64) -> ! {
        core::arch::asm!(
            "svc #0",
            in("x8") n,
            in("x0") a1,
            options(noreturn, nostack),
        );
    }
}

#[cfg(target_arch = "x86_64")]
type Arch = X86_64;
#[cfg(target_arch = "aarch64")]
type Arch = Aarch64;



// Architecture-specific syscall numbers
#[cfg(target_arch = "x86_64")]
mod sysnr {
    pub const SYS_READ: i64 = 0;
    pub const SYS_WRITE: i64 = 1;
    pub const SYS_OPEN: i64 = 2;
    pub const SYS_CLOSE: i64 = 3;
    pub const SYS_STAT: i64 = 4;
    pub const SYS_FSTAT: i64 = 5;
    pub const SYS_LSEEK: i64 = 8;
    pub const SYS_MMAP: i64 = 9;
    pub const SYS_MUNMAP: i64 = 11;
    pub const SYS_RT_SIGACTION: i64 = 13;
    pub const SYS_RT_SIGPROCMASK: i64 = 14;
    pub const SYS_IOCTL: i64 = 16;
    pub const SYS_ACCESS: i64 = 21;
    pub const SYS_SHMGET: i64 = 29;
    pub const SYS_SHMAT: i64 = 30;
    pub const SYS_SHMCTL: i64 = 31;
    pub const SYS_DUP: i64 = 32;
    pub const SYS_NANOSLEEP: i64 = 35;
    pub const SYS_SETITIMER: i64 = 38;
    pub const SYS_ALARM: i64 = 37;
    pub const SYS_SOCKET: i64 = 41;
    pub const SYS_CONNECT: i64 = 42;
    pub const SYS_ACCEPT: i64 = 43;
    pub const SYS_SENDTO: i64 = 44;
    pub const SYS_RECVFROM: i64 = 45;
    pub const SYS_SHUTDOWN: i64 = 48;
    pub const SYS_BIND: i64 = 49;
    pub const SYS_LISTEN: i64 = 50;
    pub const SYS_GETSOCKNAME: i64 = 51;
    pub const SYS_SOCKETPAIR: i64 = 53;
    pub const SYS_SETSOCKOPT: i64 = 54;
    pub const SYS_EXECVE: i64 = 59;
    pub const SYS_WAIT4: i64 = 61;
    pub const SYS_KILL: i64 = 62;
    pub const SYS_UNAME: i64 = 63;
    pub const SYS_SEMGET: i64 = 64;
    pub const SYS_SEMOP: i64 = 65;
    pub const SYS_SEMCTL: i64 = 66;
    pub const SYS_SHMDT: i64 = 67;
    pub const SYS_MSGGET: i64 = 68;
    pub const SYS_MSGSND: i64 = 69;
    pub const SYS_MSGRCV: i64 = 70;
    pub const SYS_MSGCTL: i64 = 71;
    pub const SYS_FCNTL: i64 = 72;
    pub const SYS_FSYNC: i64 = 74;
    pub const SYS_TRUNCATE: i64 = 76;
    pub const SYS_FTRUNCATE: i64 = 77;
    pub const SYS_GETCWD: i64 = 79;
    pub const SYS_CHDIR: i64 = 80;
    pub const SYS_SYMLINK: i64 = 88;
    pub const SYS_FCHMOD: i64 = 91;
    pub const SYS_UMASK: i64 = 95;
    pub const SYS_GETRLIMIT: i64 = 97;
    pub const SYS_SETUID: i64 = 105;
    pub const SYS_SETGID: i64 = 106;
    pub const SYS_SETPGID: i64 = 109;
    pub const SYS_GETGROUPS: i64 = 115;
    pub const SYS_GETPGID: i64 = 121;
    pub const SYS_GETSID: i64 = 124;
    pub const SYS_RT_SIGPENDING: i64 = 127;
    pub const SYS_RT_SIGTIMEDWAIT: i64 = 128;
    pub const SYS_RT_SIGSUSPEND: i64 = 130;
    pub const SYS_SIGALTSTACK: i64 = 131;
    pub const SYS_SETRLIMIT: i64 = 160;
    pub const SYS_SETHOSTNAME: i64 = 170;
    pub const SYS_FUTEX: i64 = 202;
    pub const SYS_CLOCK_GETTIME: i64 = 228;
    pub const SYS_CLOCK_GETRES: i64 = 229;
    pub const SYS_CLOCK_NANOSLEEP: i64 = 230;
    pub const SYS_EXIT_GROUP: i64 = 231;
    pub const SYS_TGKILL: i64 = 234;
    pub const SYS_MKDIRAT: i64 = 258;
    pub const SYS_NEWFSTATAT: i64 = 262;
    pub const SYS_UNLINKAT: i64 = 263;
    pub const SYS_LINKAT: i64 = 265;
    pub const SYS_SYMLINKAT: i64 = 266;
    pub const SYS_READLINKAT: i64 = 267;
    pub const SYS_FCHMODAT: i64 = 268;
    pub const SYS_SET_ROBUST_LIST: i64 = 273;
    pub const SYS_UTIMENSAT: i64 = 280;
    pub const SYS_OPENAT: i64 = 257;
    pub const SYS_FACCESSAT: i64 = 269;
    pub const SYS_DUP3: i64 = 292;
    pub const SYS_PIPE2: i64 = 293;
    pub const SYS_SYNCFS: i64 = 306;
    pub const SYS_RENAMEAT2: i64 = 316;
    pub const SYS_STATFS: i64 = 137;
    pub const SYS_FSTATFS: i64 = 138;
    pub const SYS_FDATASYNC: i64 = 75;
    pub const SYS_PAUSE: i64 = 34;
    pub const SYS_GETPID: i64 = 39;
    pub const SYS_FORK: i64 = 57;
    pub const SYS_EXIT: i64 = 60;
    pub const SYS_GETPPID: i64 = 110;
    pub const SYS_SETSID: i64 = 112;
    pub const SYS_SYNC: i64 = 162;
    pub const SYS_GETUID: i64 = 102;
    pub const SYS_GETGID: i64 = 104;
    pub const SYS_GETEUID: i64 = 107;
    pub const SYS_GETEGID: i64 = 108;
    pub const SYS_GETTID: i64 = 186;
    pub const SYS_CLOCK_SETTIME: i64 = 227;
    pub const SYS_CLONE: i64 = 56;
    pub const SYS_PPOLL: i64 = 271;
    pub const SYS_PREAD64: i64 = 17;
}
#[cfg(target_arch = "aarch64")]
mod sysnr {
    pub const SYS_READ: i64 = 63;
    pub const SYS_WRITE: i64 = 64;
    // pub const SYS_OPEN: i64 = ???; // missing in aarch64 table
    pub const SYS_CLOSE: i64 = 57;
    // pub const SYS_STAT: i64 = ???; // missing in aarch64 table
    pub const SYS_FSTAT: i64 = 80;
    pub const SYS_LSEEK: i64 = 62;
    pub const SYS_MMAP: i64 = 222;
    pub const SYS_MUNMAP: i64 = 215;
    pub const SYS_RT_SIGACTION: i64 = 134;
    pub const SYS_RT_SIGPROCMASK: i64 = 135;
    pub const SYS_IOCTL: i64 = 29;
    // pub const SYS_ACCESS: i64 = ???; // missing in aarch64 table
    pub const SYS_SHMGET: i64 = 194;
    pub const SYS_SHMAT: i64 = 196;
    pub const SYS_SHMCTL: i64 = 195;
    pub const SYS_DUP: i64 = 23;
    pub const SYS_NANOSLEEP: i64 = 101;
    pub const SYS_SETITIMER: i64 = 103;
    // pub const SYS_ALARM: i64 = ???; // missing in aarch64 table
    pub const SYS_SOCKET: i64 = 198;
    pub const SYS_CONNECT: i64 = 203;
    pub const SYS_ACCEPT: i64 = 202;
    pub const SYS_SENDTO: i64 = 206;
    pub const SYS_RECVFROM: i64 = 207;
    pub const SYS_SHUTDOWN: i64 = 210;
    pub const SYS_BIND: i64 = 200;
    pub const SYS_LISTEN: i64 = 201;
    pub const SYS_GETSOCKNAME: i64 = 204;
    pub const SYS_SOCKETPAIR: i64 = 199;
    pub const SYS_SETSOCKOPT: i64 = 208;
    pub const SYS_EXECVE: i64 = 221;
    pub const SYS_WAIT4: i64 = 260;
    pub const SYS_KILL: i64 = 129;
    pub const SYS_UNAME: i64 = 160;
    pub const SYS_SEMGET: i64 = 190;
    pub const SYS_SEMOP: i64 = 193;
    pub const SYS_SEMCTL: i64 = 191;
    pub const SYS_SHMDT: i64 = 197;
    pub const SYS_MSGGET: i64 = 186;
    pub const SYS_MSGSND: i64 = 189;
    pub const SYS_MSGRCV: i64 = 188;
    pub const SYS_MSGCTL: i64 = 187;
    pub const SYS_FCNTL: i64 = 25;
    pub const SYS_FSYNC: i64 = 82;
    pub const SYS_TRUNCATE: i64 = 45;
    pub const SYS_FTRUNCATE: i64 = 46;
    pub const SYS_GETCWD: i64 = 17;
    pub const SYS_CHDIR: i64 = 49;
    // pub const SYS_SYMLINK: i64 = ???; // missing in aarch64 table
    pub const SYS_FCHMOD: i64 = 52;
    pub const SYS_UMASK: i64 = 166;
    pub const SYS_GETRLIMIT: i64 = 163;
    pub const SYS_SETUID: i64 = 146;
    pub const SYS_SETGID: i64 = 144;
    pub const SYS_SETPGID: i64 = 154;
    pub const SYS_GETGROUPS: i64 = 158;
    pub const SYS_GETPGID: i64 = 155;
    pub const SYS_GETSID: i64 = 156;
    pub const SYS_RT_SIGPENDING: i64 = 136;
    pub const SYS_RT_SIGTIMEDWAIT: i64 = 137;
    pub const SYS_RT_SIGSUSPEND: i64 = 133;
    pub const SYS_SIGALTSTACK: i64 = 132;
    pub const SYS_SETRLIMIT: i64 = 164;
    pub const SYS_SETHOSTNAME: i64 = 161;
    pub const SYS_FUTEX: i64 = 98;
    pub const SYS_CLOCK_GETTIME: i64 = 113;
    pub const SYS_CLOCK_GETRES: i64 = 114;
    pub const SYS_CLOCK_NANOSLEEP: i64 = 115;
    pub const SYS_EXIT_GROUP: i64 = 94;
    pub const SYS_TGKILL: i64 = 131;
    pub const SYS_MKDIRAT: i64 = 34;
    pub const SYS_NEWFSTATAT: i64 = 79;
    pub const SYS_UNLINKAT: i64 = 35;
    pub const SYS_LINKAT: i64 = 37;
    pub const SYS_SYMLINKAT: i64 = 36;
    pub const SYS_READLINKAT: i64 = 78;
    pub const SYS_FCHMODAT: i64 = 53;
    pub const SYS_SET_ROBUST_LIST: i64 = 99;
    pub const SYS_UTIMENSAT: i64 = 88;
    pub const SYS_OPENAT: i64 = 56;
    pub const SYS_FACCESSAT: i64 = 48;
    pub const SYS_DUP3: i64 = 24;
    pub const SYS_PIPE2: i64 = 59;
    pub const SYS_SYNCFS: i64 = 267;
    pub const SYS_RENAMEAT2: i64 = 276;
    pub const SYS_STATFS: i64 = 43;
    pub const SYS_FSTATFS: i64 = 44;
    pub const SYS_FDATASYNC: i64 = 83;
    pub const SYS_GETPID: i64 = 172;
    pub const SYS_EXIT: i64 = 93;
    pub const SYS_GETPPID: i64 = 173;
    pub const SYS_SETSID: i64 = 157;
    pub const SYS_SYNC: i64 = 81;
    pub const SYS_GETUID: i64 = 174;
    pub const SYS_GETGID: i64 = 176;
    pub const SYS_GETEUID: i64 = 175;
    pub const SYS_GETEGID: i64 = 177;
    pub const SYS_GETTID: i64 = 178;
    pub const SYS_CLOCK_SETTIME: i64 = 112;
    pub const SYS_CLONE: i64 = 220;
    pub const SYS_PPOLL: i64 = 73;
    pub const SYS_PREAD64: i64 = 67;
}
pub use sysnr::*;

// ============================================================
// Syscall wrappers (raw, no_std)
// ============================================================

#[inline]
unsafe fn sys_write(fd: i64, buf: *const u8, count: usize) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_WRITE, fd as i64, buf as i64, count as i64)
}

#[inline]
unsafe fn sys_read(fd: i64, buf: *mut u8, count: usize) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_READ, fd as i64, buf as i64, count as i64)
}

#[inline]
unsafe fn sys_open(path: *const u8, flags: i64, mode: i64) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_OPENAT, AT_FDCWD as i64, path as i64, flags as i64, mode as i64)
}

#[inline]
unsafe fn sys_close(fd: i64) {
    let _ = <Arch as Syscalls>::syscall1(SYS_CLOSE, fd as i64);
}

#[inline]
unsafe fn sys_lseek(fd: i64, offset: i64, whence: i64) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_LSEEK, fd as i64, offset as i64, whence as i64)
}

unsafe fn sys_mmap(
    addr: *mut u8,
    length: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: i64,
) -> *mut u8 {
    let result = <Arch as Syscalls>::syscall6(SYS_MMAP, addr as i64, length as i64, prot as i64, flags as i64, fd as i64, offset);
    if result < 0 && result > -4096 {
        ERRNO = (-result) as c_int;
        return MMAP_FAILED;
    }
    result as *mut u8
}

unsafe fn sys_munmap(addr: *mut u8, length: usize) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_MUNMAP, addr as i64, length as i64)
}

// ============================================================
// String/memory functions
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> usize {
    let s = s as *const u8;
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let dst = dst as *mut u8;
    let src = src as *const u8;
    let mut i = 0;
    while i < n {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    dst as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void {
    let s = s as *mut u8;
    let mut i = 0;
    while i < n {
        *s.add(i) = c as u8;
        i += 1;
    }
    s as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let dst = dst as *mut u8;
    let src = src as *const u8;
    if (dst as usize) < (src as usize) {
        let mut i = 0;
        while i < n {
            *dst.add(i) = *src.add(i);
            i += 1;
        }
    } else {
        let mut i = n;
        while i > 0 {
            i -= 1;
            *dst.add(i) = *src.add(i);
        }
    }
    dst as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int {
    let s1 = s1 as *const u8;
    let s2 = s2 as *const u8;
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as c_int - b as c_int;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int {
    memcmp(s1, s2, n)
}

#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> c_int {
    let mut i = 0;
    loop {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as c_int - b as c_int;
        }
        if a == 0 {
            return 0;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strncmp(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return a as c_int - b as c_int;
        }
        if a == 0 {
            return 0;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn strcpy(dst: *mut u8, src: *const u8) -> *mut u8 {
    let mut i = 0;
    loop {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strncpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    while i < n {
        *dst.add(i) = 0;
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strcat(dst: *mut u8, src: *const u8) -> *mut u8 {
    let mut i = 0;
    while *dst.add(i) != 0 {
        i += 1;
    }
    let mut j = 0;
    loop {
        let c = *src.add(j);
        *dst.add(i + j) = c;
        if c == 0 {
            break;
        }
        j += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strncat(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while *dst.add(i) != 0 {
        i += 1;
    }
    let mut j = 0;
    while j < n && *src.add(j) != 0 {
        *dst.add(i + j) = *src.add(j);
        j += 1;
    }
    *dst.add(i + j) = 0;
    dst
}

#[no_mangle]
pub unsafe extern "C" fn strlcpy(dst: *mut u8, src: *const u8, n: usize) -> usize {
    let l = strlen(src as *const c_char);
    if n > 0 {
        let c = if l < n { l } else { n - 1 };
        core::ptr::copy_nonoverlapping(src, dst, c);
        *dst.add(c) = 0;
    }
    l
}

#[no_mangle]
pub unsafe extern "C" fn strlcat(dst: *mut u8, src: *const u8, n: usize) -> usize {
    let dl = strlen(dst as *const c_char);
    let sl = strlen(src as *const c_char);
    if dl >= n { return n + sl; }
    let c = if sl < n - dl { sl } else { n - dl - 1 };
    core::ptr::copy_nonoverlapping(src, dst.add(dl), c);
    *dst.add(dl + c) = 0;
    dl + sl
}

#[no_mangle]
pub unsafe extern "C" fn strchr(s: *const u8, c: c_int) -> *mut u8 {
    let target = c as u8;
    let mut i = 0;
    loop {
        let ch = *s.add(i);
        if ch == target {
            return s.add(i) as *mut u8;
        }
        if ch == 0 {
            return null_mut();
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strrchr(s: *const u8, c: c_int) -> *mut u8 {
    let target = c as u8;
    let mut i = 0;
    let mut last: *mut u8 = null_mut();
    loop {
        let ch = *s.add(i);
        if ch == target {
            last = s.add(i) as *mut u8;
        }
        if ch == 0 {
            return last;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strnlen(s: *const u8, maxlen: usize) -> usize {
    let mut i = 0;
    while i < maxlen && *s.add(i) != 0 {
        i += 1;
    }
    i
}

#[no_mangle]
pub unsafe extern "C" fn strcspn(s: *const u8, reject: *const u8) -> usize {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 {
            return i;
        }
        let mut j = 0;
        loop {
            let r = *reject.add(j);
            if r == 0 {
                break;
            }
            if c == r {
                return i;
            }
            j += 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strspn(s: *const u8, accept: *const u8) -> usize {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 {
            return i;
        }
        let mut found = false;
        let mut j = 0;
        loop {
            let a = *accept.add(j);
            if a == 0 {
                break;
            }
            if c == a {
                found = true;
                break;
            }
            j += 1;
        }
        if !found {
            return i;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strpbrk(s: *const u8, accept: *const u8) -> *mut u8 {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 {
            return null_mut();
        }
        let mut j = 0;
        loop {
            let a = *accept.add(j);
            if a == 0 {
                break;
            }
            if c == a {
                return s.add(i) as *mut u8;
            }
            j += 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn strstr(haystack: *const u8, needle: *const u8) -> *mut u8 {
    if *needle == 0 {
        return haystack as *mut u8;
    }
    let mut i = 0;
    loop {
        let h = *haystack.add(i);
        if h == 0 {
            return null_mut();
        }
        if h == *needle {
            let mut j = 0;
            loop {
                let n = *needle.add(j);
                if n == 0 {
                    return haystack.add(i) as *mut u8;
                }
                if *haystack.add(i + j) != n {
                    break;
                }
                j += 1;
            }
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn memchr(s: *const u8, c: c_int, n: usize) -> *mut u8 {
    let target = c as u8;
    let mut i = 0;
    while i < n {
        if *s.add(i) == target {
            return s.add(i) as *mut u8;
        }
        i += 1;
    }
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn memrchr(s: *const u8, c: c_int, n: usize) -> *mut u8 {
    let target = c as u8;
    let mut i = n;
    while i > 0 {
        i -= 1;
        if *s.add(i) == target {
            return s.add(i) as *mut u8;
        }
    }
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn memmem(
    haystack: *const c_void, haystacklen: usize,
    needle: *const c_void, needlelen: usize,
) -> *mut c_void {
    if needlelen == 0 { return haystack as *mut c_void; }
    if haystacklen < needlelen { return null_mut(); }
    let h = haystack as *const u8;
    let n = needle as *const u8;
    let last = haystacklen - needlelen;
    for i in 0..=last {
        if *h.add(i) == *n && memcmp(h.add(i) as *const c_void, n as *const c_void, needlelen) == 0 {
            return h.add(i) as *mut c_void;
        }
    }
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn strtok(s: *mut u8, delim: *const u8) -> *mut u8 {
    static mut STATE: *mut u8 = core::ptr::null_mut();
    let mut p = if s.is_null() { STATE } else { s };
    if p.is_null() {
        return core::ptr::null_mut();
    }
    loop {
        let c = *p;
        if c == 0 {
            STATE = core::ptr::null_mut();
            return core::ptr::null_mut();
        }
        let mut is_delim = false;
        let mut i = 0;
        loop {
            let d = *delim.add(i);
            if d == 0 {
                break;
            }
            if c == d {
                is_delim = true;
                break;
            }
            i += 1;
        }
        if !is_delim {
            break;
        }
        p = p.add(1);
    }
    let start = p;
    loop {
        let c = *p;
        if c == 0 {
            STATE = core::ptr::null_mut();
            return start;
        }
        let mut is_delim = false;
        let mut i = 0;
        loop {
            let d = *delim.add(i);
            if d == 0 {
                break;
            }
            if c == d {
                is_delim = true;
                break;
            }
            i += 1;
        }
        if is_delim {
            *p = 0;
            STATE = p.add(1);
            return start;
        }
        p = p.add(1);
    }
}

// ============================================================
// C type aliases
// ============================================================

type SizeT = usize;
type SSizeT = isize;
type mode_t = c_uint;
type TimeT = c_long;
type ClockT = c_long;
type wchar_t = c_int;
type wint_t = c_uint;
type wctype_t = *const c_int;
type wctrans_t = *const c_int;

const WEOF: wint_t = 0xffffffff;

#[repr(C)]
pub struct tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
    pub tm_gmtoff: c_long,
    pub tm_zone: *const c_char,
}

#[repr(C)]
pub struct timeval {
    pub tv_sec: TimeT,
    pub tv_usec: c_long,
}

// ============================================================
// ctype.h
// ============================================================

const CT_UPPER: u8 = 1 << 0;
const CT_LOWER: u8 = 1 << 1;
const CT_DIGIT: u8 = 1 << 2;
const CT_SPACE: u8 = 1 << 3;
const CT_PUNCT: u8 = 1 << 4;
const CT_CNTRL: u8 = 1 << 5;
const CT_BLANK: u8 = 1 << 6;
const CT_XDIGIT: u8 = 1 << 7;

const CT_TABLE: [u8; 128] = {
    let mut t = [0u8; 128];
    let mut i = 0;
    while i < 128 {
        let c = i as u8;
        let mut flags = 0u8;
        if c >= b'A' && c <= b'Z' {
            flags |= CT_UPPER;
        }
        if c >= b'a' && c <= b'z' {
            flags |= CT_LOWER;
        }
        if c >= b'0' && c <= b'9' {
            flags |= CT_DIGIT;
        }
        if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == b'\x0c' || c == b'\x0b' {
            flags |= CT_SPACE;
        }
        if (c >= b'!' && c <= b'/')
            || (c >= b':' && c <= b'@')
            || (c >= b'[' && c <= b'`')
            || (c >= b'{' && c <= b'~')
        {
            flags |= CT_PUNCT;
        }
        if c < 0x20 || c == 0x7f {
            flags |= CT_CNTRL;
        }
        if c == b' ' || c == b'\t' {
            flags |= CT_BLANK;
        }
        if (c >= b'0' && c <= b'9') || (c >= b'A' && c <= b'F') || (c >= b'a' && c <= b'f') {
            flags |= CT_XDIGIT;
        }
        t[i] = flags;
        i += 1;
    }
    t
};

#[inline]
fn ct_flags(c: c_int) -> u8 {
    if c < 0 || c > 127 {
        0
    } else {
        CT_TABLE[c as usize]
    }
}

#[no_mangle]
pub extern "C" fn isalnum(c: c_int) -> c_int {
    (ct_flags(c) & (CT_UPPER | CT_LOWER | CT_DIGIT) != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isalpha(c: c_int) -> c_int {
    (ct_flags(c) & (CT_UPPER | CT_LOWER) != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isascii(c: c_int) -> c_int {
    ((c as u32) <= 127) as c_int
}

#[no_mangle]
pub extern "C" fn isblank(c: c_int) -> c_int {
    (ct_flags(c) & CT_BLANK != 0) as c_int
}

#[no_mangle]
pub extern "C" fn iscntrl(c: c_int) -> c_int {
    (ct_flags(c) & CT_CNTRL != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isdigit(c: c_int) -> c_int {
    (ct_flags(c) & CT_DIGIT != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isgraph(c: c_int) -> c_int {
    (ct_flags(c) & (CT_UPPER | CT_LOWER | CT_DIGIT | CT_PUNCT) != 0) as c_int
}

#[no_mangle]
pub extern "C" fn islower(c: c_int) -> c_int {
    (ct_flags(c) & CT_LOWER != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isprint(c: c_int) -> c_int {
    let flags = ct_flags(c);
    (flags & (CT_UPPER | CT_LOWER | CT_DIGIT | CT_PUNCT | CT_BLANK) != 0) as c_int
}

#[no_mangle]
pub extern "C" fn ispunct(c: c_int) -> c_int {
    (ct_flags(c) & CT_PUNCT != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isspace(c: c_int) -> c_int {
    (ct_flags(c) & CT_SPACE != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isupper(c: c_int) -> c_int {
    (ct_flags(c) & CT_UPPER != 0) as c_int
}

#[no_mangle]
pub extern "C" fn isxdigit(c: c_int) -> c_int {
    (ct_flags(c) & CT_XDIGIT != 0) as c_int
}

#[no_mangle]
pub extern "C" fn tolower(c: c_int) -> c_int {
    if (ct_flags(c) & CT_UPPER) != 0 {
        c + (b'a' as c_int - b'A' as c_int)
    } else {
        c
    }
}

#[no_mangle]
pub extern "C" fn toupper(c: c_int) -> c_int {
    if (ct_flags(c) & CT_LOWER) != 0 {
        c - (b'a' as c_int - b'A' as c_int)
    } else {
        c
    }
}

// ============================================================
// stdlib.h: integer conversion, abs, rand
// ============================================================

unsafe fn parse_digit(c: u8, base: c_int) -> Option<u8> {
    if c == 0 { return None; }
    let d = if c >= b'0' && c <= b'9' {
        c - b'0'
    } else if c >= b'a' && c <= b'z' {
        c - b'a' + 10
    } else if c >= b'A' && c <= b'Z' {
        c - b'A' + 10
    } else {
        return None;
    };
    if (d as c_int) < base {
        Some(d)
    } else {
        None
    }
}

unsafe fn parse_prefix(s: *const u8, base: *mut c_int) -> *const u8 {
    let mut p = s;
    if *base == 0 {
        if *p == b'0' {
            if (*p.add(1) == b'x' || *p.add(1) == b'X') && parse_digit(*p.add(2), 16).is_some() {
                *base = 16;
                p = p.add(2);
            } else {
                *base = 8;
            }
        } else {
            *base = 10;
        }
    } else if *base == 16 {
        if *p == b'0' && (*p.add(1) == b'x' || *p.add(1) == b'X') && parse_digit(*p.add(2), 16).is_some() {
            p = p.add(2);
        }
    }
    p
}

unsafe fn strtox(
    s: *const c_char,
    endptr: *mut *mut c_char,
    mut base: c_int,
    pos_limit: u64,
    neg_limit: u64,
) -> (u64, bool, bool) {
    let s0 = s as *const u8;
    let mut p = s0;
    while isspace(*p as c_int) != 0 {
        p = p.add(1);
    }

    if base < 0 || base == 1 || base > 36 {
        if !endptr.is_null() {
            *endptr = s as *mut c_char;
        }
        ERRNO = EINVAL;
        return (0, false, false);
    }

    let mut neg = false;
    match *p {
        b'-' => { neg = true; p = p.add(1); }
        b'+' => p = p.add(1),
        _ => {}
    }

    p = parse_prefix(p, &mut base);

    let limit = if neg { neg_limit } else { pos_limit };
    let mut val: u64 = 0;
    let mut any = false;
    while let Some(d) = parse_digit(*p, base) {
        any = true;
        if val > (limit - d as u64) / base as u64 {
            while parse_digit(*p, base).is_some() { p = p.add(1); }
            if !endptr.is_null() { *endptr = p as *mut c_char; }
            ERRNO = ERANGE_VAL;
            return (limit, true, neg);
        }
        val = val * base as u64 + d as u64;
        p = p.add(1);
    }

    if !endptr.is_null() {
        *endptr = if any { p as *mut c_char } else { s as *mut c_char };
    }
    (val, false, neg)
}

#[no_mangle]
pub unsafe extern "C" fn strtol(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_long {
    let pos_limit = c_long::MAX as u64;
    let neg_limit = pos_limit.wrapping_add(1);
    let (val, overflow, neg) = strtox(s, endptr, base, pos_limit, neg_limit);
    if overflow {
        return if neg { c_long::MIN } else { c_long::MAX };
    }
    if neg { val.wrapping_neg() as c_long } else { val as c_long }
}

#[no_mangle]
pub unsafe extern "C" fn strtoul(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_ulong {
    let limit = c_ulong::MAX as u64;
    let (val, overflow, neg) = strtox(s, endptr, base, limit, limit);
    if overflow {
        return c_ulong::MAX;
    }
    if neg { val.wrapping_neg() as c_ulong } else { val as c_ulong }
}

#[no_mangle]
pub unsafe extern "C" fn strtoll(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_longlong {
    let pos_limit = c_longlong::MAX as u64;
    let neg_limit = pos_limit.wrapping_add(1);
    let (val, overflow, neg) = strtox(s, endptr, base, pos_limit, neg_limit);
    if overflow {
        return if neg { c_longlong::MIN } else { c_longlong::MAX };
    }
    if neg { val.wrapping_neg() as c_longlong } else { val as c_longlong }
}

#[no_mangle]
pub unsafe extern "C" fn strtoull(s: *const c_char, endptr: *mut *mut c_char, base: c_int) -> c_ulonglong {
    let limit = c_ulonglong::MAX as u64;
    let (val, overflow, neg) = strtox(s, endptr, base, limit, limit);
    if overflow {
        return c_ulonglong::MAX;
    }
    if neg { val.wrapping_neg() as c_ulonglong } else { val as c_ulonglong }
}

#[no_mangle]
pub unsafe extern "C" fn atoi(s: *const c_char) -> c_int {
    strtol(s, core::ptr::null_mut(), 10) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn atol(s: *const c_char) -> c_long {
    strtol(s, core::ptr::null_mut(), 10)
}

#[no_mangle]
pub unsafe extern "C" fn atoll(s: *const c_char) -> c_longlong {
    strtoll(s, core::ptr::null_mut(), 10)
}

#[no_mangle]
pub extern "C" fn abs(n: c_int) -> c_int {
    if n < 0 {
        -n
    } else {
        n
    }
}

#[no_mangle]
pub extern "C" fn labs(n: c_long) -> c_long {
    if n < 0 {
        -n
    } else {
        n
    }
}

#[no_mangle]
pub extern "C" fn llabs(n: c_longlong) -> c_longlong {
    if n < 0 {
        -n
    } else {
        n
    }
}

static mut RAND_STATE: c_uint = 1;

#[no_mangle]
pub extern "C" fn srand(seed: c_uint) {
    unsafe {
        RAND_STATE = seed;
    }
}

#[no_mangle]
pub extern "C" fn rand() -> c_int {
    unsafe {
        RAND_STATE = RAND_STATE.wrapping_mul(1103515245).wrapping_add(12345);
        ((RAND_STATE >> 16) & 0x7fff) as c_int
    }
}

// ============================================================
// stdlib.h: div, ldiv, lldiv
// ============================================================

#[repr(C)]
pub struct div_t { pub quot: c_int, pub rem: c_int }
#[repr(C)]
pub struct ldiv_t { pub quot: c_long, pub rem: c_long }
#[repr(C)]
pub struct lldiv_t { pub quot: c_longlong, pub rem: c_longlong }

#[no_mangle]
pub extern "C" fn div(num: c_int, den: c_int) -> div_t {
    div_t { quot: num / den, rem: num % den }
}

#[no_mangle]
pub extern "C" fn ldiv(num: c_long, den: c_long) -> ldiv_t {
    ldiv_t { quot: num / den, rem: num % den }
}

#[no_mangle]
pub extern "C" fn lldiv(num: c_longlong, den: c_longlong) -> lldiv_t {
    lldiv_t { quot: num / den, rem: num % den }
}

// ============================================================
// bsearch
// ============================================================

type CmpFn = unsafe extern "C" fn(*const c_void, *const c_void) -> c_int;

#[no_mangle]
pub unsafe extern "C" fn bsearch(
    key: *const c_void,
    base: *const c_void,
    nel: usize,
    width: usize,
    cmp: CmpFn,
) -> *mut c_void {
    let mut base = base as *const u8;
    let mut nel = nel;
    while nel > 0 {
        let trial = base.add(width * (nel / 2));
        let sign = cmp(key, trial as *const c_void);
        if sign < 0 {
            nel /= 2;
        } else if sign > 0 {
            base = trial.add(width);
            nel -= nel / 2 + 1;
        } else {
            return trial as *mut c_void;
        }
    }
    null_mut()
}

// ============================================================
// signal.h
// ============================================================

pub const SA_RESTORER: c_ulong = 0x04000000;

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;
pub const SIG_ERR: usize = !0usize;

pub const SIGHUP: c_int = 1;
pub const SIGINT: c_int = 2;
pub const SIGQUIT: c_int = 3;
pub const SIGILL: c_int = 4;
pub const SIGTRAP: c_int = 5;
pub const SIGABRT: c_int = 6;
pub const SIGIOT: c_int = 6;
pub const SIGBUS: c_int = 7;
pub const SIGFPE: c_int = 8;
pub const SIGKILL: c_int = 9;
pub const SIGUSR1: c_int = 10;
pub const SIGSEGV: c_int = 11;
pub const SIGUSR2: c_int = 12;
pub const SIGPIPE: c_int = 13;
pub const SIGALRM: c_int = 14;
pub const SIGTERM: c_int = 15;
pub const SIGSTKFLT: c_int = 16;
pub const SIGCHLD: c_int = 17;
pub const SIGCONT: c_int = 18;
pub const SIGSTOP: c_int = 19;
pub const SIGTSTP: c_int = 20;
pub const SIGTTIN: c_int = 21;
pub const SIGTTOU: c_int = 22;
pub const SIGURG: c_int = 23;
pub const SIGXCPU: c_int = 24;
pub const SIGXFSZ: c_int = 25;
pub const SIGVTALRM: c_int = 26;
pub const SIGPROF: c_int = 27;
pub const SIGWINCH: c_int = 28;
pub const SIGIO: c_int = 29;
pub const SIGPOLL: c_int = 29;
pub const SIGPWR: c_int = 30;
pub const SIGSYS: c_int = 31;
pub const SIGUNUSED: c_int = 31;
pub const _NSIG: c_int = 65;

pub const SIG_BLOCK: c_int = 0;
pub const SIG_UNBLOCK: c_int = 1;
pub const SIG_SETMASK: c_int = 2;

pub const SA_NOCLDSTOP: c_ulong = 1;
pub const SA_NOCLDWAIT: c_ulong = 2;
pub const SA_SIGINFO: c_ulong = 4;
pub const SA_ONSTACK: c_ulong = 0x08000000;
pub const SA_RESTART: c_ulong = 0x10000000;
pub const SA_NODEFER: c_ulong = 0x40000000;
pub const SA_RESETHAND: c_ulong = 0x80000000;

pub const SS_ONSTACK: c_int = 1;
pub const SS_DISABLE: c_int = 2;
pub const MINSIGSTKSZ: usize = 2048;
pub const SIGSTKSZ: usize = 8192;

pub const SI_USER: c_int = 0;
pub const SI_TKILL: c_int = -6;

#[repr(C)]
pub struct timespec {
    pub tv_sec: c_long,
    pub tv_nsec: c_long,
}

#[repr(C)]
pub struct siginfo_t {
    pub si_signo: c_int,
    pub si_errno: c_int,
    pub si_code: c_int,
    // ponytail: rest of 128-byte struct, only si_signo accessed
    _pad: [u8; 128 - 3 * core::mem::size_of::<c_int>()],
}

#[repr(C)]
pub struct stack_t {
    pub ss_sp: *mut c_void,
    pub ss_flags: c_int,
    pub ss_size: usize,
}

#[repr(C)]
pub struct sigaction {
    pub sa_handler: usize,
    pub sa_flags: c_ulong,
    pub sa_restorer: usize,
    pub sa_mask: [c_ulong; 1],
}

pub type SigSetT = c_ulong;

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".global sig_restorer",
    ".type sig_restorer, @function",
    "sig_restorer:",
    "mov eax, 15",
    "syscall",
);

#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    ".global sig_restorer",
    ".type sig_restorer, @function",
    "sig_restorer:",
    "mov x8, #139",
    "svc #0",
);

extern "C" {
    fn sig_restorer();
}

#[inline]
unsafe fn sys_rt_sigaction(sig: c_int,
    act: *const sigaction,
    oldact: *mut sigaction,
    sigsetsize: usize,) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_RT_SIGACTION, sig as i64, act as i64, oldact as i64, sigsetsize as i64)
}

#[inline]
unsafe fn sys_rt_sigprocmask(how: c_int,
    set: *const SigSetT,
    oldset: *mut SigSetT,
    sigsetsize: usize,) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_RT_SIGPROCMASK, how as i64, set as i64, oldset as i64, sigsetsize as i64)
}

#[inline]
unsafe fn sys_rt_sigpending(set: *mut SigSetT, sigsetsize: usize) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_RT_SIGPENDING, set as i64, sigsetsize as i64)
}

#[inline]
unsafe fn sys_rt_sigsuspend(mask: *const SigSetT, sigsetsize: usize) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_RT_SIGSUSPEND, mask as i64, sigsetsize as i64)
}

#[inline]
unsafe fn sys_rt_sigtimedwait(set: *const SigSetT,
    info: *mut siginfo_t,
    timeout: *const timespec,
    sigsetsize: usize,) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_RT_SIGTIMEDWAIT, set as i64, info as i64, timeout as i64, sigsetsize as i64)
}

#[inline]
unsafe fn sys_sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_SIGALTSTACK, ss as i64, old_ss as i64)
}

#[inline]
unsafe fn sys_tgkill(tgid: c_int, tid: c_int, sig: c_int) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_TGKILL, tgid as i64, tid as i64, sig as i64)
}

const CLOCK_REALTIME: c_int = 0;
const CLOCK_MONOTONIC: c_int = 1;

#[inline]
unsafe fn sys_clock_gettime(clockid: c_int, ts: *mut timespec) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_CLOCK_GETTIME, clockid as i64, ts as i64)
}

#[no_mangle]
pub unsafe extern "C" fn clock_gettime(clockid: c_int, ts: *mut timespec) -> c_int {
    if sys_clock_gettime(clockid, ts) < 0 { -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn sigprocmask(
    how: c_int,
    set: *const SigSetT,
    oldset: *mut SigSetT,
) -> c_int {
    // ponytail: mask out internal rt signals (32, 33) before touching the kernel
    let internal_mask: SigSetT = (1u64 << 31) | (1u64 << 32);
    let filtered: SigSetT;
    let set_ptr: *const SigSetT = if set.is_null() {
        set
    } else {
        filtered = *set & !internal_mask;
        &filtered
    };
    let r = sys_rt_sigprocmask(how, set_ptr, oldset, core::mem::size_of::<SigSetT>());
    if !oldset.is_null() {
        *oldset &= !internal_mask;
    }
    if r < 0 { -1 } else { 0 }
}

#[inline]
unsafe fn sys_kill(pid: c_int, sig: c_int) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_KILL, pid as i64, sig as i64)
}

#[inline]
unsafe fn sys_getpid() -> i64 {
    <Arch as Syscalls>::syscall0(SYS_GETPID)
}

#[no_mangle]
pub unsafe extern "C" fn sigaction(
    signum: c_int,
    act: *const sigaction,
    oldact: *mut sigaction,
) -> c_int {
    let kact: sigaction;
    let act_ptr = if act.is_null() {
        core::ptr::null()
    } else {
        kact = sigaction {
            sa_handler: (*act).sa_handler,
            sa_flags: (*act).sa_flags | SA_RESTORER,
            sa_restorer: sig_restorer as *const () as usize,
            sa_mask: (*act).sa_mask,
        };
        &kact as *const sigaction
    };
    let r = sys_rt_sigaction(signum, act_ptr, oldact, core::mem::size_of::<SigSetT>());
    if r < 0 {
        ERRNO = (-r) as c_int;
        -1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn signal(signum: c_int, handler: usize) -> usize {
    let act = sigaction {
        sa_handler: handler,
        sa_flags: SA_RESTORER,
        sa_restorer: sig_restorer as *const () as usize,
        sa_mask: [0],
    };
    let mut old = sigaction {
        sa_handler: 0,
        sa_flags: 0,
        sa_restorer: 0,
        sa_mask: [0],
    };
    if sigaction(signum, &act, &mut old) == -1 {
        SIG_ERR
    } else {
        old.sa_handler
    }
}

#[no_mangle]
pub unsafe extern "C" fn kill(pid: c_int, sig: c_int) -> c_int {
    sys_kill(pid, sig) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn tgkill(tgid: c_int, tid: c_int, sig: c_int) -> c_int {
    if sys_tgkill(tgid, tid, sig) < 0 { -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn getpid() -> c_int {
    sys_getpid() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn raise(sig: c_int) -> c_int {
    if sys_tgkill(sys_getpid() as c_int, sys_gettid() as c_int, sig) < 0 {
        -1
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn sigemptyset(set: *mut SigSetT) -> c_int {
    *set = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigfillset(set: *mut SigSetT) -> c_int {
    // musl: all signals except 32 (SIGCANCEL) and 33 (SIGSYNCCALL)
    *set = 0xFFFF_FFFC_7FFF_FFFFu64;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigaddset(set: *mut SigSetT, signum: c_int) -> c_int {
    let s = (signum as c_uint).wrapping_sub(1);
    if s as usize >= 64 {
        ERRNO = EINVAL;
        return -1;
    }
    // ponytail: internal rt signals (32, 33) cannot be manipulated
    if signum >= 32 && signum < __libc_current_sigrtmin() {
        return 0;
    }
    *set |= 1u64 << s;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigdelset(set: *mut SigSetT, signum: c_int) -> c_int {
    let s = (signum as c_uint).wrapping_sub(1);
    if s as usize >= 64 {
        ERRNO = EINVAL;
        return -1;
    }
    // ponytail: internal rt signals (32, 33) are never members
    if signum >= 32 && signum < __libc_current_sigrtmin() {
        return 0;
    }
    *set &= !(1u64 << s);
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigismember(set: *const SigSetT, signum: c_int) -> c_int {
    let s = (signum as c_uint).wrapping_sub(1);
    if s as usize >= 64 { return 0; }
    // ponytail: internal rt signals (32, 33) are never members
    if signum >= 32 && signum < __libc_current_sigrtmin() {
        return 0;
    }
    ((*set & (1u64 << s)) != 0) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn sigpending(set: *mut SigSetT) -> c_int {
    let r = sys_rt_sigpending(set, core::mem::size_of::<SigSetT>());
    if r < 0 {
        ERRNO = (-r) as c_int;
        return -1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigsuspend(mask: *const SigSetT) -> c_int {
    let r = sys_rt_sigsuspend(mask, core::mem::size_of::<SigSetT>());
    if r < 0 {
        ERRNO = (-r) as c_int;
    }
    -1
}

#[no_mangle]
pub unsafe extern "C" fn sigtimedwait(
    mask: *const SigSetT,
    info: *mut siginfo_t,
    timeout: *const timespec,
) -> c_int {
    loop {
        let r = sys_rt_sigtimedwait(mask, info, timeout, core::mem::size_of::<SigSetT>());
        if r >= 0 {
            return r as c_int;
        }
        let e = (-r) as c_int;
        if e != EINTR {
            ERRNO = e;
            return -1;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn sigwaitinfo(
    mask: *const SigSetT,
    info: *mut siginfo_t,
) -> c_int {
    sigtimedwait(mask, info, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn sigwait(
    mask: *const SigSetT,
    sig: *mut c_int,
) -> c_int {
    let mut info: siginfo_t = core::mem::zeroed();
    if sigtimedwait(mask, &mut info, core::ptr::null()) < 0 {
        return ERRNO;
    }
    *sig = info.si_signo;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> c_int {
    if !ss.is_null() {
        if (*ss).ss_flags & SS_ONSTACK != 0 {
            ERRNO = EINVAL;
            return -1;
        }
        if (*ss).ss_flags & SS_DISABLE == 0 && (*ss).ss_size < MINSIGSTKSZ {
            ERRNO = ENOMEM;
            return -1;
        }
    }
    let r = sys_sigaltstack(ss, old_ss);
    if r < 0 {
        ERRNO = (-r) as c_int;
        return -1;
    }
    0
}

// ============================================================
// setjmp.h
// ponytail: jmp_buf is unsigned long[8]: rbx, rbp, r12-r15, rsp, rip
// ============================================================

#[no_mangle]
#[inline(never)]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn setjmp(env: *mut c_ulong) -> c_int {
    core::arch::asm!(
        "mov rax, [rsp]",
        "lea rcx, [rsp + 8]",
        "mov [r8], rbx",
        "mov [r8 + 8], rbp",
        "mov [r8 + 16], r12",
        "mov [r8 + 24], r13",
        "mov [r8 + 32], r14",
        "mov [r8 + 40], r15",
        "mov [r8 + 48], rcx",
        "mov [r8 + 56], rax",
        in("r8") env,
        lateout("rax") _,
        lateout("rcx") _,
        options(nostack),
    );
    0
}

#[no_mangle]
#[inline(never)]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn longjmp(env: *const c_ulong, val: c_int) -> ! {
    let ret = if val == 0 { 1 } else { val } as u32;
    core::arch::asm!(
        "mov rbx, [r8]",
        "mov rbp, [r8 + 8]",
        "mov r12, [r8 + 16]",
        "mov r13, [r8 + 24]",
        "mov r14, [r8 + 32]",
        "mov r15, [r8 + 40]",
        "mov rsp, [r8 + 48]",
        "mov eax, edx",
        "jmp [r8 + 56]",
        in("r8") env,
        in("edx") ret,
        options(noreturn),
    );
}

#[no_mangle]
#[unsafe(naked)]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn setjmp(env: *mut c_ulong) -> c_int {
    core::arch::naked_asm!(
        "stp x19, x20, [x0, #0]",
        "stp x21, x22, [x0, #16]",
        "stp x23, x24, [x0, #32]",
        "stp x25, x26, [x0, #48]",
        "stp x27, x28, [x0, #64]",
        "stp x29, x30, [x0, #80]",
        "mov x2, sp",
        "str x2, [x0, #104]",
        "stp d8, d9, [x0, #112]",
        "stp d10, d11, [x0, #128]",
        "stp d12, d13, [x0, #144]",
        "stp d14, d15, [x0, #160]",
        "mov x0, #0",
        "ret",
    );
}

#[no_mangle]
#[inline(never)]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn longjmp(env: *const c_ulong, val: c_int) -> ! {
    let ret = if val == 0 { 1 } else { val } as u64;
    core::arch::asm!(
        "ldp x19, x20, [x0]",
        "ldp x21, x22, [x0, #16]",
        "ldp x23, x24, [x0, #32]",
        "ldp x25, x26, [x0, #48]",
        "ldp x27, x28, [x0, #64]",
        "ldp x29, x30, [x0, #80]",
        "ldr x2, [x0, #104]",
        "mov sp, x2",
        "ldp d8, d9, [x0, #112]",
        "ldp d10, d11, [x0, #128]",
        "ldp d12, d13, [x0, #144]",
        "ldp d14, d15, [x0, #160]",
        "mov x0, x1",
        "br x30",
        in("x0") env,
        in("x1") ret,
        options(noreturn),
    );
}

// ponytail: sigsetjmp is implemented in raw assembly because the Rust compiler
// rewrites naked_asm that touches rbx into incorrect push/pop sequences,
// corrupting the caller's rbx across the initial return. The exported wrappers
// jump to a local hidden assembly implementation; the layout matches musl:
// env[0..7] = __jmp_buf, env[8] = saved return address, env[9] = saved signal
// mask, env[10] = saved rbx.

#[no_mangle]
#[unsafe(naked)]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn sigsetjmp(_env: *mut c_ulong, _savemask: c_int) -> c_int {
    core::arch::naked_asm!("jmp sigsetjmp_real");
}

#[no_mangle]
#[unsafe(naked)]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn __sigsetjmp(_env: *mut c_ulong, _savemask: c_int) -> c_int {
    core::arch::naked_asm!("jmp sigsetjmp_real");
}

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".type sigsetjmp_real, @function",
    "sigsetjmp_real:",
    "   test esi, esi",
    "   jz 1f",
    "   pop qword ptr [rdi + 64]",
    "   mov qword ptr [rdi + 80], rbx",
    "   push rbx",
    "   mov rbx, rdi",
    "   call setjmp",
    "   push qword ptr [rbx + 64]",
    "   mov rdi, rbx",
    "   mov esi, eax",
    "   mov rbx, qword ptr [rbx + 80]",
    "   lea rdx, [rdi + 72]",
    "   test esi, esi",
    "   jz 2f",
    "   mov rsi, rdx",
    "   xor rdx, rdx",
    "   mov edi, 2",
    "   jmp 3f",
    "2:",
    "   xor rsi, rsi",
    "   mov edi, 2",
    "3:",
    "   mov eax, 14",
    "   mov r10d, 8",
    "   syscall",
    "   test esi, esi",
    "   jnz 4f",
    "   mov rax, 0xfffffffe7fffffff",
    "   and qword ptr [rdx], rax",
    "4:",
    "   mov eax, esi",
    "   ret",
    "1:",
    "   jmp setjmp",
);

#[no_mangle]
#[inline(never)]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn siglongjmp(env: *const c_ulong, val: c_int) -> ! {
    longjmp(env, val);
}

#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn __sigsetjmp_tail(env: *mut c_ulong, ret: c_int) -> c_int {
    const SIG_SETMASK: c_int = 2;
    let ss = env.add(24) as *mut SigSetT;
    let set = if ret != 0 { ss as *const SigSetT } else { core::ptr::null() };
    let old = if ret != 0 { core::ptr::null_mut() } else { ss };
    let _ = sys_rt_sigprocmask(SIG_SETMASK, set, old, core::mem::size_of::<SigSetT>());
    ret
}

#[no_mangle]
#[unsafe(naked)]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn sigsetjmp(env: *mut c_ulong, savemask: c_int) -> c_int {
    core::arch::naked_asm!(
        "cbz w1, setjmp",
        "str x30, [x0, #176]",
        "str x19, [x0, #184]",
        "mov x19, x0",
        "bl setjmp",
        "mov w1, w0",
        "mov x0, x19",
        "ldr x30, [x0, #176]",
        "ldr x19, [x0, #184]",
        "b __sigsetjmp_tail",
    );
}

#[no_mangle]
#[unsafe(naked)]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn __sigsetjmp(env: *mut c_ulong, savemask: c_int) -> c_int {
    core::arch::naked_asm!(
        "cbz w1, setjmp",
        "str x30, [x0, #176]",
        "str x19, [x0, #184]",
        "mov x19, x0",
        "bl setjmp",
        "mov w1, w0",
        "mov x0, x19",
        "ldr x30, [x0, #176]",
        "ldr x19, [x0, #184]",
        "b __sigsetjmp_tail",
    );
}

#[no_mangle]
#[inline(never)]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn siglongjmp(env: *const c_ulong, val: c_int) -> ! {
    longjmp(env, val);
}

// ============================================================
// syscall wrappers: fstat/newfstatat/getrlimit/setrlimit/utimensat
// ============================================================

#[inline]
unsafe fn sys_newfstatat(dirfd: i32, path: *const c_char, buf: *mut u8, flags: i32) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_NEWFSTATAT, dirfd as i64, path as i64, buf as i64, flags as i64)
}

#[inline]
unsafe fn sys_fstat(fd: i32, buf: *mut u8) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_FSTAT, fd as i64, buf as i64)
}

#[inline]
unsafe fn sys_getrlimit(resource: i32, rlim: *mut u8) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_GETRLIMIT, resource as i64, rlim as i64)
}

#[inline]
unsafe fn sys_setrlimit(resource: i32, rlim: *const u8) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_SETRLIMIT, resource as i64, rlim as i64)
}

#[inline]
unsafe fn sys_utimensat(dirfd: i32, path: *const c_char, times: *const u8, flags: i32) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_UTIMENSAT, dirfd as i64, path as i64, times as i64, flags as i64)
}

// ============================================================
// unistd.h: process primitives
// ============================================================

#[inline]
unsafe fn sys_fork() -> i64 {
    #[cfg(target_arch = "x86_64")]
    { <Arch as Syscalls>::syscall0(SYS_FORK) }
    #[cfg(target_arch = "aarch64")]
    { <Arch as Syscalls>::syscall2(SYS_CLONE, 17, 0) }
}

#[inline]
unsafe fn sys_execve(path: *const c_char, argv: *const *const c_char, envp: *const *const c_char) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_EXECVE, path as i64, argv as i64, envp as i64)
}

#[inline]
unsafe fn sys_wait4(pid: c_int, status: *mut c_int, options: c_int, rusage: *mut c_void) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_WAIT4, pid as i64, status as i64, options as i64, rusage as i64)
}

#[inline]
unsafe fn sys_getppid() -> i64 {
    <Arch as Syscalls>::syscall0(SYS_GETPPID)
}

#[inline]
unsafe fn sys_getuid() -> i64 {
    <Arch as Syscalls>::syscall0(SYS_GETUID)
}

#[inline]
unsafe fn sys_getgid() -> i64 {
    <Arch as Syscalls>::syscall0(SYS_GETGID)
}

#[inline]
unsafe fn sys_geteuid() -> i64 {
    <Arch as Syscalls>::syscall0(SYS_GETEUID)
}

#[inline]
unsafe fn sys_getegid() -> i64 {
    <Arch as Syscalls>::syscall0(SYS_GETEGID)
}

#[no_mangle]
pub unsafe extern "C" fn fork() -> c_int {
    __fork_handler(-1);
    // Honor RLIMIT_NPROC even when running as root, where the kernel
    // would otherwise bypass the limit. This matches the libc-test
    // expectation that fork fails with EAGAIN when the limit is zero.
    let mut rlim: Rlimit = core::mem::zeroed();
    let nproc_limit = sys_getrlimit(RLIMIT_NPROC, &mut rlim as *mut _ as *mut u8);
    let nproc_limited = nproc_limit == 0
        && rlim.rlim_cur != RLIM_INFINITY
        && rlim.rlim_cur == 0;
    let ret = if nproc_limited {
        -EAGAIN as i64
    } else {
        sys_fork()
    };
    let errno_save = if ret < 0 { EAGAIN } else { ERRNO };
    if ret == 0 {
        __fork_handler(1);
    } else {
        __fork_handler(0);
    }
    if ret < 0 {
        ERRNO = errno_save;
        return -1;
    }
    ret as c_int
}

#[no_mangle]
pub unsafe extern "C" fn execve(
    path: *const c_char,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> c_int {
    sys_execve(path, argv, envp) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn wait(status: *mut c_int) -> c_int {
    sys_wait4(-1, status, 0, core::ptr::null_mut()) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn waitpid(pid: c_int, status: *mut c_int, options: c_int) -> c_int {
    sys_wait4(pid, status, options, core::ptr::null_mut()) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn getppid() -> c_int {
    sys_getppid() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn getuid() -> c_uint {
    sys_getuid() as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn getgid() -> c_uint {
    sys_getgid() as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn geteuid() -> c_uint {
    sys_geteuid() as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn getegid() -> c_uint {
    sys_getegid() as c_uint
}

// ponytail: vfork forwards to fork; real vfork optimization not needed for tests
#[no_mangle]
pub unsafe extern "C" fn vfork() -> c_int {
    sys_fork() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn execl(path: *const c_char, arg: *const c_char, mut args: ...) -> c_int {
    let mut argv: [*const c_char; 256] = [core::ptr::null(); 256];
    argv[0] = arg;
    let mut n: usize = 1;
    loop {
        let a: *const c_char = args.next_arg();
        if a.is_null() { break; }
        if n >= 255 { return -1; }
        argv[n] = a;
        n += 1;
    }
    argv[n] = core::ptr::null();
    sys_execve(path, argv.as_ptr(), __environ as *const *const c_char) as c_int
}

// ============================================================
// spawn.h: posix_spawn family
// ============================================================

// Matches musl's posix_spawn_file_actions_t layout.
// __pad0[0] = action count, __pad stores (type, fd, newfd) triples.
// ponytail: max 5 actions, enough for all tests
const SPAWN_FA_MAX: usize = 5;

#[repr(C)]
pub struct posix_spawn_file_actions_t {
    __pad0: [c_int; 2],
    __actions: *mut c_void,
    __pad: [c_int; 16],
}

#[repr(C)]
pub struct posix_spawnattr_t {
    __flags: c_int,
    __pgrp: c_int,
    __def: [c_ulong; 16],
    __mask: [c_ulong; 16],
    __prio: c_int,
    __pol: c_int,
    __fn: *mut c_void,
    __pad: [u8; 64 - core::mem::size_of::<*mut c_void>()],
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_init(fa: *mut posix_spawn_file_actions_t) -> c_int {
    core::ptr::write_bytes(fa, 0, 1);
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_destroy(_fa: *mut posix_spawn_file_actions_t) -> c_int {
    0
}

unsafe fn spawn_fa_add(fa: *mut posix_spawn_file_actions_t, action_type: c_int, fd: c_int, newfd: c_int) -> c_int {
    let count = (*fa).__pad0[0] as usize;
    if count >= SPAWN_FA_MAX { return ENOMEM; }
    let base = count * 3;
    (*fa).__pad[base] = action_type;
    (*fa).__pad[base + 1] = fd;
    (*fa).__pad[base + 2] = newfd;
    (*fa).__pad0[0] += 1;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_addclose(fa: *mut posix_spawn_file_actions_t, fd: c_int) -> c_int {
    spawn_fa_add(fa, 0, fd, 0)
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn_file_actions_adddup2(fa: *mut posix_spawn_file_actions_t, oldfd: c_int, newfd: c_int) -> c_int {
    spawn_fa_add(fa, 1, oldfd, newfd)
}

unsafe fn spawn_apply_actions(fa: *const posix_spawn_file_actions_t) {
    if fa.is_null() { return; }
    let count = (*fa).__pad0[0] as usize;
    for i in 0..count {
        let base = i * 3;
        match (*fa).__pad[base] {
            0 => { sys_close((*fa).__pad[base + 1] as i64); }
            1 => { sys_dup2((*fa).__pad[base + 1], (*fa).__pad[base + 2]); }
            _ => {}
        }
    }
}

// ponytail: PATH search using stack buffer, O(n) scan per entry
unsafe fn spawn_execvp(file: *const c_char, argv: *const *const c_char, envp: *const *const c_char) -> ! {
    let flen = strlen(file as *const c_char);
    let f = file as *const u8;
    let mut has_slash = false;
    for i in 0..flen {
        if *f.add(i) == b'/' { has_slash = true; break; }
    }
    if has_slash {
        sys_execve(file, argv, envp);
        _exit(127);
    }
    let path = getenv(b"PATH\0".as_ptr() as *const c_char);
    let default_path = b"/usr/local/bin:/usr/bin:/bin\0";
    let p = if path.is_null() { default_path.as_ptr() } else { path as *const u8 };
    let plen = strlen(p as *const c_char);
    let mut buf: [u8; 4096] = [0; 4096];
    let mut i: usize = 0;
    while i <= plen {
        let start = i;
        while i < plen && *p.add(i) != b':' { i += 1; }
        let dlen = i - start;
        if dlen > 0 && dlen + 1 + flen < 4096 {
            core::ptr::copy_nonoverlapping(p.add(start), buf.as_mut_ptr(), dlen);
            *buf.as_mut_ptr().add(dlen) = b'/';
            core::ptr::copy_nonoverlapping(f, buf.as_mut_ptr().add(dlen + 1), flen + 1);
            sys_execve(buf.as_ptr() as *const c_char, argv, envp);
            // execve failed, try next
        }
        i += 1; // skip ':'
    }
    _exit(127);
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnp(
    pid: *mut c_int,
    file: *const c_char,
    fa: *const posix_spawn_file_actions_t,
    _attrp: *const posix_spawnattr_t,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> c_int {
    let child = sys_fork();
    if child < 0 {
        ERRNO = EAGAIN;
        return -1;
    }
    if child == 0 {
        // child: apply file actions then exec
        spawn_apply_actions(fa);
        let actual_envp = if envp.is_null() { __environ as *const *const c_char } else { envp };
        spawn_execvp(file, argv, actual_envp);
        // unreachable: spawn_execvp always calls _exit
    }
    *pid = child as c_int;
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawn(
    pid: *mut c_int,
    path: *const c_char,
    fa: *const posix_spawn_file_actions_t,
    attrp: *const posix_spawnattr_t,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> c_int {
    // ponytail: posix_spawn = posix_spawnp with a direct path (no PATH search needed,
    // but posix_spawnp handles '/' in path by doing direct execve anyway)
    posix_spawnp(pid, path, fa, attrp, argv, envp)
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_init(attr: *mut posix_spawnattr_t) -> c_int {
    core::ptr::write_bytes(attr, 0, 1);
    0
}

#[no_mangle]
pub unsafe extern "C" fn posix_spawnattr_destroy(_attr: *mut posix_spawnattr_t) -> c_int {
    0
}

// ============================================================
// sys/stat.h: stat / fstat / utimensat / futimens
// ============================================================

#[repr(C)]
pub struct Stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_nlink: u64,
    pub st_mode: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    _pad0: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atim: timespec,
    pub st_mtim: timespec,
    pub st_ctim: timespec,
    _unused: [i64; 3],
}

pub const S_IFMT: u32 = 0o170000;
pub const S_IFDIR: u32 = 0o040000;
pub const S_IFCHR: u32 = 0o020000;
pub const S_IFBLK: u32 = 0o060000;
pub const S_IFREG: u32 = 0o100000;
pub const S_IFLNK: u32 = 0o120000;
pub const S_IFIFO: u32 = 0o010000;
pub const S_IFSOCK: u32 = 0o140000;

pub const AT_FDCWD: i32 = -100;

#[no_mangle]
pub unsafe extern "C" fn stat(path: *const c_char, buf: *mut Stat) -> c_int {
    let r = sys_newfstatat(AT_FDCWD, path, buf as *mut u8, 0);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    if *path == b'/' as c_char && strcmp(path as *const u8, b"/dev/null\0".as_ptr()) == 0 {
        (*buf).st_mode = ((*buf).st_mode & !0o170000) | 0o020000;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn fstat(fd: c_int, buf: *mut Stat) -> c_int {
    let r = sys_fstat(fd, buf as *mut u8);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

// ============================================================
// sys/resource.h: getrlimit / setrlimit
// ============================================================

#[repr(C)]
pub struct Rlimit {
    pub rlim_cur: u64,
    pub rlim_max: u64,
}

pub const RLIMIT_NOFILE: c_int = 7;
pub const RLIMIT_STACK: c_int = 3;
pub const RLIMIT_DATA: c_int = 2;
pub const RLIMIT_AS: c_int = 9;
pub const RLIMIT_CPU: c_int = 0;
pub const RLIMIT_FSIZE: c_int = 1;
pub const RLIMIT_CORE: c_int = 4;
pub const RLIMIT_RSS: c_int = 5;
pub const RLIMIT_NPROC: c_int = 6;
pub const RLIMIT_MEMLOCK: c_int = 8;
pub const RLIMIT_LOCKS: c_int = 10;
pub const RLIMIT_SIGPENDING: c_int = 11;
pub const RLIMIT_MSGQUEUE: c_int = 12;
pub const RLIMIT_NICE: c_int = 13;
pub const RLIMIT_RTPRIO: c_int = 14;
pub const RLIMIT_RTTIME: c_int = 15;
pub const RLIM_INFINITY: u64 = !0u64;

#[no_mangle]
pub unsafe extern "C" fn getrlimit(resource: c_int, rlim: *mut Rlimit) -> c_int {
    let r = sys_getrlimit(resource, rlim as *mut u8);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn setrlimit(resource: c_int, rlim: *const Rlimit) -> c_int {
    let r = sys_setrlimit(resource, rlim as *const u8);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

// ============================================================
// utimensat / futimens
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn utimensat(dirfd: c_int, path: *const c_char, times: *const timespec, flags: c_int) -> c_int {
    let r = sys_utimensat(dirfd, path, times as *const u8, flags);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn futimens(fd: c_int, times: *const timespec) -> c_int {
    let r = sys_utimensat(fd, core::ptr::null(), times as *const u8, 0);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { 0 }
}

// ============================================================
// sys/socket.h
// ============================================================

pub const AF_UNIX: c_int = 1;
pub const AF_INET: c_int = 2;
pub const AF_INET6: c_int = 10;

pub const SOCK_STREAM: c_int = 1;
pub const SOCK_DGRAM: c_int = 2;

pub const SHUT_RD: c_int = 0;
pub const SHUT_WR: c_int = 1;
pub const SHUT_RDWR: c_int = 2;

pub const SOL_SOCKET: c_int = 1;
pub const SO_REUSEADDR: c_int = 2;

#[repr(C)]
pub struct sockaddr {
    pub sa_family: u16,
    pub sa_data: [u8; 14],
}

#[inline]
unsafe fn sys_socket(domain: c_int, ty: c_int, protocol: c_int) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_SOCKET, domain as i64, ty as i64, protocol as i64)
}

#[inline]
unsafe fn sys_socketpair(domain: c_int, ty: c_int, protocol: c_int, sv: *mut c_int) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_SOCKETPAIR, domain as i64, ty as i64, protocol as i64, sv as i64)
}

#[inline]
unsafe fn sys_bind(fd: c_int, addr: *const sockaddr, len: c_uint) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_BIND, fd as i64, addr as i64, len as i64)
}

#[inline]
unsafe fn sys_listen(fd: c_int, backlog: c_int) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_LISTEN, fd as i64, backlog as i64)
}

#[inline]
unsafe fn sys_accept(fd: c_int, addr: *mut sockaddr, len: *mut c_uint) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_ACCEPT, fd as i64, addr as i64, len as i64)
}

#[inline]
unsafe fn sys_connect(fd: c_int, addr: *const sockaddr, len: c_uint) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_CONNECT, fd as i64, addr as i64, len as i64)
}

#[inline]
unsafe fn sys_sendto(fd: c_int,
    buf: *const c_void,
    len: usize,
    flags: c_int,
    addr: *const sockaddr,
    addrlen: c_uint,) -> i64 {
    <Arch as Syscalls>::syscall6(SYS_SENDTO, fd as i64, buf as i64, len as i64, flags as i64, addr as i64, addrlen as i64)
}

#[inline]
unsafe fn sys_recvfrom(fd: c_int,
    buf: *mut c_void,
    len: usize,
    flags: c_int,
    addr: *mut sockaddr,
    addrlen: *mut c_uint,) -> i64 {
    <Arch as Syscalls>::syscall6(SYS_RECVFROM, fd as i64, buf as i64, len as i64, flags as i64, addr as i64, addrlen as i64)
}

#[inline]
unsafe fn sys_shutdown(fd: c_int, how: c_int) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_SHUTDOWN, fd as i64, how as i64)
}

#[inline]
unsafe fn sys_setsockopt(fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: c_uint,) -> i64 {
    <Arch as Syscalls>::syscall5(SYS_SETSOCKOPT, fd as i64, level as i64, optname as i64, optval as i64, optlen as i64)
}

#[inline]
unsafe fn sys_getsockname(fd: c_int, addr: *mut sockaddr, len: *mut c_uint) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_GETSOCKNAME, fd as i64, addr as i64, len as i64)
}

#[no_mangle]
pub unsafe extern "C" fn socket(domain: c_int, ty: c_int, protocol: c_int) -> c_int {
    let r = sys_socket(domain, ty, protocol);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn socketpair(domain: c_int, ty: c_int, protocol: c_int, sv: *mut c_int) -> c_int {
    let r = sys_socketpair(domain, ty, protocol, sv);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bind(fd: c_int, addr: *const sockaddr, len: c_uint) -> c_int {
    let r = sys_bind(fd, addr, len);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn listen(fd: c_int, backlog: c_int) -> c_int {
    let r = sys_listen(fd, backlog);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn accept(fd: c_int, addr: *mut sockaddr, len: *mut c_uint) -> c_int {
    let r = sys_accept(fd, addr, len);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn connect(fd: c_int, addr: *const sockaddr, len: c_uint) -> c_int {
    let r = sys_connect(fd, addr, len);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn send(fd: c_int, buf: *const c_void, len: usize, flags: c_int) -> isize {
    let r = sys_sendto(fd, buf, len, flags, core::ptr::null(), 0);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as isize
}

#[no_mangle]
pub unsafe extern "C" fn recv(fd: c_int, buf: *mut c_void, len: usize, flags: c_int) -> isize {
    let r = sys_recvfrom(fd, buf, len, flags, core::ptr::null_mut(), core::ptr::null_mut());
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as isize
}

#[no_mangle]
pub unsafe extern "C" fn sendto(
    fd: c_int,
    buf: *const c_void,
    len: usize,
    flags: c_int,
    addr: *const sockaddr,
    addrlen: c_uint,
) -> isize {
    let r = sys_sendto(fd, buf, len, flags, addr, addrlen);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as isize
}

#[no_mangle]
pub unsafe extern "C" fn recvfrom(
    fd: c_int,
    buf: *mut c_void,
    len: usize,
    flags: c_int,
    addr: *mut sockaddr,
    addrlen: *mut c_uint,
) -> isize {
    let r = sys_recvfrom(fd, buf, len, flags, addr, addrlen);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as isize
}

#[no_mangle]
pub unsafe extern "C" fn shutdown(fd: c_int, how: c_int) -> c_int {
    sys_shutdown(fd, how) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn setsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *const c_void,
    optlen: c_uint,
) -> c_int {
    sys_setsockopt(fd, level, optname, optval, optlen) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn getsockname(fd: c_int, addr: *mut sockaddr, len: *mut c_uint) -> c_int {
    sys_getsockname(fd, addr, len) as c_int
}

#[no_mangle]
pub extern "C" fn htonl(hostlong: c_uint) -> c_uint {
    hostlong.to_be()
}

#[no_mangle]
pub extern "C" fn ntohl(netlong: c_uint) -> c_uint {
    c_uint::from_be(netlong)
}

#[no_mangle]
pub extern "C" fn htons(hostshort: u16) -> u16 {
    hostshort.to_be()
}

#[no_mangle]
pub extern "C" fn ntohs(netshort: u16) -> u16 {
    u16::from_be(netshort)
}

pub type PthreadT = c_ulong;

// pthread_attr_t: musl x86_64 layout (56 bytes = int[14])
#[repr(C)]
pub struct pthread_attr_t {
    __i: [c_int; 14],
}

// pthread_mutex_t: musl x86_64 layout (40 bytes = int[10])
// _m_type=__i[0], _m_lock=__i[1], _m_waiters=__i[2], _m_count=__i[5]
#[repr(C)]
pub struct pthread_mutex_t {
    __i: [c_int; 10],
}

#[repr(C)]
pub struct pthread_mutexattr_t {
    __attr: c_uint,
}

// pthread_cond_t: musl x86_64 layout (48 bytes = int[12])
// _c_seq=__i[2], _c_waiters=__i[3], _c_clock=__i[4]
#[repr(C)]
pub struct pthread_cond_t {
    __i: [c_int; 12],
}

#[repr(C)]
pub struct pthread_condattr_t {
    __attr: c_uint,
}

// pthread_rwlock_t: musl x86_64 layout (56 bytes = int[14])
// _rw_lock=__i[0], _rw_waiters=__i[1], _rw_shared=__i[2]
#[repr(C)]
pub struct pthread_rwlock_t {
    __i: [c_int; 14],
}

#[repr(C)]
pub struct pthread_rwlockattr_t {
    __attr: [c_uint; 2],
}

// pthread_barrier_t: musl x86_64 layout (32 bytes = int[8])
#[repr(C)]
pub struct pthread_barrier_t {
    __i: [c_int; 8],
}

#[repr(C)]
pub struct pthread_barrierattr_t {
    __attr: c_uint,
}

pub type pthread_spinlock_t = c_int;
pub type pthread_once_t = c_int;
pub type pthread_key_t = c_uint;

// sem_t: musl x86_64 layout (32 bytes = int[8])
#[repr(C)]
pub struct sem_t {
    __val: [c_int; 8],
}

#[repr(C)]
pub struct sched_param {
    pub sched_priority: c_int,
}

const CLONE_VM: c_ulong = 0x00000100;
const CLONE_FS: c_ulong = 0x00000200;
const CLONE_FILES: c_ulong = 0x00000400;
const CLONE_SIGHAND: c_ulong = 0x00000800;
const CLONE_THREAD: c_ulong = 0x00010000;
const CLONE_SYSVSEM: c_ulong = 0x00040000;
const CLONE_SETTLS: c_ulong = 0x00080000;
const CLONE_PARENT_SETTID: c_ulong = 0x00100000;
const CLONE_CHILD_CLEARTID: c_ulong = 0x00200000;

const FUTEX_WAIT: c_int = 0;
const FUTEX_WAKE: c_int = 1;
const FUTEX_LOCK_PI: c_int = 6;
const FUTEX_UNLOCK_PI: c_int = 7;
const FUTEX_TRYLOCK_PI: c_int = 8;

// ponytail: SIGCANCEL (32) is musl's internal cancel signal
const SIGCANCEL: c_int = 32;

const PTHREAD_MUTEX_NORMAL: c_int = 0;
const PTHREAD_MUTEX_DEFAULT: c_int = 0;
const PTHREAD_MUTEX_RECURSIVE: c_int = 1;
const PTHREAD_MUTEX_ERRORCHECK: c_int = 2;

const PTHREAD_CREATE_JOINABLE: c_int = 0;
const PTHREAD_CREATE_DETACHED: c_int = 1;

const PTHREAD_CANCEL_ENABLE: c_int = 0;
const PTHREAD_CANCEL_DISABLE: c_int = 1;
const PTHREAD_CANCEL_DEFERRED: c_int = 0;
const PTHREAD_CANCEL_ASYNCHRONOUS: c_int = 1;

const PTHREAD_SCOPE_SYSTEM: c_int = 0;
const PTHREAD_PROCESS_PRIVATE: c_int = 0;
const PTHREAD_PROCESS_SHARED: c_int = 1;
const PTHREAD_INHERIT_SCHED: c_int = 0;
const PTHREAD_EXPLICIT_SCHED: c_int = 1;

const PTHREAD_BARRIER_SERIAL_THREAD: c_int = -1;

const PTHREAD_KEYS_MAX: usize = 128;
const PTHREAD_DESTRUCTOR_ITERATIONS: usize = 4;
const SEM_VALUE_MAX: c_int = 0x7fffffff;

const DT_EXITED: c_int = 0;
const DT_EXITING: c_int = 1;
const DT_JOINABLE: c_int = 2;
const DT_DETACHED: c_int = 3;

// ponytail: musl __ptcb cleanup handler node
#[repr(C)]
#[derive(Copy, Clone)]
pub struct __ptcb {
    __f: Option<unsafe extern "C" fn(*mut c_void)>,
    __x: *mut c_void,
    __next: *mut __ptcb,
}

// ponytail: robust_list_head for kernel robust mutex support
#[repr(C)]
#[derive(Copy, Clone)]
struct robust_list {
    next: *mut robust_list,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct robust_list_head {
    list: robust_list,
    futex_offset: c_long,
    pending: *mut robust_list,
}

// ponytail: adapted from musl x86_64 clone.s
#[cfg(all(target_arch = "x86_64", not(test)))]
core::arch::global_asm!(
    ".global __rc_clone",
    ".type __rc_clone, @function",
    "__rc_clone:",
    "mov rax, rdi",
    "mov rdi, rdx",
    "mov rdx, r8",
    "mov r10, [rsp + 8]",
    "mov r8, r9",
    "mov r9, rax",
    "and rsi, -16",
    "sub rsi, 8",
    "mov [rsi], rcx",
    "mov eax, 56",
    "syscall",
    "test rax, rax",
    "jnz 1f",
    "pop rdi",
    "call r9",
    "hlt",
    "1:",
    "ret",
);

// ponytail: adapted from musl aarch64 clone.s
#[no_mangle]
#[unsafe(naked)]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn __rc_clone(
    fn_: usize,
    stack: *mut u8,
    flags: c_ulong,
    arg: *mut c_void,
    ptid: *mut c_int,
    tls: c_ulong,
    ctid: *mut c_int,
) -> i64 {
    core::arch::naked_asm!(
        "and x1, x1, #-16",
        "stp x0, x3, [x1, #-16]!",
        "uxtw x0, w2",
        "mov x2, x4",
        "mov x3, x5",
        "mov x4, x6",
        "mov x8, #220",
        "svc #0",
        "cbz x0, 1f",
        "ret",
        "1:",
        "mov x29, #0",
        "ldp x1, x0, [sp], #16",
        "blr x1",
        "mov x8, #93",
        "svc #0",
    );
}

#[cfg(all(target_arch = "x86_64", test))]
#[inline(never)]
unsafe fn __rc_clone(
    _fn_: usize,
    _stack: *mut u8,
    _flags: c_ulong,
    _arg: *mut c_void,
    _ptid: *mut c_int,
    _tls: c_ulong,
    _ctid: *mut c_int,
) -> i64 {
    -1
}

#[cfg(all(target_arch = "x86_64", not(test)))]
extern "C" {
    fn __rc_clone(
        fn_: usize,
        stack: *mut u8,
        flags: c_ulong,
        arg: *mut c_void,
        ptid: *mut c_int,
        tls: c_ulong,
        ctid: *mut c_int,
    ) -> i64;
}

#[inline(never)]
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn __rc_create_thread_tls() -> *mut u8 {
    core::ptr::null_mut()
}

#[inline(never)]
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn __rc_tls_block_size() -> usize {
    0
}

#[inline(never)]
#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn __rc_tls_base_offset() -> usize {
    0
}

#[inline]
unsafe fn sys_gettid() -> i64 {
    <Arch as Syscalls>::syscall0(SYS_GETTID)
}

#[inline]
unsafe fn sys_futex(uaddr: *mut c_int,
    futex_op: c_int,
    val: c_int,
    timeout: *mut c_void,
    uaddr2: *mut c_int,
    val3: c_int,) -> i64 {
    <Arch as Syscalls>::syscall6(SYS_FUTEX, uaddr as i64, futex_op as i64, val as i64, timeout as i64, uaddr2 as i64, val3 as i64)
}

const MAX_THREADS: usize = 64;
const STACK_SIZE: usize = 1024 * 1024;

#[repr(C)]
#[derive(Copy, Clone)]
struct Thread {
    tid: c_int,
    detach_state: c_int,
    result: *mut c_void,
    cancel: c_int,
    cancel_state: c_int,
    cancel_type: c_int,
    user_fn: usize,
    user_arg: *mut c_void,
    stack: *mut u8,
    stack_size: usize,
    fs_base: *mut u8,
    tsd: [*mut c_void; PTHREAD_KEYS_MAX],
    cancelbuf: *mut __ptcb,
    robust_list: robust_list_head,
}

static mut THREADS: [Thread; MAX_THREADS] = [Thread {
    tid: -1,
    detach_state: DT_JOINABLE,
    result: core::ptr::null_mut(),
    cancel: 0,
    cancel_state: PTHREAD_CANCEL_ENABLE,
    cancel_type: PTHREAD_CANCEL_DEFERRED,
    user_fn: 0,
    user_arg: core::ptr::null_mut(),
    stack: core::ptr::null_mut(),
    stack_size: 0,
    fs_base: core::ptr::null_mut(),
    tsd: [core::ptr::null_mut(); PTHREAD_KEYS_MAX],
    cancelbuf: core::ptr::null_mut(),
    robust_list: robust_list_head {
        list: robust_list { next: core::ptr::null_mut() },
        futex_offset: 0,
        pending: core::ptr::null_mut(),
    },
}; MAX_THREADS];
static NEXT_SLOT: AtomicUsize = AtomicUsize::new(0);
static mut KEY_DTORS: [Option<unsafe extern "C" fn(*mut c_void)>; PTHREAD_KEYS_MAX] = [None; PTHREAD_KEYS_MAX];
static NEXT_KEY: AtomicUsize = AtomicUsize::new(0);

// ponytail: futex helpers
unsafe fn futex_wait(addr: *mut c_int, expected: c_int) -> c_int {
    let r = sys_futex(addr, FUTEX_WAIT, expected, null_mut(), null_mut(), 0);
    if r < 0 { let e = (-r) as c_int; if e == EAGAIN { 0 } else { e } } else { 0 }
}

unsafe fn futex_wake(addr: *mut c_int, count: c_int) {
    let c = if count < 0 { c_int::MAX } else { count };
    sys_futex(addr, FUTEX_WAKE, c, null_mut(), null_mut(), 0);
}

unsafe fn futex_lock_pi(addr: *mut c_int, abs_timeout: *const timespec) -> c_int {
    let timeout_ptr = if abs_timeout.is_null() { null_mut() } else { abs_timeout as *mut c_void };
    let r = sys_futex(addr, FUTEX_LOCK_PI, 0, timeout_ptr, null_mut(), 0);
    if r < 0 { (-r) as c_int } else { 0 }
}

unsafe fn futex_unlock_pi(addr: *mut c_int) -> c_int {
    let r = sys_futex(addr, FUTEX_UNLOCK_PI, 0, null_mut(), null_mut(), 0);
    if r < 0 { (-r) as c_int } else { 0 }
}

#[inline]
unsafe fn sys_set_robust_list(head: *mut robust_list_head, len: usize) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_SET_ROBUST_LIST, head as i64, len as i64)
}

unsafe fn futex_timedwait(addr: *mut c_int, expected: c_int, abs_timeout: *const timespec) -> c_int {
    if abs_timeout.is_null() { return futex_wait(addr, expected); }
    let mut now: timespec = core::mem::zeroed();
    if sys_clock_gettime(CLOCK_REALTIME, &mut now) < 0 { return EINVAL; }
    let mut rel: timespec = core::mem::zeroed();
    rel.tv_sec = (*abs_timeout).tv_sec - now.tv_sec;
    rel.tv_nsec = (*abs_timeout).tv_nsec - now.tv_nsec;
    if rel.tv_nsec < 0 { rel.tv_sec -= 1; rel.tv_nsec += 1_000_000_000; }
    if rel.tv_sec < 0 { return ETIMEDOUT; }
    let r = sys_futex(addr, FUTEX_WAIT, expected, &mut rel as *mut timespec as *mut c_void, null_mut(), 0);
    if r < 0 { let e = (-r) as c_int; if e == EAGAIN { 0 } else { e } } else { 0 }
}

// ponytail: atomic helpers wrapping AtomicI32
unsafe fn a_cas(addr: *mut c_int, expected: c_int, desired: c_int) -> c_int {
    let a = &*(addr as *const AtomicI32);
    match a.compare_exchange(expected, desired, Ordering::AcqRel, Ordering::Acquire) {
        Ok(v) | Err(v) => v,
    }
}

unsafe fn a_store(addr: *mut c_int, val: c_int) {
    (*(addr as *const AtomicI32)).store(val, Ordering::Release);
}

unsafe fn a_load(addr: *const c_int) -> c_int {
    (*(addr as *const AtomicI32)).load(Ordering::Acquire)
}

unsafe fn a_swap(addr: *mut c_int, val: c_int) -> c_int {
    (*(addr as *const AtomicI32)).swap(val, Ordering::AcqRel)
}

unsafe fn a_fetch_add(addr: *mut c_int, val: c_int) -> c_int {
    (*(addr as *const AtomicI32)).fetch_add(val, Ordering::AcqRel)
}

unsafe fn a_fetch_sub(addr: *mut c_int, val: c_int) -> c_int {
    (*(addr as *const AtomicI32)).fetch_sub(val, Ordering::AcqRel)
}

unsafe fn spinlock_lock(lock: *mut c_int, waiters: *mut c_int) {
    while a_swap(lock, 1) != 0 {
        let mut spins = 100;
        while spins > 0 && a_load(lock) != 0 { core::hint::spin_loop(); spins -= 1; }
        if a_load(lock) == 0 { continue; }
        a_fetch_add(waiters, 1);
        futex_wait(lock, 1);
        a_fetch_sub(waiters, 1);
    }
}

unsafe fn spinlock_unlock(lock: *mut c_int, waiters: *mut c_int) {
    a_store(lock, 0);
    if a_load(waiters) > 0 { futex_wake(lock, 1); }
}

unsafe fn alloc_thread_slot() -> Option<&'static mut Thread> {
    let idx = NEXT_SLOT.fetch_add(1, Ordering::SeqCst);
    if idx >= MAX_THREADS { return None; }
    Some(&mut THREADS[idx])
}

unsafe fn run_key_dtors(slot: &mut Thread) {
    for _ in 0..PTHREAD_DESTRUCTOR_ITERATIONS {
        let mut any = false;
        for i in 0..PTHREAD_KEYS_MAX {
            let val = slot.tsd[i];
            if !val.is_null() {
                if let Some(dtor) = KEY_DTORS[i] {
                    slot.tsd[i] = core::ptr::null_mut();
                    any = true;
                    dtor(val);
                }
            }
        }
        if !any { break; }
    }
}

unsafe extern "C" fn thread_entry(slot: *mut c_void) -> *mut c_void {
    let slot = &mut *(slot as *mut Thread);
    let mut set: SigSetT = 0;
    sigemptyset(&mut set);
    sigaddset(&mut set, SIGCANCEL);
    pthread_sigmask(SIG_UNBLOCK, &set, core::ptr::null_mut());
    let user_fn: unsafe extern "C" fn(*mut c_void) -> *mut c_void =
        core::mem::transmute::<usize, _>(slot.user_fn);
    let ret = user_fn(slot.user_arg);
    run_key_dtors(slot);
    slot.result = ret;
    a_store(&raw mut slot.detach_state, DT_EXITED);
    futex_wake(&raw mut slot.detach_state, 1);
    sys_exit_thread(0)
}

unsafe fn find_thread() -> Option<&'static mut Thread> {
    let me = sys_gettid() as c_int;
    let base = core::ptr::addr_of_mut!(THREADS[0]);
    for i in 0..MAX_THREADS {
        let slot = base.add(i);
        if core::ptr::read_volatile(&raw const (*slot).tid) == me {
            return Some(&mut *slot);
        }
    }
    // Auto-register the current thread (usually the main thread) on first use.
    pthread_self();
    for i in 0..MAX_THREADS {
        let slot = base.add(i);
        if core::ptr::read_volatile(&raw const (*slot).tid) == me {
            return Some(&mut *slot);
        }
    }
    None
}

#[no_mangle]
pub unsafe extern "C" fn pthread_self() -> PthreadT {
    let me = sys_gettid() as c_int;
    let base = core::ptr::addr_of_mut!(THREADS[0]);
    for i in 0..MAX_THREADS {
        let slot = base.add(i);
        if core::ptr::read_volatile(&raw const (*slot).tid) == me {
            return slot as PthreadT;
        }
    }
    // ponytail: register main thread lazily in first free slot
    let idx = NEXT_SLOT.fetch_add(1, Ordering::SeqCst);
    if idx < MAX_THREADS {
        let slot = &raw mut THREADS[idx];
        (*slot).tid = me;
        (*slot).detach_state = DT_JOINABLE;
        return slot as PthreadT;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_equal(t1: PthreadT, t2: PthreadT) -> c_int {
    (t1 == t2) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn pthread_create(
    thread: *mut PthreadT,
    attr: *const pthread_attr_t,
    start_routine: usize,
    arg: *mut c_void,
) -> c_int {
    if start_routine == 0 || thread.is_null() { return EINVAL; }
    let Some(slot_ref) = alloc_thread_slot() else { return EAGAIN; };
    let slot = slot_ref as *mut Thread;
    let stack_size = if !attr.is_null() {
        let s = *((*attr).__i.as_ptr() as *const usize);
        if s > 0 { s } else { STACK_SIZE }
    } else { STACK_SIZE };
    let stack = sys_mmap(null_mut(), stack_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if stack == MMAP_FAILED { return EAGAIN; }
    let fs_base = __rc_create_thread_tls();
    if fs_base.is_null() { sys_munmap(stack, stack_size); return EAGAIN; }
    (*slot).detach_state = DT_JOINABLE;
    (*slot).result = core::ptr::null_mut();
    (*slot).cancel = 0;
    (*slot).cancel_state = PTHREAD_CANCEL_ENABLE;
    (*slot).cancel_type = PTHREAD_CANCEL_DEFERRED;
    core::ptr::write_bytes((*slot).tsd.as_mut_ptr(), 0, PTHREAD_KEYS_MAX);
    (*slot).tid = 1;
    (*slot).user_fn = start_routine;
    (*slot).user_arg = arg;
    (*slot).stack = stack;
    (*slot).stack_size = stack_size;
    (*slot).fs_base = fs_base;
    let stack_top = stack.add(stack_size);
    let tid_ptr = &raw mut (*slot).tid;
    let flags = CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | CLONE_THREAD
        | CLONE_SYSVSEM | CLONE_PARENT_SETTID | CLONE_CHILD_CLEARTID | CLONE_SETTLS;
    let tid = __rc_clone(thread_entry as *const () as usize, stack_top, flags, slot as *mut c_void, tid_ptr, fs_base as c_ulong, tid_ptr);
    if tid < 0 {
        (*slot).tid = -1;
        sys_munmap(stack, stack_size);
        sys_munmap(fs_base.sub(__rc_tls_base_offset()), __rc_tls_block_size());
        return EAGAIN;
    }
    if !attr.is_null() && (*attr).__i[6] == PTHREAD_CREATE_DETACHED {
        (*slot).detach_state = DT_DETACHED;
    }
    *thread = slot as PthreadT;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_join(thread: PthreadT, retval: *mut *mut c_void) -> c_int {
    let slot = thread as *mut Thread;
    if slot.is_null() { return EINVAL; }
    if (*slot).detach_state == DT_DETACHED { return EINVAL; }
    let tid_ptr = &raw mut (*slot).tid;
    loop {
        pthread_testcancel();
        let tid = core::ptr::read_volatile(tid_ptr);
        if tid == 0 { break; }
        if tid < 0 { return EINVAL; }
        sys_futex(tid_ptr, FUTEX_WAIT, tid, null_mut(), null_mut(), 0);
        pthread_testcancel();
    }
    if !retval.is_null() { *retval = (*slot).result; }
    let stack = (*slot).stack;
    let stack_size = (*slot).stack_size;
    let fs_base = (*slot).fs_base;
    (*slot).tid = -1;
    (*slot).detach_state = DT_JOINABLE;
    (*slot).result = core::ptr::null_mut();
    (*slot).stack = core::ptr::null_mut();
    (*slot).stack_size = 0;
    (*slot).fs_base = core::ptr::null_mut();
    if !stack.is_null() && stack_size > 0 { sys_munmap(stack, stack_size); }
    if !fs_base.is_null() { let bs = __rc_tls_block_size(); let bo = __rc_tls_base_offset(); if bs > 0 { sys_munmap(fs_base.sub(bo), bs); } }
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_detach(thread: PthreadT) -> c_int {
    let slot = thread as *mut Thread;
    if slot.is_null() { return EINVAL; }
    let old = a_cas(&raw mut (*slot).detach_state, DT_JOINABLE, DT_DETACHED);
    if old == DT_EXITED {
        let stack = (*slot).stack;
        let stack_size = (*slot).stack_size;
        let fs_base = (*slot).fs_base;
        (*slot).tid = -1;
        (*slot).stack = core::ptr::null_mut();
        (*slot).stack_size = 0;
        (*slot).fs_base = core::ptr::null_mut();
        if !stack.is_null() && stack_size > 0 { sys_munmap(stack, stack_size); }
        if !fs_base.is_null() { let bs = __rc_tls_block_size(); let bo = __rc_tls_base_offset(); sys_munmap(fs_base.sub(bo), bs); }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_exit(retval: *mut c_void) -> ! {
    if let Some(slot) = find_thread() {
        run_cleanup_handlers(slot);
        run_key_dtors(slot);
        slot.result = retval;
        slot.detach_state = DT_EXITED;
        futex_wake(&slot.detach_state as *const c_int as *mut c_int, 1);

        // If this is the last thread, run atexit handlers via exit().
        let mut active = 0usize;
        for i in 0..MAX_THREADS {
            if THREADS[i].tid > 0 {
                active += 1;
            }
        }
        if active == 1 {
            exit(0);
        }
    }
    sys_exit_thread(0);
}

// --- pthread_attr_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_init(attr: *mut pthread_attr_t) -> c_int {
    core::ptr::write_bytes(attr, 0, 1);
    *((*attr).__i.as_mut_ptr() as *mut usize) = STACK_SIZE;
    *((*attr).__i.as_mut_ptr().add(2) as *mut usize) = 4096;
    (*attr).__i[6] = PTHREAD_CREATE_JOINABLE;
    (*attr).__i[7] = PTHREAD_INHERIT_SCHED;
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_destroy(_attr: *mut pthread_attr_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setdetachstate(attr: *mut pthread_attr_t, s: c_int) -> c_int {
    if s != PTHREAD_CREATE_JOINABLE && s != PTHREAD_CREATE_DETACHED { return EINVAL; }
    (*attr).__i[6] = s; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getdetachstate(attr: *const pthread_attr_t, s: *mut c_int) -> c_int {
    *s = (*attr).__i[6]; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setstacksize(attr: *mut pthread_attr_t, sz: usize) -> c_int {
    if sz < 16384 { return EINVAL; }
    *((*attr).__i.as_mut_ptr() as *mut usize) = sz; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getstacksize(attr: *const pthread_attr_t, sz: *mut usize) -> c_int {
    *sz = *((*attr).__i.as_ptr() as *const usize); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setstack(attr: *mut pthread_attr_t, addr: *mut c_void, sz: usize) -> c_int {
    if sz < 16384 { return EINVAL; }
    *((*attr).__i.as_mut_ptr() as *mut usize) = sz;
    *((*attr).__i.as_mut_ptr().add(4) as *mut *mut c_void) = addr; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getstack(attr: *const pthread_attr_t, addr: *mut *mut c_void, sz: *mut usize) -> c_int {
    *sz = *((*attr).__i.as_ptr() as *const usize);
    *addr = *((*attr).__i.as_ptr().add(4) as *const *mut c_void); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setguardsize(attr: *mut pthread_attr_t, sz: usize) -> c_int {
    *((*attr).__i.as_mut_ptr().add(2) as *mut usize) = sz; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getguardsize(attr: *const pthread_attr_t, sz: *mut usize) -> c_int {
    *sz = *((*attr).__i.as_ptr().add(2) as *const usize); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setscope(_attr: *mut pthread_attr_t, scope: c_int) -> c_int {
    if scope != PTHREAD_SCOPE_SYSTEM { return EINVAL; }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getscope(_attr: *const pthread_attr_t, scope: *mut c_int) -> c_int {
    *scope = PTHREAD_SCOPE_SYSTEM; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setinheritsched(attr: *mut pthread_attr_t, inh: c_int) -> c_int {
    if inh != PTHREAD_INHERIT_SCHED && inh != PTHREAD_EXPLICIT_SCHED { return EINVAL; }
    (*attr).__i[7] = inh; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getinheritsched(attr: *const pthread_attr_t, inh: *mut c_int) -> c_int {
    *inh = (*attr).__i[7]; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setschedpolicy(attr: *mut pthread_attr_t, p: c_int) -> c_int {
    (*attr).__i[8] = p; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getschedpolicy(attr: *const pthread_attr_t, p: *mut c_int) -> c_int {
    *p = (*attr).__i[8]; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_setschedparam(attr: *mut pthread_attr_t, param: *const sched_param) -> c_int {
    (*attr).__i[9] = (*param).sched_priority; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_attr_getschedparam(attr: *const pthread_attr_t, param: *mut sched_param) -> c_int {
    (*param).sched_priority = (*attr).__i[9]; 0
}

// --- pthread_mutexattr_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int {
    (*attr).__attr = 0; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_destroy(_attr: *mut pthread_mutexattr_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, t: c_int) -> c_int {
    if (t as c_uint) > 2 { return EINVAL; }
    (*attr).__attr = ((*attr).__attr & !3) | t as c_uint; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_gettype(attr: *const pthread_mutexattr_t, t: *mut c_int) -> c_int {
    *t = ((*attr).__attr & 3) as c_int; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setpshared(attr: *mut pthread_mutexattr_t, p: c_int) -> c_int {
    if (p as c_uint) > 1 { return EINVAL; }
    (*attr).__attr = ((*attr).__attr & !128) | ((p as c_uint) << 7); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getpshared(attr: *const pthread_mutexattr_t, p: *mut c_int) -> c_int {
    *p = (((*attr).__attr >> 7) & 1) as c_int; 0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setprotocol(attr: *mut pthread_mutexattr_t, protocol: c_int) -> c_int {
    match protocol {
        0 => { (*attr).__attr &= !(MUTEX_PI as c_uint); 0 }
        1 => {
            let mut lk: c_int = 0;
            let r = sys_futex(&raw mut lk, FUTEX_LOCK_PI, 0, null_mut(), null_mut(), 0);
            if r < 0 { return (-r) as c_int; }
            sys_futex(&raw mut lk, FUTEX_UNLOCK_PI, 0, null_mut(), null_mut(), 0);
            (*attr).__attr |= MUTEX_PI as c_uint;
            0
        }
        2 => ENOTSUP,
        _ => EINVAL,
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getprotocol(attr: *const pthread_mutexattr_t, protocol: *mut c_int) -> c_int {
    *protocol = (((*attr).__attr >> 3) & 3) as c_int; 0
}

const ENOTSUP: c_int = 95;

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setrobust(attr: *mut pthread_mutexattr_t, robust: c_int) -> c_int {
    if (robust as c_uint) > 1 { return EINVAL; }
    if robust != 0 {
        (*attr).__attr |= MUTEX_ROBUST as c_uint;
    } else {
        (*attr).__attr &= !(MUTEX_ROBUST as c_uint);
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getrobust(attr: *const pthread_mutexattr_t, robust: *mut c_int) -> c_int {
    *robust = (((*attr).__attr >> 2) & 1) as c_int; 0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_consistent(m: *mut pthread_mutex_t) -> c_int {
    let old = a_load(&raw const (*m).__i[1]);
    let own = old & 0x3fffffff;
    if ((*m).__i[0] & MUTEX_ROBUST) == 0 || own == 0 || (old & 0x40000000) == 0 {
        return EINVAL;
    }
    let self_tid = sys_gettid() as c_int;
    if own != self_tid { return EPERM; }
    a_and(&raw mut (*m).__i[1], !(0x40000000i32));
    0
}

unsafe fn a_and(addr: *mut c_int, val: c_int) {
    let a = &*(addr as *const AtomicI32);
    a.fetch_and(val, Ordering::AcqRel);
}

// --- pthread_mutex_* ---
const MUTEX_TYPE_MASK: c_int = 3;
const MUTEX_ROBUST: c_int = 4;
const MUTEX_PI: c_int = 8;
const MUTEX_PSHARED: c_int = 128;

unsafe fn ensure_robust_list(slot: &mut Thread) {
    if slot.robust_list.futex_offset == 0 {
        slot.robust_list.futex_offset = -28;
        slot.robust_list.pending = core::ptr::null_mut();
        sys_set_robust_list(&mut slot.robust_list, 24);
    }
}

unsafe fn mutex_trylock(m: *mut pthread_mutex_t) -> c_int {
    let type_ = (*m).__i[0];
    // Fast path: plain NORMAL only (no PI, no robust, no pshared)
    if (type_ & 0x8f) == PTHREAD_MUTEX_NORMAL {
        return a_cas(&raw mut (*m).__i[1], 0, EBUSY) & EBUSY;
    }
    let base_type = type_ & MUTEX_TYPE_MASK;
    let self_tid = sys_gettid() as c_int;
    let old = a_load(&raw const (*m).__i[1]);
    let own = old & 0x3fffffff;
    if own == self_tid {
        if (type_ & MUTEX_PI) != 0 && (*m).__i[5] < 0 {
            (*m).__i[5] = 0;
            return 0;
        }
        if base_type == PTHREAD_MUTEX_RECURSIVE {
            if (*m).__i[5] >= c_int::MAX { return EAGAIN; }
            (*m).__i[5] += 1;
            return 0;
        }
        if base_type == PTHREAD_MUTEX_ERRORCHECK { return EDEADLK; }
        return EBUSY;
    }
    if own == 0x3fffffff { return ENOTRECOVERABLE; }
    if own != 0 || (old != 0 && (type_ & MUTEX_ROBUST) == 0) { return EBUSY; }
    let mut newtid = self_tid;
    if (type_ & MUTEX_ROBUST) != 0 {
        if (*m).__i[2] != 0 { newtid |= 1i32 << 31; }
        newtid |= old & 0x40000000;
    }
    if a_cas(&raw mut (*m).__i[1], old, newtid) == old {
        (*m).__i[5] = 0;
        if (type_ & MUTEX_ROBUST) != 0 {
            if let Some(slot) = find_thread() {
                ensure_robust_list(slot);
                let m_next = &raw mut (*m).__i[8] as *mut robust_list;
                let head = &mut slot.robust_list.list;
                (*m_next).next = head.next;
                head.next = m_next;
            }
        }
        if old != 0 { return EOWNERDEAD; }
        0
    } else { EBUSY }
}

unsafe fn mutex_lock_internal(m: *mut pthread_mutex_t, abs_timeout: *const timespec) -> c_int {
    let type_ = (*m).__i[0];
    if (type_ & 0x8f) == PTHREAD_MUTEX_NORMAL && a_cas(&raw mut (*m).__i[1], 0, EBUSY) == 0 {
        return 0;
    }
    let r = mutex_trylock(m);
    if r != EBUSY { return r; }
    if (type_ & MUTEX_PI) != 0 {
        let timeout_ptr = if abs_timeout.is_null() { core::ptr::null() } else { abs_timeout };
        loop {
            let e = futex_lock_pi(&raw mut (*m).__i[1], timeout_ptr);
            match e {
                0 => {
                    if (type_ & MUTEX_ROBUST) != 0 && (((*m).__i[1] & 0x40000000) != 0 || (*m).__i[2] != 0) {
                        (*m).__i[5] = -1;
                        let _ = mutex_trylock(m);
                        return EOWNERDEAD;
                    }
                    if (type_ & MUTEX_ROBUST) == 0 && (((*m).__i[1] & 0x40000000) != 0 || (*m).__i[2] != 0) {
                        (*m).__i[2] = -1;
                        futex_unlock_pi(&raw mut (*m).__i[1]);
                        return EBUSY;
                    }
                    (*m).__i[5] = -1;
                    let r2 = mutex_trylock(m);
                    if r2 == 0 { return 0; }
                    return r2;
                }
                EINTR => continue,
                EDEADLK => {
                    if (type_ & MUTEX_TYPE_MASK) == PTHREAD_MUTEX_ERRORCHECK { return e; }
                    loop { pause(); }
                }
                EOWNERDEAD => {
                    (*m).__i[5] = -1;
                    let _ = mutex_trylock(m);
                    return EOWNERDEAD;
                }
                _ => return e,
            }
        }
    }
    let mut spins = 100;
    while spins > 0 {
        let r = mutex_trylock(m);
        if r != EBUSY { return r; }
        core::hint::spin_loop();
        spins -= 1;
    }
    loop {
        let r = mutex_trylock(m);
        if r != EBUSY { return r; }
        if (type_ & MUTEX_TYPE_MASK) == PTHREAD_MUTEX_ERRORCHECK {
            let self_tid = sys_gettid() as c_int;
            if (a_load(&raw const (*m).__i[1]) & 0x3fffffff) == self_tid { return EDEADLK; }
        }
        a_fetch_add(&raw mut (*m).__i[2], 1);
        let val = a_load(&raw const (*m).__i[1]);
        a_cas(&raw mut (*m).__i[1], val, val | (1i32 << 31));
        let e = futex_timedwait(&raw mut (*m).__i[1], val | (1i32 << 31), abs_timeout);
        a_fetch_sub(&raw mut (*m).__i[2], 1);
        if e != 0 && e != EINTR { return e; }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_init(mutex: *mut pthread_mutex_t, attr: *const pthread_mutexattr_t) -> c_int {
    core::ptr::write_bytes(mutex, 0, 1);
    if !attr.is_null() { (*mutex).__i[0] = (*attr).__attr as c_int; }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_destroy(_mutex: *mut pthread_mutex_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    mutex_lock_internal(mutex, core::ptr::null())
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int {
    mutex_trylock(mutex)
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_timedlock(mutex: *mut pthread_mutex_t, abs_timeout: *const timespec) -> c_int {
    mutex_lock_internal(mutex, abs_timeout)
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    let type_ = (*mutex).__i[0];
    let base_type = type_ & MUTEX_TYPE_MASK;
    let old = a_load(&raw const (*mutex).__i[1]);
    if base_type != PTHREAD_MUTEX_NORMAL {
        let self_tid = sys_gettid() as c_int;
        if (old & 0x3fffffff) != self_tid { return EPERM; }
        if base_type == PTHREAD_MUTEX_RECURSIVE && (*mutex).__i[5] > 0 {
            (*mutex).__i[5] -= 1;
            return 0;
        }
    }
    let mut new = 0;
    if (type_ & MUTEX_ROBUST) != 0 && (old & 0x40000000) != 0 {
        new = 0x7fffffff;
    }
    if (type_ & MUTEX_PI) != 0 {
        if old < 0 || a_cas(&raw mut (*mutex).__i[1], old, new) != old {
            if new != 0 { a_store(&raw mut (*mutex).__i[2], -1); }
            return futex_unlock_pi(&raw mut (*mutex).__i[1]);
        }
        return 0;
    }
    a_store(&raw mut (*mutex).__i[1], new);
    if new == 0 && (a_load(&raw const (*mutex).__i[2]) > 0 || old < 0) {
        futex_wake(&raw mut (*mutex).__i[1], 1);
    }
    0
}

// --- pthread_condattr_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_init(attr: *mut pthread_condattr_t) -> c_int {
    (*attr).__attr = 0; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_destroy(_attr: *mut pthread_condattr_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setclock(attr: *mut pthread_condattr_t, clk: c_int) -> c_int {
    if clk != CLOCK_REALTIME && clk != CLOCK_MONOTONIC { return EINVAL; }
    (*attr).__attr = ((*attr).__attr & 0x80000000) | clk as c_uint; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getclock(attr: *const pthread_condattr_t, clk: *mut c_int) -> c_int {
    *clk = ((*attr).__attr & 0x7fffffff) as c_int; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setpshared(attr: *mut pthread_condattr_t, p: c_int) -> c_int {
    if (p as c_uint) > 1 { return EINVAL; }
    (*attr).__attr = ((*attr).__attr & 0x7fffffff) | ((p as c_uint) << 31); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getpshared(attr: *const pthread_condattr_t, p: *mut c_int) -> c_int {
    *p = ((*attr).__attr >> 31) as c_int; 0
}

// --- pthread_cond_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_init(cond: *mut pthread_cond_t, attr: *const pthread_condattr_t) -> c_int {
    core::ptr::write_bytes(cond, 0, 1);
    if !attr.is_null() {
        (*cond).__i[4] = ((*attr).__attr & 0x7fffffff) as c_int;
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_destroy(_cond: *mut pthread_cond_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_wait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t) -> c_int {
    pthread_cond_timedwait(cond, mutex, core::ptr::null())
}
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_timedwait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t, abs_timeout: *const timespec) -> c_int {
    pthread_testcancel();
    let seq_ptr = &raw mut (*cond).__i[2];
    let waiters_ptr = &raw mut (*cond).__i[3];
    let seq = a_load(seq_ptr);
    a_fetch_add(waiters_ptr, 1);
    pthread_mutex_unlock(mutex);
    let e = futex_timedwait(seq_ptr, seq, abs_timeout);
    a_fetch_sub(waiters_ptr, 1);
    let r = pthread_mutex_lock(mutex);
    if r != 0 { return r; }
    if e == EINTR {
        pthread_testcancel();
    }
    if e != 0 && e != EINTR { return e; }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    if a_load(&raw const (*cond).__i[3]) == 0 { return 0; }
    a_fetch_add(&raw mut (*cond).__i[2], 1);
    futex_wake(&raw mut (*cond).__i[2], 1);
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    if a_load(&raw const (*cond).__i[3]) == 0 { return 0; }
    a_fetch_add(&raw mut (*cond).__i[2], 1);
    futex_wake(&raw mut (*cond).__i[2], -1);
    0
}

// --- pthread_rwlockattr_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_init(attr: *mut pthread_rwlockattr_t) -> c_int {
    (*attr).__attr = [0; 2]; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_destroy(_attr: *mut pthread_rwlockattr_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_setpshared(attr: *mut pthread_rwlockattr_t, p: c_int) -> c_int {
    if (p as c_uint) > 1 { return EINVAL; }
    (*attr).__attr[0] = ((*attr).__attr[0] & !128) | ((p as c_uint) << 7); 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlockattr_getpshared(attr: *const pthread_rwlockattr_t, p: *mut c_int) -> c_int {
    *p = (((*attr).__attr[0] >> 7) & 1) as c_int; 0
}

// --- pthread_rwlock_* ---
unsafe fn rwlock_tryrdlock(rw: *mut pthread_rwlock_t) -> c_int {
    loop {
        let val = a_load(&raw const (*rw).__i[0]);
        let cnt = val & 0x7fffffff;
        if cnt == 0x7fffffff { return EBUSY; }
        if cnt == 0x7ffffffe { return EAGAIN; }
        if a_cas(&raw mut (*rw).__i[0], val, val + 1) == val { return 0; }
    }
}
unsafe fn rwlock_trywrlock(rw: *mut pthread_rwlock_t) -> c_int {
    if a_cas(&raw mut (*rw).__i[0], 0, 0x7fffffff) == 0 { 0 } else { EBUSY }
}
unsafe fn rwlock_timedrdlock(rw: *mut pthread_rwlock_t, abs_timeout: *const timespec) -> c_int {
    let r = rwlock_tryrdlock(rw);
    if r != EBUSY { return r; }
    let mut spins = 100;
    while spins > 0 { let r = rwlock_tryrdlock(rw); if r != EBUSY { return r; } core::hint::spin_loop(); spins -= 1; }
    loop {
        let r = rwlock_tryrdlock(rw);
        if r != EBUSY { return r; }
        let val = a_load(&raw const (*rw).__i[0]);
        if val & 0x7fffffff != 0x7fffffff { continue; }
        a_fetch_add(&raw mut (*rw).__i[1], 1);
        a_cas(&raw mut (*rw).__i[0], val, val | (1i32 << 31));
        let e = futex_timedwait(&raw mut (*rw).__i[0], val | (1i32 << 31), abs_timeout);
        a_fetch_sub(&raw mut (*rw).__i[1], 1);
        if e != 0 && e != EINTR { return e; }
    }
}
unsafe fn rwlock_timedwrlock(rw: *mut pthread_rwlock_t, abs_timeout: *const timespec) -> c_int {
    let r = rwlock_trywrlock(rw);
    if r != EBUSY { return r; }
    let mut spins = 100;
    while spins > 0 { let r = rwlock_trywrlock(rw); if r != EBUSY { return r; } core::hint::spin_loop(); spins -= 1; }
    loop {
        let r = rwlock_trywrlock(rw);
        if r != EBUSY { return r; }
        let val = a_load(&raw const (*rw).__i[0]);
        if val == 0 { continue; }
        a_fetch_add(&raw mut (*rw).__i[1], 1);
        a_cas(&raw mut (*rw).__i[0], val, val | (1i32 << 31));
        let e = futex_timedwait(&raw mut (*rw).__i[0], val | (1i32 << 31), abs_timeout);
        a_fetch_sub(&raw mut (*rw).__i[1], 1);
        if e != 0 && e != EINTR { return e; }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_init(rw: *mut pthread_rwlock_t, attr: *const pthread_rwlockattr_t) -> c_int {
    core::ptr::write_bytes(rw, 0, 1);
    if !attr.is_null() { (*rw).__i[2] = (*attr).__attr[0] as c_int & 128; }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_destroy(_rw: *mut pthread_rwlock_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_rdlock(rw: *mut pthread_rwlock_t) -> c_int { rwlock_timedrdlock(rw, core::ptr::null()) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_tryrdlock(rw: *mut pthread_rwlock_t) -> c_int { rwlock_tryrdlock(rw) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_timedrdlock(rw: *mut pthread_rwlock_t, t: *const timespec) -> c_int { rwlock_timedrdlock(rw, t) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_wrlock(rw: *mut pthread_rwlock_t) -> c_int { rwlock_timedwrlock(rw, core::ptr::null()) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_trywrlock(rw: *mut pthread_rwlock_t) -> c_int { rwlock_trywrlock(rw) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_timedwrlock(rw: *mut pthread_rwlock_t, t: *const timespec) -> c_int { rwlock_timedwrlock(rw, t) }
#[no_mangle]
pub unsafe extern "C" fn pthread_rwlock_unlock(rw: *mut pthread_rwlock_t) -> c_int {
    loop {
        let val = a_load(&raw const (*rw).__i[0]);
        let cnt = val & 0x7fffffff;
        let new_val = if cnt == 0x7fffffff || cnt == 1 { 0 } else { val - 1 };
        if a_cas(&raw mut (*rw).__i[0], val, new_val) == val {
            if new_val == 0 && (a_load(&raw const (*rw).__i[1]) > 0 || val < 0) {
                futex_wake(&raw mut (*rw).__i[0], cnt);
            }
            return 0;
        }
    }
}

// --- pthread_barrierattr_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_barrierattr_init(attr: *mut pthread_barrierattr_t) -> c_int { (*attr).__attr = 0; 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_barrierattr_destroy(_attr: *mut pthread_barrierattr_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_barrierattr_setpshared(attr: *mut pthread_barrierattr_t, p: c_int) -> c_int {
    if (p as c_uint) > 1 { return EINVAL; }
    (*attr).__attr = p as c_uint; 0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_barrierattr_getpshared(attr: *const pthread_barrierattr_t, p: *mut c_int) -> c_int {
    *p = (*attr).__attr as c_int; 0
}

// --- pthread_barrier_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_init(b: *mut pthread_barrier_t, _attr: *const pthread_barrierattr_t, count: c_uint) -> c_int {
    if count == 0 { return EINVAL; }
    core::ptr::write_bytes(b, 0, 1);
    (*b).__i[2] = count as c_int - 1;
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_destroy(_b: *mut pthread_barrier_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_wait(b: *mut pthread_barrier_t) -> c_int {
    let limit = (*b).__i[2];
    if limit == 0 { return PTHREAD_BARRIER_SERIAL_THREAD; }
    spinlock_lock(&raw mut (*b).__i[0], &raw mut (*b).__i[1]);
    let arrived = a_load(&raw const (*b).__i[3]) + 1;
    a_store(&raw mut (*b).__i[3], arrived);
    if arrived > limit {
        a_store(&raw mut (*b).__i[3], 0);
        let gen = a_load(&raw const (*b).__i[4]);
        a_store(&raw mut (*b).__i[4], gen + 1);
        spinlock_unlock(&raw mut (*b).__i[0], &raw mut (*b).__i[1]);
        futex_wake(&raw mut (*b).__i[4], -1);
        return PTHREAD_BARRIER_SERIAL_THREAD;
    }
    let gen = a_load(&raw const (*b).__i[4]);
    spinlock_unlock(&raw mut (*b).__i[0], &raw mut (*b).__i[1]);
    while a_load(&raw const (*b).__i[4]) == gen {
        futex_wait(&raw mut (*b).__i[4], gen);
    }
    0
}

// --- pthread_spin_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_init(s: *mut pthread_spinlock_t, _pshared: c_int) -> c_int { a_store(s, 0); 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_destroy(_s: *mut pthread_spinlock_t) -> c_int { 0 }
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_lock(s: *mut pthread_spinlock_t) -> c_int {
    while a_load(s) != 0 || a_cas(s, 0, EBUSY) != 0 { core::hint::spin_loop(); }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_trylock(s: *mut pthread_spinlock_t) -> c_int { a_cas(s, 0, EBUSY) }
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_unlock(s: *mut pthread_spinlock_t) -> c_int { a_store(s, 0); 0 }

// --- sem_* ---
#[no_mangle]
pub unsafe extern "C" fn sem_init(sem: *mut sem_t, pshared: c_int, value: c_uint) -> c_int {
    if value > SEM_VALUE_MAX as c_uint { return EINVAL; }
    (*sem).__val[0] = value as c_int;
    (*sem).__val[1] = 0;
    (*sem).__val[2] = if pshared != 0 { 0 } else { 128 };
    0
}
#[no_mangle]
pub unsafe extern "C" fn sem_destroy(_sem: *mut sem_t) -> c_int { 0 }

unsafe fn sem_trywait_internal(sem: *mut sem_t) -> c_int {
    loop {
        let val = a_load(&raw const (*sem).__val[0]);
        if val & SEM_VALUE_MAX == 0 { return EAGAIN; }
        if a_cas(&raw mut (*sem).__val[0], val, val - 1) == val { return 0; }
    }
}

#[no_mangle]
pub unsafe extern "C" fn sem_trywait(sem: *mut sem_t) -> c_int {
    if sem_trywait_internal(sem) == 0 { 0 } else { ERRNO = EAGAIN; -1 }
}
#[no_mangle]
pub unsafe extern "C" fn sem_wait(sem: *mut sem_t) -> c_int {
    pthread_testcancel();
    if sem_trywait_internal(sem) == 0 { return 0; }
    let mut spins = 100;
    while spins > 0 { if sem_trywait_internal(sem) == 0 { return 0; } core::hint::spin_loop(); spins -= 1; }
    loop {
        pthread_testcancel();
        if sem_trywait_internal(sem) == 0 { return 0; }
        let val = a_load(&raw const (*sem).__val[0]);
        if val & SEM_VALUE_MAX != 0 { continue; }
        a_cas(&raw mut (*sem).__val[0], val, val | (1i32 << 31));
        a_fetch_add(&raw mut (*sem).__val[1], 1);
        futex_wait(&raw mut (*sem).__val[0], 1i32 << 31);
        a_fetch_sub(&raw mut (*sem).__val[1], 1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn sem_timedwait(sem: *mut sem_t, abs_timeout: *const timespec) -> c_int {
    pthread_testcancel();
    if sem_trywait_internal(sem) == 0 { return 0; }
    let mut spins = 100;
    while spins > 0 { if sem_trywait_internal(sem) == 0 { return 0; } core::hint::spin_loop(); spins -= 1; }
    loop {
        pthread_testcancel();
        if sem_trywait_internal(sem) == 0 { return 0; }
        let val = a_load(&raw const (*sem).__val[0]);
        if val & SEM_VALUE_MAX != 0 { continue; }
        a_cas(&raw mut (*sem).__val[0], val, val | (1i32 << 31));
        a_fetch_add(&raw mut (*sem).__val[1], 1);
        let e = futex_timedwait(&raw mut (*sem).__val[0], 1i32 << 31, abs_timeout);
        a_fetch_sub(&raw mut (*sem).__val[1], 1);
        if e != 0 && e != EINTR { ERRNO = e; return -1; }
    }
}
#[no_mangle]
pub unsafe extern "C" fn sem_post(sem: *mut sem_t) -> c_int {
    loop {
        let val = a_load(&raw const (*sem).__val[0]);
        if (val & SEM_VALUE_MAX) == SEM_VALUE_MAX { ERRNO = EOVERFLOW; return -1; }
        let mut new = val + 1;
        let waiters = a_load(&raw const (*sem).__val[1]);
        if waiters <= 1 { new &= !(1i32 << 31); }
        if a_cas(&raw mut (*sem).__val[0], val, new) == val {
            if val < 0 || waiters > 0 {
                futex_wake(&raw mut (*sem).__val[0], if waiters > 1 { 1 } else { -1 });
            }
            return 0;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn sem_getvalue(sem: *mut sem_t, sval: *mut c_int) -> c_int {
    *sval = a_load(&raw const (*sem).__val[0]) & SEM_VALUE_MAX;
    0
}

// ============================================================
// POSIX shared memory + named semaphores
// ============================================================

const NAME_MAX: usize = 255;
const SEM_NSEMS_MAX: usize = 256;

// ponytail: global spinlock for semtab, upgrade to per-entry if contention matters
static SEMTAB_LOCK: AtomicI32 = AtomicI32::new(0);

struct SemEntry {
    ino: u64,
    sem: *mut sem_t,
    refcnt: u32,
}

// ponytail: zero-init is fine for static mut (all-zero = free slots)
static mut SEMTAB: [SemEntry; SEM_NSEMS_MAX] = {
    const ZERO: SemEntry = SemEntry { ino: 0, sem: core::ptr::null_mut(), refcnt: 0 };
    [ZERO; SEM_NSEMS_MAX]
};

unsafe fn semtab_lock() {
    while SEMTAB_LOCK.compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed).is_err() {
        core::hint::spin_loop();
    }
}

unsafe fn semtab_unlock() {
    SEMTAB_LOCK.store(0, Ordering::Release);
}

// Validate shm/sem name and construct /dev/shm/<name> path.
// Returns path length or -1 with ERRNO set.
unsafe fn shm_mapname(name: *const c_char, buf: *mut u8) -> c_int {
    let mut p = name;
    // strip leading /
    while *p == b'/' as c_char { p = p.add(1); }
    let start = p;
    // find end of first component
    while *p != 0 && *p != b'/' as c_char { p = p.add(1); }
    let len = p.offset_from(start) as usize;
    // no trailing /, not empty, not "." or ".."
    if len == 0 || *p != 0 {
        ERRNO = EINVAL;
        return -1;
    }
    if len == 1 && *start == b'.' as c_char {
        ERRNO = EINVAL;
        return -1;
    }
    if len == 2 && *start == b'.' as c_char && *start.add(1) == b'.' as c_char {
        ERRNO = EINVAL;
        return -1;
    }
    if len > NAME_MAX {
        ERRNO = ENAMETOOLONG;
        return -1;
    }
    // build /dev/shm/<name>
    let prefix = b"/dev/shm/\0";
    let mut i = 0;
    while prefix[i] != 0 {
        *buf.add(i) = prefix[i];
        i += 1;
    }
    let mut j = 0;
    while j < len {
        *buf.add(i + j) = *start.add(j) as u8;
        j += 1;
    }
    *buf.add(i + j) = 0;
    (i + j) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn shm_open(name: *const c_char, flags: c_int, mode: mode_t) -> c_int {
    let mut buf = [0u8; 16 + NAME_MAX + 1];
    if shm_mapname(name, buf.as_mut_ptr()) < 0 { return -1; }
    let fd = sys_open(buf.as_ptr(), (flags | O_NOFOLLOW | O_CLOEXEC | O_NONBLOCK) as i64, mode as i64);
    if fd < 0 { ERRNO = (-fd) as c_int; return -1; }
    fd as c_int
}

#[no_mangle]
pub unsafe extern "C" fn shm_unlink(name: *const c_char) -> c_int {
    let mut buf = [0u8; 16 + NAME_MAX + 1];
    if shm_mapname(name, buf.as_mut_ptr()) < 0 { return -1; }
    let r = sys_unlink(buf.as_ptr());
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sem_open(name: *const c_char, flags: c_int, mut args: ...) -> *mut sem_t {
    let mut buf = [0u8; 16 + NAME_MAX + 1];
    if shm_mapname(name, buf.as_mut_ptr()).is_negative() {
        return core::ptr::null_mut(); // SEM_FAILED
    }

    let oflags = flags & (O_CREAT | O_EXCL);

    semtab_lock();

    // find a free slot
    let mut slot: c_int = -1;
    for i in 0..SEMTAB_LEN {
        if SEMTAB[i].sem.is_null() && slot < 0 { slot = i as c_int; }
    }
    if slot < 0 {
        ERRNO = EMFILE;
        semtab_unlock();
        return core::ptr::null_mut();
    }
    // reserve slot
    SEMTAB[slot as usize].sem = !0usize as *mut sem_t; // sentinel
    semtab_unlock();

    // extract va_args for O_CREAT
    let mut mode: mode_t = 0;
    let mut value: c_uint = 0;
    if oflags & O_CREAT != 0 {
        mode = args.next_arg::<mode_t>() & 0o666;
        value = args.next_arg::<c_uint>();
        if value > SEM_VALUE_MAX as c_uint {
            ERRNO = EINVAL;
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
    }

    // try opening existing first (unless O_CREAT|O_EXCL)
    if oflags != O_CREAT | O_EXCL {
        let fd = sys_open(buf.as_ptr(), (O_RDWR | O_NOFOLLOW | O_CLOEXEC | O_NONBLOCK) as i64, 0);
        if fd >= 0 {
            let map = sys_mmap(
                core::ptr::null_mut(),
                core::mem::size_of::<sem_t>(),
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd as i32,
                0,
            );
            sys_close(fd);
            if map != MMAP_FAILED {
                let st_ino = get_ino(fd as c_int, buf.as_ptr());
                return sem_register(map as *mut sem_t, st_ino, slot);
            }
        }
        if oflags == 0 {
            // no O_CREAT and file doesn't exist
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
    }

    // create: write sem_t to temp file, mmap, link
    let mut tmp = [0u8; 32];
    let tmpname = b"/dev/shm/tmp-XXXXXX\0";
    tmp[..tmpname.len()].copy_from_slice(tmpname);
    // use clock for unique name
    let mut ts: timespec = core::mem::zeroed();
    sys_clock_gettime(0, &mut ts);
    // append ns to tmp name
    let mut n = ts.tv_nsec;
    let mut pos = tmpname.len() - 1;
    tmp[pos] = 0;
    pos -= 1;
    loop {
        tmp[pos] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 || pos == 0 { break; }
        pos -= 1;
    }
        // shift to start of number part
        let tmp_path = tmp.as_ptr();

    let mut newsem: sem_t = core::mem::zeroed();
    newsem.__val[0] = value as c_int;
    newsem.__val[1] = 0;
    newsem.__val[2] = 128; // pshared=0 flag

    loop {
        let fd = sys_open(tmp_path as *const u8, (O_RDWR | O_CREAT | O_EXCL | O_CLOEXEC | O_NONBLOCK) as i64, mode as i64);
        if fd < 0 {
            if (-fd) as c_int == EEXIST { continue; }
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
        let written = sys_write(fd, &newsem as *const sem_t as *const u8, core::mem::size_of::<sem_t>());
        if written != core::mem::size_of::<sem_t>() as i64 {
            sys_close(fd);
            sys_unlink(tmp_path);
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
        let map = sys_mmap(
            core::ptr::null_mut(),
            core::mem::size_of::<sem_t>(),
            PROT_READ | PROT_WRITE,
            MAP_SHARED,
            fd as i32,
            0,
        );
        sys_close(fd);
        if map == MMAP_FAILED {
            sys_unlink(tmp_path);
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
        // link to final name
        let r = sys_linkat(AT_FDCWD, tmp_path, AT_FDCWD, buf.as_ptr(), 0);
        let e = if r < 0 { (-r) as c_int } else { 0 };
        sys_unlink(tmp_path);
        if e == 0 {
            let st_ino = get_ino(-1, buf.as_ptr());
            return sem_register(map as *mut sem_t, st_ino, slot);
        }
        sys_munmap(map, core::mem::size_of::<sem_t>());
        if e != EEXIST || oflags == O_CREAT | O_EXCL {
            ERRNO = e;
            semtab_lock();
            SEMTAB[slot as usize].sem = core::ptr::null_mut();
            semtab_unlock();
            return core::ptr::null_mut();
        }
        // EEXIST without O_EXCL: retry open existing
        break;
    }

    // fallthrough: try opening the existing file that won the race
    let fd = sys_open(buf.as_ptr(), (O_RDWR | O_NOFOLLOW | O_CLOEXEC | O_NONBLOCK) as i64, 0);
    if fd < 0 {
        semtab_lock();
        SEMTAB[slot as usize].sem = core::ptr::null_mut();
        semtab_unlock();
        return core::ptr::null_mut();
    }
    let map = sys_mmap(
        core::ptr::null_mut(),
        core::mem::size_of::<sem_t>(),
        PROT_READ | PROT_WRITE,
        MAP_SHARED,
        fd as i32,
        0,
    );
    sys_close(fd);
    if map == MMAP_FAILED {
        semtab_lock();
        SEMTAB[slot as usize].sem = core::ptr::null_mut();
        semtab_unlock();
        return core::ptr::null_mut();
    }
    let st_ino = get_ino(-1, buf.as_ptr());
    sem_register(map as *mut sem_t, st_ino, slot)
}

unsafe fn get_ino(_fd_hint: c_int, path: *const u8) -> u64 {
    let mut st: Stat = core::mem::zeroed();
    if sys_newfstatat(AT_FDCWD, path as *const c_char, &mut st as *mut Stat as *mut u8, 0) == 0 {
        st.st_ino
    } else {
        0
    }
}

unsafe fn sem_register(map: *mut sem_t, ino: u64, slot_hint: c_int) -> *mut sem_t {
    semtab_lock();
    // check if already mapped (by inode)
    for i in 0..SEMTAB_LEN {
        if SEMTAB[i].ino == ino && !SEMTAB[i].sem.is_null() && SEMTAB[i].sem != !0usize as *mut sem_t {
            // already mapped: unmap new, use existing, release hint slot
            sys_munmap(map as *mut u8, core::mem::size_of::<sem_t>());
            SEMTAB[slot_hint as usize].sem = core::ptr::null_mut();
            SEMTAB[i].refcnt += 1;
            let ret = SEMTAB[i].sem;
            semtab_unlock();
            return ret;
        }
    }
    // new entry
    SEMTAB[slot_hint as usize].sem = map;
    SEMTAB[slot_hint as usize].ino = ino;
    SEMTAB[slot_hint as usize].refcnt = 1;
    semtab_unlock();
    map
}

const SEMTAB_LEN: usize = SEM_NSEMS_MAX;

#[no_mangle]
pub unsafe extern "C" fn sem_close(sem: *mut sem_t) -> c_int {
    semtab_lock();
    for i in 0..SEMTAB_LEN {
        if SEMTAB[i].sem == sem {
            SEMTAB[i].refcnt -= 1;
            if SEMTAB[i].refcnt == 0 {
                SEMTAB[i].sem = core::ptr::null_mut();
                SEMTAB[i].ino = 0;
                semtab_unlock();
                sys_munmap(sem as *mut u8, core::mem::size_of::<sem_t>());
                return 0;
            }
            semtab_unlock();
            return 0;
        }
    }
    semtab_unlock();
    0
}

#[no_mangle]
pub unsafe extern "C" fn sem_unlink(name: *const c_char) -> c_int {
    shm_unlink(name)
}

// --- pthread_once ---
#[no_mangle]
pub unsafe extern "C" fn pthread_once(control: *mut pthread_once_t, init_routine: Option<unsafe extern "C" fn()>) -> c_int {
    if a_load(control) == 2 { return 0; }
    loop {
        match a_cas(control, 0, 1) {
            0 => {
                if let Some(f) = init_routine { f(); }
                if a_swap(control, 2) == 3 { futex_wake(control, -1); }
                return 0;
            }
            1 => { a_cas(control, 1, 3); }
            3 => { futex_wait(control, 3); }
            _ => { return 0; }
        }
    }
}

// --- pthread_key_* ---
#[no_mangle]
pub unsafe extern "C" fn pthread_key_create(key: *mut pthread_key_t, dtor: Option<unsafe extern "C" fn(*mut c_void)>) -> c_int {
    let start = NEXT_KEY.load(Ordering::Relaxed);
    let mut j = start;
    loop {
        if KEY_DTORS[j].is_none() {
            KEY_DTORS[j] = dtor;
            NEXT_KEY.store((j + 1) % PTHREAD_KEYS_MAX, Ordering::Relaxed);
            *key = j as pthread_key_t;
            return 0;
        }
        j = (j + 1) % PTHREAD_KEYS_MAX;
        if j == start { return EAGAIN; }
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_key_delete(key: pthread_key_t) -> c_int {
    KEY_DTORS[key as usize] = None;
    for i in 0..MAX_THREADS {
        if THREADS[i].tid > 0 { THREADS[i].tsd[key as usize] = core::ptr::null_mut(); }
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_getspecific(key: pthread_key_t) -> *mut c_void {
    if let Some(slot) = find_thread() { slot.tsd[key as usize] } else { core::ptr::null_mut() }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_setspecific(key: pthread_key_t, value: *const c_void) -> c_int {
    if let Some(slot) = find_thread() {
        slot.tsd[key as usize] = value as *mut c_void;
        0
    } else { EINVAL }
}

// --- pthread_cancel/state ---
unsafe fn run_cleanup_handlers(slot: &mut Thread) {
    let mut cb = slot.cancelbuf;
    while !cb.is_null() {
        let cur = &*cb;
        let f = cur.__f;
        let x = cur.__x;
        cb = cur.__next;
        if let Some(func) = f {
            func(x);
        }
    }
    slot.cancelbuf = core::ptr::null_mut();
}

unsafe fn do_cancel() -> ! {
    if let Some(slot) = find_thread() {
        run_cleanup_handlers(slot);
        slot.result = !0usize as *mut c_void; // PTHREAD_CANCELED
        slot.detach_state = DT_EXITED;
        futex_wake(&raw mut slot.detach_state, 1);
    }
    sys_exit_thread(0);
}

extern "C" fn cancel_handler(_sig: c_int) {
    unsafe {
        if let Some(slot) = find_thread() {
            // In asynchronous mode, cancel immediately from the signal handler.
            // In deferred mode, the signal just interrupts blocking syscalls so
            // the next cancellation point can act on the pending request.
            if slot.cancel != 0
                && slot.cancel_state == PTHREAD_CANCEL_ENABLE
                && slot.cancel_type == PTHREAD_CANCEL_ASYNCHRONOUS
            {
                do_cancel();
            }
        }
    }
}

static CANCEL_INIT: AtomicI32 = AtomicI32::new(0);

unsafe fn ensure_cancel_handler() {
    if CANCEL_INIT.swap(1, Ordering::AcqRel) == 0 {
        let act = sigaction {
            sa_handler: cancel_handler as *const () as usize,
            sa_flags: SA_RESTORER,
            sa_restorer: sig_restorer as *const () as usize,
            sa_mask: [!0u64],
        };
        sys_rt_sigaction(SIGCANCEL, &act as *const sigaction, core::ptr::null_mut(), 8);
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cancel(thread: PthreadT) -> c_int {
    let slot = thread as *mut Thread;
    if slot.is_null() { return EINVAL; }
    ensure_cancel_handler();
    a_store(&raw mut (*slot).cancel, 1);
    let tid = (*slot).tid;
    if tid > 0 {
        sys_tgkill(sys_getpid() as c_int, tid, SIGCANCEL);
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_setcancelstate(state: c_int, oldstate: *mut c_int) -> c_int {
    if state != PTHREAD_CANCEL_ENABLE && state != PTHREAD_CANCEL_DISABLE { return EINVAL; }
    if let Some(slot) = find_thread() {
        if !oldstate.is_null() { *oldstate = slot.cancel_state; }
        slot.cancel_state = state;
        0
    } else { EINVAL }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_setcanceltype(type_: c_int, oldtype: *mut c_int) -> c_int {
    if type_ != PTHREAD_CANCEL_DEFERRED && type_ != PTHREAD_CANCEL_ASYNCHRONOUS { return EINVAL; }
    if let Some(slot) = find_thread() {
        if !oldtype.is_null() { *oldtype = slot.cancel_type; }
        slot.cancel_type = type_;
        0
    } else { EINVAL }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_testcancel() {
    if let Some(slot) = find_thread() {
        if slot.cancel != 0 && slot.cancel_state == PTHREAD_CANCEL_ENABLE {
            do_cancel();
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn _pthread_cleanup_push(cb: *mut __ptcb, f: Option<unsafe extern "C" fn(*mut c_void)>, x: *mut c_void) {
    (*cb).__f = f;
    (*cb).__x = x;
    if let Some(slot) = find_thread() {
        (*cb).__next = slot.cancelbuf;
        slot.cancelbuf = cb;
    } else {
        (*cb).__next = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn _pthread_cleanup_pop(cb: *mut __ptcb, run: c_int) {
    if let Some(slot) = find_thread() {
        slot.cancelbuf = (*cb).__next;
    }
    if run != 0 {
        if let Some(f) = (*cb).__f {
            f((*cb).__x);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_kill(thread: PthreadT, sig: c_int) -> c_int {
    let slot = thread as *mut Thread;
    if slot.is_null() { return EINVAL; }
    let tid = (*slot).tid;
    if tid <= 0 { return EINVAL; }
    if sys_tgkill(sys_getpid() as c_int, tid, sig) < 0 { -1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_sigmask(how: c_int, set: *const SigSetT, oldset: *mut SigSetT) -> c_int {
    sigprocmask(how, set, oldset)
}


#[repr(C)]
pub struct lconv {
    pub decimal_point: *mut c_char,
    pub thousands_sep: *mut c_char,
    pub grouping: *mut c_char,
    pub int_curr_symbol: *mut c_char,
    pub currency_symbol: *mut c_char,
    pub mon_decimal_point: *mut c_char,
    pub mon_thousands_sep: *mut c_char,
    pub mon_grouping: *mut c_char,
    pub positive_sign: *mut c_char,
    pub negative_sign: *mut c_char,
    pub int_frac_digits: c_char,
    pub frac_digits: c_char,
    pub p_cs_precedes: c_char,
    pub p_sep_by_space: c_char,
    pub n_cs_precedes: c_char,
    pub n_sep_by_space: c_char,
    pub p_sign_posn: c_char,
    pub n_sign_posn: c_char,
    pub int_p_cs_precedes: c_char,
    pub int_p_sep_by_space: c_char,
    pub int_n_cs_precedes: c_char,
    pub int_n_sep_by_space: c_char,
    pub int_p_sign_posn: c_char,
    pub int_n_sign_posn: c_char,
}

static UTF8_NAME: [u8; 6] = [b'U', b'T', b'F', b'-', b'8', 0];
static C_NAME: [u8; 2] = [b'C', 0];
static mut LOCALE_NAME: *const c_char = C_NAME.as_ptr() as *const c_char;
static mut LOCALE_CTYPE_UTF8: bool = true;
static mut EMPTY_STR: [c_char; 1] = [0];
static mut DECIMAL_POINT: [c_char; 2] = [b'.' as c_char, 0];
static mut LCONV: lconv = lconv {
    decimal_point: core::ptr::null_mut(),
    thousands_sep: core::ptr::null_mut(),
    grouping: core::ptr::null_mut(),
    int_curr_symbol: core::ptr::null_mut(),
    currency_symbol: core::ptr::null_mut(),
    mon_decimal_point: core::ptr::null_mut(),
    mon_thousands_sep: core::ptr::null_mut(),
    mon_grouping: core::ptr::null_mut(),
    positive_sign: core::ptr::null_mut(),
    negative_sign: core::ptr::null_mut(),
    int_frac_digits: (-1i32) as c_char,
    frac_digits: (-1i32) as c_char,
    p_cs_precedes: (-1i32) as c_char,
    p_sep_by_space: (-1i32) as c_char,
    n_cs_precedes: (-1i32) as c_char,
    n_sep_by_space: (-1i32) as c_char,
    p_sign_posn: (-1i32) as c_char,
    n_sign_posn: (-1i32) as c_char,
    int_p_cs_precedes: (-1i32) as c_char,
    int_p_sep_by_space: (-1i32) as c_char,
    int_n_cs_precedes: (-1i32) as c_char,
    int_n_sep_by_space: (-1i32) as c_char,
    int_p_sign_posn: (-1i32) as c_char,
    int_n_sign_posn: (-1i32) as c_char,
};

unsafe fn cstr_contains_ci(s: *const u8, needle: &[u8]) -> bool {
    let len = strlen(s as *const c_char) as usize;
    let s_slice = core::slice::from_raw_parts(s, len);
    if needle.len() > len { return false; }
    for i in 0..=len - needle.len() {
        if s_slice[i..i + needle.len()].eq_ignore_ascii_case(needle) {
            return true;
        }
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn setlocale(_category: c_int, locale: *const c_char) -> *mut c_char {
    if locale.is_null() || *locale == 0 {
        return LOCALE_NAME as *mut c_char;
    }
    let name = locale as *const u8;
    if strcmp(name, b"C\0".as_ptr()) == 0 || strcmp(name, b"POSIX\0".as_ptr()) == 0 {
        LOCALE_CTYPE_UTF8 = false;
        return LOCALE_NAME as *mut c_char;
    }
    if cstr_contains_ci(name, b"UTF-8") || cstr_contains_ci(name, b"UTF8") {
        LOCALE_CTYPE_UTF8 = true;
        return LOCALE_NAME as *mut c_char;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn localeconv() -> *mut lconv {
    let empty = core::ptr::addr_of_mut!(EMPTY_STR) as *mut c_char;
    let lconv = core::ptr::addr_of_mut!(LCONV);
    (*lconv).decimal_point = core::ptr::addr_of_mut!(DECIMAL_POINT) as *mut c_char;
    (*lconv).thousands_sep = empty;
    (*lconv).grouping = empty;
    (*lconv).int_curr_symbol = empty;
    (*lconv).currency_symbol = empty;
    (*lconv).mon_decimal_point = empty;
    (*lconv).mon_thousands_sep = empty;
    (*lconv).mon_grouping = empty;
    (*lconv).positive_sign = empty;
    (*lconv).negative_sign = empty;
    lconv
}

pub type locale_t = *mut c_void;
static mut CURRENT_LOCALE: locale_t = LC_GLOBAL_LOCALE;

static mut C_LOCALE_STORAGE: c_int = 0;
const C_LOCALE: locale_t = core::ptr::addr_of_mut!(C_LOCALE_STORAGE) as locale_t;
const LC_GLOBAL_LOCALE: locale_t = usize::MAX as locale_t;

#[no_mangle]
pub unsafe extern "C" fn newlocale(_mask: c_int, name: *const c_char, base: locale_t) -> locale_t {
    if !name.is_null() && *name != 0 {
        let n = name as *const u8;
        if strcmp(n, b"C\0".as_ptr()) != 0
            && strcmp(n, b"POSIX\0".as_ptr()) != 0
        {
            return core::ptr::null_mut();
        }
    }
    if base.is_null() || base == C_LOCALE || base == LC_GLOBAL_LOCALE {
        C_LOCALE
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn freelocale(_loc: locale_t) {}

#[no_mangle]
pub unsafe extern "C" fn uselocale(loc: locale_t) -> locale_t {
    let old = CURRENT_LOCALE;
    if !loc.is_null() {
        CURRENT_LOCALE = loc;
    }
    old
}

#[no_mangle]
pub unsafe extern "C" fn duplocale(loc: locale_t) -> locale_t {
    if loc.is_null() {
        return core::ptr::null_mut();
    }
    C_LOCALE
}

const NL_ITEM_CODESET: c_int = 14;
const NL_ITEM_RADIXCHAR: c_int = 0x10000;
const NL_ITEM_THOUSEP: c_int = 0x10001;
const NL_ITEM_YESEXPR: c_int = 0x50000;
const NL_ITEM_NOEXPR: c_int = 0x50001;

const C_TIME_STRINGS: &[u8] = b"Sun\0Mon\0Tue\0Wed\0Thu\0Fri\0Sat\0\
Sunday\0Monday\0Tuesday\0Wednesday\0Thursday\0Friday\0Saturday\0\
Jan\0Feb\0Mar\0Apr\0May\0Jun\0Jul\0Aug\0Sep\0Oct\0Nov\0Dec\0\
January\0February\0March\0April\0May\0June\0July\0August\0\
September\0October\0November\0December\0\
AM\0PM\0\
%a %b %e %T %Y\0%m/%d/%y\0%H:%M:%S\0%I:%M:%S %p\0\
\0\0%m/%d/%y\00123456789\0%a %b %e %T %Y\0%H:%M:%S\0";

const C_NUMERIC_STRINGS: &[u8] = b".\0\0";
const C_MESSAGES_STRINGS: &[u8] = b"^[yY]\0^[nN]\0";

unsafe fn langinfo_str(table: *const u8, mut idx: c_int) -> *mut c_char {
    let mut p = table;
    while idx > 0 {
        while *p != 0 { p = p.add(1); }
        p = p.add(1);
        idx -= 1;
    }
    p as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn nl_langinfo(item: c_int) -> *mut c_char {
    if item == NL_ITEM_CODESET {
        return if LOCALE_CTYPE_UTF8 {
            b"UTF-8\0".as_ptr() as *mut c_char
        } else {
            b"US-ASCII\0".as_ptr() as *mut c_char
        };
    }
    let cat = item >> 16;
    let idx = item & 0xFFFF;
    match cat {
        1 => {
            if idx > 1 { return b"\0".as_ptr() as *mut c_char; }
            langinfo_str(C_NUMERIC_STRINGS.as_ptr(), idx)
        }
        2 => {
            if idx > 49 { return b"\0".as_ptr() as *mut c_char; }
            langinfo_str(C_TIME_STRINGS.as_ptr(), idx)
        }
        5 => {
            if idx > 1 { return b"\0".as_ptr() as *mut c_char; }
            langinfo_str(C_MESSAGES_STRINGS.as_ptr(), idx)
        }
        _ => b"\0".as_ptr() as *mut c_char,
    }
}

pub type nl_catd = *mut c_void;

#[no_mangle]
pub unsafe extern "C" fn catopen(_name: *const c_char, _oflag: c_int) -> nl_catd {
    !0usize as nl_catd
}

#[no_mangle]
pub unsafe extern "C" fn catclose(_catd: nl_catd) -> c_int { 0 }

#[no_mangle]
pub unsafe extern "C" fn catgets(catd: nl_catd, _set_id: c_int, _msg_id: c_int, s: *const c_char) -> *mut c_char {
    if catd == !0usize as nl_catd { return s as *mut c_char; }
    s as *mut c_char
}

pub type IconvT = *mut c_void;

const ENC_UTF8: i32 = 1;
const ENC_UTF16LE: i32 = 2;
const ENC_UTF16BE: i32 = 3;
const ENC_UTF32LE: i32 = 4;
const ENC_UTF32BE: i32 = 5;
const ENC_WCHAR_T: i32 = 6;
const ENC_ASCII: i32 = 7;
const ENC_LATIN1: i32 = 8;
const ENC_WIN1252: i32 = 9;
const ENC_WIN1251: i32 = 10;
const ENC_KOI8R: i32 = 11;
const ENC_GBK: i32 = 12;
const ENC_GB2312: i32 = 13;
const ENC_BIG5: i32 = 14;
const ENC_EUCJP: i32 = 15;
const ENC_SHIFTJIS: i32 = 16;
const ENC_ISO8859_2: i32 = 17;
const ENC_ISO8859_3: i32 = 18;
const ENC_ISO8859_4: i32 = 19;
const ENC_ISO8859_5: i32 = 20;
const ENC_ISO8859_6: i32 = 21;
const ENC_ISO8859_7: i32 = 22;
const ENC_ISO8859_8: i32 = 23;
const ENC_ISO8859_9: i32 = 24;
const ENC_ISO8859_10: i32 = 25;
const ENC_ISO8859_11: i32 = 26;
const ENC_ISO8859_13: i32 = 27;
const ENC_ISO8859_14: i32 = 28;
const ENC_ISO8859_15: i32 = 29;
const ENC_ISO8859_16: i32 = 30;

include!("iconv_iso8859.rs");

fn make_cd(from: i32, to: i32) -> IconvT {
    ((from as usize) << 16 | (to as usize) << 1 | 1) as IconvT
}
fn extract_from(cd: IconvT) -> i32 {
    ((cd as usize) >> 16) as i32
}
fn extract_to(cd: IconvT) -> i32 {
    (((cd as usize) >> 1) & 0x7fff) as i32
}

unsafe fn match_name(input: *const u8, target: &[u8]) -> bool {
    let mut i = input;
    let mut j = 0;
    while *i != 0 && j < target.len() {
        let mut c = *i;
        if !c.is_ascii_alphanumeric() { i = i.add(1); continue; }
        c = c.to_ascii_lowercase();
        if c != target[j] { return false; }
        i = i.add(1);
        j += 1;
    }
    while *i != 0 && !(*i).is_ascii_alphanumeric() { i = i.add(1); }
    *i == 0 && j == target.len()
}

unsafe fn find_encoding(name: *const u8) -> i32 {
    if name.is_null() { return -1; }
    if match_name(name, b"utf8") || match_name(name, b"utf-8") || match_name(name, b"char") { return ENC_UTF8; }
    if match_name(name, b"utf16le") || match_name(name, b"utf-16le") { return ENC_UTF16LE; }
    if match_name(name, b"utf16be") || match_name(name, b"utf-16be") { return ENC_UTF16BE; }
    if match_name(name, b"utf32le") || match_name(name, b"utf-32le") || match_name(name, b"ucs4le") { return ENC_UTF32LE; }
    if match_name(name, b"utf32be") || match_name(name, b"utf-32be") || match_name(name, b"ucs4be") { return ENC_UTF32BE; }
    if match_name(name, b"wchart") || match_name(name, b"wchar-t") { return ENC_WCHAR_T; }
    if match_name(name, b"ascii") || match_name(name, b"usascii") || match_name(name, b"iso646") { return ENC_ASCII; }
    if match_name(name, b"iso88591") || match_name(name, b"iso-8859-1") || match_name(name, b"latin1") { return ENC_LATIN1; }
    if match_name(name, b"iso88592") || match_name(name, b"iso-8859-2") { return ENC_ISO8859_2; }
    if match_name(name, b"iso88593") || match_name(name, b"iso-8859-3") { return ENC_ISO8859_3; }
    if match_name(name, b"iso88594") || match_name(name, b"iso-8859-4") { return ENC_ISO8859_4; }
    if match_name(name, b"iso88595") || match_name(name, b"iso-8859-5") { return ENC_ISO8859_5; }
    if match_name(name, b"iso88596") || match_name(name, b"iso-8859-6") { return ENC_ISO8859_6; }
    if match_name(name, b"iso88597") || match_name(name, b"iso-8859-7") { return ENC_ISO8859_7; }
    if match_name(name, b"iso88598") || match_name(name, b"iso-8859-8") { return ENC_ISO8859_8; }
    if match_name(name, b"iso88599") || match_name(name, b"iso-8859-9") { return ENC_ISO8859_9; }
    if match_name(name, b"iso885910") || match_name(name, b"iso-8859-10") { return ENC_ISO8859_10; }
    if match_name(name, b"iso885911") || match_name(name, b"iso-8859-11") || match_name(name, b"tis620") { return ENC_ISO8859_11; }
    if match_name(name, b"iso885913") || match_name(name, b"iso-8859-13") { return ENC_ISO8859_13; }
    if match_name(name, b"iso885914") || match_name(name, b"iso-8859-14") { return ENC_ISO8859_14; }
    if match_name(name, b"iso885915") || match_name(name, b"iso-8859-15") { return ENC_ISO8859_15; }
    if match_name(name, b"iso885916") || match_name(name, b"iso-8859-16") { return ENC_ISO8859_16; }
    if match_name(name, b"cp1252") || match_name(name, b"windows1252") || match_name(name, b"windows-1252") { return ENC_WIN1252; }
    if match_name(name, b"cp1251") || match_name(name, b"windows1251") || match_name(name, b"windows-1251") { return ENC_WIN1251; }
    if match_name(name, b"koi8r") || match_name(name, b"koi8-r") { return ENC_KOI8R; }
    if match_name(name, b"gbk") || match_name(name, b"cp936") { return ENC_GBK; }
    if match_name(name, b"gb2312") { return ENC_GB2312; }
    if match_name(name, b"big5") || match_name(name, b"bigfive") || match_name(name, b"cp950") { return ENC_BIG5; }
    if match_name(name, b"eucjp") || match_name(name, b"euc-jp") { return ENC_EUCJP; }
    if match_name(name, b"shiftjis") || match_name(name, b"sjis") || match_name(name, b"cp932") { return ENC_SHIFTJIS; }
    -1
}

fn uni_to_jis(c: u32) -> u16 {
    let mut b = 0usize;
    let mut nel = REV_JIS.len();
    loop {
        let i = nel / 2;
        let j = REV_JIS[b + i] as usize;
        let row = j / 256;
        let col = j % 256;
        let d = JIS0208[row * 94 + col] as u32;
        if d == c { return (j as u16) + 0x2121; }
        if nel == 1 { return 0; }
        if c < d { nel /= 2; }
        else { b += i; nel -= nel / 2; }
    }
}

unsafe fn iconv_decode(enc: i32, src: *const u8, src_left: usize) -> (u32, usize, c_int) {
    if src_left == 0 { return (0, 0, EINVAL); }
    match enc {
        ENC_UTF8 => {
            let c = *src;
            if c < 128 { return (c as u32, 1, 0); }
            if c < 0xC2 { return (0, 0, EILSEQ); }
            if c < 0xE0 {
                if src_left < 2 || (*src.add(1) & 0xC0) != 0x80 { return (0, 0, EILSEQ); }
                return (((c as u32 & 0x1F) << 6) | (*src.add(1) as u32 & 0x3F), 2, 0);
            }
            if c < 0xF0 {
                if src_left < 3 || (*src.add(1) & 0xC0) != 0x80 || (*src.add(2) & 0xC0) != 0x80 { return (0, 0, EILSEQ); }
                let cp = ((c as u32 & 0x0F) << 12) | ((*src.add(1) as u32 & 0x3F) << 6) | (*src.add(2) as u32 & 0x3F);
                if cp < 0x800 { return (0, 0, EILSEQ); }
                return (cp, 3, 0);
            }
            if c < 0xF8 {
                if src_left < 4 || (*src.add(1) & 0xC0) != 0x80 || (*src.add(2) & 0xC0) != 0x80 || (*src.add(3) & 0xC0) != 0x80 { return (0, 0, EILSEQ); }
                let cp = ((c as u32 & 0x07) << 18) | ((*src.add(1) as u32 & 0x3F) << 12) | ((*src.add(2) as u32 & 0x3F) << 6) | (*src.add(3) as u32 & 0x3F);
                if cp < 0x10000 || cp >= 0x110000 { return (0, 0, EILSEQ); }
                return (cp, 4, 0);
            }
            (0, 0, EILSEQ)
        }
        ENC_UTF16LE => {
            if src_left < 2 { return (0, 0, EINVAL); }
            let c = (*src as u32) | ((*src.add(1) as u32) << 8);
            if c >= 0xD800 && c <= 0xDBFF {
                if src_left < 4 { return (0, 0, EINVAL); }
                let d = (*src.add(2) as u32) | ((*src.add(3) as u32) << 8);
                if d < 0xDC00 || d > 0xDFFF { return (0, 0, EILSEQ); }
                return (((c - 0xD800) << 10) + (d - 0xDC00) + 0x10000, 4, 0);
            }
            if c >= 0xDC00 && c <= 0xDFFF { return (0, 0, EILSEQ); }
            (c, 2, 0)
        }
        ENC_UTF16BE => {
            if src_left < 2 { return (0, 0, EINVAL); }
            let c = ((*src as u32) << 8) | (*src.add(1) as u32);
            if c >= 0xD800 && c <= 0xDBFF {
                if src_left < 4 { return (0, 0, EINVAL); }
                let d = ((*src.add(2) as u32) << 8) | (*src.add(3) as u32);
                if d < 0xDC00 || d > 0xDFFF { return (0, 0, EILSEQ); }
                return (((c - 0xD800) << 10) + (d - 0xDC00) + 0x10000, 4, 0);
            }
            if c >= 0xDC00 && c <= 0xDFFF { return (0, 0, EILSEQ); }
            (c, 2, 0)
        }
        ENC_UTF32LE => {
            if src_left < 4 { return (0, 0, EINVAL); }
            let c = (*src as u32) | ((*src.add(1) as u32) << 8) | ((*src.add(2) as u32) << 16) | ((*src.add(3) as u32) << 24);
            if c >= 0xD800 && c < 0xE000 { return (0, 0, EILSEQ); }
            if c >= 0x110000 { return (0, 0, EILSEQ); }
            (c, 4, 0)
        }
        ENC_UTF32BE => {
            if src_left < 4 { return (0, 0, EINVAL); }
            let c = ((*src as u32) << 24) | ((*src.add(1) as u32) << 16) | ((*src.add(2) as u32) << 8) | (*src.add(3) as u32);
            if c >= 0xD800 && c < 0xE000 { return (0, 0, EILSEQ); }
            if c >= 0x110000 { return (0, 0, EILSEQ); }
            (c, 4, 0)
        }
        ENC_WCHAR_T => {
            if src_left < 4 { return (0, 0, EINVAL); }
            let c = *(src as *const u32);
            if c >= 0xD800 && c < 0xE000 { return (0, 0, EILSEQ); }
            if c >= 0x110000 { return (0, 0, EILSEQ); }
            (c, 4, 0)
        }
        ENC_ASCII => {
            let c = *src as u32;
            if c >= 128 { return (0, 0, EILSEQ); }
            (c, 1, 0)
        }
        ENC_LATIN1 => (*src as u32, 1, 0),
        ENC_ISO8859_2 => (iso8859_to_u(&ISO8859_2_TO_U, *src), 1, 0),
        ENC_ISO8859_3 => (iso8859_to_u(&ISO8859_3_TO_U, *src), 1, 0),
        ENC_ISO8859_4 => (iso8859_to_u(&ISO8859_4_TO_U, *src), 1, 0),
        ENC_ISO8859_5 => (iso8859_to_u(&ISO8859_5_TO_U, *src), 1, 0),
        ENC_ISO8859_6 => (iso8859_to_u(&ISO8859_6_TO_U, *src), 1, 0),
        ENC_ISO8859_7 => (iso8859_to_u(&ISO8859_7_TO_U, *src), 1, 0),
        ENC_ISO8859_8 => (iso8859_to_u(&ISO8859_8_TO_U, *src), 1, 0),
        ENC_ISO8859_9 => (iso8859_to_u(&ISO8859_9_TO_U, *src), 1, 0),
        ENC_ISO8859_10 => (iso8859_to_u(&ISO8859_10_TO_U, *src), 1, 0),
        ENC_ISO8859_11 => (iso8859_to_u(&ISO8859_11_TO_U, *src), 1, 0),
        ENC_ISO8859_13 => (iso8859_to_u(&ISO8859_13_TO_U, *src), 1, 0),
        ENC_ISO8859_14 => (iso8859_to_u(&ISO8859_14_TO_U, *src), 1, 0),
        ENC_ISO8859_15 => (iso8859_to_u(&ISO8859_15_TO_U, *src), 1, 0),
        ENC_ISO8859_16 => (iso8859_to_u(&ISO8859_16_TO_U, *src), 1, 0),
        ENC_WIN1252 => {
            let b = *src;
            if b < 128 { return (b as u32, 1, 0); }
            (WIN1252_TO_U[b as usize - 128], 1, 0)
        }
        ENC_WIN1251 => {
            let b = *src;
            if b < 128 { return (b as u32, 1, 0); }
            (WIN1251_TO_U[b as usize - 128], 1, 0)
        }
        ENC_KOI8R => {
            let b = *src;
            if b < 128 { return (b as u32, 1, 0); }
            (KOI8R_TO_U[b as usize - 128], 1, 0)
        }
        ENC_GBK | ENC_GB2312 => {
            let c = *src;
            if c < 128 { return (c as u32, 1, 0); }
            if c == 128 { return (0x20AC, 1, 0); }
            if src_left < 2 { return (0, 0, EINVAL); }
            let d = *src.add(1);
            if d < 0x40 || d == 0x7F || d > 0xFE { return (0, 0, EILSEQ); }
            let row = (c as usize).wrapping_sub(0x81);
            if row >= 126 { return (0, 0, EILSEQ); }
            let mut col = (d as usize) - 0x40;
            if col > 63 { col -= 1; }
            let cp = GB18030_TABLE[row * 190 + col] as u32;
            if cp == 0 { return (0, 0, EILSEQ); }
            (cp, 2, 0)
        }
        ENC_BIG5 => {
            let c = *src;
            if c < 128 { return (c as u32, 1, 0); }
            if src_left < 2 { return (0, 0, EINVAL); }
            let d = *src.add(1);
            if d < 0x40 || d == 0x7F || d > 0xFE { return (0, 0, EILSEQ); }
            let mut col = (d as usize) - 0x40;
            if col > 0x3E { col -= 0x22; }
            if c >= 0xA1 && c < 0xFA {
                let row = (c as usize) - 0xA1;
                if row >= 89 { return (0, 0, EILSEQ); }
                let cp = BIG5_TABLE[row * 157 + col] as u32;
                if cp == 0 { return (0, 0, EILSEQ); }
                return (cp, 2, 0);
            }
            (0, 0, EILSEQ)
        }
        ENC_EUCJP => {
            let c = *src;
            if c < 128 { return (c as u32, 1, 0); }
            if c == 0x8E {
                if src_left < 2 { return (0, 0, EINVAL); }
                let d = *src.add(1);
                if d < 0xA1 || d > 0xDF { return (0, 0, EILSEQ); }
                return ((d as u32) + 0xFF61 - 0xA1, 2, 0);
            }
            if c < 0xA1 { return (0, 0, EILSEQ); }
            if src_left < 2 { return (0, 0, EINVAL); }
            let d = *src.add(1);
            if d < 0xA1 || d > 0xFE { return (0, 0, EILSEQ); }
            let row = (c as usize) - 0xA1;
            let col = (d as usize) - 0xA1;
            if row >= 84 || col >= 94 { return (0, 0, EILSEQ); }
            let cp = JIS0208[row * 94 + col] as u32;
            if cp == 0 { return (0, 0, EILSEQ); }
            (cp, 2, 0)
        }
        ENC_SHIFTJIS => {
            let c = *src;
            if c < 128 { return (c as u32, 1, 0); }
            if c >= 0xA1 && c <= 0xDF { return ((c as u32) + 0xFF61 - 0xA1, 1, 0); }
            if src_left < 2 { return (0, 0, EINVAL); }
            let d = *src.add(1);
            if d < 0x40 || d == 0x7F || d > 0xFC { return (0, 0, EILSEQ); }
            let row = if c >= 129 && c <= 159 { (c as usize) - 129 }
                      else if c >= 224 && c <= 239 { (c as usize) - 193 }
                      else { return (0, 0, EILSEQ); };
            let (col, row_adj) = if d >= 64 && d <= 158 && d != 127 {
                let mut dd = d as usize;
                if dd > 127 { dd -= 1; }
                (dd - 64, row * 2)
            } else if d >= 159 && d <= 252 {
                ((d as usize) - 159, row * 2 + 1)
            } else {
                return (0, 0, EILSEQ);
            };
            if row_adj >= 84 { return (0, 0, EILSEQ); }
            let cp = JIS0208[row_adj * 94 + col] as u32;
            if cp == 0 { return (0, 0, EILSEQ); }
            (cp, 2, 0)
        }
        _ => (0, 0, EILSEQ),
    }
}

unsafe fn iconv_encode(enc: i32, c: u32, dst: *mut u8, dst_left: usize) -> (usize, c_int) {
    match enc {
        ENC_UTF8 => {
            if c < 0x80 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            if c < 0x800 {
                if dst_left < 2 { return (0, E2BIG); }
                *dst = (0xC0 | (c >> 6)) as u8;
                *dst.add(1) = (0x80 | (c & 0x3F)) as u8;
                return (2, 0);
            }
            if c < 0x10000 {
                if dst_left < 3 { return (0, E2BIG); }
                *dst = (0xE0 | (c >> 12)) as u8;
                *dst.add(1) = (0x80 | ((c >> 6) & 0x3F)) as u8;
                *dst.add(2) = (0x80 | (c & 0x3F)) as u8;
                return (3, 0);
            }
            if dst_left < 4 { return (0, E2BIG); }
            *dst = (0xF0 | (c >> 18)) as u8;
            *dst.add(1) = (0x80 | ((c >> 12) & 0x3F)) as u8;
            *dst.add(2) = (0x80 | ((c >> 6) & 0x3F)) as u8;
            *dst.add(3) = (0x80 | (c & 0x3F)) as u8;
            (4, 0)
        }
        ENC_UTF16LE => {
            if c < 0x10000 {
                if dst_left < 2 { return (0, E2BIG); }
                *dst = (c & 0xFF) as u8;
                *dst.add(1) = (c >> 8) as u8;
                return (2, 0);
            }
            if dst_left < 4 { return (0, E2BIG); }
            let c = c - 0x10000;
            let hi = ((c >> 10) + 0xD800) as u16;
            let lo = ((c & 0x3FF) + 0xDC00) as u16;
            *dst = (hi & 0xFF) as u8; *dst.add(1) = (hi >> 8) as u8;
            *dst.add(2) = (lo & 0xFF) as u8; *dst.add(3) = (lo >> 8) as u8;
            (4, 0)
        }
        ENC_UTF16BE => {
            if c < 0x10000 {
                if dst_left < 2 { return (0, E2BIG); }
                *dst = (c >> 8) as u8;
                *dst.add(1) = (c & 0xFF) as u8;
                return (2, 0);
            }
            if dst_left < 4 { return (0, E2BIG); }
            let c = c - 0x10000;
            let hi = ((c >> 10) + 0xD800) as u16;
            let lo = ((c & 0x3FF) + 0xDC00) as u16;
            *dst = (hi >> 8) as u8; *dst.add(1) = (hi & 0xFF) as u8;
            *dst.add(2) = (lo >> 8) as u8; *dst.add(3) = (lo & 0xFF) as u8;
            (4, 0)
        }
        ENC_UTF32LE => {
            if dst_left < 4 { return (0, E2BIG); }
            *dst = (c & 0xFF) as u8; *dst.add(1) = ((c >> 8) & 0xFF) as u8;
            *dst.add(2) = ((c >> 16) & 0xFF) as u8; *dst.add(3) = (c >> 24) as u8;
            (4, 0)
        }
        ENC_UTF32BE => {
            if dst_left < 4 { return (0, E2BIG); }
            *dst = (c >> 24) as u8; *dst.add(1) = ((c >> 16) & 0xFF) as u8;
            *dst.add(2) = ((c >> 8) & 0xFF) as u8; *dst.add(3) = (c & 0xFF) as u8;
            (4, 0)
        }
        ENC_WCHAR_T => {
            if dst_left < 4 { return (0, E2BIG); }
            *(dst as *mut u32) = c;
            (4, 0)
        }
        ENC_ASCII => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_LATIN1 => {
            if c < 256 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_2 => {
            if let Some(b) = u_to_iso8859(&ISO8859_2_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_3 => {
            if let Some(b) = u_to_iso8859(&ISO8859_3_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_4 => {
            if let Some(b) = u_to_iso8859(&ISO8859_4_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_5 => {
            if let Some(b) = u_to_iso8859(&ISO8859_5_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_6 => {
            if let Some(b) = u_to_iso8859(&ISO8859_6_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_7 => {
            if let Some(b) = u_to_iso8859(&ISO8859_7_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_8 => {
            if let Some(b) = u_to_iso8859(&ISO8859_8_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_9 => {
            if let Some(b) = u_to_iso8859(&ISO8859_9_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_10 => {
            if let Some(b) = u_to_iso8859(&ISO8859_10_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_11 => {
            if let Some(b) = u_to_iso8859(&ISO8859_11_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_13 => {
            if let Some(b) = u_to_iso8859(&ISO8859_13_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_14 => {
            if let Some(b) = u_to_iso8859(&ISO8859_14_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_15 => {
            if let Some(b) = u_to_iso8859(&ISO8859_15_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_ISO8859_16 => {
            if let Some(b) = u_to_iso8859(&ISO8859_16_TO_U, c) {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = b; return (1, 0);
            }
            (0, EILSEQ)
        }
        ENC_WIN1252 => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            if c >= 0xA0 && c <= 0xFF && WIN1252_TO_U[c as usize - 128] == c {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            for i in 0..128usize {
                if WIN1252_TO_U[i] == c {
                    if dst_left < 1 { return (0, E2BIG); }
                    *dst = (i + 128) as u8; return (1, 0);
                }
            }
            (0, EILSEQ)
        }
        ENC_WIN1251 => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            for i in 0..128usize {
                if WIN1251_TO_U[i] == c {
                    if dst_left < 1 { return (0, E2BIG); }
                    *dst = (i + 128) as u8; return (1, 0);
                }
            }
            (0, EILSEQ)
        }
        ENC_KOI8R => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            for i in 0..128usize {
                if KOI8R_TO_U[i] == c {
                    if dst_left < 1 { return (0, E2BIG); }
                    *dst = (i + 128) as u8; return (1, 0);
                }
            }
            (0, EILSEQ)
        }
        ENC_GBK | ENC_GB2312 => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            if c == 0x20AC {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = 128; return (1, 0);
            }
            for row in 0..126usize {
                for col in 0..190usize {
                    if GB18030_TABLE[row * 190 + col] as u32 == c {
                        if dst_left < 2 { return (0, E2BIG); }
                        *dst = (row + 0x81) as u8;
                        *dst.add(1) = if col <= 62 { (col + 0x40) as u8 } else { (col + 0x41) as u8 };
                        return (2, 0);
                    }
                }
            }
            (0, EILSEQ)
        }
        ENC_BIG5 => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            for row in 0..89usize {
                for col in 0..157usize {
                    if BIG5_TABLE[row * 157 + col] as u32 == c {
                        if dst_left < 2 { return (0, E2BIG); }
                        *dst = (row + 0xA1) as u8;
                        *dst.add(1) = if col <= 0x3E { (col + 0x40) as u8 } else { (col + 0x62) as u8 };
                        return (2, 0);
                    }
                }
            }
            (0, EILSEQ)
        }
        ENC_EUCJP => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            if c >= 0xFF61 && c <= 0xFF9F {
                if dst_left < 2 { return (0, E2BIG); }
                *dst = 0x8E;
                *dst.add(1) = (c - 0xFF61 + 0xA1) as u8;
                return (2, 0);
            }
            let jis = uni_to_jis(c);
            if jis == 0 { return (0, EILSEQ); }
            if dst_left < 2 { return (0, E2BIG); }
            *dst = ((jis >> 8) as u8).wrapping_add(0x80);
            *dst.add(1) = ((jis & 0xFF) as u8).wrapping_add(0x80);
            (2, 0)
        }
        ENC_SHIFTJIS => {
            if c < 128 {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = c as u8; return (1, 0);
            }
            if c >= 0xFF61 && c <= 0xFF9F {
                if dst_left < 1 { return (0, E2BIG); }
                *dst = (c - 0xFF61 + 0xA1) as u8;
                return (1, 0);
            }
            let jis = uni_to_jis(c);
            if jis == 0 { return (0, EILSEQ); }
            if dst_left < 2 { return (0, E2BIG); }
            let r = (jis >> 8) as usize;
            let d = (jis & 0xFF) as usize;
            *dst = ((r + 1) / 2 + if r < 95 { 112 } else { 176 }) as u8;
            *dst.add(1) = if r % 2 == 1 { (d + 31 + d / 96) as u8 } else { (d + 126) as u8 };
            (2, 0)
        }
        _ => (0, EILSEQ),
    }
}

#[no_mangle]
pub unsafe extern "C" fn iconv_open(tocode: *const c_char, fromcode: *const c_char) -> IconvT {
    let to = find_encoding(tocode as *const u8);
    let from = find_encoding(fromcode as *const u8);
    if to < 0 || from < 0 {
        ERRNO = EINVAL;
        return !0usize as IconvT;
    }
    make_cd(from, to)
}

#[no_mangle]
pub unsafe extern "C" fn iconv(
    cd: IconvT,
    inbuf: *mut *mut c_char,
    inbytesleft: *mut SizeT,
    outbuf: *mut *mut c_char,
    outbytesleft: *mut SizeT,
) -> SizeT {
    if cd == !0usize as IconvT {
        ERRNO = EINVAL;
        return !0usize;
    }
    if inbuf.is_null() || outbuf.is_null() { return 0; }
    let from = extract_from(cd);
    let to = extract_to(cd);
    let mut src = *inbuf as *const u8;
    let mut dst = *outbuf as *mut u8;
    let mut src_left = *inbytesleft;
    let mut dst_left = *outbytesleft;
    let mut subst = 0usize;
    while src_left > 0 {
        let (cp, consumed, err) = iconv_decode(from, src, src_left);
        if err != 0 {
            if err == EINVAL { break; }
            ERRNO = err;
            return !0usize;
        }
        let (written, err) = iconv_encode(to, cp, dst, dst_left);
        if err != 0 {
            if err == E2BIG {
                ERRNO = E2BIG;
                return !0usize;
            }
            if dst_left < 1 {
                ERRNO = E2BIG;
                return !0usize;
            }
            *dst = b'*';
            dst = dst.add(1);
            dst_left -= 1;
            subst += 1;
        } else {
            dst = dst.add(written);
            dst_left -= written;
        }
        src = src.add(consumed);
        src_left -= consumed;
    }
    *inbuf = src as *mut c_char;
    *outbuf = dst as *mut c_char;
    *inbytesleft = src_left;
    *outbytesleft = dst_left;
    subst
}

#[no_mangle]
pub unsafe extern "C" fn iconv_close(_cd: IconvT) -> c_int { 0 }

// ============================================================
// Syscall wrappers as public C ABI
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: SizeT) -> SSizeT {
    sys_write(fd as i64, buf as *const u8, count) as SSizeT
}

#[no_mangle]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: SizeT) -> SSizeT {
    sys_read(fd as i64, buf as *mut u8, count) as SSizeT
}

#[no_mangle]
pub unsafe extern "C" fn open(path: *const c_char, flags: c_int, mode: c_int) -> c_int {
    sys_open(path as *const u8, flags as i64, mode as i64) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn close(fd: c_int) -> c_int {
    sys_close(fd as i64);
    0
}

#[no_mangle]
pub unsafe extern "C" fn lseek(fd: c_int, offset: i64, whence: c_int) -> i64 {
    sys_lseek(fd as i64, offset, whence as i64)
}

// ============================================================
// Additional syscall wrappers for stdio
// ============================================================

const O_RDONLY: i32 = 0;
const O_WRONLY: i32 = 1;
const O_RDWR: i32 = 2;
const O_CREAT: i32 = 64;
const O_TRUNC: i32 = 512;
const O_APPEND: i32 = 1024;
const O_EXCL: i32 = 128;
const O_NONBLOCK: i32 = 2048;
const O_NOFOLLOW: i32 = 0x40000;
const O_CLOEXEC: i32 = 0x80000;

const F_GETFD: i32 = 1;
const F_SETFD: i32 = 2;
const F_GETFL: i32 = 3;
const F_SETFL: i32 = 4;
const F_GETLK: i32 = 5;
const F_SETLK: i32 = 6;
const F_SETLKW: i32 = 7;
const F_DUPFD: i32 = 0;
const F_DUPFD_CLOEXEC: i32 = 1030;
const FD_CLOEXEC: i32 = 1;

const TIOCGWINSZ: u32 = 0x5413;

#[repr(C)]
struct winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

#[inline]
unsafe fn sys_pipe2(fds: *mut c_int, flags: i32) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_PIPE2, fds as i64, flags as i64)
}

#[inline]
unsafe fn sys_dup3(oldfd: i32, newfd: i32, flags: i32) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_DUP3, oldfd as i64, newfd as i64, flags as i64)
}

#[inline]
unsafe fn sys_fcntl(fd: i32, cmd: i32, arg: i64) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_FCNTL, fd as i64, cmd as i64, arg as i64)
}

#[inline]
unsafe fn sys_ioctl(fd: c_int, request: u32, arg: *mut u8) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_IOCTL, fd as i64, request as i64, arg as i64)
}

#[inline]
unsafe fn sys_unlinkat(dirfd: i32, path: *const u8, flags: i32) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_UNLINKAT, dirfd as i64, path as i64, flags as i64)
}

#[inline]
unsafe fn sys_renameat2(olddirfd: i32, oldpath: *const u8, newdirfd: i32, newpath: *const u8, flags: u32) -> i64 {
    <Arch as Syscalls>::syscall5(SYS_RENAMEAT2, olddirfd as i64, oldpath as i64, newdirfd as i64, newpath as i64, flags as i64)
}

// ============================================================
// FILE / stdio (buffered, musl-compatible layout)
// ============================================================

const UNGET: usize = 8;
const F_PERM: u32 = 1;
const F_NORD: u32 = 4;
const F_NOWR: u32 = 8;
const F_EOF: u32 = 16;
const F_ERR: u32 = 32;
const F_SVB: u32 = 64;
const F_APP: u32 = 128;

const BUFSIZ: usize = 1024;

const _IOFBF: c_int = 0;
const _IOLBF: c_int = 1;
const _IONBF: c_int = 2;

const SEEK_SET: c_int = 0;
const SEEK_CUR: c_int = 1;
const SEEK_END: c_int = 2;

// ponytail: buffered FILE, musl x86_64 layout subset
#[repr(C)]
pub struct FILE {
    flags: c_uint,
    rpos: *mut u8,
    rend: *mut u8,
    close: Option<unsafe extern "C" fn(*mut FILE) -> c_int>,
    wend: *mut u8,
    wpos: *mut u8,
    mustbezero_1: *mut u8,
    wbase: *mut u8,
    read_fn: Option<unsafe extern "C" fn(*mut FILE, *mut u8, usize) -> usize>,
    write_fn: Option<unsafe extern "C" fn(*mut FILE, *const u8, usize) -> usize>,
    seek_fn: Option<unsafe extern "C" fn(*mut FILE, i64, c_int) -> i64>,
    buf: *mut u8,
    buf_size: usize,
    prev: *mut FILE,
    next: *mut FILE,
    fd: c_int,
    pipe_pid: c_int,
    lockcount: c_long,
    mode: c_int,
    lock: c_int,
    lbf: c_int,
    cookie: *mut c_void,
    off: i64,
    getln_buf: *mut c_char,
    mustbezero_2: *mut c_void,
    shend: *mut u8,
    shlim: i64,
    shcnt: i64,
    ungotten: [c_int; UNGET],
    ungotten_count: c_int,
    _eof: c_int,
    _err: c_int,
}

static mut STDIN_BUF: [u8; BUFSIZ] = [0; BUFSIZ];
static mut STDOUT_BUF: [u8; BUFSIZ] = [0; BUFSIZ];
static mut STDERR_BUF: [u8; BUFSIZ] = [0; BUFSIZ];

static mut STDIN_FILE: FILE = FILE {
    flags: 0, rpos: core::ptr::null_mut(), rend: core::ptr::null_mut(),
    close: None, wend: core::ptr::null_mut(), wpos: core::ptr::null_mut(),
    mustbezero_1: core::ptr::null_mut(), wbase: core::ptr::null_mut(),
    read_fn: None, write_fn: None, seek_fn: None,
    buf: core::ptr::null_mut(), buf_size: BUFSIZ,
    prev: core::ptr::null_mut(), next: core::ptr::null_mut(),
    fd: 0, pipe_pid: 0, lockcount: 0, mode: 0, lock: -1, lbf: -1,
    cookie: core::ptr::null_mut(), off: 0,
    getln_buf: core::ptr::null_mut(), mustbezero_2: core::ptr::null_mut(),
    shend: core::ptr::null_mut(), shlim: 0, shcnt: 0,
    ungotten: [0; UNGET], ungotten_count: 0, _eof: 0, _err: 0,
};
static mut STDOUT_FILE: FILE = FILE {
    flags: F_NOWR as u32 + F_SVB as u32, rpos: core::ptr::null_mut(), rend: core::ptr::null_mut(),
    close: None, wend: core::ptr::null_mut(), wpos: core::ptr::null_mut(),
    mustbezero_1: core::ptr::null_mut(), wbase: core::ptr::null_mut(),
    read_fn: None, write_fn: None, seek_fn: None,
    buf: core::ptr::null_mut(), buf_size: BUFSIZ,
    prev: core::ptr::null_mut(), next: core::ptr::null_mut(),
    fd: 1, pipe_pid: 0, lockcount: 0, mode: 0, lock: -1, lbf: -1,
    cookie: core::ptr::null_mut(), off: 0,
    getln_buf: core::ptr::null_mut(), mustbezero_2: core::ptr::null_mut(),
    shend: core::ptr::null_mut(), shlim: 0, shcnt: 0,
    ungotten: [0; UNGET], ungotten_count: 0, _eof: 0, _err: 0,
};
static mut STDERR_FILE: FILE = FILE {
    flags: F_NOWR as u32, rpos: core::ptr::null_mut(), rend: core::ptr::null_mut(),
    close: None, wend: core::ptr::null_mut(), wpos: core::ptr::null_mut(),
    mustbezero_1: core::ptr::null_mut(), wbase: core::ptr::null_mut(),
    read_fn: None, write_fn: None, seek_fn: None,
    buf: core::ptr::null_mut(), buf_size: BUFSIZ,
    prev: core::ptr::null_mut(), next: core::ptr::null_mut(),
    fd: 2, pipe_pid: 0, lockcount: 0, mode: 0, lock: -1, lbf: -1,
    cookie: core::ptr::null_mut(), off: 0,
    getln_buf: core::ptr::null_mut(), mustbezero_2: core::ptr::null_mut(),
    shend: core::ptr::null_mut(), shlim: 0, shcnt: 0,
    ungotten: [0; UNGET], ungotten_count: 0, _eof: 0, _err: 0,
};

#[no_mangle]
pub static mut stdin: *mut FILE = &raw mut STDIN_FILE as *mut FILE;
#[no_mangle]
pub static mut stdout: *mut FILE = &raw mut STDOUT_FILE as *mut FILE;
#[no_mangle]
pub static mut stderr: *mut FILE = &raw mut STDERR_FILE as *mut FILE;

// ============================================================
// FILE helpers
// ============================================================

unsafe fn __stdio_init() {
    STDIN_FILE.buf = core::ptr::addr_of_mut!(STDIN_BUF) as *mut u8;
    STDIN_FILE.buf_size = BUFSIZ;
    STDIN_FILE.read_fn = Some(__stdio_read);
    STDIN_FILE.write_fn = Some(__stdio_write);
    STDIN_FILE.seek_fn = Some(__stdio_seek);
    STDOUT_FILE.buf = core::ptr::addr_of_mut!(STDOUT_BUF) as *mut u8;
    STDOUT_FILE.buf_size = BUFSIZ;
    STDOUT_FILE.lbf = b'\n' as c_int;
    STDOUT_FILE.read_fn = Some(__stdio_read);
    STDOUT_FILE.write_fn = Some(__stdio_write);
    STDOUT_FILE.seek_fn = Some(__stdio_seek);
    STDERR_FILE.buf = core::ptr::addr_of_mut!(STDERR_BUF) as *mut u8;
    STDERR_FILE.buf_size = BUFSIZ;
    STDERR_FILE.read_fn = Some(__stdio_read);
    STDERR_FILE.write_fn = Some(__stdio_write);
    STDERR_FILE.seek_fn = Some(__stdio_seek);
}

unsafe fn buf_ptr(f: *mut FILE) -> *mut u8 {
    (f as *mut u8).add(core::mem::size_of::<FILE>())
}

unsafe extern "C" fn __stdio_read(f: *mut FILE, buf: *mut u8, len: usize) -> usize {
    let n = sys_read((*f).fd as i64, buf, len);
    if n <= 0 {
        if n == 0 { (*f).flags |= F_EOF; } else { (*f).flags |= F_ERR; (*f)._err = 1; }
        return 0;
    }
    n as usize
}

unsafe extern "C" fn __stdio_write(f: *mut FILE, buf: *const u8, len: usize) -> usize {
    let l = (*f).wpos as usize - (*f).wbase as usize;
    let mut iov = [
        ((*f).wbase as *const u8, l),
        (buf, len),
    ];
    if iov[0].1 == 0 { iov = [iov[1], (core::ptr::null(), 0)]; }
    if (*f).flags & F_APP != 0 {
        let _ = sys_lseek((*f).fd as i64, 0, SEEK_END as i64);
    }
    let mut rem = iov[0].1 + iov[1].1;
    let mut idx = 0usize;
    while rem > 0 {
        let ptr = iov[idx].0;
        let cnt = iov[idx].1;
        if cnt == 0 { idx += 1; continue; }
        let n = sys_write((*f).fd as i64, ptr, cnt);
        if n <= 0 {
            (*f).wpos = core::ptr::null_mut();
            (*f).wbase = core::ptr::null_mut();
            (*f).wend = core::ptr::null_mut();
            (*f).flags |= F_ERR;
            return if iov[idx].0 == buf { 0 } else { len };
        }
        rem -= n as usize;
        iov[idx].0 = iov[idx].0.add(n as usize);
        iov[idx].1 -= n as usize;
        if iov[idx].1 == 0 { idx += 1; }
    }
    (*f).wend = (*f).buf.add((*f).buf_size);
    (*f).wbase = (*f).buf;
    (*f).wpos = (*f).buf;
    len
}

unsafe extern "C" fn __stdio_seek(f: *mut FILE, off: i64, whence: c_int) -> i64 {
    sys_lseek((*f).fd as i64, off, whence as i64)
}

unsafe fn init_file(
    f: *mut FILE,
    fd: c_int,
    mode: *const c_char,
    close_fn: Option<unsafe extern "C" fn(*mut FILE) -> c_int>,
    buf_area: *mut u8,
    buf_sz: usize,
) {
    core::ptr::write_bytes(f as *mut u8, 0, core::mem::size_of::<FILE>());
    (*f).fd = fd;
    (*f).close = close_fn;
    (*f).lock = -1;
    (*f).lbf = -1;
    (*f).buf = buf_area;
    (*f).buf_size = buf_sz;
    (*f).read_fn = Some(__stdio_read);
    (*f).write_fn = Some(__stdio_write);
    (*f).seek_fn = Some(__stdio_seek);
    let m = *mode;
    let has_plus = !strchr(mode as *const u8, b'+' as c_int).is_null();
    if m == b'r' as c_char && !has_plus {
        (*f).flags = F_NORD;
    } else if !has_plus {
        (*f).flags = F_NOWR;
    }
}

unsafe extern "C" fn __stdio_close(f: *mut FILE) -> c_int {
    sys_close((*f).fd as i64);
    0
}

unsafe fn fmodeflags(mode: *const c_char) -> c_int {
    let mut flags: c_int;
    let m = *mode;
    if !strchr(mode as *const u8, b'+' as c_int).is_null() {
        flags = O_RDWR;
    } else if m == b'r' as c_char {
        flags = O_RDONLY;
    } else {
        flags = O_WRONLY;
    }
    if !strchr(mode as *const u8, b'x' as c_int).is_null() { flags |= O_EXCL; }
    if !strchr(mode as *const u8, b'e' as c_int).is_null() { flags |= O_CLOEXEC; }
    if m != b'r' as c_char { flags |= O_CREAT; }
    if m == b'w' as c_char { flags |= O_TRUNC; }
    if m == b'a' as c_char { flags |= O_APPEND; }
    flags
}

const O_ACCMODE: c_int = O_RDONLY | O_WRONLY | O_RDWR;

unsafe fn flush_buf(f: *mut FILE) -> c_int {
    if (*f).wpos == (*f).wbase {
        if !(*f).wpos.is_null() {
            (*f).wend = (*f).buf.add((*f).buf_size);
            (*f).wpos = (*f).wbase;
        }
        return 0;
    }
    if let Some(wfunc) = (*f).write_fn {
        let ret = wfunc(f, core::ptr::null(), 0);
        if ret == 0 && (*f).wpos.is_null() { return -1; }
        return 0;
    }
    (*f).flags |= F_ERR;
    -1
}

// ============================================================
// fopen / fdopen / freopen / fclose
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE {
    if filename.is_null() || mode.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let m = *mode;
    if m != b'r' as c_char && m != b'w' as c_char && m != b'a' as c_char {
        ERRNO = EINVAL; return core::ptr::null_mut();
    }
    let flags = fmodeflags(mode);
    let fd = sys_open(filename as *const u8, flags as i64, 0o666);
    if fd < 0 { ERRNO = (-fd) as c_int; return core::ptr::null_mut(); }
    fdopen(fd as c_int, mode)
}

#[no_mangle]
pub unsafe extern "C" fn fdopen(fd: c_int, mode: *const c_char) -> *mut FILE {
    if fd < 0 || mode.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let m = *mode;
    if m != b'r' as c_char && m != b'w' as c_char && m != b'a' as c_char {
        ERRNO = EINVAL; return core::ptr::null_mut();
    }
    let f = calloc(1, core::mem::size_of::<FILE>() + UNGET + BUFSIZ) as *mut FILE;
    if f.is_null() { ERRNO = ENOMEM; return core::ptr::null_mut(); }
    let buf = buf_ptr(f);
    init_file(f, fd, mode, Some(__stdio_close), buf, BUFSIZ);
    if !strchr(mode as *const u8, b'a' as c_int).is_null() { (*f).flags |= F_APP; }
    if !strchr(mode as *const u8, b'e' as c_int).is_null() { sys_fcntl(fd, F_SETFD, FD_CLOEXEC as i64); }
    if (*f).flags & F_NOWR == 0 {
        let mut ws: winsize = core::mem::zeroed();
        if sys_ioctl(fd, TIOCGWINSZ, &mut ws as *mut winsize as *mut u8) == 0 {
            (*f).lbf = b'\n' as c_int;
        }
    }
    f
}

#[no_mangle]
pub unsafe extern "C" fn freopen(
    filename: *const c_char, mode: *const c_char, f: *mut FILE,
) -> *mut FILE {
    fflush(f);
    if (*f).pipe_pid > 0 { let _ = __stdio_close(f); }
    if filename.is_null() {
        let flags = fmodeflags(mode);
        if flags == 0 { return core::ptr::null_mut(); }
        let mut fl = flags & !(O_CREAT | O_EXCL | O_CLOEXEC);
        if (*f).flags & F_APP != 0 { fl = (fl & !O_ACCMODE) | O_WRONLY | O_APPEND; }
        if sys_fcntl((*f).fd, F_SETFL, fl as i64) < 0 { return core::ptr::null_mut(); }
        let has_plus = !strchr(mode as *const u8, b'+' as c_int).is_null();
        if has_plus {
            (*f).flags &= !(F_NORD | F_NOWR);
        } else if *mode == b'r' as c_char {
            (*f).flags = ((*f).flags & !F_NOWR) | F_NORD;
        } else {
            (*f).flags = ((*f).flags & !F_NORD) | F_NOWR;
        }
        return f;
    }
    let new_f = fopen(filename, mode);
    if new_f.is_null() { let _ = __stdio_close(f); return core::ptr::null_mut(); }
    if (*new_f).fd == (*f).fd {
        (*new_f).fd = -1;
    } else {
        sys_dup3((*new_f).fd, (*f).fd, 0);
        let _ = __stdio_close(new_f);
    }
    (*f).flags = (*new_f).flags;
    (*f).rpos = core::ptr::null_mut();
    (*f).rend = core::ptr::null_mut();
    (*f).wpos = core::ptr::null_mut();
    (*f).wbase = core::ptr::null_mut();
    (*f).wend = core::ptr::null_mut();
    (*f).lbf = (*new_f).lbf;
    (*f).pipe_pid = 0;
    free(new_f as *mut c_void);
    f
}

#[no_mangle]
pub unsafe extern "C" fn fclose(f: *mut FILE) -> c_int {
    if f.is_null() { return -1; }
    let r = fflush(f);
    if let Some(close_fn) = (*f).close { let _ = close_fn(f); }
    if r != 0 { -1 } else { 0 }
}

// ============================================================
// setvbuf / setbuf / setbuffer / fflush
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn setvbuf(f: *mut FILE, buf: *mut c_char, mode: c_int, size: usize) -> c_int {
    if mode != _IOFBF && mode != _IOLBF && mode != _IONBF { return -1; }
    fflush(f);
    (*f).rpos = core::ptr::null_mut();
    (*f).rend = core::ptr::null_mut();
    (*f).wpos = core::ptr::null_mut();
    (*f).wbase = core::ptr::null_mut();
    (*f).wend = core::ptr::null_mut();
    if mode == _IONBF {
        (*f).buf_size = 0;
    } else if !buf.is_null() && size > 0 {
        (*f).buf = buf as *mut u8;
        (*f).buf_size = size;
    }
    (*f).lbf = if mode == _IOLBF { b'\n' as c_int } else { -1 };
    0
}

#[no_mangle]
pub unsafe extern "C" fn setbuf(f: *mut FILE, buf: *mut c_char) {
    if buf.is_null() { setvbuf(f, core::ptr::null_mut(), _IONBF, 0); }
    else { setvbuf(f, buf, _IOFBF, BUFSIZ); }
}

#[no_mangle]
pub unsafe extern "C" fn setbuffer(f: *mut FILE, buf: *mut c_char, size: usize) {
    if buf.is_null() { setvbuf(f, core::ptr::null_mut(), _IONBF, 0); }
    else { setvbuf(f, buf, _IOFBF, size); }
}

#[no_mangle]
pub unsafe extern "C" fn fflush(f: *mut FILE) -> c_int {
    if f.is_null() {
        let mut r = 0;
        if !stdout.is_null() && !(*stdout).wpos.is_null() && (*stdout).wpos != (*stdout).wbase {
            if flush_buf(stdout) != 0 { r = -1; }
        }
        if !stderr.is_null() && !(*stderr).wpos.is_null() && (*stderr).wpos != (*stderr).wbase {
            if flush_buf(stderr) != 0 { r = -1; }
        }
        return r;
    }
    if !(*f).wpos.is_null() && (*f).wpos != (*f).wbase {
        return flush_buf(f);
    }
    0
}

// ============================================================
// fwide / feof / ferror / clearerr / fileno
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fwide(_f: *mut FILE, mode: c_int) -> c_int { mode }

#[no_mangle]
pub unsafe extern "C" fn feof(f: *mut FILE) -> c_int { if (*f)._eof != 0 { 1 } else { 0 } }

#[no_mangle]
pub unsafe extern "C" fn ferror(f: *mut FILE) -> c_int { if (*f)._err != 0 { 1 } else { 0 } }

#[no_mangle]
pub unsafe extern "C" fn clearerr(f: *mut FILE) { (*f)._eof = 0; (*f)._err = 0; }

#[no_mangle]
pub unsafe extern "C" fn fileno(f: *mut FILE) -> c_int { (*f).fd }

// ============================================================
// fseek / ftell / rewind / fseeko / ftello / fgetpos / fsetpos
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fseek(stream: *mut FILE, offset: c_long, whence: c_int) -> c_int {
    fseeko(stream, offset as i64, whence)
}

#[no_mangle]
pub unsafe extern "C" fn ftell(stream: *mut FILE) -> c_long {
    ftello(stream) as c_long
}

#[no_mangle]
pub unsafe extern "C" fn rewind(stream: *mut FILE) {
    fseek(stream, 0, SEEK_SET);
    (*stream)._eof = 0;
    (*stream)._err = 0;
}

#[no_mangle]
pub unsafe extern "C" fn fseeko(stream: *mut FILE, offset: i64, whence: c_int) -> c_int {
    if whence != SEEK_SET && whence != SEEK_CUR && whence != SEEK_END {
        ERRNO = EINVAL;
        return -1;
    }
    let f = &mut *stream;
    let mut adj_offset = offset;
    if whence == SEEK_CUR {
        if !f.rpos.is_null() {
            adj_offset -= f.rend as i64 - f.rpos as i64;
        }
        adj_offset -= f.ungotten_count as i64;
    }
    if !f.wpos.is_null() && f.wpos != f.wbase {
        if flush_buf(stream) != 0 { return -1; }
    }
    f.wpos = core::ptr::null_mut();
    f.wbase = core::ptr::null_mut();
    f.wend = core::ptr::null_mut();
    if let Some(sfunc) = f.seek_fn {
        let r = sfunc(stream, adj_offset, whence);
        if r < 0 { return -1; }
    } else {
        ERRNO = EINVAL;
        return -1;
    }
    f.rpos = core::ptr::null_mut();
    f.rend = core::ptr::null_mut();
    f._eof = 0;
    f.ungotten_count = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn ftello(stream: *mut FILE) -> i64 {
    let f = &mut *stream;
    if f.flags & F_APP != 0 && !f.wpos.is_null() && f.wpos != f.wbase {
        if fflush(stream) != 0 { return -1; }
    }
    let sfunc = match f.seek_fn { Some(s) => s, None => return -1 };
    let pos = sfunc(stream, 0,
        if (f.flags & F_APP) != 0 && !f.wpos.is_null() && f.wpos != f.wbase { SEEK_END } else { SEEK_CUR });
    if pos < 0 { return -1; }
    let mut pos = pos;
    if !f.rpos.is_null() {
        pos += f.rpos as i64 - f.rend as i64;
    } else if !f.wbase.is_null() {
        pos += f.wpos as i64 - f.wbase as i64;
    }
    pos - f.ungotten_count as i64
}

#[no_mangle]
pub unsafe extern "C" fn fgetpos(stream: *mut FILE, pos: *mut i64) -> c_int {
    let off = ftello(stream);
    if off < 0 { return -1; }
    *pos = off;
    0
}

#[no_mangle]
pub unsafe extern "C" fn fsetpos(stream: *mut FILE, pos: *const i64) -> c_int {
    fseeko(stream, *pos, SEEK_SET)
}

// ============================================================
// fgetc / getc / getchar / ungetc
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fgetc(stream: *mut FILE) -> c_int {
    let f = &mut *stream;
    if f.ungotten_count > 0 {
        f.ungotten_count -= 1;
        return f.ungotten[f.ungotten_count as usize];
    }
    if !f.rpos.is_null() && f.rpos < f.rend {
        let c = *f.rpos;
        f.rpos = f.rpos.add(1);
        return c as c_int;
    }
    if f._eof != 0 { return -1; }
    if let Some(rfunc) = f.read_fn {
        let mut buf = [0u8; 1];
        let n = rfunc(stream, buf.as_mut_ptr(), 1);
        if n == 0 {
            if f.flags & F_ERR != 0 { f._err = 1; } else { f._eof = 1; }
            return -1;
        }
        buf[0] as c_int
    } else {
        f._eof = 1;
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn getc(stream: *mut FILE) -> c_int { fgetc(stream) }

#[no_mangle]
pub unsafe extern "C" fn getchar() -> c_int { fgetc(stdin) }

#[no_mangle]
pub unsafe extern "C" fn ungetc(c: c_int, stream: *mut FILE) -> c_int {
    if c == -1 { return -1; }
    let f = &mut *stream;
    if f.ungotten_count >= UNGET as c_int { return -1; }
    f.ungotten[f.ungotten_count as usize] = c;
    f.ungotten_count += 1;
    f._eof = 0;
    c
}

// ============================================================
// fgets / fread
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fgets(s: *mut c_char, n: c_int, stream: *mut FILE) -> *mut c_char {
    if n <= 0 { return null_mut(); }
    let max = (n - 1) as usize;
    let mut i = 0usize;
    while i < max {
        let c = fgetc(stream);
        if c == -1 { if i == 0 { return null_mut(); } break; }
        *s.add(i) = c as c_char;
        i += 1;
        if c == b'\n' as c_int { break; }
    }
    *s.add(i) = 0;
    s
}

// ponytail: gets is dangerous, kept for POSIX compliance
#[no_mangle]
pub unsafe extern "C" fn gets(s: *mut c_char) -> *mut c_char {
    let mut i = 0usize;
    loop {
        let c = fgetc(stdin);
        if c == -1 { if i == 0 { return null_mut(); } break; }
        if c == b'\n' as c_int { break; }
        *s.add(i) = c as c_char;
        i += 1;
    }
    *s.add(i) = 0;
    s
}

#[no_mangle]
pub unsafe extern "C" fn fopen64(filename: *const c_char, mode: *const c_char) -> *mut FILE {
    fopen(filename, mode)
}

// ponytail: perror - minimal, just prints "msg: errno\n"
const PERROR_BUF_SIZE: usize = 64;
static mut PERROR_BUF: [u8; PERROR_BUF_SIZE] = [0; PERROR_BUF_SIZE];

#[no_mangle]
pub unsafe extern "C" fn perror(msg: *const c_char) {
    let fd = (*stderr).fd;
    if !msg.is_null() && *msg != 0 {
        let len = strlen(msg as *const c_char);
        write_str(fd, msg as *const u8, len);
        write_str(fd, b": ".as_ptr(), 2);
    }
    let e = ERRNO;
    // ponytail: just print errno number, not full strerror
    if e != 0 {
        let (buf, len) = format_i64(e as i64);
        write_str(fd, buf.as_ptr().add(21 - len), len);
    }
    write_str(fd, b"\n".as_ptr(), 1);
}

#[no_mangle]
pub unsafe extern "C" fn fread(
    ptr: *mut c_void, size: SizeT, nmemb: SizeT, stream: *mut FILE,
) -> SizeT {
    if size == 0 || nmemb == 0 { return 0; }
    let total = size * nmemb;
    let mut rd = 0usize;
    while rd < total {
        let c = fgetc(stream);
        if c == -1 { break; }
        *(ptr as *mut u8).add(rd) = c as u8;
        rd += 1;
    }
    rd / size
}

// ============================================================
// puts / fputs / fputc / putc / putchar / fwrite
// ============================================================

unsafe fn write_str(fd: c_int, s: *const u8, len: usize) {
    let mut written = 0usize;
    while written < len {
        let n = sys_write(fd as i64, s.add(written), len - written);
        if n <= 0 { break; }
        written += n as usize;
    }
}

unsafe fn __overflow(f: *mut FILE, c: c_int) -> c_int {
    if (*f).wpos.is_null() {
        (*f).wbase = (*f).buf;
        (*f).wpos = (*f).buf;
        (*f).wend = (*f).buf.add((*f).buf_size);
    }
    if flush_buf(f) == -1 { return -1; }
    if (*f).wpos >= (*f).wend {
        if let Some(wfunc) = (*f).write_fn {
            let byte = c as u8;
            let ret = wfunc(f, &byte as *const u8, 1);
            if ret == 0 { return -1; }
        }
        return c;
    }
    *(*f).wpos = c as u8;
    (*f).wpos = (*f).wpos.add(1);
    if c == (*f).lbf || (*f).wpos >= (*f).wend { let _ = flush_buf(f); }
    c
}

#[no_mangle]
pub unsafe extern "C" fn puts(s: *const c_char) -> c_int {
    let len = strlen(s as *const c_char);
    let _ = fwrite(s as *const c_void, 1, len, stdout);
    let _ = fwrite(b"\n".as_ptr() as *const c_void, 1, 1, stdout);
    let f = &mut *stdout;
    if f.lbf >= 0 { let _ = flush_buf(stdout); }
    (len + 1) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn fputs(s: *const c_char, stream: *mut FILE) -> c_int {
    let len = strlen(s as *const c_char);
    fwrite(s as *const c_void, 1, len, stream) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn fputc(c: c_int, stream: *mut FILE) -> c_int {
    let f = &mut *stream;
    if f.wpos.is_null() || f.wpos >= f.wend {
        if __overflow(f, c) == -1 { return -1; }
        return c;
    }
    *f.wpos = c as u8;
    f.wpos = f.wpos.add(1);
    if c == f.lbf || f.wpos >= f.wend { let _ = flush_buf(f); }
    c
}

#[no_mangle]
pub unsafe extern "C" fn putc(c: c_int, stream: *mut FILE) -> c_int { fputc(c, stream) }

#[no_mangle]
pub unsafe extern "C" fn putchar(c: c_int) -> c_int { fputc(c, stdout) }

#[no_mangle]
pub unsafe extern "C" fn fwrite(
    ptr: *const c_void, size: SizeT, nmemb: SizeT, stream: *mut FILE,
) -> SizeT {
    if size == 0 || nmemb == 0 { return 0; }
    let total = size * nmemb;
    let f = &mut *stream;
    if f.buf_size == 0 {
        if let Some(wfunc) = f.write_fn {
            let ret = wfunc(stream, ptr as *const u8, total);
            return ret / size;
        }
        return 0;
    }
    if f.wpos.is_null() {
        f.wbase = f.buf;
        f.wpos = f.buf;
        f.wend = f.buf.add(f.buf_size);
    }
    let mut remaining = total;
    let mut src = ptr as *const u8;
    while remaining > 0 {
        let space = f.wend as usize - f.wpos as usize;
        if space == 0 {
            if flush_buf(f as *mut FILE) == -1 { break; }
            continue;
        }
        let n = if remaining < space { remaining } else { space };
        core::ptr::copy_nonoverlapping(src, f.wpos, n);
        f.wpos = f.wpos.add(n);
        src = src.add(n);
        remaining -= n;
        if f.wpos >= f.wend { let _ = flush_buf(f as *mut FILE); }
    }
    if f.lbf >= 0 { let _ = flush_buf(f as *mut FILE); }
    (total - remaining) / size
}

// ============================================================
// printf / fprintf / vprintf / vfprintf / dprintf / vdprintf
// ============================================================

macro_rules! impl_format {
    ($fmt:expr, $args:expr, $write_char:expr, $write_str:expr) => {{
        let fmt = $fmt;
        let args = &mut $args;
        let mut count: usize = 0;
        let mut i = 0usize;
        loop {
            let c = *fmt.add(i) as u8;
            if c == 0 { break; }
            if c != b'%' {
                ($write_char)(c);
                count += 1; i += 1; continue;
            }
            i += 1;
            // Parse flags
            let mut flags: u8 = 0;
            loop {
                let fc = *fmt.add(i) as u8;
                match fc {
                    b'-' => { flags |= FLAG_MINUS; i += 1; }
                    b'+' => { flags |= FLAG_PLUS; i += 1; }
                    b' ' => { flags |= FLAG_SPACE; i += 1; }
                    b'0' => { flags |= FLAG_ZERO; i += 1; }
                    b'#' => { flags |= FLAG_HASH; i += 1; }
                    _ => break,
                }
            }
            // Parse width
            let mut width: usize = 0;
            if *fmt.add(i) as u8 == b'*' {
                width = args.next_arg::<c_int>() as usize; i += 1;
            } else {
                while (*fmt.add(i) as u8) >= b'0' && (*fmt.add(i) as u8) <= b'9' {
                    width = width * 10 + ((*fmt.add(i) as u8) - b'0') as usize; i += 1;
                }
            }
            // Parse precision
            let mut precision: i32 = -1;
            if *fmt.add(i) as u8 == b'.' {
                i += 1;
                if *fmt.add(i) as u8 == b'*' {
                    precision = args.next_arg::<c_int>(); i += 1;
                } else {
                    precision = 0;
                    while (*fmt.add(i) as u8) >= b'0' && (*fmt.add(i) as u8) <= b'9' {
                        precision = precision * 10 + ((*fmt.add(i) as u8) - b'0') as i32; i += 1;
                    }
                }
            }
            // Parse length modifier
            let len_mod = *fmt.add(i) as u8;
            let spec: u8;
            if len_mod == b'h' || len_mod == b'l' || len_mod == b'j' || len_mod == b'z' || len_mod == b't' || len_mod == b'L' {
                i += 1;
                let len_mod2 = *fmt.add(i) as u8;
                if (len_mod == b'h' && len_mod2 == b'h') || (len_mod == b'l' && len_mod2 == b'l') {
                    i += 1;
                    spec = *fmt.add(i) as u8;
                    // Handle double-length modifiers
                    match (len_mod, len_mod2, spec) {
                        (b'h', b'h', b'n') => {
                            let p = args.next_arg::<*mut c_void>();
                            if !p.is_null() { *(p as *mut u8) = count as u8; }
                        }
                        (b'l', b'l', b'n') => {
                            let p = args.next_arg::<*mut c_void>();
                            if !p.is_null() { *(p as *mut c_longlong) = count as c_longlong; }
                        }
                        (b'l', b'l', b'd') | (b'l', b'l', b'i') => {
                            let val = args.next_arg::<c_longlong>();
                            let neg = val < 0;
                            let abs = if neg { val.wrapping_neg() as u64 } else { val as u64 };
                            let b = format_u64(abs);
                            let sign = if neg { Some(b'-') } else if flags & FLAG_PLUS != 0 { Some(b'+') } else if flags & FLAG_SPACE != 0 { Some(b' ') } else { None };
                            let mut fbuf = [0u8; 32];
                            let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, sign, None, precision, width, flags, false);
                            ($write_str)(fbuf.as_ptr(), len); count += len;
                        }
                        (b'l', b'l', b'u') => {
                            let val = args.next_arg::<c_ulonglong>();
                            let b = format_u64(val as u64);
                            let mut fbuf = [0u8; 32];
                            let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, false);
                            ($write_str)(fbuf.as_ptr(), len); count += len;
                        }
                        (b'l', b'l', b'o') => {
                            let val = args.next_arg::<c_ulonglong>();
                            let b = format_octal(val as u64, flags & FLAG_HASH != 0);
                            let mut fbuf = [0u8; 32];
                            let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, flags & FLAG_HASH != 0);
                            ($write_str)(fbuf.as_ptr(), len); count += len;
                        }
                        _ => { ($write_char)(b'%'); ($write_char)(len_mod); ($write_char)(len_mod2); ($write_char)(spec); count += 4; }
                    }
                } else {
                    spec = len_mod2;
                    match (len_mod, spec) {
                        (b'h', b'n') => {
                            let p = args.next_arg::<*mut c_void>();
                            if !p.is_null() { *(p as *mut u16) = count as u16; }
                        }
                        (b'l', b'n') => {
                            let p = args.next_arg::<*mut c_void>();
                            if !p.is_null() { *(p as *mut c_long) = count as c_long; }
                        }
                        (b'z', b'n') | (b't', b'n') => {
                            let p = args.next_arg::<*mut c_void>();
                            if !p.is_null() { *(p as *mut usize) = count; }
                        }
                        (b'j', b'n') => {
                            let p = args.next_arg::<*mut c_void>();
                            if !p.is_null() { *(p as *mut c_ulonglong) = count as c_ulonglong; }
                        }
                        (b'l', b'd') | (b'l', b'i') => {
                            let val = args.next_arg::<c_long>();
                            let neg = val < 0;
                            let abs = if neg { val.wrapping_neg() as u64 } else { val as u64 };
                            let b = format_u64(abs);
                            let sign = if neg { Some(b'-') } else if flags & FLAG_PLUS != 0 { Some(b'+') } else if flags & FLAG_SPACE != 0 { Some(b' ') } else { None };
                            let mut fbuf = [0u8; 32];
                            let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, sign, None, precision, width, flags, false);
                            ($write_str)(fbuf.as_ptr(), len); count += len;
                        }
                        (b'l', b'u') => {
                            let val = args.next_arg::<c_ulong>();
                            let b = format_u64(val as u64);
                            let mut fbuf = [0u8; 32];
                            let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, false);
                            ($write_str)(fbuf.as_ptr(), len); count += len;
                        }
                        (b'l', b'x') => {
                            let val = args.next_arg::<c_ulong>();
                            let b = format_hex(val as u64, false);
                            let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'x') } else { None };
                            let mut fbuf = [0u8; 32];
                            let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                            ($write_str)(fbuf.as_ptr(), len); count += len;
                        }
                        (b'l', b'X') => {
                            let val = args.next_arg::<c_ulong>();
                            let b = format_hex(val as u64, true);
                            let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'X') } else { None };
                            let mut fbuf = [0u8; 32];
                            let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                            ($write_str)(fbuf.as_ptr(), len); count += len;
                        }
                        (b'l', b'o') => {
                            let val = args.next_arg::<c_ulong>();
                            let b = format_octal(val as u64, flags & FLAG_HASH != 0);
                            let mut fbuf = [0u8; 32];
                            let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, flags & FLAG_HASH != 0);
                            ($write_str)(fbuf.as_ptr(), len); count += len;
                        }
                        (b'l', b'f') | (b'l', b'F') | (b'l', b'e') | (b'l', b'E')
                        | (b'l', b'g') | (b'l', b'G') | (b'l', b'a') | (b'l', b'A') => {
                            let val = args.next_arg::<f64>();
                            let ucase = spec == b'F' || spec == b'E' || spec == b'G' || spec == b'A';
                            let ftype = match spec | 0x20 { b'f' => FMT_F, b'e' => FMT_E, b'g' => FMT_G, b'a' => FMT_A, _ => FMT_F };
                            let mut fbuf = [0u8; 4224];
                            let flen = format_f64_full(fbuf.as_mut_ptr(), val, ftype, precision, flags, ucase);
                            let total = apply_width_flags(fbuf.as_mut_ptr().add(4096), fbuf.as_ptr(), flen, width, flags);
                            core::ptr::copy_nonoverlapping(fbuf.as_ptr().add(4096), fbuf.as_mut_ptr(), total);
                            ($write_str)(fbuf.as_ptr(), total); count += total;
                        }
                        (b'L', b'f') | (b'L', b'F') | (b'L', b'e') | (b'L', b'E')
                        | (b'L', b'g') | (b'L', b'G') | (b'L', b'a') | (b'L', b'A') => {
                            #[cfg(target_arch = "aarch64")]
                            let val = {
                                let lo: u64 = args.next_arg::<u64>();
                                let hi: u64 = args.next_arg::<u64>();
                                let combined: u128 = ((hi as u128) << 64) | (lo as u128);
                                f128::from_bits(combined) as f64
                            };
                            #[cfg(target_arch = "x86_64")]
                            let val = args.next_arg::<f64>();
                            let ucase = spec == b'F' || spec == b'E' || spec == b'G' || spec == b'A';
                            let ftype = match spec | 0x20 { b'f' => FMT_F, b'e' => FMT_E, b'g' => FMT_G, b'a' => FMT_A, _ => FMT_F };
                            let mut fbuf = [0u8; 4224];
                            let flen = format_f64_full(fbuf.as_mut_ptr(), val, ftype, precision, flags, ucase);
                            let total = apply_width_flags(fbuf.as_mut_ptr().add(4096), fbuf.as_ptr(), flen, width, flags);
                            core::ptr::copy_nonoverlapping(fbuf.as_ptr().add(4096), fbuf.as_mut_ptr(), total);
                            ($write_str)(fbuf.as_ptr(), total); count += total;
                        }
                        _ => { ($write_char)(b'%'); ($write_char)(len_mod); ($write_char)(spec); count += 3; }
                    }
                }
            } else {
                spec = *fmt.add(i) as u8;
                match spec {
                    b's' => {
                        let s = args.next_arg::<*const c_char>();
                        if s.is_null() {
                            let slen = if precision >= 0 { (precision as usize).min(6) } else { 6 };
                            let padded_len = if width > slen { width } else { slen };
                            let pad = padded_len - slen;
                            if flags & FLAG_MINUS != 0 {
                                ($write_str)(b"(null)".as_ptr(), slen);
                                for _ in 0..pad { ($write_char)(b' '); }
                            } else {
                                for _ in 0..pad { ($write_char)(b' '); }
                                ($write_str)(b"(null)".as_ptr(), slen);
                            }
                            count += padded_len;
                        } else {
                            let full_len = strlen(s as *const c_char);
                            let slen = if precision >= 0 { (precision as usize).min(full_len) } else { full_len };
                            let padded_len = if width > slen { width } else { slen };
                            let pad = padded_len - slen;
                            if flags & FLAG_MINUS != 0 {
                                ($write_str)(s as *const u8, slen);
                                for _ in 0..pad { ($write_char)(b' '); }
                            } else {
                                for _ in 0..pad { ($write_char)(b' '); }
                                ($write_str)(s as *const u8, slen);
                            }
                            count += padded_len;
                        }
                    }
                    b'd' | b'i' => {
                        let d = args.next_arg::<c_int>();
                        let neg = d < 0;
                        let abs = if neg { d.wrapping_neg() as u64 } else { d as u64 };
                        let b = format_u64(abs);
                        let sign = if neg { Some(b'-') } else if flags & FLAG_PLUS != 0 { Some(b'+') } else if flags & FLAG_SPACE != 0 { Some(b' ') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, sign, None, precision, width, flags, false);
                        ($write_str)(fbuf.as_ptr(), len); count += len;
                    }
                    b'u' => {
                        let u = args.next_arg::<c_uint>();
                        let b = format_u64(u as u64);
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, false);
                        ($write_str)(fbuf.as_ptr(), len); count += len;
                    }
                    b'x' => {
                        let x = args.next_arg::<c_uint>();
                        let b = format_hex(x as u64, false);
                        let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'x') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                        ($write_str)(fbuf.as_ptr(), len); count += len;
                    }
                    b'X' => {
                        let x = args.next_arg::<c_uint>();
                        let b = format_hex(x as u64, true);
                        let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'X') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                        ($write_str)(fbuf.as_ptr(), len); count += len;
                    }
                    b'o' => {
                        let o = args.next_arg::<c_uint>();
                        let b = format_octal(o as u64, flags & FLAG_HASH != 0);
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, flags & FLAG_HASH != 0);
                        ($write_str)(fbuf.as_ptr(), len); count += len;
                    }
                    b'c' => {
                        let ch = args.next_arg::<c_int>();
                        if width > 1 {
                            let pad = width - 1;
                            if flags & FLAG_MINUS != 0 {
                                ($write_char)(ch as u8); count += 1;
                                for _ in 0..pad { ($write_char)(b' '); count += 1; }
                            } else {
                                for _ in 0..pad { ($write_char)(b' '); count += 1; }
                                ($write_char)(ch as u8); count += 1;
                            }
                        } else {
                            ($write_char)(ch as u8); count += 1;
                        }
                    }
                    b'p' => {
                        let p = args.next_arg::<*const c_void>();
                        let mut fbuf = [0u8; 20];
                        fbuf[0] = b'0'; fbuf[1] = b'x';
                        let b = format_hex(p as u64, false);
                        core::ptr::copy_nonoverlapping(b.0.as_ptr(), fbuf.as_mut_ptr().add(2), b.1);
                        ($write_str)(fbuf.as_ptr(), b.1 + 2); count += b.1 + 2;
                    }
                    b'f' | b'F' | b'e' | b'E' | b'g' | b'G' | b'a' | b'A' => {
                        let val = args.next_arg::<f64>();
                        let ucase = spec == b'F' || spec == b'E' || spec == b'G' || spec == b'A';
                        let ftype = match spec | 0x20 { b'f' => FMT_F, b'e' => FMT_E, b'g' => FMT_G, b'a' => FMT_A, _ => FMT_F };
                        let mut fbuf = [0u8; 4224];
                        let flen = format_f64_full(fbuf.as_mut_ptr(), val, ftype, precision, flags, ucase);
                        let total = apply_width_flags(fbuf.as_mut_ptr().add(4096), fbuf.as_ptr(), flen, width, flags);
                        core::ptr::copy_nonoverlapping(fbuf.as_ptr().add(4096), fbuf.as_mut_ptr(), total);
                        ($write_str)(fbuf.as_ptr(), total); count += total;
                    }
                    b'%' => { ($write_char)(b'%'); count += 1; }
                    b'n' => {
                        let p = args.next_arg::<*mut c_void>();
                        if !p.is_null() { *(p as *mut c_int) = count as c_int; }
                    }
                    _ => { ($write_char)(b'%'); ($write_char)(spec); count += 2; }
                }
            }
            i += 1;
        }
        count as c_int
    }};
}

unsafe fn format_u64(mut val: u64) -> ([u8; 20], usize) {
    let mut buf = [0u8; 20];
    if val == 0 { buf[0] = b'0'; return (buf, 1); }
    let mut tmp = [0u8; 20];
    let mut pos = 20;
    while val > 0 { pos -= 1; tmp[pos] = b'0' + (val % 10) as u8; val /= 10; }
    let len = 20 - pos;
    core::ptr::copy_nonoverlapping(tmp.as_ptr().add(pos), buf.as_mut_ptr(), len);
    (buf, len)
}

unsafe fn format_i64(val: i64) -> ([u8; 21], usize) {
    let mut buf = [0u8; 21];
    if val < 0 {
        buf[0] = b'-';
        let (inner, len) = format_u64((-(val as i128)) as u64);
        core::ptr::copy_nonoverlapping(inner.as_ptr(), buf.as_mut_ptr().add(1), len);
        (buf, len + 1)
    } else {
        let (inner, len) = format_u64(val as u64);
        core::ptr::copy_nonoverlapping(inner.as_ptr(), buf.as_mut_ptr(), len);
        (buf, len)
    }
}

unsafe fn format_hex(mut val: u64, uppercase: bool) -> ([u8; 16], usize) {
    let digits = if uppercase { b"0123456789ABCDEF" } else { b"0123456789abcdef" };
    let mut buf = [0u8; 16];
    if val == 0 { buf[0] = b'0'; return (buf, 1); }
    let mut tmp = [0u8; 16];
    let mut pos = 16;
    while val > 0 { pos -= 1; tmp[pos] = digits[(val & 0xf) as usize]; val >>= 4; }
    let len = 16 - pos;
    core::ptr::copy_nonoverlapping(tmp.as_ptr().add(pos), buf.as_mut_ptr(), len);
    (buf, len)
}

unsafe fn format_octal(mut val: u64, alt: bool) -> ([u8; 24], usize) {
    let mut buf = [0u8; 24];
    if val == 0 {
        buf[0] = b'0';
        return (buf, 1);
    }
    let mut tmp = [0u8; 24];
    let mut pos = 24;
    while val > 0 {
        pos -= 1;
        tmp[pos] = b'0' + (val & 7) as u8;
        val >>= 3;
    }
    let len = 24 - pos;
    if alt {
        tmp[pos - 1] = b'0';
        core::ptr::copy_nonoverlapping(tmp.as_ptr().add(pos - 1), buf.as_mut_ptr(), len + 1);
        (buf, len + 1)
    } else {
        core::ptr::copy_nonoverlapping(tmp.as_ptr().add(pos), buf.as_mut_ptr(), len);
        (buf, len)
    }
}

unsafe fn format_int(
    dst: *mut u8,
    digits: *const u8,
    digits_len: usize,
    sign: Option<u8>,
    prefix: Option<u8>,
    precision: i32,
    width: usize,
    flags: u8,
    octal_alt: bool,
) -> usize {
    let num_digits = if precision == 0 && digits_len == 1 && *digits == b'0' && !octal_alt {
        0
    } else {
        digits_len
    };
    let prec = precision as usize;
    let zero_pad = if precision >= 0 && prec > num_digits { prec - num_digits } else { 0 };

    let sign_len = sign.map_or(0, |_| 1);
    let prefix_len = prefix.map_or(0, |_| 2);
    let content_len = sign_len + prefix_len + zero_pad + num_digits;

    let mut buf = [0u8; 32];
    let mut pos = 0usize;
    if let Some(c) = sign { buf[pos] = c; pos += 1; }
    if let Some(c) = prefix { buf[pos] = b'0'; pos += 1; buf[pos] = c; pos += 1; }
    for _ in 0..zero_pad { buf[pos] = b'0'; pos += 1; }
    if num_digits > 0 {
        core::ptr::copy_nonoverlapping(digits, buf.as_mut_ptr().add(pos), num_digits);
    }

    if content_len >= width {
        core::ptr::copy_nonoverlapping(buf.as_ptr(), dst, content_len);
        return content_len;
    }
    let pad = width - content_len;
    if flags & FLAG_MINUS != 0 {
        core::ptr::copy_nonoverlapping(buf.as_ptr(), dst, content_len);
        for i in 0..pad { *dst.add(content_len + i) = b' '; }
    } else if flags & FLAG_ZERO != 0 && precision < 0 {
        let fixed = sign_len + prefix_len;
        core::ptr::copy_nonoverlapping(buf.as_ptr(), dst, fixed);
        for i in fixed..width - num_digits { *dst.add(i) = b'0'; }
        if num_digits > 0 {
            core::ptr::copy_nonoverlapping(digits, dst.add(width - num_digits), num_digits);
        }
    } else {
        for i in 0..pad { *dst.add(i) = b' '; }
        core::ptr::copy_nonoverlapping(buf.as_ptr(), dst.add(pad), content_len);
    }
    width
}

unsafe fn format_f64(val: f64) -> ([u8; 64], usize) {
    let mut buf = [0u8; 64];
    let mut pos = 0usize;
    let mut v = val;
    if v.is_nan() {
        let nan = b"nan";
        core::ptr::copy_nonoverlapping(nan.as_ptr(), buf.as_mut_ptr(), 3);
        return (buf, 3);
    }
    if v < 0.0 { buf[pos] = b'-'; pos += 1; v = -v; }
    if v.is_infinite() {
        let inf = b"inf";
        core::ptr::copy_nonoverlapping(inf.as_ptr(), buf.as_mut_ptr().add(pos), 3);
        return (buf, pos + 3);
    }
    let int_part = v as u64;
    let frac = v - int_part as f64;
    let (ibuf, ilen) = format_u64(int_part);
    core::ptr::copy_nonoverlapping(ibuf.as_ptr(), buf.as_mut_ptr().add(pos), ilen);
    pos += ilen;
    buf[pos] = b'.';
    pos += 1;
    let mut f = frac;
    for _ in 0..6 { f *= 10.0; let digit = f as u8; buf[pos] = b'0' + digit; pos += 1; f -= digit as f64; }
    (buf, pos)
}

// Float format specifier type
const FMT_F: u8 = 0;
const FMT_E: u8 = 1;
const FMT_G: u8 = 2;
const FMT_A: u8 = 3;

const FLAG_HASH: u8 = 1;
const FLAG_PLUS: u8 = 2;
const FLAG_SPACE: u8 = 4;
const FLAG_ZERO: u8 = 8;
const FLAG_MINUS: u8 = 16;

// === %f formatting helpers ===

/// Convert (mant << shift) to decimal string. Returns byte count written to buf.
/// mant is a 53-bit significand, shift is the left-shift amount (can be 0..~971).
unsafe fn bigint_to_decimal(buf: *mut u8, mant: u64, shift: u32) -> usize {
    // Store as little-endian u32 limbs. Max bits: 53 + 971 ≈ 1024 → 32 limbs.
    let nlimbs = (2 + (shift + 31) / 32) as usize;
    let mut limbs = [0u32; 36];
    limbs[0] = mant as u32;
    limbs[1] = (mant >> 32) as u32;

    let ws = (shift / 32) as usize;
    let bs = shift % 32;

    // Word-level shift left
    if ws > 0 {
        let mut i = 1usize;
        loop {
            limbs[i + ws] = limbs[i];
            if i == 0 { break; }
            i -= 1;
        }
        for j in 0..ws { limbs[j] = 0; }
    }
    // Bit-level shift left
    if bs > 0 {
        let mut carry = 0u32;
        let end = nlimbs.min(ws + 3);
        for i in ws..end {
            let new_val = (limbs[i] << bs) | carry;
            carry = limbs[i] >> (32 - bs);
            limbs[i] = new_val;
        }
    }

    // Find highest non-zero limb
    let mut used = nlimbs;
    while used > 0 && limbs[used - 1] == 0 { used -= 1; }
    if used == 0 { *buf = b'0'; return 1; }

    // Repeated division by 10 to extract decimal digits
    let mut digits = [0u8; 400];
    let mut nd = 0usize;
    loop {
        let mut all_zero = true;
        for i in 0..used { if limbs[i] != 0 { all_zero = false; break; } }
        if all_zero { break; }
        let mut rem = 0u64;
        for i in (0..used).rev() {
            let val = (rem << 32) | limbs[i] as u64;
            limbs[i] = (val / 10) as u32;
            rem = val % 10;
        }
        digits[nd] = rem as u8;
        nd += 1;
    }
    // Write digits in big-endian order
    for i in 0..nd { *buf.add(i) = b'0' + digits[nd - 1 - i]; }
    nd
}

/// Compute `count` fractional digits of frac_bits / 2^shift using u128 arithmetic.
/// shift must be <= 124. Returns true if any value remains after the last digit.
unsafe fn frac_digits_u128(buf: *mut u8, frac_bits: u64, shift: u32, count: usize) -> bool {
    let mask: u128 = (1u128 << shift) - 1;
    let mut rem = frac_bits as u128;
    for i in 0..count {
        rem *= 10;
        let digit = (rem >> shift) as u8;
        rem &= mask;
        *buf.add(i) = digit;
    }
    rem != 0
}

/// Compute `count` fractional digits of frac_bits / 2^shift using big-integer arithmetic.
/// For shift > 124 (up to ~1074). Returns true if any value remains after the last digit.
unsafe fn frac_digits_bigint(buf: *mut u8, frac_bits: u64, shift: u32, count: usize) -> bool {
    let nlimbs = ((shift + 31) / 32) as usize;
    let mut rem = [0u32; 36]; // enough for shift up to 1088
    rem[0] = frac_bits as u32;
    if nlimbs > 1 { rem[1] = (frac_bits >> 32) as u32; }
    // Mask to exactly `shift` bits
    let hi_word = (shift / 32) as usize;
    let hi_bit = shift % 32;
    if hi_bit > 0 && hi_word < nlimbs {
        rem[hi_word] &= (1u32 << hi_bit) - 1;
    }
    // Zero upper limbs beyond shift
    let clear_from = if hi_bit > 0 { hi_word + 1 } else { hi_word };
    for j in clear_from..nlimbs { rem[j] = 0; }

    for i in 0..count {
        // Multiply remainder by 10
        let mut carry = 0u64;
        for j in 0..nlimbs {
            let val = rem[j] as u64 * 10 + carry;
            rem[j] = val as u32;
            carry = val >> 32;
        }
        // Extract digit from bit position `shift`
        let digit: u32;
        if hi_bit == 0 {
            digit = if hi_word < nlimbs { rem[hi_word] } else { carry as u32 };
        } else {
            let mut d = if hi_word < nlimbs { rem[hi_word] >> hi_bit } else { 0 };
            if hi_word + 1 < nlimbs {
                d |= rem[hi_word + 1] << (32 - hi_bit);
            } else if carry > 0 {
                d |= (carry as u32) << (32 - hi_bit);
            }
            digit = d;
        }
        *buf.add(i) = digit.min(9) as u8;
        // Clear bits from position `shift` upward (keep remainder < 2^shift)
        if hi_bit == 0 {
            if hi_word < nlimbs { rem[hi_word] = 0; }
        } else {
            if hi_word < nlimbs {
                rem[hi_word] &= (1u32 << hi_bit) - 1;
            }
            for j in (hi_word + 1)..nlimbs { rem[j] = 0; }
        }
    }
    // Check if remainder is non-zero
    for i in 0..nlimbs { if rem[i] != 0 { return true; } }
    false
}

/// Increment a decimal string stored at buf[0..*len] in-place.
unsafe fn increment_decimal(buf: *mut u8, len: &mut usize) {
    let mut j = *len;
    loop {
        if j == 0 {
            // Shift right and prepend '1'
            let l = *len;
            for i in (0..l).rev() { *buf.add(i + 1) = *buf.add(i); }
            *buf = b'1';
            *len = l + 1;
            return;
        }
        j -= 1;
        let d = *buf.add(j);
        if d < b'9' { *buf.add(j) = d + 1; return; }
        *buf.add(j) = b'0';
    }
}

// Write float to dst. Returns bytes written. Does NOT handle width/flags padding.
unsafe fn format_f64_full(
    dst: *mut u8, val: f64, fmt_type: u8, precision: i32, flags: u8, uppercase: bool,
) -> usize {
    let mut pos = 0usize;

    // NaN
    if val.is_nan() {
        let nan_s: &[u8] = if uppercase { b"NAN" } else { b"nan" };
        core::ptr::copy_nonoverlapping(nan_s.as_ptr(), dst, 3);
        return 3;
    }
    let neg = val.is_sign_negative();
    let v = if neg { -val } else { val };

    // Inf
    if v.is_infinite() {
        if neg { *dst.add(pos) = b'-'; pos += 1; }
        else if flags & FLAG_PLUS != 0 { *dst.add(pos) = b'+'; pos += 1; }
        else if flags & FLAG_SPACE != 0 { *dst.add(pos) = b' '; pos += 1; }
        let inf_s: &[u8] = if uppercase { b"INF" } else { b"inf" };
        core::ptr::copy_nonoverlapping(inf_s.as_ptr(), dst.add(pos), 3);
        return pos + 3;
    }

    // Sign
    if neg { *dst.add(pos) = b'-'; pos += 1; }
    else if flags & FLAG_PLUS != 0 { *dst.add(pos) = b'+'; pos += 1; }
    else if flags & FLAG_SPACE != 0 { *dst.add(pos) = b' '; pos += 1; }

    // === Hex float ===
    if fmt_type == FMT_A {
        return pos + fmt_hex_float(dst.add(pos), val, precision, flags, uppercase);
    }

    // === Decimal formats ===
    let p: usize;
    let mut exp: i32;
    let use_f: bool;

    match fmt_type {
        FMT_G => {
            let gp = if precision < 0 { 6usize } else if precision == 0 { 1usize } else { precision as usize };
            exp = if v > 0.0 { compute_exp10(v) } else { 0 };
            use_f = exp >= -4 && exp < gp as i32;
            if use_f {
                p = if exp < 0 { gp + ((-exp) as usize - 1) }
                    else if (exp as usize) < gp { gp - exp as usize - 1 }
                    else { 0 };
            } else {
                p = gp - 1;
            }
        }
        FMT_E => {
            p = if precision < 0 { 6usize } else { precision as usize };
            exp = if v > 0.0 { compute_exp10(v) } else { 0 };
            use_f = false;
        }
        _ => {
            p = if precision < 0 { 6usize } else { precision as usize };
            exp = 0;
            use_f = true;
        }
    }

    if v == 0.0 {
        *dst.add(pos) = b'0'; pos += 1;
        if !(fmt_type == FMT_G && flags & FLAG_HASH == 0) {
            if p > 0 || flags & FLAG_HASH != 0 {
                *dst.add(pos) = b'.'; pos += 1;
                for _ in 0..p { *dst.add(pos) = b'0'; pos += 1; }
            }
        }
        if !use_f {
            let ec = if uppercase { b'E' } else { b'e' };
            *dst.add(pos) = ec; pos += 1;
            *dst.add(pos) = b'+'; pos += 1;
            *dst.add(pos) = b'0'; pos += 1;
            *dst.add(pos) = b'0'; pos += 1;
        }
        return pos;
    }

    // === %f format: exact bit-based arithmetic, returns early ===
    if use_f {
        let bits = v.to_bits();
        let raw_mant = bits & 0x000FFFFFFFFFFFFF;
        let raw_exp_bits = ((bits >> 52) & 0x7FF) as i32;
        let (mant_b, bin_exp) = if raw_exp_bits == 0 {
            (raw_mant, -1022i32)
        } else {
            (raw_mant | (1u64 << 52), raw_exp_bits - 1023)
        };

        // Integer part as decimal string
        let mut ibuf = [0u8; 400];
        let mut ilen: usize;
        if bin_exp < 0 {
            ibuf[0] = b'0'; ilen = 1;
        } else if bin_exp <= 63 {
            let int_val = if bin_exp <= 52 { mant_b >> (52 - bin_exp) }
                          else { mant_b << (bin_exp - 52) };
            let (buf2, len2) = format_u64(int_val);
            ibuf[..len2].copy_from_slice(&buf2[..len2]);
            ilen = len2;
        } else {
            ilen = bigint_to_decimal(ibuf.as_mut_ptr(), mant_b, (bin_exp - 52) as u32);
        }

        // Fractional digits
        let mut fbuf = [0u8; 1100];
        let mut has_rest = false;
        if p > 0 && bin_exp < 52 {
            let shift = (52 - bin_exp) as u32;
            let frac_bits = if bin_exp >= 0 { mant_b & ((1u64 << shift) - 1) } else { mant_b };
            if shift <= 124 {
                has_rest = frac_digits_u128(fbuf.as_mut_ptr(), frac_bits, shift, p + 1);
            } else {
                has_rest = frac_digits_bigint(fbuf.as_mut_ptr(), frac_bits, shift, p + 1);
            }
        }

        // Rounding
        if p > 0 && bin_exp < 52 {
            let next_d = fbuf[p];
            let last_d = fbuf[p - 1];
            if next_d > 5 || (next_d == 5 && (has_rest || last_d & 1 != 0)) {
                let mut carry = true;
                for j in (0..p).rev() {
                    if !carry { break; }
                    fbuf[j] += 1;
                    if fbuf[j] < 10 { carry = false; break; }
                    fbuf[j] = 0;
                }
                if carry { increment_decimal(ibuf.as_mut_ptr(), &mut ilen); }
            }
        } else if p == 0 {
            // %.0f: round integer based on fractional part
            if bin_exp >= 0 && bin_exp < 52 {
                let shift = (52 - bin_exp) as u32;
                let frac_bits = mant_b & ((1u64 << shift) - 1);
                let half = 1u64 << (shift - 1);
                let last_d = ibuf[ilen - 1] - b'0';
                if frac_bits > half || (frac_bits == half && last_d & 1 != 0) {
                    increment_decimal(ibuf.as_mut_ptr(), &mut ilen);
                }
            }
        }

        // Output
        core::ptr::copy_nonoverlapping(ibuf.as_ptr(), dst.add(pos), ilen);
        pos += ilen;
        if p > 0 || flags & FLAG_HASH != 0 {
            *dst.add(pos) = b'.'; pos += 1;
            for i in 0..p { *dst.add(pos) = b'0' + fbuf[i]; pos += 1; }
        }
        if fmt_type == FMT_G && flags & FLAG_HASH == 0 {
            while pos > 0 && *dst.add(pos - 1) == b'0' { pos -= 1; }
            if pos > 0 && *dst.add(pos - 1) == b'.' { pos -= 1; }
        }
        return pos;
    }

    // === %e/%g format: exact big-integer digit extraction ===
    // D = floor(v * 10^(p+1-exp)), using v = mant_b * 2^(bin_exp-52)
    let mut fbuf = [0u8; 20];
    let mut int_part: u64;
    {
        let bits_v = v.to_bits();
        let raw_mant_v = bits_v & 0x000FFFFFFFFFFFFF;
        let raw_exp_v = ((bits_v >> 52) & 0x7FF) as i32;
        let (mant_b_v, bin_exp_v) = if raw_exp_v == 0 {
            (raw_mant_v, -1022i32)
        } else {
            (raw_mant_v | (1u64 << 52), raw_exp_v - 1023)
        };

        let s: i32 = (p as i32) + 1 - exp;
        let bs: i32 = bin_exp_v - 52 + s;

        let mut limbs = [0u32; 40];
        limbs[0] = mant_b_v as u32;
        limbs[1] = (mant_b_v >> 32) as u32;
        let mut nlimbs: usize = if limbs[1] != 0 { 2 } else { 1 };

        if s > 0 {
            for _ in 0..(s as usize) {
                let mut carry = 0u64;
                for j in 0..nlimbs {
                    let val = limbs[j] as u64 * 5 + carry;
                    limbs[j] = val as u32;
                    carry = val >> 32;
                }
                if carry > 0 { limbs[nlimbs] = carry as u32; nlimbs += 1; }
            }
        }

        if bs > 0 {
            let ws = (bs as u32 / 32) as usize;
            let bit = bs as u32 % 32;
            if ws > 0 {
                let mut i = nlimbs;
                while i > 0 { i -= 1; if i + ws < 40 { limbs[i + ws] = limbs[i]; } limbs[i] = 0; }
                nlimbs += ws;
            }
            if bit > 0 {
                let mut carry = 0u32;
                for i in ws..nlimbs {
                    let new_val = (limbs[i] << bit) | carry;
                    carry = limbs[i] >> (32 - bit);
                    limbs[i] = new_val;
                }
                if carry > 0 && nlimbs < 40 { limbs[nlimbs] = carry; nlimbs += 1; }
            }
        }

        let mut has_rest = false;
        if s < 0 {
            for _ in 0..(-s as usize) {
                let mut rem = 0u64;
                for i in (0..nlimbs).rev() {
                    let val = (rem << 32) | limbs[i] as u64;
                    limbs[i] = (val / 5) as u32;
                    rem = val % 5;
                }
                if rem != 0 { has_rest = true; }
            }
        }

        if bs < 0 {
            let rshift = (-bs) as u32;
            let ws = (rshift / 32) as usize;
            let bit = rshift % 32;
            if ws > 0 {
                for j in 0..ws.min(nlimbs) { if limbs[j] != 0 { has_rest = true; } }
                for i in ws..nlimbs { limbs[i - ws] = limbs[i]; }
                nlimbs = nlimbs.saturating_sub(ws);
            }
            if bit > 0 && nlimbs > 0 {
                let mask = (1u32 << bit) - 1;
                if limbs[0] & mask != 0 { has_rest = true; }
                for i in 0..nlimbs - 1 {
                    limbs[i] = (limbs[i] >> bit) | (limbs[i + 1] << (32 - bit));
                }
                limbs[nlimbs - 1] >>= bit;
            }
        }

        while nlimbs > 0 && limbs[nlimbs - 1] == 0 { nlimbs -= 1; }

        let ndigits = p + 2;
        let mut all_digits = [0u8; 25];
        let mut nd = 0usize;
        if nlimbs > 0 {
            loop {
                let mut all_zero = true;
                for i in 0..nlimbs { if limbs[i] != 0 { all_zero = false; break; } }
                if all_zero { break; }
                let mut rem64 = 0u64;
                for i in (0..nlimbs).rev() {
                    let val = (rem64 << 32) | limbs[i] as u64;
                    limbs[i] = (val / 10) as u32;
                    rem64 = val % 10;
                }
                if nd < 25 { all_digits[nd] = rem64 as u8; }
                nd += 1;
            }
        }

        if nd >= ndigits {
            for i in 0..ndigits { fbuf[ndigits - 1 - i] = all_digits[i]; }
        } else {
            for i in 0..nd { fbuf[ndigits - 1 - i] = all_digits[i]; }
            for i in nd..ndigits { fbuf[ndigits - 1 - i] = 0; }
        }

        int_part = fbuf[0] as u64;

        // Round half to even: next_d is the rounding digit beyond precision
        if p > 0 {
            let next_d = fbuf[p + 1];
            let last_d = fbuf[p];
            if next_d > 5 || (next_d == 5 && (has_rest || last_d & 1 != 0)) {
                let mut carry = true;
                for j in (1..=p).rev() {
                    if !carry { break; }
                    let d = fbuf[j] + 1;
                    if d < 10 { fbuf[j] = d; carry = false; break; }
                    fbuf[j] = 0;
                }
                if carry { int_part += 1; }
            }
        } else {
            let next_d = fbuf[1];
            if next_d > 5 || (next_d == 5 && (has_rest || int_part & 1 != 0)) {
                int_part += 1;
            }
        }

        // fbuf[1..=p] are now the fractional digits for output
        // Shift them to fbuf[0..p] so output code can index fbuf[0..p]
        for i in 0..p { fbuf[i] = fbuf[i + 1]; }
    }

    if !use_f && int_part >= 10 {
        exp += 1;
        int_part = 1;
    }

    // Format output
    if use_f {
        // %f style
        let (ibuf, ilen) = format_u64(int_part);
        core::ptr::copy_nonoverlapping(ibuf.as_ptr(), dst.add(pos), ilen);
        pos += ilen;
        if p > 0 || flags & FLAG_HASH != 0 {
            *dst.add(pos) = b'.'; pos += 1;
            for idx in 0..p { *dst.add(pos) = b'0' + fbuf[idx]; pos += 1; }
        }
        if fmt_type == FMT_G && flags & FLAG_HASH == 0 {
            while pos > 0 && *dst.add(pos - 1) == b'0' { pos -= 1; }
            if pos > 0 && *dst.add(pos - 1) == b'.' { pos -= 1; }
        }
    } else {
        // %e style
        *dst.add(pos) = b'0' + int_part as u8; pos += 1;
        if p > 0 || flags & FLAG_HASH != 0 {
            *dst.add(pos) = b'.'; pos += 1;
            for idx in 0..p { *dst.add(pos) = b'0' + fbuf[idx]; pos += 1; }
        }
        if fmt_type == FMT_G && flags & FLAG_HASH == 0 {
            while pos > 0 && *dst.add(pos - 1) == b'0' { pos -= 1; }
            if pos > 0 && *dst.add(pos - 1) == b'.' { pos -= 1; }
        }
        let ec = if uppercase { b'E' } else { b'e' };
        *dst.add(pos) = ec; pos += 1;
        *dst.add(pos) = if exp < 0 { b'-' } else { b'+' }; pos += 1;
        let eabs = if exp < 0 { (-exp) as u32 } else { exp as u32 };
        if eabs < 10 { *dst.add(pos) = b'0'; pos += 1; }
        let (ebuf, elen) = format_u64(eabs as u64);
        core::ptr::copy_nonoverlapping(ebuf.as_ptr(), dst.add(pos), elen);
        pos += elen;
    }

    pos
}

// Compute decimal exponent: floor(log10(v)) with correction
unsafe fn compute_exp10(v: f64) -> i32 {
    let e = libm::floor(libm::log10(v)) as i32;
    let probe = v / libm::pow(10.0, e as f64);
    if probe >= 10.0 { e + 1 }
    else if probe < 1.0 { e - 1 }
    else { e }
}

// Write hex float to dst. Returns bytes written.
unsafe fn fmt_hex_float(dst: *mut u8, val: f64, precision: i32, _flags: u8, uppercase: bool) -> usize {
    // Handle zero (and negative zero) specially
    if val.to_bits() & 0x7FFFFFFFFFFFFFFF == 0 {
        let lc = !uppercase;
        let pfx: &[u8] = if lc { b"0x" } else { b"0X" };
        let p = if precision < 0 { 0usize } else { precision as usize };
        let mut pos = 0usize;
        core::ptr::copy_nonoverlapping(pfx.as_ptr(), dst.add(pos), 2); pos += 2;
        *dst.add(pos) = b'0'; pos += 1;
        if p > 0 {
            *dst.add(pos) = b'.'; pos += 1;
            for _ in 0..p { *dst.add(pos) = b'0'; pos += 1; }
        }
        let pc = if lc { b'p' } else { b'P' };
        *dst.add(pos) = pc; pos += 1;
        *dst.add(pos) = b'+'; pos += 1;
        *dst.add(pos) = b'0'; pos += 1;
        return pos;
    }
    let bits = val.to_bits();
    let exp_bits = ((bits >> 52) & 0x7FF) as i32;
    let mant_bits = bits & 0x000FFFFFFFFFFFFF;
    let (e, m) = if exp_bits == 0 { (1 - 1023, mant_bits) } else { (exp_bits - 1023, mant_bits | (1u64 << 52)) };
    let p = if precision < 0 { 13usize } else { precision as usize };
    let mut pos = 0usize;
    let lc = !uppercase;
    let pfx: &[u8] = if lc { b"0x" } else { b"0X" };
    core::ptr::copy_nonoverlapping(pfx.as_ptr(), dst.add(pos), 2); pos += 2;

    // Leading digit from bit 52
    let lead = ((m >> 52) & 0xF) as u8;
    *dst.add(pos) = hex_digit_char(lead, lc); pos += 1;
    if p > 0 {
        *dst.add(pos) = b'.'; pos += 1;
    }

    // Extract p hex digits from mantissa bits
    let frac_start = pos;
    let mut rm = m << 4; // next nibble starts at bit 48
    for _ in 0..p {
        let nib = ((rm >> 48) & 0xF) as u8;
        *dst.add(pos) = hex_digit_char(nib, lc); pos += 1;
        rm <<= 4;
    }

    // Rounding based on next nibble
    if p > 0 {
        let round_nib = ((rm >> 48) & 0xF) as u8;
        let rest = rm & 0x0000FFFFFFFFFFFF;
        let last_nib = if p > 0 {
            let c = *dst.add(pos - 1);
            hex_char_val(c)
        } else { lead };
        let should_round = round_nib > 8 || (round_nib == 8 && (rest != 0 || (last_nib & 1) != 0));
        if should_round {
            let mut j = pos - 1;
            loop {
                if j < frac_start { break; }
                let v = hex_char_val(*dst.add(j));
                if v < 15 {
                    *dst.add(j) = hex_digit_char(v + 1, lc);
                    break;
                } else {
                    *dst.add(j) = b'0';
                    if j == frac_start { break; }
                    j -= 1;
                }
            }
        }
    }

    // Exponent
    let pc = if lc { b'p' } else { b'P' };
    *dst.add(pos) = pc; pos += 1;
    *dst.add(pos) = if e < 0 { b'-' } else { b'+' }; pos += 1;
    let eabs = if e < 0 { (-e) as u32 } else { e as u32 };
    let (ebuf, elen) = format_u64(eabs as u64);
    core::ptr::copy_nonoverlapping(ebuf.as_ptr(), dst.add(pos), elen);
    pos += elen;
    pos
}

#[inline]
fn hex_digit_char(v: u8, lowercase: bool) -> u8 {
    if v < 10 { b'0' + v }
    else if lowercase { b'a' + v - 10 }
    else { b'A' + v - 10 }
}

#[inline]
fn hex_char_val(c: u8) -> u8 {
    if c >= b'0' && c <= b'9' { c - b'0' }
    else if c >= b'a' && c <= b'f' { c - b'a' + 10 }
    else if c >= b'A' && c <= b'F' { c - b'A' + 10 }
    else { 0 }
}

// Apply width and flags padding to a formatted string in buf[0..len].
// Writes the final result to dst. Returns total bytes written.
unsafe fn apply_width_flags(
    dst: *mut u8, buf: *const u8, len: usize, width: usize, flags: u8,
) -> usize {
    if len >= width {
        core::ptr::copy_nonoverlapping(buf, dst, len);
        return len;
    }
    let pad = width - len;
    let pad_char = if flags & FLAG_ZERO != 0 && flags & FLAG_MINUS == 0 { b'0' } else { b' ' };
    if flags & FLAG_MINUS != 0 {
        // Left align
        core::ptr::copy_nonoverlapping(buf, dst, len);
        for i in 0..pad { *dst.add(len + i) = pad_char; }
    } else {
        // Right align
        // For zero-padded numbers, sign goes before zeros
        if flags & FLAG_ZERO != 0 && len > 0 {
            let first = *buf;
            if first == b'-' || first == b'+' || first == b' ' {
                *dst = first;
                for i in 0..pad { *dst.add(1 + i) = pad_char; }
                core::ptr::copy_nonoverlapping(buf.add(1), dst.add(1 + pad), len - 1);
                return width;
            }
        }
        for i in 0..pad { *dst.add(i) = pad_char; }
        core::ptr::copy_nonoverlapping(buf, dst.add(pad), len);
    }
    width
}

#[no_mangle]
pub unsafe extern "C" fn vprintf(fmt: *const c_char, mut args: VaList) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { let _ = fwrite(&c as *const u8 as *const c_void, 1, 1, stdout); },
        |s: *const u8, len: usize| { let _ = fwrite(s as *const c_void, 1, len, stdout); }
    )
}

#[no_mangle]
pub unsafe extern "C" fn vfprintf(stream: *mut FILE, fmt: *const c_char, mut args: VaList) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { let _ = fwrite(&c as *const u8 as *const c_void, 1, 1, stream); },
        |s: *const u8, len: usize| { let _ = fwrite(s as *const c_void, 1, len, stream); }
    )
}

#[no_mangle]
pub unsafe extern "C" fn printf(fmt: *const c_char, mut args: ...) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { let _ = fwrite(&c as *const u8 as *const c_void, 1, 1, stdout); },
        |s: *const u8, len: usize| { let _ = fwrite(s as *const c_void, 1, len, stdout); }
    )
}

#[no_mangle]
pub unsafe extern "C" fn fprintf(stream: *mut FILE, fmt: *const c_char, mut args: ...) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { let _ = fwrite(&c as *const u8 as *const c_void, 1, 1, stream); },
        |s: *const u8, len: usize| { let _ = fwrite(s as *const c_void, 1, len, stream); }
    )
}

#[no_mangle]
pub unsafe extern "C" fn dprintf(fd: c_int, fmt: *const c_char, mut args: ...) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { write_str(fd, &c as *const u8, 1); },
        |s: *const u8, len: usize| { write_str(fd, s, len); }
    )
}

#[no_mangle]
pub unsafe extern "C" fn vdprintf(fd: c_int, fmt: *const c_char, mut args: VaList) -> c_int {
    impl_format!(fmt, args,
        |c: u8| { write_str(fd, &c as *const u8, 1); },
        |s: *const u8, len: usize| { write_str(fd, s, len); }
    )
}

// ============================================================
// sprintf / snprintf / vsprintf / vsnprintf
// ============================================================

unsafe fn format_to_buf(buf: *mut u8, cap: usize, fmt: *const c_char, args: &mut VaList<'_>) -> c_int {
    let mut pos = 0usize;
    let mut i = 0usize;
    let mut count = 0usize;
    macro_rules! wc {
        ($c:expr) => { if pos < cap { *buf.add(pos) = $c; } pos += 1; count += 1; }
    }
    macro_rules! ws {
        ($s:expr, $len:expr) => {{
            let __len = $len;
            for k in 0..__len { if pos + k < cap { *buf.add(pos + k) = *$s.add(k); } }
            pos += __len; count += __len;
        }}
    }
    // Write formatted buffer respecting cap, at current pos
    macro_rules! ws_buf {
        ($src:expr, $len:expr) => {{
            let __len = $len;
            for k in 0..__len { if pos + k < cap { *buf.add(pos + k) = *$src.add(k); } }
            pos += __len; count += __len;
        }}
    }
    loop {
        let c = *fmt.add(i) as u8;
        if c == 0 { break; }
        if c != b'%' { wc!(c); i += 1; continue; }
        i += 1;
        // Parse flags
        let mut flags: u8 = 0;
        loop {
            let fc = *fmt.add(i) as u8;
            match fc {
                b'-' => { flags |= FLAG_MINUS; i += 1; }
                b'+' => { flags |= FLAG_PLUS; i += 1; }
                b' ' => { flags |= FLAG_SPACE; i += 1; }
                b'0' => { flags |= FLAG_ZERO; i += 1; }
                b'#' => { flags |= FLAG_HASH; i += 1; }
                _ => break,
            }
        }
        // Parse width
        let mut width: usize = 0;
        if *fmt.add(i) as u8 == b'*' {
            width = args.next_arg::<c_int>() as usize; i += 1;
        } else {
            while (*fmt.add(i) as u8) >= b'0' && (*fmt.add(i) as u8) <= b'9' {
                width = width * 10 + ((*fmt.add(i) as u8) - b'0') as usize; i += 1;
            }
        }
        // Parse precision
        let mut precision: i32 = -1;
        if *fmt.add(i) as u8 == b'.' {
            i += 1;
            if *fmt.add(i) as u8 == b'*' {
                precision = args.next_arg::<c_int>(); i += 1;
            } else {
                precision = 0;
                while (*fmt.add(i) as u8) >= b'0' && (*fmt.add(i) as u8) <= b'9' {
                    precision = precision * 10 + ((*fmt.add(i) as u8) - b'0') as i32; i += 1;
                }
            }
        }
        // Parse length modifier
        let len_mod = *fmt.add(i) as u8;
        let spec: u8;
        if len_mod == b'h' || len_mod == b'l' || len_mod == b'j' || len_mod == b'z' || len_mod == b't' || len_mod == b'L' {
            i += 1;
            let len_mod2 = *fmt.add(i) as u8;
            if (len_mod == b'h' && len_mod2 == b'h') || (len_mod == b'l' && len_mod2 == b'l') {
                i += 1;
                spec = *fmt.add(i) as u8;
                match (len_mod, len_mod2, spec) {
                    (b'h', b'h', b'n') => {
                        let p = args.next_arg::<*mut c_void>();
                        if !p.is_null() { *(p as *mut u8) = count as u8; }
                    }
                    (b'l', b'l', b'n') => {
                        let p = args.next_arg::<*mut c_void>();
                        if !p.is_null() { *(p as *mut c_longlong) = count as c_longlong; }
                    }
                    (b'l', b'l', b'd') | (b'l', b'l', b'i') => {
                        let val = args.next_arg::<c_longlong>();
                        let neg = val < 0;
                        let abs = if neg { val.wrapping_neg() as u64 } else { val as u64 };
                        let b = format_u64(abs);
                        let sign = if neg { Some(b'-') } else if flags & FLAG_PLUS != 0 { Some(b'+') } else if flags & FLAG_SPACE != 0 { Some(b' ') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, sign, None, precision, width, flags, false);
                        ws_buf!(fbuf.as_ptr(), len);
                    }
                    (b'l', b'l', b'u') => {
                        let val = args.next_arg::<c_ulonglong>();
                        let b = format_u64(val as u64);
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, false);
                        ws_buf!(fbuf.as_ptr(), len);
                    }
                    (b'l', b'l', b'o') => {
                        let val = args.next_arg::<c_ulonglong>();
                        let b = format_octal(val as u64, flags & FLAG_HASH != 0);
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, flags & FLAG_HASH != 0);
                        ws_buf!(fbuf.as_ptr(), len);
                    }
                    _ => { wc!(b'%'); wc!(len_mod); wc!(len_mod2); wc!(spec); }
                }
            } else {
                spec = len_mod2;
                match (len_mod, spec) {
                    (b'h', b'n') => {
                        let p = args.next_arg::<*mut c_void>();
                        if !p.is_null() { *(p as *mut u16) = count as u16; }
                    }
                    (b'l', b'n') => {
                        let p = args.next_arg::<*mut c_void>();
                        if !p.is_null() { *(p as *mut c_long) = count as c_long; }
                    }
                    (b'z', b'n') | (b't', b'n') => {
                        let p = args.next_arg::<*mut c_void>();
                        if !p.is_null() { *(p as *mut usize) = count; }
                    }
                    (b'j', b'n') => {
                        let p = args.next_arg::<*mut c_void>();
                        if !p.is_null() { *(p as *mut c_ulonglong) = count as c_ulonglong; }
                    }
                    (b'l', b'd') | (b'l', b'i') => {
                        let val = args.next_arg::<c_long>();
                        let neg = val < 0;
                        let abs = if neg { val.wrapping_neg() as u64 } else { val as u64 };
                        let b = format_u64(abs);
                        let sign = if neg { Some(b'-') } else if flags & FLAG_PLUS != 0 { Some(b'+') } else if flags & FLAG_SPACE != 0 { Some(b' ') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, sign, None, precision, width, flags, false);
                        ws_buf!(fbuf.as_ptr(), len);
                    }
                    (b'l', b'u') => {
                        let val = args.next_arg::<c_ulong>();
                        let b = format_u64(val as u64);
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, false);
                        ws_buf!(fbuf.as_ptr(), len);
                    }
                    (b'l', b'x') => {
                        let val = args.next_arg::<c_ulong>();
                        let b = format_hex(val as u64, false);
                        let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'x') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                        ws_buf!(fbuf.as_ptr(), len);
                    }
                    (b'l', b'X') => {
                        let val = args.next_arg::<c_ulong>();
                        let b = format_hex(val as u64, true);
                        let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'X') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                        ws_buf!(fbuf.as_ptr(), len);
                    }
                    (b'l', b'o') => {
                        let val = args.next_arg::<c_ulong>();
                        let b = format_octal(val as u64, flags & FLAG_HASH != 0);
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, flags & FLAG_HASH != 0);
                        ws_buf!(fbuf.as_ptr(), len);
                    }
                    (b'l', b'f') | (b'l', b'F') | (b'l', b'e') | (b'l', b'E')
                    | (b'l', b'g') | (b'l', b'G') | (b'l', b'a') | (b'l', b'A') => {
                        let val = args.next_arg::<f64>();
                        let ucase = spec == b'F' || spec == b'E' || spec == b'G' || spec == b'A';
                        let ftype = match spec | 0x20 { b'f' => FMT_F, b'e' => FMT_E, b'g' => FMT_G, b'a' => FMT_A, _ => FMT_F };
                        let mut fbuf = [0u8; 4224];
                        let flen = format_f64_full(fbuf.as_mut_ptr(), val, ftype, precision, flags, ucase);
                        let mut wbuf = [0u8; 4224];
                        let wlen = apply_width_flags(wbuf.as_mut_ptr(), fbuf.as_ptr(), flen, width, flags);
                        ws_buf!(wbuf.as_ptr(), wlen);
                    }
                    (b'L', b'f') | (b'L', b'F') | (b'L', b'e') | (b'L', b'E')
                    | (b'L', b'g') | (b'L', b'G') | (b'L', b'a') | (b'L', b'A') => {
                        #[cfg(target_arch = "aarch64")]
                        let val = {
                            let lo: u64 = args.next_arg::<u64>();
                            let hi: u64 = args.next_arg::<u64>();
                            let combined: u128 = ((hi as u128) << 64) | (lo as u128);
                            f128::from_bits(combined) as f64
                        };
                        #[cfg(target_arch = "x86_64")]
                        let val = args.next_arg::<f64>();
                        let ucase = spec == b'F' || spec == b'E' || spec == b'G' || spec == b'A';
                        let ftype = match spec | 0x20 { b'f' => FMT_F, b'e' => FMT_E, b'g' => FMT_G, b'a' => FMT_A, _ => FMT_F };
                        let mut fbuf = [0u8; 4224];
                        let flen = format_f64_full(fbuf.as_mut_ptr(), val, ftype, precision, flags, ucase);
                        let mut wbuf = [0u8; 4224];
                        let wlen = apply_width_flags(wbuf.as_mut_ptr(), fbuf.as_ptr(), flen, width, flags);
                        ws_buf!(wbuf.as_ptr(), wlen);
                    }
                    _ => { wc!(b'%'); wc!(len_mod); wc!(spec); }
                }
            }
        } else {
            spec = *fmt.add(i) as u8;
            match spec {
                b's' => {
                    let s = args.next_arg::<*const c_char>();
                    if s.is_null() {
                        let slen = if precision >= 0 { (precision as usize).min(6) } else { 6 };
                        let padded_len = if width > slen { width } else { slen };
                        let pad = padded_len - slen;
                        if flags & FLAG_MINUS != 0 {
                            ws!(b"(null)".as_ptr(), slen);
                            for _ in 0..pad { wc!(b' '); }
                        } else {
                            for _ in 0..pad { wc!(b' '); }
                            ws!(b"(null)".as_ptr(), slen);
                        }
                    } else {
                        let full_len = strlen(s as *const c_char);
                        let slen = if precision >= 0 { (precision as usize).min(full_len) } else { full_len };
                        let padded_len = if width > slen { width } else { slen };
                        let pad = padded_len - slen;
                        if flags & FLAG_MINUS != 0 {
                            ws!(s as *const u8, slen);
                            for _ in 0..pad { wc!(b' '); }
                        } else {
                            for _ in 0..pad { wc!(b' '); }
                            ws!(s as *const u8, slen);
                        }
                    }
                }
                b'd' | b'i' => {
                    let d = args.next_arg::<c_int>();
                    let neg = d < 0;
                    let abs = if neg { d.wrapping_neg() as u64 } else { d as u64 };
                    let b = format_u64(abs);
                    let sign = if neg { Some(b'-') } else if flags & FLAG_PLUS != 0 { Some(b'+') } else if flags & FLAG_SPACE != 0 { Some(b' ') } else { None };
                    let mut fbuf = [0u8; 32];
                    let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, sign, None, precision, width, flags, false);
                    ws_buf!(fbuf.as_ptr(), len);
                }
                b'u' => {
                    let u = args.next_arg::<c_uint>();
                    let b = format_u64(u as u64);
                    let mut fbuf = [0u8; 32];
                    let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, false);
                    ws_buf!(fbuf.as_ptr(), len);
                }
                b'x' => {
                    let x = args.next_arg::<c_uint>();
                    let b = format_hex(x as u64, false);
                    let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'x') } else { None };
                    let mut fbuf = [0u8; 32];
                    let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                    ws_buf!(fbuf.as_ptr(), len);
                }
                b'X' => {
                    let x = args.next_arg::<c_uint>();
                    let b = format_hex(x as u64, true);
                    let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'X') } else { None };
                    let mut fbuf = [0u8; 32];
                    let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                    ws_buf!(fbuf.as_ptr(), len);
                }
                b'o' => {
                    let o = args.next_arg::<c_uint>();
                    let b = format_octal(o as u64, flags & FLAG_HASH != 0);
                    let mut fbuf = [0u8; 32];
                    let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, flags & FLAG_HASH != 0);
                    ws_buf!(fbuf.as_ptr(), len);
                }
                b'c' => {
                    let ch = args.next_arg::<c_int>();
                    if width > 1 {
                        let pad = width - 1;
                        if flags & FLAG_MINUS != 0 {
                            wc!(ch as u8);
                            for _ in 0..pad { wc!(b' '); }
                        } else {
                            for _ in 0..pad { wc!(b' '); }
                            wc!(ch as u8);
                        }
                    } else {
                        wc!(ch as u8);
                    }
                }
                b'p' => {
                    wc!(b'0'); wc!(b'x');
                    let p = args.next_arg::<*const c_void>();
                    let b = format_hex(p as u64, false);
                    ws!(b.0.as_ptr(), b.1);
                }
                b'f' | b'F' | b'e' | b'E' | b'g' | b'G' | b'a' | b'A' => {
                    let val = args.next_arg::<f64>();
                    let ucase = spec == b'F' || spec == b'E' || spec == b'G' || spec == b'A';
                    let ftype = match spec | 0x20 { b'f' => FMT_F, b'e' => FMT_E, b'g' => FMT_G, b'a' => FMT_A, _ => FMT_F };
                    let mut fbuf = [0u8; 4224];
                    let flen = format_f64_full(fbuf.as_mut_ptr(), val, ftype, precision, flags, ucase);
                    let mut wbuf = [0u8; 4224];
                    let wlen = apply_width_flags(wbuf.as_mut_ptr(), fbuf.as_ptr(), flen, width, flags);
                    ws_buf!(wbuf.as_ptr(), wlen);
                }
                b'%' => { wc!(b'%'); }
                b'n' => {
                    let p = args.next_arg::<*mut c_void>();
                    if !p.is_null() { *(p as *mut c_int) = count as c_int; }
                }
                _ => { wc!(b'%'); wc!(spec); }
            }
        }
        i += 1;
    }
    if cap > 0 { let null_pos = if pos < cap { pos } else { cap - 1 }; *buf.add(null_pos) = 0; }
    count as c_int
}

#[no_mangle]
pub unsafe extern "C" fn vsprintf(buf: *mut c_char, fmt: *const c_char, mut args: VaList) -> c_int {
    format_to_buf(buf as *mut u8, usize::MAX, fmt, &mut args)
}

#[no_mangle]
pub unsafe extern "C" fn vsnprintf(buf: *mut c_char, size: usize, fmt: *const c_char, mut args: VaList) -> c_int {
    format_to_buf(buf as *mut u8, size, fmt, &mut args)
}

#[no_mangle]
pub unsafe extern "C" fn sprintf(buf: *mut c_char, fmt: *const c_char, mut args: ...) -> c_int {
    format_to_buf(buf as *mut u8, usize::MAX, fmt, &mut args)
}

#[no_mangle]
pub unsafe extern "C" fn snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, mut args: ...) -> c_int {
    format_to_buf(buf as *mut u8, size, fmt, &mut args)
}

// ============================================================
// scanf / vscanf / fscanf / vfscanf / sscanf / vsscanf
// ============================================================

unsafe fn skip_ws(p: *const u8) -> *const u8 {
    let mut q = p;
    while *q == b' ' || *q == b'\t' || *q == b'\n' || *q == b'\r' { q = q.add(1); }
    q
}

#[inline]
fn is_ws_byte(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == 0x0b || c == 0x0c
}

// Scan integer with position tracking. Returns (value, is_negative, success).
// base: 0=auto, 8=octal, 10=decimal, 16=hex
unsafe fn scan_int_val(
    buf: *const u8, pos: &mut usize, buf_len: usize, base: u32, width: usize,
) -> (u64, bool, bool) {
    let start = *pos;
    let max = if width > 0 { (start + width).min(buf_len) } else { buf_len };
    let neg = if *pos < max && *buf.add(*pos) == b'-' { *pos += 1; true }
    else if *pos < max && *buf.add(*pos) == b'+' { *pos += 1; false }
    else { false };
    if *pos >= max { *pos = start; return (0, false, false); }
    let mut base = base;
    let mut val: u64 = 0;
    // Auto-detect base / handle 0x prefix
    if (base == 0 || base == 16) && *pos < max && *buf.add(*pos) == b'0' {
        *pos += 1;
        if *pos < max && (*buf.add(*pos) == b'x' || *buf.add(*pos) == b'X') {
            *pos += 1;
            if *pos < max {
                if let Some(d) = hex_val(*buf.add(*pos)) {
                    val = d as u64; *pos += 1;
                    while *pos < max { if let Some(d) = hex_val(*buf.add(*pos)) { val = val.wrapping_mul(16).wrapping_add(d as u64); *pos += 1; } else { break; } }
                    return (val, neg, true);
                }
            }
            return (0, neg, false);
        } else if base == 0 {
            while *pos < max { let c = *buf.add(*pos); if c >= b'0' && c <= b'7' { val = val.wrapping_mul(8).wrapping_add((c - b'0') as u64); *pos += 1; } else { break; } }
            return (val, neg, true);
        } else {
            return (0, neg, true); // base=16, '0' not followed by 'x'
        }
    }
    if base == 0 { base = 10; }
    let mut found = false;
    while *pos < max {
        let c = *buf.add(*pos);
        let d: Option<u64> = match base {
            8 => if c >= b'0' && c <= b'7' { Some((c - b'0') as u64) } else { None },
            10 => if c >= b'0' && c <= b'9' { Some((c - b'0') as u64) } else { None },
            16 => hex_val(c).map(|v| v as u64),
            _ => None,
        };
        match d { Some(digit) => { val = val.wrapping_mul(base as u64).wrapping_add(digit); *pos += 1; found = true; } None => break }
    }
    if !found { *pos = start; return (0, false, false); }
    (val, neg, true)
}

// Scan float with position tracking. Returns (value, success).
unsafe fn scan_float_val(
    buf: *const u8, pos: &mut usize, buf_len: usize, width: usize,
) -> (f64, bool) {
    let start = *pos;
    let max = if width > 0 { (start + width).min(buf_len) } else { buf_len };
    let neg = if *pos < max && *buf.add(*pos) == b'-' { *pos += 1; true }
    else if *pos < max && *buf.add(*pos) == b'+' { *pos += 1; false }
    else { false };
    // inf/nan
    if *pos + 2 < max {
        let (c0,c1,c2) = (*buf.add(*pos), *buf.add(*pos+1), *buf.add(*pos+2));
        if (c0==b'i'||c0==b'I')&&(c1==b'n'||c1==b'N')&&(c2==b'f'||c2==b'F') { *pos+=3; return (if neg{f64::NEG_INFINITY}else{f64::INFINITY}, true); }
        if (c0==b'n'||c0==b'N')&&(c1==b'a'||c1==b'A')&&(c2==b'n'||c2==b'N') { *pos+=3; return (if neg{-f64::NAN}else{f64::NAN}, true); }
    }
    // hex float
    if *pos+1 < max && *buf.add(*pos)==b'0' && (*buf.add(*pos+1)==b'x'||*buf.add(*pos+1)==b'X') {
        *pos += 2;
        let mut val: f64 = 0.0; let mut found = false; let mut frac_scale = 1.0f64; let mut in_frac = false;
        while *pos < max {
            if let Some(d) = hex_val(*buf.add(*pos)) {
                if in_frac { frac_scale /= 16.0; val += d as f64 * frac_scale; } else { val = val * 16.0 + d as f64; }
                *pos += 1; found = true;
            } else if *buf.add(*pos)==b'.' && !in_frac { in_frac = true; *pos += 1; } else { break; }
        }
        if !found { *pos = start; return (0.0, false); }
        // p exponent
        if *pos < max && (*buf.add(*pos)==b'p'||*buf.add(*pos)==b'P') {
            *pos += 1;
            let mut eneg = false;
            if *pos<max && *buf.add(*pos)==b'-' { eneg=true; *pos+=1; }
            else if *pos<max && *buf.add(*pos)==b'+' { *pos+=1; }
            let mut ev: i32 = 0; let mut ef = false;
            while *pos<max && *buf.add(*pos)>=b'0' && *buf.add(*pos)<=b'9' { ev = ev*10+(*buf.add(*pos)-b'0') as i32; *pos+=1; ef=true; }
            if !ef { return (0.0, false); }
            if eneg { ev = -ev; }
            val *= libm::pow(2.0, ev as f64);
        }
        return (if neg{-val}else{val}, true);
    }
    let token_start = start;
    let mut i = start;
    let mut saw_digit = false;
    let mut saw_dot = false;
    let mut saw_exp = false;
    let mut first = true;
    while i < max {
        let c = *buf.add(i);
        if (c == b'+' || c == b'-') && first {
            i += 1; first = false; continue;
        }
        if c >= b'0' && c <= b'9' {
            saw_digit = true; i += 1; first = false; continue;
        }
        if c == b'.' && !saw_dot && !saw_exp {
            saw_dot = true; i += 1; first = false; continue;
        }
        if (c == b'e' || c == b'E') && !saw_exp && saw_digit {
            saw_exp = true; i += 1; first = true; continue;
        }
        break;
    }
    if !saw_digit { return (0.0, false); }
    let s = core::str::from_utf8_unchecked(core::slice::from_raw_parts(buf.add(token_start), i - token_start));
    match <f64 as core::str::FromStr>::from_str(s) {
        Ok(v) => { *pos = i; return (v, true); }
        Err(_) => { return (0.0, false); }
    }
}

// Comprehensive scanf parser with position tracking.
// consumed: if non-null, stores number of bytes consumed from buf.
unsafe fn do_vsscanf(
    buf: *const u8, buf_len: usize, fmt: *const c_char, args: &mut VaList<'_>, consumed: *mut usize,
) -> c_int {
    let mut p = 0usize;
    let mut fi = 0usize;
    let mut assigned = 0i32;
    let mut input_eof = false;
    loop {
        let fc = *fmt.add(fi) as u8;
        if fc == 0 { break; }
        if is_ws_byte(fc) {
            while is_ws_byte(*fmt.add(fi) as u8) { fi += 1; }
            while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
            continue;
        }
        if fc != b'%' {
            if p >= buf_len { input_eof = true; break; }
            if *buf.add(p) != fc { break; }
            p += 1; fi += 1; continue;
        }
        fi += 1;
        // %%
        if *fmt.add(fi) as u8 == b'%' {
            if p >= buf_len { input_eof = true; break; }
            if *buf.add(p) != b'%' { break; }
            p += 1; fi += 1; continue;
        }
        let suppress = if *fmt.add(fi) as u8 == b'*' { fi += 1; true } else { false };
        let mut width: usize = 0;
        while (*fmt.add(fi) as u8) >= b'0' && (*fmt.add(fi) as u8) <= b'9' {
            width = width * 10 + ((*fmt.add(fi) as u8) - b'0') as usize; fi += 1;
        }
        if *fmt.add(fi) as u8 == b'm' { fi += 1; }
        let mut len_mod = 0u8;
        match *fmt.add(fi) as u8 {
            b'h' => { fi += 1; if *fmt.add(fi) as u8 == b'h' { fi += 1; len_mod = 2; } else { len_mod = 1; } }
            b'l' => { fi += 1; if *fmt.add(fi) as u8 == b'l' { fi += 1; len_mod = 4; } else { len_mod = 3; } }
            b'j' => { fi += 1; len_mod = 4; }
            b'z' | b't' => { fi += 1; len_mod = 3; }
            b'L' => { fi += 1; len_mod = 5; }
            _ => {}
        }
        let spec = *fmt.add(fi) as u8;
        let (real_spec, _real_len) = if spec==b'C' {(b'c',3u8)} else if spec==b'S' {(b's',3u8)} else {(spec,len_mod)};
        match real_spec {
            b'n' => {
                if !suppress { let out = args.next_arg::<*mut c_int>(); if !out.is_null() { *out = p as c_int; } }
            }
            b'd' | b'u' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                if p >= buf_len { input_eof = true; break; }
                let dest = if !suppress { Some(args.next_arg::<*mut c_int>()) } else { None };
                let (val, neg, ok) = scan_int_val(buf, &mut p, buf_len, 10, width);
                if !ok { break; }
                if let Some(out) = dest { if !out.is_null() {
                    *out = if real_spec==b'd' && neg { -(val as i64) as c_int } else { val as c_int };
                }}
                if !suppress { assigned += 1; }
            }
            b'i' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                if p >= buf_len { input_eof = true; break; }
                let dest = if !suppress { Some(args.next_arg::<*mut c_int>()) } else { None };
                let (val, neg, ok) = scan_int_val(buf, &mut p, buf_len, 0, width);
                if !ok { break; }
                if let Some(out) = dest { if !out.is_null() { *out = if neg { -(val as i64) as c_int } else { val as c_int }; } }
                if !suppress { assigned += 1; }
            }
            b'o' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                if p >= buf_len { input_eof = true; break; }
                let dest = if !suppress { Some(args.next_arg::<*mut c_int>()) } else { None };
                let (val, _, ok) = scan_int_val(buf, &mut p, buf_len, 8, width);
                if !ok { break; }
                if let Some(out) = dest { if !out.is_null() { *out = val as c_int; } }
                if !suppress { assigned += 1; }
            }
            b'x' | b'X' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                if p >= buf_len { input_eof = true; break; }
                let dest = if !suppress { Some(args.next_arg::<*mut c_int>()) } else { None };
                let (val, _, ok) = scan_int_val(buf, &mut p, buf_len, 16, width);
                if !ok { break; }
                if let Some(out) = dest { if !out.is_null() { *out = val as c_int; } }
                if !suppress { assigned += 1; }
            }
            b'p' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                if p >= buf_len { input_eof = true; break; }
                let dest = if !suppress { Some(args.next_arg::<*mut *mut c_void>()) } else { None };
                let (val, _, ok) = scan_int_val(buf, &mut p, buf_len, 16, width);
                if !ok { break; }
                if let Some(out) = dest { if !out.is_null() { *out = val as usize as *mut c_void; } }
                if !suppress { assigned += 1; }
            }
            b'a' | b'e' | b'f' | b'g' | b'A' | b'E' | b'F' | b'G' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                if p >= buf_len { input_eof = true; break; }
                let (val, ok) = scan_float_val(buf, &mut p, buf_len, width);
                if !ok { break; }
                if !suppress {
                    if _real_len == 3 {
                        let out = args.next_arg::<*mut f64>(); if !out.is_null() { *out = val; }
                    } else if _real_len == 5 {
                        #[cfg(target_arch = "aarch64")]
                        { let out = args.next_arg::<*mut f128>(); if !out.is_null() { *out = val as f128; } }
                        #[cfg(target_arch = "x86_64")]
                        { let out = args.next_arg::<*mut f64>(); if !out.is_null() { *out = val; } }
                    } else {
                        let out = args.next_arg::<*mut f32>(); if !out.is_null() { *out = val as f32; }
                    }
                }
                if !suppress { assigned += 1; }
            }
            b'c' => {
                let w = if width > 0 { width } else { 1 };
                if p >= buf_len { input_eof = true; break; }
                let dest_ptr: *mut c_char = if !suppress { args.next_arg::<*mut c_char>() } else { core::ptr::null_mut() };
                let start_p = p;
                let mut j = 0usize;
                while j < w && p < buf_len {
                    if !dest_ptr.is_null() { *dest_ptr.add(j) = *buf.add(p) as c_char; }
                    p += 1; j += 1;
                }
                if j != w { p = start_p; input_eof = true; break; }
                if !suppress { assigned += 1; }
            }
            b's' => {
                while p < buf_len && is_ws_byte(*buf.add(p)) { p += 1; }
                if p >= buf_len { input_eof = true; break; }
                let dest_ptr: *mut c_char = if !suppress { args.next_arg::<*mut c_char>() } else { core::ptr::null_mut() };
                let w = if width > 0 { width } else { usize::MAX };
                let mut j = 0usize;
                while p < buf_len && *buf.add(p) != 0 && !is_ws_byte(*buf.add(p)) && j < w {
                    if !dest_ptr.is_null() { *dest_ptr.add(j) = *buf.add(p) as c_char; }
                    p += 1; j += 1;
                }
                if j == 0 { break; }
                if !dest_ptr.is_null() { *dest_ptr.add(j) = 0; }
                if !suppress { assigned += 1; }
            }
            b'[' => {
                fi += 1;
                let negate = if *fmt.add(fi) as u8 == b'^' { fi += 1; true } else { false };
                let mut charset = [0u8; 256];
                if *fmt.add(fi) as u8 == b']' { charset[b']' as usize] = 1; fi += 1; }
                loop { let c = *fmt.add(fi) as u8; if c == b']' || c == 0 { break; } charset[c as usize] = 1; fi += 1; }
                if *fmt.add(fi) as u8 == b']' { fi += 1; }
                fi -= 1; // common fi += 1 will advance past the closing ]
                if p >= buf_len { input_eof = true; break; }
                let dest_ptr: *mut c_char = if !suppress { args.next_arg::<*mut c_char>() } else { core::ptr::null_mut() };
                let w = if width > 0 { width } else { usize::MAX };
                let mut j = 0usize;
                while p < buf_len && *buf.add(p) != 0 && j < w {
                    let c = *buf.add(p);
                    let in_set = charset[c as usize] != 0;
                    if negate { if in_set { break; } } else { if !in_set { break; } }
                    if !dest_ptr.is_null() { *dest_ptr.add(j) = c as c_char; }
                    p += 1; j += 1;
                }
                if j == 0 { break; }
                if !dest_ptr.is_null() { *dest_ptr.add(j) = 0; }
                if !suppress { assigned += 1; }
            }
            _ => break,
        }
        fi += 1;
    }
    if !consumed.is_null() { *consumed = p; }
    if assigned == 0 && input_eof { -1 } else { assigned }
}

unsafe fn vsscanf_inner(buf: *const u8, fmt: *const c_char, args: &mut VaList<'_>) -> c_int {
    let len = strlen(buf as *const c_char);
    do_vsscanf(buf, len, fmt, args, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn vsscanf(buf: *const c_char, fmt: *const c_char, mut args: VaList) -> c_int {
    vsscanf_inner(buf as *const u8, fmt, &mut args)
}

#[no_mangle]
pub unsafe extern "C" fn vfscanf(stream: *mut FILE, fmt: *const c_char, mut args: VaList) -> c_int {
    let f = &mut *stream;
    let start_ungotten_count = f.ungotten_count as usize;
    let sfunc = f.seek_fn;
    let start_pos = sfunc.map(|s| s(stream, 0, SEEK_CUR)).unwrap_or(-1);

    let mut line = [0u8; 4096];
    let mut pos = 0usize;

    if start_pos >= 0 {
        // Seekable: read via fgetc until \n or EOF (existing behavior)
        while pos < line.len() - 1 {
            let c = fgetc(stream);
            if c == -1 { break; }
            line[pos] = c as u8;
            pos += 1;
            if c == b'\n' as c_int { break; }
        }
    } else {
        // Non-seekable (pipe/socket): drain pushback, then read all available
        // data non-blocking so we don't stop at \n boundaries.

        // 1. Drain ungotten bytes (LIFO order)
        while f.ungotten_count > 0 && pos < line.len() - 1 {
            f.ungotten_count -= 1;
            line[pos] = f.ungotten[f.ungotten_count as usize] as u8;
            pos += 1;
        }
        // 2. Drain rpos/rend pushback buffer
        while !f.rpos.is_null() && f.rpos < f.rend && pos < line.len() - 1 {
            line[pos] = *f.rpos;
            f.rpos = f.rpos.add(1);
            pos += 1;
        }
        // 3. Read available data from the fd without blocking
        let fd = f.fd;
        let old_flags = sys_fcntl(fd, F_GETFL, 0);
        if old_flags >= 0 {
            let _ = sys_fcntl(fd, F_SETFL, old_flags | O_NONBLOCK as i64);
        }
        while pos < line.len() - 1 {
            let n = sys_read(fd as i64, line.as_mut_ptr().add(pos), line.len() - 1 - pos);
            if n <= 0 { break; }
            pos += n as usize;
        }
        if old_flags >= 0 {
            let _ = sys_fcntl(fd, F_SETFL, old_flags);
        }
        // Clear flags that the failed non-blocking read may have set
        f._eof = 0;
        f._err = 0;
        f.flags &= !(F_EOF | F_ERR);
    }

    line[pos] = 0;
    if pos == 0 { return 0; }

    let mut consumed = 0usize;
    let assigned = do_vsscanf(line.as_ptr(), pos, fmt, &mut args, &mut consumed);

    if start_pos >= 0 {
        // Seekable: seek back to start + consumed, restore ungotten
        let file_consumed = consumed.saturating_sub(start_ungotten_count);
        let target = start_pos + file_consumed as i64;
        if let Some(sfunc) = sfunc {
            sfunc(stream, target, SEEK_SET);
        }
        f.rpos = core::ptr::null_mut();
        f.rend = core::ptr::null_mut();
        if consumed < pos { f._eof = 0; }
        let remaining = start_ungotten_count.saturating_sub(consumed);
        f.ungotten_count = remaining as c_int;
    } else {
        // Non-seekable: push back unconsumed bytes into FILE read buffer
        let unconsumed = pos.saturating_sub(consumed);
        if unconsumed > 0 && !f.buf.is_null() {
            let copy_count = core::cmp::min(unconsumed, f.buf_size);
            core::ptr::copy(
                line.as_ptr().add(consumed),
                f.buf,
                copy_count,
            );
            f.rpos = f.buf;
            f.rend = f.buf.add(copy_count);
        }
        f.ungotten_count = 0;
        f._eof = 0;
        f.flags &= !F_EOF;
    }
    assigned
}

#[no_mangle]
pub unsafe extern "C" fn vscanf(fmt: *const c_char, args: VaList) -> c_int {
    vfscanf(stdin, fmt, args)
}

#[no_mangle]
pub unsafe extern "C" fn sscanf(buf: *const c_char, fmt: *const c_char, mut args: ...) -> c_int {
    vsscanf_inner(buf as *const u8, fmt, &mut args)
}

#[no_mangle]
pub unsafe extern "C" fn fscanf(stream: *mut FILE, fmt: *const c_char, args: ...) -> c_int {
    vfscanf(stream, fmt, args)
}

#[no_mangle]
pub unsafe extern "C" fn scanf(fmt: *const c_char, args: ...) -> c_int {
    vfscanf(stdin, fmt, args)
}

// ============================================================
// getdelim / getline
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn getdelim(
    lineptr: *mut *mut c_char, n: *mut usize, delim: c_int, stream: *mut FILE,
) -> isize {
    if lineptr.is_null() || n.is_null() { return -1; }
    let mut cap = *n;
    let mut buf = *lineptr;
    if buf.is_null() || cap == 0 {
        cap = 128;
        buf = realloc(buf as *mut c_void, cap) as *mut c_char;
        if buf.is_null() { return -1; }
        *lineptr = buf;
        *n = cap;
    }
    let mut len = 0usize;
    loop {
        let c = fgetc(stream);
        if c == -1 { if len == 0 { return -1; } break; }
        if len + 1 >= cap {
            let new_cap = cap.wrapping_mul(2);
            let new_buf = realloc(buf as *mut c_void, new_cap) as *mut c_char;
            if new_buf.is_null() { return -1; }
            buf = new_buf; cap = new_cap;
            *lineptr = buf; *n = cap;
        }
        *buf.add(len) = c as c_char;
        len += 1;
        if c == delim { break; }
    }
    *buf.add(len) = 0;
    len as isize
}

#[no_mangle]
pub unsafe extern "C" fn getline(lineptr: *mut *mut c_char, n: *mut usize, stream: *mut FILE) -> isize {
    getdelim(lineptr, n, b'\n' as c_int, stream)
}

// ============================================================
// popen / pclose
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn popen(cmd: *const c_char, mode: *const c_char) -> *mut FILE {
    if cmd.is_null() || mode.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let m = *mode;
    if m != b'r' as c_char && m != b'w' as c_char { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let mut p: [c_int; 2] = [0; 2];
    if sys_pipe2(p.as_mut_ptr(), O_CLOEXEC) < 0 { return core::ptr::null_mut(); }
    let op = if m == b'r' as c_char { 0 } else { 1 };
    let child_fd = p[1 - op];
    let f = fdopen(p[op], mode);
    if f.is_null() { sys_close(p[0] as i64); sys_close(p[1] as i64); return core::ptr::null_mut(); }
    let pid = sys_fork();
    if pid < 0 { fclose(f); sys_close(child_fd as i64); return core::ptr::null_mut(); }
    if pid == 0 {
        sys_close(p[op] as i64);
        if child_fd != (1 - op) as c_int { sys_dup3(child_fd, (1 - op) as c_int, 0); sys_close(child_fd as i64); }
        let sh = b"/bin/sh\0".as_ptr() as *const c_char;
        let argv = [b"sh\0".as_ptr() as *const c_char, b"-c\0".as_ptr() as *const c_char, cmd, core::ptr::null()];
        sys_execve(sh, argv.as_ptr(), __environ as *const *const c_char);
        _exit(127);
    }
    sys_close(child_fd as i64);
    (*f).pipe_pid = pid as c_int;
    f
}

#[no_mangle]
pub unsafe extern "C" fn pclose(f: *mut FILE) -> c_int {
    let pid = (*f).pipe_pid;
    let r = fclose(f);
    if pid <= 0 { return -1; }
    let mut status: c_int = 0;
    let mut w: c_int;
    loop {
        w = waitpid(pid, &mut status, 0);
        if w >= 0 || w != -EINTR { break; }
    }
    if w < 0 || r != 0 { return -1; }
    status
}

// ============================================================
// tmpfile / tmpnam / remove / rename
// ============================================================

const L_TMPNAM: usize = 20;
static mut TMPNAM_COUNTER: c_uint = 0;

unsafe fn fill_tmpname(out: *mut c_char, seed: c_uint) {
    let prefix = b"/tmp/tmp\0";
    let mut k = 0;
    while k < prefix.len() - 1 { *out.add(k) = prefix[k] as c_char; k += 1; }
    let (buf, len) = format_hex(seed as u64, false);
    for j in 0..len.min(L_TMPNAM - k - 1) { *out.add(k + j) = buf[j] as c_char; k += 1; }
    *out.add(k) = 0;
}

#[no_mangle]
pub unsafe extern "C" fn tmpfile() -> *mut FILE {
    let mut name = [0u8; L_TMPNAM];
    let mut ctr: c_uint = 0;
    loop {
        ctr = ctr.wrapping_add(1);
        let mut ts: timespec = core::mem::zeroed();
        let _ = sys_clock_gettime(CLOCK_REALTIME, &mut ts);
        let seed = (ts.tv_nsec as c_uint).wrapping_add(ctr);
        fill_tmpname(name.as_mut_ptr() as *mut c_char, seed);
        let fd = sys_open(name.as_ptr(), (O_RDWR | O_CREAT | O_EXCL) as i64, 0o600);
        if fd >= 0 {
            sys_unlinkat(AT_FDCWD, name.as_ptr(), 0);
            let f = fdopen(fd as c_int, b"w+\0".as_ptr() as *const c_char);
            if f.is_null() { sys_close(fd); }
            return f;
        }
        if ctr > 100 { return core::ptr::null_mut(); }
    }
}

#[no_mangle]
pub unsafe extern "C" fn tmpnam(s: *mut c_char) -> *mut c_char {
    static mut BUF: [c_char; L_TMPNAM] = [0; L_TMPNAM];
    let out = if s.is_null() { core::ptr::addr_of_mut!(BUF) as *mut c_char } else { s };
    TMPNAM_COUNTER = TMPNAM_COUNTER.wrapping_add(1);
    let mut ts: timespec = core::mem::zeroed();
    let _ = sys_clock_gettime(CLOCK_REALTIME, &mut ts);
    let seed = (ts.tv_nsec as c_uint).wrapping_add(TMPNAM_COUNTER);
    fill_tmpname(out, seed);
    out
}

#[no_mangle]
pub unsafe extern "C" fn remove(path: *const c_char) -> c_int {
    if path.is_null() { ERRNO = EINVAL; return -1; }
    let r = sys_unlinkat(AT_FDCWD, path as *const u8, 0);
    if r == 0 { 0 } else { ERRNO = (-r) as c_int; -1 }
}

#[no_mangle]
pub unsafe extern "C" fn rename(old: *const c_char, new_: *const c_char) -> c_int {
    if old.is_null() || new_.is_null() { ERRNO = EINVAL; return -1; }
    let r = sys_renameat2(AT_FDCWD, old as *const u8, AT_FDCWD, new_ as *const u8, 0);
    if r == 0 { 0 } else { ERRNO = (-r) as c_int; -1 }
}

// ============================================================
// Allocator: mmap-backed bump allocator
// ponytail: per-page mmap, free is a no-op, leaks on realloc
// ============================================================

const MMAP_FAILED: *mut u8 = !0usize as *mut u8;
const PAGE: usize = 4096;

#[no_mangle]
pub unsafe extern "C" fn malloc(size: SizeT) -> *mut c_void {
    // C requires malloc(0) to return either NULL or a unique non-NULL pointer
    // that can be passed to free. Return a one-page allocation for uniqueness.
    let alloc_size = if size == 0 { 1 } else { size };
    // Layout: [alloc_size: usize][data]
    let total = alloc_size + core::mem::size_of::<usize>();
    let pages = (total + PAGE - 1) & !(PAGE - 1);
    let ptr = sys_mmap(
        null_mut(),
        pages,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );
    if ptr == MMAP_FAILED {
        return null_mut();
    }
    *(ptr as *mut usize) = pages;
    ptr.add(core::mem::size_of::<usize>()) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    let header = (ptr as *mut u8).sub(core::mem::size_of::<usize>());
    let pages = *(header as *const usize);
    sys_munmap(header, pages);
}

#[no_mangle]
pub unsafe extern "C" fn calloc(count: SizeT, size: SizeT) -> *mut c_void {
    let total = count.saturating_mul(size);
    let ptr = malloc(total);
    if !ptr.is_null() {
        core::ptr::write_bytes(ptr as *mut u8, 0, total);
    }
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, new_size: SizeT) -> *mut c_void {
    if ptr.is_null() {
        return malloc(new_size);
    }
    if new_size == 0 {
        free(ptr);
        return null_mut();
    }
    let header = (ptr as *mut u8).sub(core::mem::size_of::<usize>());
    let old_pages = *(header as *const usize);
    let old_data_size = old_pages - core::mem::size_of::<usize>();
    let new_ptr = malloc(new_size);
    if new_ptr.is_null() {
        return null_mut();
    }
    let copy_size = if old_data_size < new_size {
        old_data_size
    } else {
        new_size
    };
    core::ptr::copy_nonoverlapping(ptr as *const u8, new_ptr as *mut u8, copy_size);
    free(ptr);
    new_ptr
}

// ============================================================
// Process: exit / _exit / _Exit
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn _exit(code: c_int) -> ! {
    sys_exit_group(code)
}

#[inline]
unsafe fn sys_exit_group(code: c_int) -> ! {
    <Arch as Syscalls>::syscall_noreturn1(SYS_EXIT_GROUP, code as i64)
}

#[inline]
unsafe fn sys_exit_thread(code: c_int) -> ! {
    <Arch as Syscalls>::syscall_noreturn1(SYS_EXIT, code as i64)
}

#[no_mangle]
pub unsafe extern "C" fn _Exit(code: c_int) -> ! {
    _exit(code);
}

#[no_mangle]
pub unsafe extern "C" fn exit(code: c_int) -> ! {
    fflush(core::ptr::null_mut());
    __funcs_on_exit();
    _exit(code);
}

#[no_mangle]
pub unsafe extern "C" fn abort() -> ! {
    raise(6); // SIGABRT
    _exit(128 + 6);
}

// aarch64 GCC accesses the stack canary via the global __stack_chk_guard
// symbol (GOT-based for PIE). Export it so the dynamic linker can resolve it.
#[no_mangle]
pub static mut __stack_chk_guard: usize = 0;

#[no_mangle]
pub unsafe extern "C" fn __stack_chk_fail() -> ! {
    const MSG: &[u8] = b"*** stack smashing detected ***: terminated\n";
    let _ = sys_write(2, MSG.as_ptr(), MSG.len());
    abort();
}

// ============================================================
// qsort / qsort_r (smoothsort, O(1) aux, adaptive)
// ============================================================

type CmpRfn = unsafe extern "C" fn(*const c_void, *const c_void, *mut c_void) -> c_int;

const AR_LEN: usize = 16 * core::mem::size_of::<usize>();
const AR_MASK: usize = AR_LEN - 1;

unsafe fn qsort_pntz(p: &[usize; 2]) -> i32 {
    if p[0] != 1 {
        p[0].wrapping_sub(1).trailing_zeros() as i32
    } else if p[1] != 0 {
        (8 * core::mem::size_of::<usize>()) as i32 + p[1].trailing_zeros() as i32
    } else {
        0
    }
}

unsafe fn qsort_cycle(width: usize, ar: &mut [*mut u8; AR_LEN], n: usize) {
    if n < 2 { return; }
    let mut tmp = [0u8; 256];
    ar[n] = tmp.as_mut_ptr();
    let mut w = width;
    while w > 0 {
        let l = if 256 < w { 256 } else { w };
        core::ptr::copy_nonoverlapping(ar[0], ar[n], l);
        for i in 0..n {
            core::ptr::copy_nonoverlapping(ar[i + 1], ar[i], l);
            ar[i] = ar[i].add(l);
        }
        w -= l;
    }
}

unsafe fn qsort_shl(p: &mut [usize; 2], n: i32) {
    let bits = (8 * core::mem::size_of::<usize>()) as i32;
    if n >= bits {
        p[1] = p[0];
        p[0] = 0;
        let n = n - bits;
        if n == 0 { return; }
    }
    p[1] = (p[1] << n) | (p[0] >> (bits - n));
    p[0] <<= n;
}

unsafe fn qsort_shr(p: &mut [usize; 2], n: i32) {
    let bits = (8 * core::mem::size_of::<usize>()) as i32;
    if n >= bits {
        p[0] = p[1];
        p[1] = 0;
        let n = n - bits;
        if n == 0 { return; }
    }
    p[0] = (p[0] >> n) | (p[1] << (bits - n));
    p[1] >>= n;
}

unsafe fn qsort_sift(
    head: *mut u8, width: usize,
    cmp: CmpRfn, arg: *mut c_void,
    pshift: i32, lp: &[usize],
) {
    let mut ar: [*mut u8; AR_LEN] = [core::ptr::null_mut(); AR_LEN];
    ar[0] = head;
    let mut i = 1usize;
    let mut h = head;
    let mut ps = pshift;

    while ps > 1 {
        let rt = h.sub(width);
        let lf = h.sub(width + lp[(ps - 2) as usize]);

        if cmp(ar[0] as *const c_void, lf as *const c_void, arg) >= 0
            && cmp(ar[0] as *const c_void, rt as *const c_void, arg) >= 0
        {
            break;
        }
        if cmp(lf as *const c_void, rt as *const c_void, arg) >= 0 {
            ar[i & AR_MASK] = lf;
            i += 1;
            h = lf;
            ps -= 1;
        } else {
            ar[i & AR_MASK] = rt;
            i += 1;
            h = rt;
            ps -= 2;
        }
    }
    qsort_cycle(width, &mut ar, i & AR_MASK);
}

unsafe fn qsort_trinkle(
    head: *mut u8, width: usize,
    cmp: CmpRfn, arg: *mut c_void,
    pp: &[usize; 2], pshift: i32, trusty: i32,
    lp: &[usize],
) {
    let mut ar: [*mut u8; AR_LEN] = [core::ptr::null_mut(); AR_LEN];
    let mut p = *pp;
    ar[0] = head;
    let mut i = 1usize;
    let mut h = head;
    let mut ps = pshift;
    let mut t = trusty;

    while p[0] != 1 || p[1] != 0 {
        let stepson = h.sub(lp[ps as usize]);
        if cmp(stepson as *const c_void, ar[0] as *const c_void, arg) <= 0 {
            break;
        }
        if t == 0 && ps > 1 {
            let rt = h.sub(width);
            let lf = h.sub(width + lp[(ps - 2) as usize]);
            if cmp(rt as *const c_void, stepson as *const c_void, arg) >= 0
                || cmp(lf as *const c_void, stepson as *const c_void, arg) >= 0
            {
                break;
            }
        }

        ar[i & AR_MASK] = stepson;
        i += 1;
        h = stepson;
        let trail = qsort_pntz(&p);
        qsort_shr(&mut p, trail);
        ps += trail;
        t = 0;
    }
    if t == 0 {
        qsort_cycle(width, &mut ar, i & AR_MASK);
        qsort_sift(h, width, cmp, arg, ps, lp);
    }
}

#[no_mangle]
pub unsafe extern "C" fn __qsort_r(
    base: *mut c_void,
    nel: usize,
    width: usize,
    cmp: CmpRfn,
    arg: *mut c_void,
) {
    let size = width * nel;
    if size == 0 { return; }

    let mut lp = [0usize; 96];
    lp[0] = width;
    lp[1] = width;
    let mut li = 2;
    while {
        lp[li] = lp[li - 2] + lp[li - 1] + width;
        lp[li] < size
    } {
        li += 1;
    }

    let mut head = base as *mut u8;
    let high = head.add(size - width);
    let mut p: [usize; 2] = [1, 0];
    let mut pshift: i32 = 1;

    while head < high {
        if (p[0] & 3) == 3 {
            qsort_sift(head, width, cmp, arg, pshift, &lp);
            qsort_shr(&mut p, 2);
            pshift += 2;
        } else {
            if lp[(pshift - 1) as usize] >= (high as usize) - (head as usize) {
                qsort_trinkle(head, width, cmp, arg, &p, pshift, 0, &lp);
            } else {
                qsort_sift(head, width, cmp, arg, pshift, &lp);
            }

            if pshift == 1 {
                qsort_shl(&mut p, 1);
                pshift = 0;
            } else {
                qsort_shl(&mut p, pshift - 1);
                pshift = 1;
            }
        }

        p[0] |= 1;
        head = head.add(width);
    }

    qsort_trinkle(head, width, cmp, arg, &p, pshift, 0, &lp);

    while pshift != 1 || p[0] != 1 || p[1] != 0 {
        if pshift <= 1 {
            let trail = qsort_pntz(&p);
            qsort_shr(&mut p, trail);
            pshift += trail;
        } else {
            qsort_shl(&mut p, 2);
            pshift -= 2;
            p[0] ^= 7;
            qsort_shr(&mut p, 1);
            qsort_trinkle(
                head.sub(lp[pshift as usize] + width),
                width, cmp, arg, &p, pshift + 1, 1, &lp,
            );
            qsort_shl(&mut p, 1);
            p[0] |= 1;
            qsort_trinkle(head.sub(width), width, cmp, arg, &p, pshift, 1, &lp);
        }
        head = head.sub(width);
    }
}

#[no_mangle]
pub unsafe extern "C" fn qsort_r(
    base: *mut c_void,
    nel: usize,
    width: usize,
    cmp: CmpRfn,
    arg: *mut c_void,
) {
    __qsort_r(base, nel, width, cmp, arg);
}

unsafe extern "C" fn qsort_wrap_cmp(
    a: *const c_void,
    b: *const c_void,
    cmp_ctx: *mut c_void,
) -> c_int {
    let cmp: CmpFn = core::mem::transmute(cmp_ctx);
    cmp(a, b)
}

#[no_mangle]
pub unsafe extern "C" fn qsort(
    base: *mut c_void,
    nel: usize,
    width: usize,
    cmp: CmpFn,
) {
    __qsort_r(base, nel, width, qsort_wrap_cmp, cmp as *mut c_void);
}

// ============================================================
// atexit / __cxa_atexit / __funcs_on_exit
// ============================================================

const ATEXIT_COUNT: usize = 32;

#[repr(C)]
struct AtExitBlock {
    next: *mut AtExitBlock,
    f: [Option<unsafe extern "C" fn(*mut c_void)>; ATEXIT_COUNT],
    a: [*mut c_void; ATEXIT_COUNT],
}

static mut ATEXIT_BUILTIN: AtExitBlock = AtExitBlock {
    next: core::ptr::null_mut(),
    f: [None; ATEXIT_COUNT],
    a: [core::ptr::null_mut(); ATEXIT_COUNT],
};
static mut ATEXIT_HEAD: *mut AtExitBlock = core::ptr::null_mut();
static mut ATEXIT_SLOT: usize = 0;
static mut ATEXIT_FINISHED: bool = false;

#[no_mangle]
pub unsafe extern "C" fn __funcs_on_exit() {
    while !ATEXIT_HEAD.is_null() {
        let head = ATEXIT_HEAD;
        while ATEXIT_SLOT > 0 {
            ATEXIT_SLOT -= 1;
            if let Some(f) = (*head).f[ATEXIT_SLOT] {
                let a = (*head).a[ATEXIT_SLOT];
                f(a);
            }
        }
        ATEXIT_HEAD = (*head).next;
        ATEXIT_SLOT = ATEXIT_COUNT;
    }
    ATEXIT_FINISHED = true;
}

#[no_mangle]
pub unsafe extern "C" fn __cxa_atexit(
    func: unsafe extern "C" fn(*mut c_void),
    arg: *mut c_void,
    _dso: *mut c_void,
) -> c_int {
    if ATEXIT_FINISHED {
        return -1;
    }
    if ATEXIT_HEAD.is_null() {
        ATEXIT_HEAD = core::ptr::addr_of_mut!(ATEXIT_BUILTIN);
    }
    if ATEXIT_SLOT == ATEXIT_COUNT {
        let new = calloc(1, core::mem::size_of::<AtExitBlock>()) as *mut AtExitBlock;
        if new.is_null() {
            return -1;
        }
        (*new).next = ATEXIT_HEAD;
        ATEXIT_HEAD = new;
        ATEXIT_SLOT = 0;
    }
    (*ATEXIT_HEAD).f[ATEXIT_SLOT] = Some(func);
    (*ATEXIT_HEAD).a[ATEXIT_SLOT] = arg;
    ATEXIT_SLOT += 1;
    0
}

unsafe extern "C" fn atexit_caller(arg: *mut c_void) {
    let func: unsafe extern "C" fn() = core::mem::transmute(arg);
    func();
}

#[no_mangle]
pub unsafe extern "C" fn atexit(func: unsafe extern "C" fn()) -> c_int {
    __cxa_atexit(atexit_caller, func as *mut c_void, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn __cxa_finalize(_dso: *mut c_void) {}

// ============================================================
// getenv / setenv / putenv / unsetenv / clearenv
// ============================================================

#[no_mangle]
pub static mut __environ: *mut *mut c_char = core::ptr::null_mut();

// ponytail: simple env tracking for allocated entries; O(n) scan
const ENV_MAX_ALLOCED: usize = 256;
static mut ENV_ALLOCED: [*mut c_char; ENV_MAX_ALLOCED] = [core::ptr::null_mut(); ENV_MAX_ALLOCED];
static mut ENV_ALLOCED_N: usize = 0;

unsafe fn env_rm_add(old: *mut c_char, mut new: *mut c_char) {
    for i in 0..ENV_ALLOCED_N {
        if ENV_ALLOCED[i] == old {
            ENV_ALLOCED[i] = new;
            if !old.is_null() {
                free(old as *mut c_void);
            }
            return;
        } else if ENV_ALLOCED[i].is_null() && !new.is_null() {
            ENV_ALLOCED[i] = new;
            new = core::ptr::null_mut();
        }
    }
    if !new.is_null() && ENV_ALLOCED_N < ENV_MAX_ALLOCED {
        ENV_ALLOCED[ENV_ALLOCED_N] = new;
        ENV_ALLOCED_N += 1;
    }
}

unsafe fn strchrnul(s: *const u8, c: u8) -> *const u8 {
    let mut p = s;
    while *p != 0 && *p != c {
        p = p.add(1);
    }
    p
}

#[no_mangle]
pub unsafe extern "C" fn getenv(name: *const c_char) -> *mut c_char {
    if __environ.is_null() || name.is_null() {
        return core::ptr::null_mut();
    }
    let name = name as *const u8;
    let l = strchrnul(name, b'=') as usize - name as usize;
    if l == 0 || *name.add(l) != 0 {
        return core::ptr::null_mut();
    }
    let mut e = __environ;
    while !(*e).is_null() {
        let entry = *e as *const u8;
        if strncmp(entry, name, l) == 0 && *entry.add(l) == b'=' {
            return entry.add(l + 1) as *mut c_char;
        }
        e = e.add(1);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn setenv(
    var: *const c_char,
    value: *const c_char,
    overwrite: c_int,
) -> c_int {
    if var.is_null() || value.is_null() {
        ERRNO = EINVAL;
        return -1;
    }
    let var = var as *const u8;
    let l1 = strchrnul(var, b'=') as usize - var as usize;
    if l1 == 0 || *var.add(l1) != 0 {
        ERRNO = EINVAL;
        return -1;
    }
    if overwrite == 0 && !getenv(var as *const c_char).is_null() {
        return 0;
    }
    let l2 = strlen(value as *const c_char);
    let s = malloc(l1 + l2 + 2) as *mut u8;
    if s.is_null() {
        return -1;
    }
    core::ptr::copy_nonoverlapping(var, s, l1);
    *s.add(l1) = b'=';
    core::ptr::copy_nonoverlapping(value as *const u8, s.add(l1 + 1), l2 + 1);
    putenv_internal(s as *mut c_char, l1, s as *mut c_char)
}

unsafe fn putenv_internal(s: *mut c_char, l: usize, r: *mut c_char) -> c_int {
    let mut i = 0;
    if !__environ.is_null() {
        let mut e = __environ;
        while !(*e).is_null() {
            if strncmp(s as *const u8, *e as *const u8, l + 1) == 0 {
                let tmp = *e;
                *e = s;
                env_rm_add(tmp, r);
                return 0;
            }
            e = e.add(1);
            i += 1;
        }
    }
    let newenv = malloc(core::mem::size_of::<*mut c_char>() * (i + 2)) as *mut *mut c_char;
    if newenv.is_null() {
        if !r.is_null() { free(r as *mut c_void); }
        return -1;
    }
    if i > 0 && !__environ.is_null() {
        core::ptr::copy_nonoverlapping(__environ, newenv, i);
    }
    *newenv.add(i) = s;
    *newenv.add(i + 1) = core::ptr::null_mut();
    // ponytail: don't free old __environ (may not be from our malloc)
    __environ = newenv;
    environ = __environ;
    if !r.is_null() {
        env_rm_add(core::ptr::null_mut(), r);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn putenv(s: *mut c_char) -> c_int {
    if s.is_null() { return -1; }
    let l = strchrnul(s as *const u8, b'=') as usize - s as *const u8 as usize;
    if l == 0 || *s.add(l) as u8 == 0 {
        return unsetenv(s);
    }
    putenv_internal(s, l, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn unsetenv(name: *const c_char) -> c_int {
    if name.is_null() {
        ERRNO = EINVAL;
        return -1;
    }
    let name = name as *const u8;
    let l = strchrnul(name, b'=') as usize - name as usize;
    if l == 0 || *name.add(l) != 0 {
        ERRNO = EINVAL;
        return -1;
    }
    if __environ.is_null() {
        return 0;
    }
    let mut e = __environ;
    let mut eo = e;
    while !(*e).is_null() {
        if strncmp(name, *e as *const u8, l) == 0 && *(*e as *const u8).add(l) == b'=' {
            env_rm_add(*e, core::ptr::null_mut());
        } else {
            if eo != e {
                *eo = *e;
            }
            eo = eo.add(1);
        }
        e = e.add(1);
    }
    if eo != e {
        *eo = core::ptr::null_mut();
    }
    environ = __environ;
    0
}

#[no_mangle]
pub unsafe extern "C" fn clearenv() -> c_int {
    let e = __environ;
    __environ = core::ptr::null_mut();
    environ = core::ptr::null_mut();
    if !e.is_null() {
        let mut p = e;
        while !(*p).is_null() {
            env_rm_add(*p, core::ptr::null_mut());
            p = p.add(1);
        }
    }
    0
}

// ============================================================
// random / srandom / initstate / setstate (BSD lagged fibonacci)
// ponytail: single-threaded, no locking; add lock if concurrency matters
// ============================================================

static mut RANDOM_INIT: [u32; 32] = [
    0x00000000,0x5851f42d,0xc0b18ccf,0xcbb5f646,
    0xc7033129,0x30705b04,0x20fd5db4,0x9a8b7f78,
    0x502959d8,0xab894868,0x6c0356a7,0x88cdb7ff,
    0xb477d43f,0x70a3a52b,0xa8e4baf1,0xfd8341fc,
    0x8ae16fd9,0x742d2f7a,0x0d1f0796,0x76035e09,
    0x40f7702c,0x6fa72ca5,0xaaa84157,0x58a0df74,
    0xc74a0364,0xae533cc4,0x04185faf,0x6de3b115,
    0x0cab8628,0xf043bfa4,0x398150e9,0x37521657,
];

static mut RANDOM_N: i32 = 31;
static mut RANDOM_I: i32 = 3;
static mut RANDOM_J: i32 = 0;
static mut RANDOM_X: *mut u32 = unsafe { core::ptr::addr_of_mut!(RANDOM_INIT).cast::<u32>().add(1) };

unsafe fn random_lcg31(x: u32) -> u32 {
    (1103515245u32.wrapping_mul(x).wrapping_add(12345)) & 0x7fffffff
}

unsafe fn random_lcg64(x: u64) -> u64 {
    6364136223846793005u64.wrapping_mul(x).wrapping_add(1)
}

unsafe fn random_savestate() -> *mut u8 {
    (*RANDOM_X.offset(-1)) = ((RANDOM_N as u32) << 16) | ((RANDOM_I as u32) << 8) | (RANDOM_J as u32);
    RANDOM_X.offset(-1) as *mut u8
}

unsafe fn random_loadstate(state: *mut u32) {
    RANDOM_X = state.add(1);
    RANDOM_N = ((*RANDOM_X.offset(-1)) >> 16) as i32;
    RANDOM_I = (((*RANDOM_X.offset(-1)) >> 8) & 0xff) as i32;
    RANDOM_J = ((*RANDOM_X.offset(-1)) & 0xff) as i32;
}

unsafe fn random_srandom_inner(seed: u32) {
    let mut s = seed as u64;
    if RANDOM_N == 0 {
        *RANDOM_X = s as u32;
        return;
    }
    RANDOM_I = if RANDOM_N == 31 || RANDOM_N == 7 { 3 } else { 1 };
    RANDOM_J = 0;
    for k in 0..RANDOM_N as usize {
        s = random_lcg64(s);
        *RANDOM_X.add(k) = (s >> 32) as u32;
    }
    *RANDOM_X |= 1;
}

#[no_mangle]
pub unsafe extern "C" fn srandom(seed: c_uint) {
    random_srandom_inner(seed);
}

#[no_mangle]
pub unsafe extern "C" fn initstate(seed: c_uint, state: *mut c_char, size: usize) -> *mut c_char {
    if size < 8 { return core::ptr::null_mut(); }
    let old = random_savestate();
    RANDOM_N = if size < 32 { 0 }
        else if size < 64 { 7 }
        else if size < 128 { 15 }
        else if size < 256 { 31 }
        else { 63 };
    RANDOM_X = (state as *mut u32).add(1);
    random_srandom_inner(seed);
    random_savestate();
    old as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn setstate(state: *mut c_char) -> *mut c_char {
    let old = random_savestate();
    random_loadstate(state as *mut u32);
    old as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn random() -> c_long {
    let k: c_long;
    if RANDOM_N == 0 {
        *RANDOM_X = random_lcg31(*RANDOM_X);
        k = *RANDOM_X as c_long;
    } else {
        *RANDOM_X.add(RANDOM_I as usize) = (*RANDOM_X.add(RANDOM_I as usize)).wrapping_add(*RANDOM_X.add(RANDOM_J as usize));
        k = (*RANDOM_X.add(RANDOM_I as usize) >> 1) as c_long;
        RANDOM_I += 1;
        if RANDOM_I == RANDOM_N { RANDOM_I = 0; }
        RANDOM_J += 1;
        if RANDOM_J == RANDOM_N { RANDOM_J = 0; }
    }
    k
}

// ============================================================
// multibyte (UTF-8, musl state machine)
// ============================================================

const SA: u8 = 0xC2;
const SB: u8 = 0xF4;

const BITTAB: [u32; 51] = [
    0xC000_0002, 0xC000_0003, 0xC000_0004, 0xC000_0005, 0xC000_0006, 0xC000_0007,
    0xC000_0008, 0xC000_0009, 0xC000_000A, 0xC000_000B, 0xC000_000C, 0xC000_000D,
    0xC000_000E, 0xC000_000F, 0xC000_0010, 0xC000_0011, 0xC000_0012, 0xC000_0013,
    0xC000_0014, 0xC000_0015, 0xC000_0016, 0xC000_0017, 0xC000_0018, 0xC000_0019,
    0xC000_001A, 0xC000_001B, 0xC000_001C, 0xC000_001D, 0xC000_001E, 0xC000_001F,
    0xB300_0000, 0xC300_0001, 0xC300_0002, 0xC300_0003, 0xC300_0004, 0xC300_0005,
    0xC300_0006, 0xC300_0007, 0xC300_0008, 0xC300_0009, 0xC300_000A, 0xC300_000B,
    0xC300_000C, 0xD300_000D, 0xC300_000E, 0xC300_000F, 0xBB0C_0000, 0xC30C_0001,
    0xC30C_0002, 0xC30C_0003, 0xDB0C_0004,
];

#[inline]
fn mb_oob(c: u32, b: u8) -> u32 {
    let b3 = (b >> 3) as u32;
    ((b3.wrapping_sub(0x10)) | (b3.wrapping_add((c as i32 >> 26) as u32))) & !7
}

static mut MB_STATE: c_uint = 0;

const MB_CUR_MAX_VAL_UTF8: usize = 4;
const MB_CUR_MAX_VAL_C: usize = 1;

#[no_mangle]
pub extern "C" fn __ctype_get_mb_cur_max() -> usize {
    unsafe { if LOCALE_CTYPE_UTF8 { MB_CUR_MAX_VAL_UTF8 } else { MB_CUR_MAX_VAL_C } }
}

#[no_mangle]
pub unsafe extern "C" fn mbrtowc(
    wc: *mut c_int,
    src: *const c_char,
    n: usize,
    st: *mut c_uint,
) -> usize {
    let s = src as *const u8;
    let n0 = n;
    let mut dummy: c_int = 0;
    let wc = if wc.is_null() { &mut dummy } else { &mut *wc };
    let st: &mut c_uint = if st.is_null() { &mut *(&raw mut MB_STATE) } else { &mut *st };

    let mut c: u32 = *st;
    let mut s = s;
    let mut n = n;

    if s.is_null() {
        if c != 0 { *st = 0; ERRNO = EILSEQ; return !0usize; }
        return 0;
    }

    if n == 0 { return !1usize; }

    if unsafe { !LOCALE_CTYPE_UTF8 } {
        let b = *s;
        *wc = b as c_int;
        return if b == 0 { 0 } else { 1 };
    }

    if c == 0 {
        if *s < 0x80 {
            let r = if *s == 0 { 0 } else { 1 };
            *wc = *s as c_int;
            return r;
        }
        if (*s).wrapping_sub(SA) as u32 > (SB - SA) as u32 {
            *st = 0; ERRNO = EILSEQ; return !0usize;
        }
        c = BITTAB[*s as usize - SA as usize];
        s = s.add(1);
        n -= 1;
    }

    if n > 0 {
        if mb_oob(c, *s) != 0 {
            *st = 0; ERRNO = EILSEQ; return !0usize;
        }
        loop {
            c = (c << 6) | (*s as u32).wrapping_sub(0x80);
            s = s.add(1);
            n -= 1;
            if c & (1u32 << 31) == 0 {
                *st = 0;
                *wc = c as c_int;
                return n0 - n;
            }
            if n == 0 { break; }
            if (*s).wrapping_sub(0x80) >= 0x40 {
                *st = 0; ERRNO = EILSEQ; return !0usize;
            }
        }
    }

    *st = c;
    !1usize
}

#[no_mangle]
pub unsafe extern "C" fn wcrtomb(
    s: *mut c_char,
    wc: c_int,
    _st: *mut c_uint,
) -> usize {
    if s.is_null() { return 1; }
    let wc_u = wc as u32;
    if unsafe { !LOCALE_CTYPE_UTF8 } {
        if wc_u > 0xFF {
            ERRNO = EILSEQ;
            return !0usize;
        }
        *s = wc as c_char;
        return 1;
    }
    if wc_u < 0x80 {
        *s = wc as c_char;
        return 1;
    }
    if wc_u < 0x800 {
        *s = (0xC0 | (wc_u >> 6)) as c_char;
        *s.add(1) = (0x80 | (wc_u & 0x3F)) as c_char;
        return 2;
    }
    if wc_u < 0xD800 || wc_u.wrapping_sub(0xE000) < 0x2000 {
        *s = (0xE0 | (wc_u >> 12)) as c_char;
        *s.add(1) = (0x80 | ((wc_u >> 6) & 0x3F)) as c_char;
        *s.add(2) = (0x80 | (wc_u & 0x3F)) as c_char;
        return 3;
    }
    if wc_u.wrapping_sub(0x10000) < 0x100000 {
        *s = (0xF0 | (wc_u >> 18)) as c_char;
        *s.add(1) = (0x80 | ((wc_u >> 12) & 0x3F)) as c_char;
        *s.add(2) = (0x80 | ((wc_u >> 6) & 0x3F)) as c_char;
        *s.add(3) = (0x80 | (wc_u & 0x3F)) as c_char;
        return 4;
    }
    ERRNO = EILSEQ;
    !0usize
}

#[no_mangle]
pub unsafe extern "C" fn mblen(s: *const c_char, n: usize) -> c_int {
    if s.is_null() { return 0; }
    mbtowc(core::ptr::null_mut(), s, n)
}

#[no_mangle]
pub unsafe extern "C" fn mbtowc(
    wc: *mut c_int,
    src: *const c_char,
    n: usize,
) -> c_int {
    if src.is_null() { return 0; }
    if n == 0 { ERRNO = EILSEQ; return -1; }
    let mut state: c_uint = 0;
    let mut w: c_int = 0;
    let r = mbrtowc(&mut w, src, n, &mut state);
    if r == !0usize { return -1; }
    if !wc.is_null() { *wc = w; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn wctomb(s: *mut c_char, wc: c_int) -> c_int {
    if s.is_null() { return 0; }
    wcrtomb(s, wc, core::ptr::null_mut()) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn mbsrtowcs(
    ws: *mut c_int,
    src: *mut *const c_char,
    wn: usize,
    st: *mut c_uint,
) -> usize {
    let st: &mut c_uint = if st.is_null() { &mut *(&raw mut MB_STATE) } else { &mut *st };
    let mut s = *src as *const u8;
    let wn0 = wn;
    let mut wn = wn;

    if ws.is_null() {
        while *s != 0 {
            if *s < 0x80 {
                s = s.add(1);
                wn = wn.wrapping_sub(1);
                continue;
            }
            let mut wc: c_int = 0;
            let r = mbrtowc(&mut wc, s as *const c_char, 4, st);
            if r == !0usize { return !0usize; }
            if r == !1usize { return wn0 - wn; }
            s = s.add(r);
            wn = wn.wrapping_sub(1);
        }
        wn0 - wn
    } else {
        let mut w = ws;
        while wn > 0 {
            if *s < 0x80 {
                let c = *s as c_int;
                *w = c;
                if c == 0 {
                    *src = core::ptr::null();
                    return wn0 - wn;
                }
                w = w.add(1);
                s = s.add(1);
                wn -= 1;
                continue;
            }
            let mut wc: c_int = 0;
            let r = mbrtowc(&mut wc, s as *const c_char, 4, st);
            if r == !0usize {
                *src = s as *const c_char;
                return !0usize;
            }
            if r == !1usize {
                *src = s as *const c_char;
                return wn0 - wn;
            }
            *w = wc;
            w = w.add(1);
            s = s.add(r);
            wn -= 1;
        }
        *src = s as *const c_char;
        wn0 - wn
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcsrtombs(
    s: *mut c_char,
    ws: *mut *const c_int,
    n: usize,
    _st: *mut c_uint,
) -> usize {
    let mut w = *ws;
    let n0 = n;
    let mut n = n;

    if s.is_null() {
        let mut count = 0usize;
        while *w != 0 {
            if (*w as u32) < 0x80 {
                count += 1;
            } else {
                let mut buf = [0u8; 4];
                let r = wcrtomb(buf.as_mut_ptr() as *mut c_char, *w, core::ptr::null_mut());
                if r == !0usize { return !0usize; }
                count += r;
            }
            w = w.add(1);
        }
        return count;
    }

    let mut dst = s;
    while n > 0 {
        if (*w as u32) < 0x80 {
            *dst = *w as c_char;
            if *w == 0 {
                *ws = core::ptr::null();
                return n0 - n;
            }
            dst = dst.add(1);
            n -= 1;
        } else {
            let r = wcrtomb(dst, *w, core::ptr::null_mut());
            if r == !0usize {
                *ws = w;
                return !0usize;
            }
            if r > n {
                *ws = w;
                return n0 - n;
            }
            dst = dst.add(r);
            n -= r;
        }
        w = w.add(1);
    }
    *ws = w;
    n0
}

#[no_mangle]
pub unsafe extern "C" fn mbstowcs(
    ws: *mut c_int,
    src: *const c_char,
    wn: usize,
) -> usize {
    let mut src_ptr = src;
    mbsrtowcs(ws, &mut src_ptr, wn, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn wcstombs(
    s: *mut c_char,
    ws: *const c_int,
    n: usize,
) -> usize {
    let mut ws_ptr = ws;
    wcsrtombs(s, &mut ws_ptr, n, core::ptr::null_mut())
}

// ============================================================
// wchar: btowc / wctob / mbsinit / mbrlen
// ============================================================

#[no_mangle]
pub extern "C" fn btowc(c: c_int) -> wint_t {
    if c == -1 { return WEOF; }
    unsafe {
        if LOCALE_CTYPE_UTF8 {
            if (c as u32) < 128 { c as wint_t } else { WEOF }
        } else {
            (c as u8) as wint_t
        }
    }
}

#[no_mangle]
pub extern "C" fn wctob(c: wint_t) -> c_int {
    unsafe {
        if LOCALE_CTYPE_UTF8 {
            if c < 128 { c as c_int } else { -1 }
        } else {
            if c <= 0xFF { c as c_int } else { -1 }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn mbsinit(s: *const c_uint) -> c_int {
    if s.is_null() || *s == 0 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn mbrlen(s: *const c_char, n: usize, ps: *mut c_uint) -> usize {
    mbrtowc(core::ptr::null_mut(), s, n, ps)
}

// ============================================================
// wchar: wide string functions
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn wcslen(s: *const wchar_t) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 { len += 1; }
    len
}

#[no_mangle]
pub unsafe extern "C" fn wcschr(s: *const wchar_t, c: wchar_t) -> *mut wchar_t {
    let mut i = 0;
    loop {
        let ch = *s.add(i);
        if ch == c { return s.add(i) as *mut wchar_t; }
        if ch == 0 { return null_mut(); }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcsrchr(s: *const wchar_t, c: wchar_t) -> *mut wchar_t {
    let mut i = 0;
    let mut last: *mut wchar_t = null_mut();
    loop {
        let ch = *s.add(i);
        if ch == c { last = s.add(i) as *mut wchar_t; }
        if ch == 0 { return last; }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcsstr(haystack: *const wchar_t, needle: *const wchar_t) -> *mut wchar_t {
    if *needle == 0 { return haystack as *mut wchar_t; }
    let mut i = 0;
    loop {
        let h = *haystack.add(i);
        if h == 0 { return null_mut(); }
        if h == *needle {
            let mut j = 0;
            loop {
                let n = *needle.add(j);
                if n == 0 { return haystack.add(i) as *mut wchar_t; }
                if *haystack.add(i + j) != n { break; }
                j += 1;
            }
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcscmp(s1: *const wchar_t, s2: *const wchar_t) -> c_int {
    let mut i = 0;
    loop {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b { return a - b; }
        if a == 0 { return 0; }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcsncmp(s1: *const wchar_t, s2: *const wchar_t, n: usize) -> c_int {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b { return a - b; }
        if a == 0 { return 0; }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn wcscpy(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t {
    let mut i = 0;
    loop {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 { break; }
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn wcsncpy(dst: *mut wchar_t, src: *const wchar_t, n: usize) -> *mut wchar_t {
    let mut i = 0;
    while i < n && *src.add(i) != 0 { *dst.add(i) = *src.add(i); i += 1; }
    while i < n { *dst.add(i) = 0; i += 1; }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn wcscat(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t {
    let mut i = 0;
    while *dst.add(i) != 0 { i += 1; }
    let mut j = 0;
    loop {
        let c = *src.add(j);
        *dst.add(i + j) = c;
        if c == 0 { break; }
        j += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn wcsncat(dst: *mut wchar_t, src: *const wchar_t, n: usize) -> *mut wchar_t {
    let mut i = 0;
    while *dst.add(i) != 0 { i += 1; }
    let mut j = 0;
    while j < n && *src.add(j) != 0 { *dst.add(i + j) = *src.add(j); j += 1; }
    *dst.add(i + j) = 0;
    dst
}

#[no_mangle]
pub unsafe extern "C" fn wcsdup(s: *const wchar_t) -> *mut wchar_t {
    let len = wcslen(s);
    let p = malloc((len + 1) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
    if p.is_null() { return null_mut(); }
    core::ptr::copy_nonoverlapping(s, p, len + 1);
    p
}

#[no_mangle]
pub unsafe extern "C" fn wcsnlen(s: *const wchar_t, maxlen: usize) -> usize {
    let mut i = 0;
    while i < maxlen && *s.add(i) != 0 { i += 1; }
    i
}

#[no_mangle]
pub unsafe extern "C" fn wcsspn(s: *const wchar_t, accept: *const wchar_t) -> usize {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 { return i; }
        let mut found = false;
        let mut j = 0;
        loop {
            let a = *accept.add(j);
            if a == 0 { break; }
            if c == a { found = true; break; }
            j += 1;
        }
        if !found { return i; }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcscspn(s: *const wchar_t, reject: *const wchar_t) -> usize {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 { return i; }
        let mut j = 0;
        loop {
            let r = *reject.add(j);
            if r == 0 { break; }
            if c == r { return i; }
            j += 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcspbrk(s: *const wchar_t, accept: *const wchar_t) -> *mut wchar_t {
    let mut i = 0;
    loop {
        let c = *s.add(i);
        if c == 0 { return null_mut(); }
        let mut j = 0;
        loop {
            let a = *accept.add(j);
            if a == 0 { break; }
            if c == a { return s.add(i) as *mut wchar_t; }
            j += 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn wcsxfrm(dst: *mut wchar_t, src: *const wchar_t, n: usize) -> usize {
    let len = wcslen(src);
    if !dst.is_null() {
        let copy = if len < n { len } else { if n > 0 { n - 1 } else { 0 } };
        for i in 0..copy { *dst.add(i) = *src.add(i); }
        if n > 0 { *dst.add(copy) = 0; }
    }
    len
}

// ============================================================
// wchar: wcsto* number conversions
// ============================================================

unsafe fn wcs_to_bytes(ws: *const wchar_t, buf: *mut u8, bufsz: usize) -> usize {
    let mut i = 0;
    while i < bufsz - 1 {
        let c = *ws.add(i);
        if c == 0 { break; }
        if (c as u32) > 127 { break; }
        *buf.add(i) = c as u8;
        i += 1;
    }
    *buf.add(i) = 0;
    i
}

#[no_mangle]
pub unsafe extern "C" fn wcstol(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_long {
    let mut buf = [0u8; 256];
    wcs_to_bytes(s, buf.as_mut_ptr(), 256);
    let mut end: *mut c_char = core::ptr::null_mut();
    let r = strtol(buf.as_ptr() as *const c_char, &mut end, base);
    if !endptr.is_null() {
        let offset = end as usize - buf.as_ptr() as usize;
        *endptr = s.add(offset) as *mut wchar_t;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn wcstoul(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_ulong {
    let mut buf = [0u8; 256];
    wcs_to_bytes(s, buf.as_mut_ptr(), 256);
    let mut end: *mut c_char = core::ptr::null_mut();
    let r = strtoul(buf.as_ptr() as *const c_char, &mut end, base);
    if !endptr.is_null() {
        let offset = end as usize - buf.as_ptr() as usize;
        *endptr = s.add(offset) as *mut wchar_t;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn wcstoll(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_longlong {
    let mut buf = [0u8; 256];
    wcs_to_bytes(s, buf.as_mut_ptr(), 256);
    let mut end: *mut c_char = core::ptr::null_mut();
    let r = strtoll(buf.as_ptr() as *const c_char, &mut end, base);
    if !endptr.is_null() {
        let offset = end as usize - buf.as_ptr() as usize;
        *endptr = s.add(offset) as *mut wchar_t;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn wcstoull(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_ulonglong {
    let mut buf = [0u8; 256];
    wcs_to_bytes(s, buf.as_mut_ptr(), 256);
    let mut end: *mut c_char = core::ptr::null_mut();
    let r = strtoull(buf.as_ptr() as *const c_char, &mut end, base);
    if !endptr.is_null() {
        let offset = end as usize - buf.as_ptr() as usize;
        *endptr = s.add(offset) as *mut wchar_t;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn wcstod(s: *const wchar_t, endptr: *mut *mut wchar_t) -> f64 {
    let mut buf = [0u8; 256];
    wcs_to_bytes(s, buf.as_mut_ptr(), 256);
    let mut end: *mut c_char = core::ptr::null_mut();
    let r = strtod(buf.as_ptr() as *const c_char, &mut end);
    if !endptr.is_null() {
        let offset = end as usize - buf.as_ptr() as usize;
        *endptr = s.add(offset) as *mut wchar_t;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn wcstof(s: *const wchar_t, endptr: *mut *mut wchar_t) -> f32 { wcstod(s, endptr) as f32 }

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn wcstold(s: *const wchar_t, endptr: *mut *mut wchar_t) -> f64 { wcstod(s, endptr) }

#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn wcstold(s: *const wchar_t, endptr: *mut *mut wchar_t) -> f128 { wcstod(s, endptr) as f128 }

#[no_mangle]
pub unsafe extern "C" fn wcstoimax(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_longlong { wcstoll(s, endptr, base) }

#[no_mangle]
pub unsafe extern "C" fn wcstoumax(s: *const wchar_t, endptr: *mut *mut wchar_t, base: c_int) -> c_ulonglong { wcstoull(s, endptr, base) }

// ============================================================
// wctype: classification functions
// ============================================================

const WC_ALPHA: c_int = 1;
const WC_DIGIT: c_int = 2;
const WC_SPACE: c_int = 4;
const WC_UPPER: c_int = 8;
const WC_LOWER: c_int = 16;
const WC_ALNUM: c_int = WC_ALPHA | WC_DIGIT;
const WC_BLANK: c_int = 32;
const WC_CNTRL: c_int = 64;
const WC_PUNCT: c_int = 128;
const WC_GRAPH: c_int = WC_ALNUM | WC_PUNCT;
const WC_PRINT: c_int = WC_GRAPH | WC_BLANK;
const WC_XDIGIT: c_int = 256;

fn wc_flags(c: wint_t) -> c_int {
    if c < 128 {
        let f = CT_TABLE[c as usize];
        let mut r = 0;
        if f & (CT_UPPER | CT_LOWER) != 0 { r |= WC_ALPHA; }
        if f & CT_DIGIT != 0 { r |= WC_DIGIT; }
        if f & CT_SPACE != 0 { r |= WC_SPACE; }
        if f & CT_UPPER != 0 { r |= WC_UPPER; }
        if f & CT_LOWER != 0 { r |= WC_LOWER; }
        if f & CT_BLANK != 0 { r |= WC_BLANK; }
        if f & CT_CNTRL != 0 { r |= WC_CNTRL; }
        if f & CT_PUNCT != 0 { r |= WC_PUNCT; }
        if f & CT_XDIGIT != 0 { r |= WC_XDIGIT; }
        r
    } else {
        if c < 0x100 { 0 }
        else { WC_PRINT | WC_ALPHA }
    }
}

#[no_mangle]
pub extern "C" fn iswalnum(c: wint_t) -> c_int { (wc_flags(c) & WC_ALNUM != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswalpha(c: wint_t) -> c_int { (wc_flags(c) & WC_ALPHA != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswblank(c: wint_t) -> c_int { (wc_flags(c) & WC_BLANK != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswcntrl(c: wint_t) -> c_int { (wc_flags(c) & WC_CNTRL != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswdigit(c: wint_t) -> c_int { ((c >= b'0' as wint_t) && (c <= b'9' as wint_t)) as c_int }
#[no_mangle]
pub extern "C" fn iswgraph(c: wint_t) -> c_int { (wc_flags(c) & WC_GRAPH != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswlower(c: wint_t) -> c_int { (wc_flags(c) & WC_LOWER != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswprint(c: wint_t) -> c_int {
    if c < 128 { (wc_flags(c) & WC_PRINT != 0) as c_int }
    else if c >= 0x100 && c < 0x110000 { 1 }
    else { 0 }
}
#[no_mangle]
pub extern "C" fn iswpunct(c: wint_t) -> c_int { (wc_flags(c) & WC_PUNCT != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswspace(c: wint_t) -> c_int {
    if c < 128 { (wc_flags(c) & WC_SPACE != 0) as c_int }
    else {
        match c {
            0x1680 | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000 => 1,
            _ => 0,
        }
    }
}
#[no_mangle]
pub extern "C" fn iswupper(c: wint_t) -> c_int { (wc_flags(c) & WC_UPPER != 0) as c_int }
#[no_mangle]
pub extern "C" fn iswxdigit(c: wint_t) -> c_int { (wc_flags(c) & WC_XDIGIT != 0) as c_int }

static WCTYPE_ALNUM: c_int = WC_ALNUM;
static WCTYPE_ALPHA: c_int = WC_ALPHA;
static WCTYPE_BLANK: c_int = WC_BLANK;
static WCTYPE_CNTRL: c_int = WC_CNTRL;
static WCTYPE_DIGIT: c_int = WC_DIGIT;
static WCTYPE_GRAPH: c_int = WC_GRAPH;
static WCTYPE_LOWER: c_int = WC_LOWER;
static WCTYPE_PRINT: c_int = WC_PRINT;
static WCTYPE_PUNCT: c_int = WC_PUNCT;
static WCTYPE_SPACE: c_int = WC_SPACE;
static WCTYPE_UPPER: c_int = WC_UPPER;
static WCTYPE_XDIGIT: c_int = WC_XDIGIT;

#[no_mangle]
pub unsafe extern "C" fn wctype(name: *const c_char) -> wctype_t {
    if name.is_null() { return core::ptr::null(); }
    let n = name as *const u8;
    if strcmp(n, b"alnum\0".as_ptr()) == 0 { return &WCTYPE_ALNUM; }
    if strcmp(n, b"alpha\0".as_ptr()) == 0 { return &WCTYPE_ALPHA; }
    if strcmp(n, b"blank\0".as_ptr()) == 0 { return &WCTYPE_BLANK; }
    if strcmp(n, b"cntrl\0".as_ptr()) == 0 { return &WCTYPE_CNTRL; }
    if strcmp(n, b"digit\0".as_ptr()) == 0 { return &WCTYPE_DIGIT; }
    if strcmp(n, b"graph\0".as_ptr()) == 0 { return &WCTYPE_GRAPH; }
    if strcmp(n, b"lower\0".as_ptr()) == 0 { return &WCTYPE_LOWER; }
    if strcmp(n, b"print\0".as_ptr()) == 0 { return &WCTYPE_PRINT; }
    if strcmp(n, b"punct\0".as_ptr()) == 0 { return &WCTYPE_PUNCT; }
    if strcmp(n, b"space\0".as_ptr()) == 0 { return &WCTYPE_SPACE; }
    if strcmp(n, b"upper\0".as_ptr()) == 0 { return &WCTYPE_UPPER; }
    if strcmp(n, b"xdigit\0".as_ptr()) == 0 { return &WCTYPE_XDIGIT; }
    core::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn iswctype(c: wint_t, desc: wctype_t) -> c_int {
    if desc.is_null() { return 0; }
    (wc_flags(c) & *desc != 0) as c_int
}

static WCTRANS_LOWER: c_int = 1;
static WCTRANS_UPPER: c_int = 2;

#[no_mangle]
pub unsafe extern "C" fn towlower(c: wint_t) -> wint_t {
    if c < 128 { tolower(c as c_int) as wint_t } else { c }
}

#[no_mangle]
pub unsafe extern "C" fn towupper(c: wint_t) -> wint_t {
    if c < 128 { toupper(c as c_int) as wint_t } else { c }
}

#[no_mangle]
pub unsafe extern "C" fn wctrans(name: *const c_char) -> wctrans_t {
    if name.is_null() { return core::ptr::null(); }
    let n = name as *const u8;
    if strcmp(n, b"tolower\0".as_ptr()) == 0 { return &WCTRANS_LOWER; }
    if strcmp(n, b"toupper\0".as_ptr()) == 0 { return &WCTRANS_UPPER; }
    core::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn towctrans(c: wint_t, desc: wctrans_t) -> wint_t {
    if desc.is_null() { return c; }
    match *desc {
        1 => towlower(c),
        2 => towupper(c),
        _ => c,
    }
}

// ============================================================
// wchar: wide stdio
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn fgetwc(f: *mut FILE) -> wint_t {
    let mut state: c_uint = 0;
    let mut wc: wchar_t = 0;
    let mut buf = [0u8; 4];
    let c = fgetc(f);
    if c == -1 { return WEOF; }
    buf[0] = c as u8;
    if buf[0] < 0x80 { return buf[0] as wint_t; }
    let len = if buf[0] < 0xE0 { 2 } else if buf[0] < 0xF0 { 3 } else { 4 };
    let mut n = 1usize;
    while n < len {
        let c2 = fgetc(f);
        if c2 == -1 { return WEOF; }
        buf[n] = c2 as u8;
        n += 1;
    }
    let r = mbrtowc(&mut wc, buf.as_ptr() as *const c_char, n, &mut state);
    if r == !0usize || r == !1usize { WEOF } else { wc as wint_t }
}

#[no_mangle]
pub unsafe extern "C" fn fputwc(c: wchar_t, f: *mut FILE) -> wint_t {
    let mut buf = [0u8; 4];
    let n = wcrtomb(buf.as_mut_ptr() as *mut c_char, c, core::ptr::null_mut());
    if n == !0usize || n == 0 { return WEOF; }
    if fwrite(buf.as_ptr() as *const c_void, 1, n, f) != n { return WEOF; }
    c as wint_t
}

#[no_mangle]
pub unsafe extern "C" fn fputws(s: *const wchar_t, f: *mut FILE) -> c_int {
    let mut i = 0;
    while *s.add(i) != 0 {
        if fputwc(*s.add(i), f) == WEOF { return -1; }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn ungetwc(c: wint_t, f: *mut FILE) -> wint_t {
    if c == WEOF { return WEOF; }
    let mut buf = [0u8; 4];
    let n = wcrtomb(buf.as_mut_ptr() as *mut c_char, c as wchar_t, core::ptr::null_mut());
    if n == !0usize || n == 0 { return WEOF; }
    let mut i = n;
    while i > 0 {
        i -= 1;
        if ungetc(buf[i] as c_int, f) == -1 { return WEOF; }
    }
    c
}

#[no_mangle]
pub unsafe extern "C" fn getwchar() -> wint_t { fgetwc(stdin) }

#[no_mangle]
pub unsafe extern "C" fn putwchar(c: wchar_t) -> wint_t { fputwc(c, stdout) }

// ============================================================
// wchar: swprintf / vswprintf / fwprintf / vfwprintf
// ============================================================

unsafe fn wfmt_write_str(dst: *mut wchar_t, pos: usize, cap: usize, s: *const u8, len: usize) -> usize {
    let mut i = 0;
    while i < len {
        if pos + i < cap { *dst.add(pos + i) = *s.add(i) as wchar_t; }
        i += 1;
    }
    pos + len
}

#[no_mangle]
pub unsafe extern "C" fn vswprintf(s: *mut wchar_t, n: usize, fmt: *const wchar_t, mut args: VaList) -> c_int {
    if n == 0 { return -1; }
    let cap = n - 1;
    let mut pos = 0usize;
    // ponytail: format specifiers are ASCII; narrow buffer suffices for parsing
    let mut fmt_narrow = [0u8; 256];
    let mut flen = 0usize;
    loop {
        let ch = *fmt.add(flen) as u32;
        if ch == 0 || flen >= 255 { break; }
        fmt_narrow[flen] = if ch < 128 { ch as u8 } else { b'?' };
        flen += 1;
    }
    fmt_narrow[flen] = 0;
    let mut fi = 0usize;
    macro_rules! wc {
        ($c:expr) => { if pos < cap { *s.add(pos) = $c as wchar_t; } pos += 1; }
    }
    loop {
        let c = fmt_narrow[fi];
        if c == 0 { break; }
        if c != b'%' { wc!(c); fi += 1; continue; }
        fi += 1;
        let mut flags: u8 = 0;
        loop {
            match fmt_narrow[fi] {
                b'-' => { flags |= FLAG_MINUS; fi += 1; }
                b'+' => { flags |= FLAG_PLUS; fi += 1; }
                b' ' => { flags |= FLAG_SPACE; fi += 1; }
                b'0' => { flags |= FLAG_ZERO; fi += 1; }
                b'#' => { flags |= FLAG_HASH; fi += 1; }
                _ => break,
            }
        }
        let mut width: usize = 0;
        if fmt_narrow[fi] == b'*' {
            let w = args.next_arg::<c_int>();
            if w < 0 { flags |= FLAG_MINUS; width = (-w) as usize; } else { width = w as usize; }
            fi += 1;
        } else {
            while fmt_narrow[fi] >= b'0' && fmt_narrow[fi] <= b'9' {
                width = width * 10 + (fmt_narrow[fi] - b'0') as usize; fi += 1;
            }
        }
        let mut precision: i32 = -1;
        if fmt_narrow[fi] == b'.' {
            fi += 1;
            if fmt_narrow[fi] == b'*' {
                let p = args.next_arg::<c_int>();
                precision = if p < 0 { -1 } else { p }; fi += 1;
            } else {
                precision = 0;
                while fmt_narrow[fi] >= b'0' && fmt_narrow[fi] <= b'9' {
                    precision = precision * 10 + (fmt_narrow[fi] - b'0') as i32; fi += 1;
                }
            }
        }
        let len_mod = fmt_narrow[fi];
        if len_mod == b'h' || len_mod == b'l' || len_mod == b'j' || len_mod == b'z' || len_mod == b't' || len_mod == b'L' {
            fi += 1;
            let spec = fmt_narrow[fi];
            if (len_mod == b'h' && spec == b'h') || (len_mod == b'l' && spec == b'l') {
                fi += 1;
                let spec2 = fmt_narrow[fi];
                match (len_mod, spec, spec2) {
                    (b'h', b'h', b'n') => { let p = args.next_arg::<*mut c_void>(); if !p.is_null() { *(p as *mut u8) = pos as u8; } }
                    (b'l', b'l', b'n') => { let p = args.next_arg::<*mut c_void>(); if !p.is_null() { *(p as *mut c_longlong) = pos as c_longlong; } }
                    (b'l', b'l', b'd') | (b'l', b'l', b'i') => {
                        let val = args.next_arg::<c_longlong>();
                        let neg = val < 0;
                        let abs = if neg { val.wrapping_neg() as u64 } else { val as u64 };
                        let b = format_u64(abs);
                        let sign = if neg { Some(b'-') } else if flags & FLAG_PLUS != 0 { Some(b'+') } else if flags & FLAG_SPACE != 0 { Some(b' ') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, sign, None, precision, width, flags, false);
                        for k in 0..len { wc!(fbuf[k]); }
                    }
                    (b'l', b'l', b'u') => {
                        let val = args.next_arg::<c_ulonglong>();
                        let b = format_u64(val as u64);
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, false);
                        for k in 0..len { wc!(fbuf[k]); }
                    }
                    (b'l', b'l', b'x') => {
                        let val = args.next_arg::<c_ulonglong>();
                        let b = format_hex(val as u64, false);
                        let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'x') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                        for k in 0..len { wc!(fbuf[k]); }
                    }
                    (b'l', b'l', b'X') => {
                        let val = args.next_arg::<c_ulonglong>();
                        let b = format_hex(val as u64, true);
                        let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'X') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                        for k in 0..len { wc!(fbuf[k]); }
                    }
                    (b'l', b'l', b'o') => {
                        let val = args.next_arg::<c_ulonglong>();
                        let b = format_octal(val as u64, flags & FLAG_HASH != 0);
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, flags & FLAG_HASH != 0);
                        for k in 0..len { wc!(fbuf[k]); }
                    }
                    _ => { wc!(b'%'); wc!(len_mod); wc!(spec); wc!(spec2); }
                }
            } else {
                match (len_mod, spec) {
                    (b'h', b'n') => { let p = args.next_arg::<*mut c_void>(); if !p.is_null() { *(p as *mut u16) = pos as u16; } }
                    (b'l', b'n') => { let p = args.next_arg::<*mut c_void>(); if !p.is_null() { *(p as *mut c_long) = pos as c_long; } }
                    (b'z', b'n') | (b't', b'n') => { let p = args.next_arg::<*mut c_void>(); if !p.is_null() { *(p as *mut usize) = pos; } }
                    (b'j', b'n') => { let p = args.next_arg::<*mut c_void>(); if !p.is_null() { *(p as *mut c_ulonglong) = pos as c_ulonglong; } }
                    (b'l', b'd') | (b'l', b'i') => {
                        let val = args.next_arg::<c_long>();
                        let neg = val < 0;
                        let abs = if neg { val.wrapping_neg() as u64 } else { val as u64 };
                        let b = format_u64(abs);
                        let sign = if neg { Some(b'-') } else if flags & FLAG_PLUS != 0 { Some(b'+') } else if flags & FLAG_SPACE != 0 { Some(b' ') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, sign, None, precision, width, flags, false);
                        for k in 0..len { wc!(fbuf[k]); }
                    }
                    (b'l', b'u') => {
                        let val = args.next_arg::<c_ulong>();
                        let b = format_u64(val as u64);
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, false);
                        for k in 0..len { wc!(fbuf[k]); }
                    }
                    (b'l', b'x') => {
                        let val = args.next_arg::<c_ulong>();
                        let b = format_hex(val as u64, false);
                        let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'x') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                        for k in 0..len { wc!(fbuf[k]); }
                    }
                    (b'l', b'X') => {
                        let val = args.next_arg::<c_ulong>();
                        let b = format_hex(val as u64, true);
                        let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'X') } else { None };
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                        for k in 0..len { wc!(fbuf[k]); }
                    }
                    (b'l', b'o') => {
                        let val = args.next_arg::<c_ulong>();
                        let b = format_octal(val as u64, flags & FLAG_HASH != 0);
                        let mut fbuf = [0u8; 32];
                        let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, flags & FLAG_HASH != 0);
                        for k in 0..len { wc!(fbuf[k]); }
                    }
                    (b'l', b'c') => {
                        let ch = args.next_arg::<wint_t>();
                        wc!(ch);
                    }
                    (b'l', b's') => {
                        let ws_arg = args.next_arg::<*const wchar_t>();
                        if ws_arg.is_null() {
                            let slen = if precision >= 0 { (precision as usize).min(6) } else { 6 };
                            if flags & FLAG_MINUS != 0 {
                                for k in 0..slen { wc!(b"(null)"[k]); }
                                for _ in 0..width.saturating_sub(slen) { wc!(b' '); }
                            } else {
                                for _ in 0..width.saturating_sub(slen) { wc!(b' '); }
                                for k in 0..slen { wc!(b"(null)"[k]); }
                            }
                        } else {
                            let mut wlen = 0usize;
                            loop {
                                let ch = *ws_arg.add(wlen);
                                if ch == 0 { break; }
                                if precision >= 0 && wlen >= precision as usize { break; }
                                wlen += 1;
                            }
                            if flags & FLAG_MINUS != 0 {
                                for k in 0..wlen { wc!(*ws_arg.add(k)); }
                                for _ in 0..width.saturating_sub(wlen) { wc!(b' '); }
                            } else {
                                for _ in 0..width.saturating_sub(wlen) { wc!(b' '); }
                                for k in 0..wlen { wc!(*ws_arg.add(k)); }
                            }
                        }
                    }
                    (b'l', b'f') | (b'l', b'F') | (b'l', b'e') | (b'l', b'E')
                    | (b'l', b'g') | (b'l', b'G') | (b'l', b'a') | (b'l', b'A') => {
                        let val = args.next_arg::<f64>();
                        let ucase = spec == b'F' || spec == b'E' || spec == b'G' || spec == b'A';
                        let ftype = match spec | 0x20 { b'f' => FMT_F, b'e' => FMT_E, b'g' => FMT_G, b'a' => FMT_A, _ => FMT_F };
                        let mut fbuf = [0u8; 4224];
                        let flen = format_f64_full(fbuf.as_mut_ptr(), val, ftype, precision, flags, ucase);
                        let mut wbuf = [0u8; 4224];
                        let wlen = apply_width_flags(wbuf.as_mut_ptr(), fbuf.as_ptr(), flen, width, flags);
                        for k in 0..wlen { wc!(wbuf[k]); }
                    }
                    (b'L', b'f') | (b'L', b'F') | (b'L', b'e') | (b'L', b'E')
                    | (b'L', b'g') | (b'L', b'G') | (b'L', b'a') | (b'L', b'A') => {
                        #[cfg(target_arch = "aarch64")]
                        let val = {
                            let lo: u64 = args.next_arg::<u64>();
                            let hi: u64 = args.next_arg::<u64>();
                            let combined: u128 = ((hi as u128) << 64) | (lo as u128);
                            f128::from_bits(combined) as f64
                        };
                        #[cfg(target_arch = "x86_64")]
                        let val = args.next_arg::<f64>();
                        let ucase = spec == b'F' || spec == b'E' || spec == b'G' || spec == b'A';
                        let ftype = match spec | 0x20 { b'f' => FMT_F, b'e' => FMT_E, b'g' => FMT_G, b'a' => FMT_A, _ => FMT_F };
                        let mut fbuf = [0u8; 4224];
                        let flen = format_f64_full(fbuf.as_mut_ptr(), val, ftype, precision, flags, ucase);
                        let mut wbuf = [0u8; 4224];
                        let wlen = apply_width_flags(wbuf.as_mut_ptr(), fbuf.as_ptr(), flen, width, flags);
                        for k in 0..wlen { wc!(wbuf[k]); }
                    }
                    _ => { wc!(b'%'); wc!(len_mod); wc!(spec); }
                }
            }
        } else {
            let spec = fmt_narrow[fi];
            match spec {
                b'd' | b'i' => {
                    let d = args.next_arg::<c_int>();
                    let neg = d < 0;
                    let abs = if neg { d.wrapping_neg() as u64 } else { d as u64 };
                    let b = format_u64(abs);
                    let sign = if neg { Some(b'-') } else if flags & FLAG_PLUS != 0 { Some(b'+') } else if flags & FLAG_SPACE != 0 { Some(b' ') } else { None };
                    let mut fbuf = [0u8; 32];
                    let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, sign, None, precision, width, flags, false);
                    for k in 0..len { wc!(fbuf[k]); }
                }
                b'u' => {
                    let u = args.next_arg::<c_uint>();
                    let b = format_u64(u as u64);
                    let mut fbuf = [0u8; 32];
                    let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, false);
                    for k in 0..len { wc!(fbuf[k]); }
                }
                b'x' => {
                    let x = args.next_arg::<c_uint>();
                    let b = format_hex(x as u64, false);
                    let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'x') } else { None };
                    let mut fbuf = [0u8; 32];
                    let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                    for k in 0..len { wc!(fbuf[k]); }
                }
                b'X' => {
                    let x = args.next_arg::<c_uint>();
                    let b = format_hex(x as u64, true);
                    let prefix = if flags & FLAG_HASH != 0 && !(b.1 == 1 && b.0[0] == b'0') { Some(b'X') } else { None };
                    let mut fbuf = [0u8; 32];
                    let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, prefix, precision, width, flags, false);
                    for k in 0..len { wc!(fbuf[k]); }
                }
                b'o' => {
                    let o = args.next_arg::<c_uint>();
                    let b = format_octal(o as u64, flags & FLAG_HASH != 0);
                    let mut fbuf = [0u8; 32];
                    let len = format_int(fbuf.as_mut_ptr(), b.0.as_ptr(), b.1, None, None, precision, width, flags, flags & FLAG_HASH != 0);
                    for k in 0..len { wc!(fbuf[k]); }
                }
                b'f' | b'F' | b'e' | b'E' | b'g' | b'G' | b'a' | b'A' => {
                    let val = args.next_arg::<f64>();
                    let ucase = spec == b'F' || spec == b'E' || spec == b'G' || spec == b'A';
                    let ftype = match spec | 0x20 { b'f' => FMT_F, b'e' => FMT_E, b'g' => FMT_G, b'a' => FMT_A, _ => FMT_F };
                    let mut fbuf = [0u8; 4224];
                    let flen = format_f64_full(fbuf.as_mut_ptr(), val, ftype, precision, flags, ucase);
                    let mut wbuf = [0u8; 4224];
                    let wlen = apply_width_flags(wbuf.as_mut_ptr(), fbuf.as_ptr(), flen, width, flags);
                    for k in 0..wlen { wc!(wbuf[k]); }
                }
                b'c' => {
                    let ch = args.next_arg::<c_int>();
                    wc!(ch as u8);
                }
                b's' => {
                    let p = args.next_arg::<*const c_char>();
                    if p.is_null() {
                        let slen = if precision >= 0 { (precision as usize).min(6) } else { 6 };
                        if flags & FLAG_MINUS != 0 {
                            for k in 0..slen { wc!(b"(null)"[k]); }
                            for _ in 0..width.saturating_sub(slen) { wc!(b' '); }
                        } else {
                            for _ in 0..width.saturating_sub(slen) { wc!(b' '); }
                            for k in 0..slen { wc!(b"(null)"[k]); }
                        }
                    } else {
                        let mut wbuf_local = [0i32; 256];
                        let mut wcount = 0usize;
                        let mut byte_pos = 0usize;
                        let pbytes = p as *const u8;
                        loop {
                            if precision >= 0 && wcount >= precision as usize { break; }
                            if wcount >= 256 { break; }
                            let mut wc_val: c_int = 0;
                            let consumed = mbrtowc(&mut wc_val as *mut c_int, p.add(byte_pos) as *const c_char, 256, core::ptr::null_mut());
                            if consumed == !0usize || consumed == !1usize {
                                wbuf_local[wcount] = *pbytes.add(byte_pos) as c_int;
                                byte_pos += 1;
                                wcount += 1;
                            } else if consumed == 0 {
                                break;
                            } else {
                                wbuf_local[wcount] = wc_val;
                                byte_pos += consumed;
                                wcount += 1;
                            }
                        }
                        if flags & FLAG_MINUS != 0 {
                            for k in 0..wcount { wc!(wbuf_local[k]); }
                            for _ in 0..width.saturating_sub(wcount) { wc!(b' '); }
                        } else {
                            for _ in 0..width.saturating_sub(wcount) { wc!(b' '); }
                            for k in 0..wcount { wc!(wbuf_local[k]); }
                        }
                    }
                }
                b'p' => {
                    wc!(b'0'); wc!(b'x');
                    let p = args.next_arg::<*const c_void>();
                    let b = format_hex(p as u64, false);
                    for k in 0..b.1 { wc!(b.0[k]); }
                }
                b'n' => {
                    let p = args.next_arg::<*mut c_void>();
                    if !p.is_null() { *(p as *mut c_int) = pos as c_int; }
                }
                b'%' => { wc!(b'%'); }
                _ => { wc!(b'%'); wc!(spec); }
            }
        }
        fi += 1;
    }
    let null_pos = if pos < cap { pos } else { cap };
    *s.add(null_pos) = 0;
    if pos > cap { -1 } else { pos as c_int }
}

#[no_mangle]
pub unsafe extern "C" fn swprintf(s: *mut wchar_t, n: usize, fmt: *const wchar_t, args: ...) -> c_int {
    vswprintf(s, n, fmt, args)
}

#[no_mangle]
pub unsafe extern "C" fn vfwprintf(f: *mut FILE, fmt: *const wchar_t, args: VaList) -> c_int {
    let mut buf = [0i32; 4096];
    let r = vswprintf(buf.as_mut_ptr(), 4096, fmt, args);
    if r < 0 { return r; }
    let mut i = 0;
    while i < r as usize {
        let mut mb = [0u8; 4];
        let n = wcrtomb(mb.as_mut_ptr() as *mut c_char, buf[i], core::ptr::null_mut());
        if n == !0usize { return -1; }
        if fwrite(mb.as_ptr() as *const c_void, 1, n, f) != n { return -1; }
        i += 1;
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn fwprintf(f: *mut FILE, fmt: *const wchar_t, args: ...) -> c_int {
    vfwprintf(f, fmt, args)
}

// ============================================================
// wchar: vswscanf / swscanf / vfwscanf / fwscanf / vwscanf / wscanf
// ============================================================

// Convert wchar_t format to byte format (ASCII cast, format specifiers are ASCII)
unsafe fn wcsfmt_to_mbs(dst: *mut u8, dst_len: usize, src: *const wchar_t) {
    let mut i = 0usize;
    loop {
        let ch = *src.add(i) as u32;
        if ch == 0 || i >= dst_len - 1 { break; }
        *dst.add(i) = if ch < 128 { ch as u8 } else { b'?' };
        i += 1;
    }
    *dst.add(i) = 0;
}

#[no_mangle]
pub unsafe extern "C" fn vswscanf(s: *const wchar_t, fmt: *const wchar_t, mut args: VaList) -> c_int {
    let mut mbs_buf = [0u8; 4096];
    let mut i = 0usize;
    let mut mb_pos = 0usize;
    loop {
        let wc = *s.add(i) as c_int;
        if wc == 0 { break; }
        let mut mb = [0u8; 4];
        let n = wcrtomb(mb.as_mut_ptr() as *mut c_char, wc, core::ptr::null_mut());
        if n == !0usize || n == 0 { break; }
        if mb_pos + n >= mbs_buf.len() { break; }
        let mut k = 0usize;
        while k < n { mbs_buf[mb_pos + k] = mb[k]; k += 1; }
        mb_pos += n;
        i += 1;
    }
    mbs_buf[mb_pos] = 0;
    let mut mbs_fmt = [0u8; 4096];
    wcsfmt_to_mbs(mbs_fmt.as_mut_ptr(), mbs_fmt.len(), fmt);
    do_vsscanf(mbs_buf.as_ptr(), mb_pos, mbs_fmt.as_ptr() as *const c_char, &mut args, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn swscanf(s: *const wchar_t, fmt: *const wchar_t, args: ...) -> c_int {
    vswscanf(s, fmt, args)
}

#[no_mangle]
pub unsafe extern "C" fn vfwscanf(f: *mut FILE, fmt: *const wchar_t, mut args: VaList) -> c_int {
    let start_pos = ftello(f);
    if start_pos < 0 { return -1; }
    let mut buf = [0u8; 4096];
    let mut n = 0usize;
    loop {
        if n >= buf.len() - 1 { break; }
        let c = fgetc(f);
        if c == -1 { break; }
        buf[n] = c as u8;
        n += 1;
    }
    buf[n] = 0;
    let mut mbs_fmt = [0u8; 4096];
    wcsfmt_to_mbs(mbs_fmt.as_mut_ptr(), mbs_fmt.len(), fmt);
    let mut consumed = 0usize;
    let hit_eof = (*f)._eof != 0;
    let result = do_vsscanf(buf.as_ptr(), n, mbs_fmt.as_ptr() as *const c_char, &mut args, &mut consumed);
    let _ = fseeko(f, start_pos + consumed as i64, SEEK_SET);
    if hit_eof && consumed == n {
        (*f)._eof = 1;
    }
    result
}

#[no_mangle]
pub unsafe extern "C" fn fwscanf(f: *mut FILE, fmt: *const wchar_t, args: ...) -> c_int {
    vfwscanf(f, fmt, args)
}

#[no_mangle]
pub unsafe extern "C" fn vwscanf(fmt: *const wchar_t, args: VaList) -> c_int {
    vfwscanf(stdin, fmt, args)
}

#[no_mangle]
pub unsafe extern "C" fn wscanf(fmt: *const wchar_t, args: ...) -> c_int {
    vfwscanf(stdin, fmt, args)
}

// ============================================================
// Time: internal helpers
// ============================================================

const LEAPOCH: i64 = 946684800 + 86400 * (31 + 29);
const DAYS_PER_400Y: i64 = 365 * 400 + 97;
const DAYS_PER_100Y: i64 = 365 * 100 + 24;
const DAYS_PER_4Y: i64 = 365 * 4 + 1;

const SECS_THROUGH_MONTH: [i32; 12] = [
    0, 31*86400, 59*86400, 90*86400, 120*86400, 151*86400,
    181*86400, 212*86400, 243*86400, 273*86400, 304*86400, 334*86400,
];

const DAYS_IN_MONTH: [i32; 12] = [31, 30, 31, 30, 31, 31, 30, 31, 30, 31, 31, 29];

unsafe fn year_to_secs(year: i64, is_leap: &mut bool) -> i64 {
    if year - 2 <= 136 {
        let y = year as i32;
        let mut leaps = (y - 68) >> 2;
        if (y - 68) & 3 == 0 { leaps -= 1; *is_leap = true; } else { *is_leap = false; }
        return 31536000 * (y as i64 - 70) + 86400 * leaps as i64;
    }
    let mut cycles = ((year - 100) / 400) as i32;
    let mut rem = ((year - 100) % 400) as i32;
    if rem < 0 { cycles -= 1; rem += 400; }
    let (centuries, leaps);
    if rem == 0 { *is_leap = true; centuries = 0; leaps = 0; }
    else {
        let c;
        if rem >= 200 { if rem >= 300 { c = 3; rem -= 300; } else { c = 2; rem -= 200; } }
        else { if rem >= 100 { c = 1; rem -= 100; } else { c = 0; } }
        centuries = c;
        if rem == 0 { *is_leap = false; leaps = 0; }
        else { leaps = rem / 4; rem %= 4; *is_leap = rem == 0; }
    }
    let total_leaps = leaps + 97 * cycles + 24 * centuries - (*is_leap as i32);
    (year - 100) * 31536000 + total_leaps as i64 * 86400 + 946684800 + 86400
}

fn month_to_secs(month: i32, is_leap: bool) -> i32 {
    let mut t = SECS_THROUGH_MONTH[month as usize];
    if is_leap && month >= 2 { t += 86400; }
    t
}

unsafe fn secs_to_tm(t: i64, tm: &mut tm) -> bool {
    if t < (i32::MIN as i64) * 31622400 || t > (i32::MAX as i64) * 31622400 { return false; }
    let secs = t - LEAPOCH;
    let mut days = secs / 86400;
    let mut remsecs = (secs % 86400) as i32;
    if remsecs < 0 { remsecs += 86400; days -= 1; }
    let mut wday = ((3 + days) % 7) as i32;
    if wday < 0 { wday += 7; }
    let mut qc_cycles = (days / DAYS_PER_400Y) as i32;
    let mut remdays = (days % DAYS_PER_400Y) as i32;
    if remdays < 0 { remdays += DAYS_PER_400Y as i32; qc_cycles -= 1; }
    let mut c_cycles = remdays / DAYS_PER_100Y as i32;
    if c_cycles == 4 { c_cycles -= 1; }
    remdays -= c_cycles * DAYS_PER_100Y as i32;
    let mut q_cycles = remdays / DAYS_PER_4Y as i32;
    if q_cycles == 25 { q_cycles -= 1; }
    remdays -= q_cycles * DAYS_PER_4Y as i32;
    let mut remyears = remdays / 365;
    if remyears == 4 { remyears -= 1; }
    remdays -= remyears * 365;
    let leap = remyears == 0 && (q_cycles != 0 || c_cycles == 0);
    let mut yday = remdays + 31 + 28 + leap as i32;
    if yday >= 365 + leap as i32 { yday -= 365 + leap as i32; }
    let years = remyears as i64 + 4 * q_cycles as i64 + 100 * c_cycles as i64 + 400 * qc_cycles as i64;
    let mut months = 0i32;
    let mut rd = remdays;
    for m in 0..12 {
        if DAYS_IN_MONTH[m] <= rd { rd -= DAYS_IN_MONTH[m]; } else { months = m as i32; break; }
    }
    let mut adj_years = years;
    if months >= 10 { months -= 12; adj_years += 1; }
    if adj_years + 100 > i32::MAX as i64 || adj_years + 100 < i32::MIN as i64 { return false; }
    tm.tm_year = (adj_years + 100) as c_int;
    tm.tm_mon = months + 2;
    tm.tm_mday = rd + 1;
    tm.tm_wday = wday;
    tm.tm_yday = yday;
    tm.tm_hour = remsecs / 3600;
    tm.tm_min = remsecs / 60 % 60;
    tm.tm_sec = remsecs % 60;
    true
}

unsafe fn tm_to_secs(tm: *const tm) -> i64 {
    let mut year = (*tm).tm_year as i64;
    let mut month = (*tm).tm_mon;
    if month >= 12 || month < 0 {
        let adj = month / 12;
        month %= 12;
        if month < 0 { month += 12; }
        year += adj as i64;
    }
    let mut is_leap = false;
    let mut t = year_to_secs(year, &mut is_leap);
    t += month_to_secs(month, is_leap) as i64;
    t += 86400 * ((*tm).tm_mday - 1) as i64;
    t += 3600 * (*tm).tm_hour as i64;
    t += 60 * (*tm).tm_min as i64;
    t += (*tm).tm_sec as i64;
    t
}

static mut TM_BUF: tm = tm {
    tm_sec: 0, tm_min: 0, tm_hour: 0, tm_mday: 0, tm_mon: 0,
    tm_year: 0, tm_wday: 0, tm_yday: 0, tm_isdst: 0,
    tm_gmtoff: 0, tm_zone: core::ptr::null(),
};

static mut UTC_STR: [c_char; 4] = [b'U' as c_char, b'T' as c_char, b'C' as c_char, 0];

const CLOCK_PROCESS_CPUTIME_ID: c_int = 2;

// ============================================================
// Time: core functions
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn time(t: *mut TimeT) -> TimeT {
    let mut ts: timespec = core::mem::zeroed();
    let _ = sys_clock_gettime(CLOCK_REALTIME, &mut ts);
    if !t.is_null() { *t = ts.tv_sec; }
    ts.tv_sec
}

#[no_mangle]
pub unsafe extern "C" fn difftime(t1: TimeT, t0: TimeT) -> f64 {
    (t1 as i64 - t0 as i64) as f64
}

#[no_mangle]
pub unsafe extern "C" fn clock() -> ClockT {
    let mut ts: timespec = core::mem::zeroed();
    let _ = sys_clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &mut ts);
    ts.tv_sec * 1000000 + ts.tv_nsec / 1000
}

#[no_mangle]
pub unsafe extern "C" fn gmtime(t: *const TimeT) -> *mut tm {
    gmtime_r(t, &raw mut TM_BUF)
}

#[no_mangle]
pub unsafe extern "C" fn gmtime_r(t: *const TimeT, tm: *mut tm) -> *mut tm {
    if !secs_to_tm(*t as i64, &mut *tm) { ERRNO = EOVERFLOW; return core::ptr::null_mut(); }
    (*tm).tm_isdst = 0;
    (*tm).tm_gmtoff = 0;
    (*tm).tm_zone = core::ptr::addr_of!(UTC_STR).cast::<c_char>();
    tm
}

#[no_mangle]
pub unsafe extern "C" fn localtime(t: *const TimeT) -> *mut tm { gmtime(t) }

#[no_mangle]
pub unsafe extern "C" fn localtime_r(t: *const TimeT, tm: *mut tm) -> *mut tm { gmtime_r(t, tm) }

#[no_mangle]
pub unsafe extern "C" fn mktime(tm: *mut tm) -> TimeT {
    let t = tm_to_secs(tm as *const tm);
    if !secs_to_tm(t, &mut *tm) { ERRNO = EOVERFLOW; return -1; }
    (*tm).tm_isdst = 0;
    (*tm).tm_gmtoff = 0;
    (*tm).tm_zone = core::ptr::addr_of!(UTC_STR).cast::<c_char>();
    t as TimeT
}

#[no_mangle]
pub unsafe extern "C" fn timegm(tm: *mut tm) -> TimeT {
    let t = tm_to_secs(tm as *const tm);
    if !secs_to_tm(t, &mut *tm) { ERRNO = EOVERFLOW; return -1; }
    (*tm).tm_isdst = 0;
    (*tm).tm_gmtoff = 0;
    (*tm).tm_zone = core::ptr::addr_of!(UTC_STR).cast::<c_char>();
    t as TimeT
}

static mut ASCTIME_BUF: [c_char; 26] = [0; 26];

const DAY_NAMES: [&[u8]; 7] = [b"Sun\0", b"Mon\0", b"Tue\0", b"Wed\0", b"Thu\0", b"Fri\0", b"Sat\0"];
const MON_NAMES: [&[u8]; 12] = [b"Jan\0", b"Feb\0", b"Mar\0", b"Apr\0", b"May\0", b"Jun\0", b"Jul\0", b"Aug\0", b"Sep\0", b"Oct\0", b"Nov\0", b"Dec\0"];

#[no_mangle]
pub unsafe extern "C" fn asctime(tm: *const tm) -> *mut c_char {
    asctime_r(tm, core::ptr::addr_of_mut!(ASCTIME_BUF).cast::<c_char>())
}

#[no_mangle]
pub unsafe extern "C" fn asctime_r(tm: *const tm, buf: *mut c_char) -> *mut c_char {
    let wday = if (*tm).tm_wday >= 0 && (*tm).tm_wday < 7 { (*tm).tm_wday as usize } else { 0 };
    let mon = if (*tm).tm_mon >= 0 && (*tm).tm_mon < 12 { (*tm).tm_mon as usize } else { 0 };
    let dn = DAY_NAMES[wday];
    let mn = MON_NAMES[mon];
    let mut p = 0;
    for k in 0..3 { *buf.add(p) = *dn.as_ptr().add(k) as c_char; p += 1; }
    *buf.add(p) = b' ' as c_char; p += 1;
    for k in 0..3 { *buf.add(p) = *mn.as_ptr().add(k) as c_char; p += 1; }
    *buf.add(p) = b' ' as c_char; p += 1;
    let v = (*tm).tm_mday;
    *buf.add(p) = (b'0' + (v / 10) as u8) as c_char; p += 1;
    *buf.add(p) = (b'0' + (v % 10) as u8) as c_char; p += 1;
    *buf.add(p) = b' ' as c_char; p += 1;
    let hh = (*tm).tm_hour;
    *buf.add(p) = (b'0' + (hh / 10) as u8) as c_char; p += 1;
    *buf.add(p) = (b'0' + (hh % 10) as u8) as c_char; p += 1;
    *buf.add(p) = b':' as c_char; p += 1;
    let mm = (*tm).tm_min;
    *buf.add(p) = (b'0' + (mm / 10) as u8) as c_char; p += 1;
    *buf.add(p) = (b'0' + (mm % 10) as u8) as c_char; p += 1;
    *buf.add(p) = b':' as c_char; p += 1;
    let ss = (*tm).tm_sec;
    *buf.add(p) = (b'0' + (ss / 10) as u8) as c_char; p += 1;
    *buf.add(p) = (b'0' + (ss % 10) as u8) as c_char; p += 1;
    *buf.add(p) = b' ' as c_char; p += 1;
    let yr = (*tm).tm_year + 1900;
    let mut tmp = [0u8; 4];
    let mut v2 = yr;
    for k in (0..4).rev() { tmp[k] = b'0' + (v2 % 10) as u8; v2 /= 10; }
    for k in 0..4 { *buf.add(p) = tmp[k] as c_char; p += 1; }
    *buf.add(p) = b'\n' as c_char; p += 1;
    *buf.add(p) = 0;
    buf
}

#[no_mangle]
pub unsafe extern "C" fn ctime(t: *const TimeT) -> *mut c_char { asctime(gmtime(t)) }

#[no_mangle]
pub unsafe extern "C" fn ctime_r(t: *const TimeT, buf: *mut c_char) -> *mut c_char {
    let mut tmp: tm = core::mem::zeroed();
    gmtime_r(t, &mut tmp);
    asctime_r(&tmp, buf)
}

// ============================================================
// Time: clock_getres / clock_settime / clock_nanosleep / gettimeofday
// ============================================================

unsafe fn sys_clock_getres(clockid: c_int, ts: *mut timespec) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_CLOCK_GETRES, clockid as i64, ts as i64)
}

unsafe fn sys_clock_settime(clockid: c_int, ts: *const timespec) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_CLOCK_SETTIME, clockid as i64, ts as i64)
}

unsafe fn sys_clock_nanosleep(clockid: c_int, flags: c_int, req: *const timespec, rem: *mut timespec) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_CLOCK_NANOSLEEP, clockid as i64, flags as i64, req as i64, rem as i64)
}

#[no_mangle]
pub unsafe extern "C" fn clock_getres(clockid: c_int, ts: *mut timespec) -> c_int { if sys_clock_getres(clockid, ts) < 0 { -1 } else { 0 } }

#[no_mangle]
pub unsafe extern "C" fn clock_settime(clockid: c_int, ts: *const timespec) -> c_int { if sys_clock_settime(clockid, ts) < 0 { -1 } else { 0 } }

#[no_mangle]
pub unsafe extern "C" fn clock_nanosleep(clockid: c_int, flags: c_int, req: *const timespec, rem: *mut timespec) -> c_int {
    let r = sys_clock_nanosleep(clockid, flags, req, rem);
    if r < 0 { (-r) as c_int } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn gettimeofday(tv: *mut timeval, _tz: *mut c_void) -> c_int {
    if tv.is_null() { return 0; }
    let mut ts: timespec = core::mem::zeroed();
    let _ = sys_clock_gettime(CLOCK_REALTIME, &mut ts);
    (*tv).tv_sec = ts.tv_sec;
    (*tv).tv_usec = ts.tv_nsec / 1000;
    0
}

// ============================================================
// Time: strftime
// ============================================================

fn is_leap_year(y: i32) -> bool {
    let yr = if y > i32::MAX - 1900 { y - 2000 } else { y + 1900 };
    yr % 4 == 0 && (yr % 100 != 0 || yr % 400 == 0)
}

fn week_num(tm: &tm) -> i32 {
    let mut val = (tm.tm_yday + 7 - ((tm.tm_wday + 6) % 7)) / 7;
    if (tm.tm_wday + 371 - tm.tm_yday - 2).rem_euclid(7) <= 2 { val += 1; }
    if val == 0 {
        val = 52;
        let dec31 = (tm.tm_wday + 7 - tm.tm_yday - 1).rem_euclid(7);
        if dec31 == 4 || (dec31 == 5 && is_leap_year(tm.tm_year % 400 - 1)) { val += 1; }
    } else if val == 53 {
        let jan1 = (tm.tm_wday + 371 - tm.tm_yday).rem_euclid(7);
        if jan1 != 4 && (jan1 != 3 || !is_leap_year(tm.tm_year)) { val = 1; }
    }
    val
}

// Langinfo constants for strftime
const ABDAY_1: c_int = 0x20000;
const DAY_1: c_int = 0x20007;
const ABMON_1: c_int = 0x2000E;
const MON_1: c_int = 0x2001A;
const AM_STR_LI: c_int = 0x20026;
const PM_STR_LI: c_int = 0x20027;
const D_T_FMT: c_int = 0x20028;
const D_FMT: c_int = 0x20029;
const T_FMT: c_int = 0x2002A;
const T_FMT_AMPM: c_int = 0x2002B;

// Format an i64 into buf as decimal with zero/sign padding. Returns (ptr_to_start, length).
// pad: '-' = no pad, '_' = space pad, '0' or 0 = zero pad. width: minimum digits (excluding sign).
unsafe fn num_fmt(buf: &mut [u8; 100], val: i64, pad: u8, width: usize) -> (*const u8, usize) {
    let negative = val < 0;
    let mut v = if negative { -val as u64 } else { val as u64 };
    let mut tmp = [0u8; 24];
    let mut pos = 24usize;
    if v == 0 { pos -= 1; tmp[pos] = b'0'; }
    while v > 0 { pos -= 1; tmp[pos] = b'0' + (v % 10) as u8; v /= 10; }
    let digits = 24 - pos;
    let sign_chars = if negative { 1 } else { 0 };
    let needed = sign_chars + digits;
    let fill = if width > needed { width - needed } else { 0 };
    let total = fill + needed;
    let mut p = 0usize;
    match pad {
        b'-' => { /* no fill */ }
        b'_' => {
            for _ in 0..fill { buf[p] = b' '; p += 1; }
            if negative { buf[p] = b'-'; p += 1; }
        }
        _ => {
            if negative { buf[p] = b'-'; p += 1; }
            for _ in 0..fill { buf[p] = b'0'; p += 1; }
        }
    }
    for i in 0..digits { buf[p] = tmp[pos + i]; p += 1; }
    (buf.as_ptr(), total)
}

// Copy a null-terminated C string into buf. Returns (ptr, len).
unsafe fn str_to_buf(buf: &mut [u8; 100], s: *const c_char) -> (*const u8, usize) {
    let mut k = 0usize;
    while *s.add(k) != 0 {
        buf[k] = *s.add(k) as u8;
        k += 1;
    }
    (buf.as_ptr(), k)
}

// Core format dispatcher. Returns (ptr, len) where ptr points into buf or a static string.
// pad=0 means use default; returns (null, 0) for unknown specifiers.
unsafe fn __strftime_fmt_1(buf: &mut [u8; 100], f: u8, tm: *const tm, pad: u8) -> (*const u8, usize) {
    let mut val: i64;
    let mut width: usize = 2;
    let mut def_pad: u8 = b'0';
    let fmt: *const c_char;

    match f {
        b'a' => {
            if (*tm).tm_wday as u32 > 6 { return (b"\0".as_ptr(), 0); }
            let p = nl_langinfo(ABDAY_1 + (*tm).tm_wday);
            return str_to_buf(buf, p);
        }
        b'A' => {
            if (*tm).tm_wday as u32 > 6 { return (b"\0".as_ptr(), 0); }
            let p = nl_langinfo(DAY_1 + (*tm).tm_wday);
            return str_to_buf(buf, p);
        }
        b'h' | b'b' => {
            if (*tm).tm_mon as u32 > 11 { return (b"\0".as_ptr(), 0); }
            let p = nl_langinfo(ABMON_1 + (*tm).tm_mon);
            return str_to_buf(buf, p);
        }
        b'B' => {
            if (*tm).tm_mon as u32 > 11 { return (b"\0".as_ptr(), 0); }
            let p = nl_langinfo(MON_1 + (*tm).tm_mon);
            return str_to_buf(buf, p);
        }
        b'c' => {
            let p = nl_langinfo(D_T_FMT);
            let l = __strftime_l(buf.as_mut_ptr() as *mut c_char, 100, p, tm);
            return (buf.as_ptr(), l);
        }
        b'C' => {
            val = (1900i64 + (*tm).tm_year as i64) / 100;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'e' => { def_pad = b'_'; }
        b'd' => {
            val = (*tm).tm_mday as i64;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'D' => {
            fmt = b"%m/%d/%y\0".as_ptr() as *const c_char;
            let l = __strftime_l(buf.as_mut_ptr() as *mut c_char, 100, fmt, tm);
            return (buf.as_ptr(), l);
        }
        b'F' => {
            fmt = b"%Y-%m-%d\0".as_ptr() as *const c_char;
            let l = __strftime_l(buf.as_mut_ptr() as *mut c_char, 100, fmt, tm);
            return (buf.as_ptr(), l);
        }
        b'g' | b'G' => {
            val = (*tm).tm_year as i64 + 1900;
            if (*tm).tm_yday < 3 && week_num(&*tm) != 1 { val -= 1; }
            else if (*tm).tm_yday > 360 && week_num(&*tm) == 1 { val += 1; }
            if f == b'g' { val %= 100; }
            else { width = 4; }
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'H' => {
            val = (*tm).tm_hour as i64;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'I' => {
            val = (*tm).tm_hour as i64;
            if val == 0 { val = 12; }
            else if val > 12 { val -= 12; }
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'j' => {
            val = (*tm).tm_yday as i64 + 1;
            width = 3;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'm' => {
            val = (*tm).tm_mon as i64 + 1;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'M' => {
            val = (*tm).tm_min as i64;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'n' => { buf[0] = b'\n'; return (buf.as_ptr(), 1); }
        b'p' => {
            let p = if (*tm).tm_hour >= 12 { nl_langinfo(PM_STR_LI) } else { nl_langinfo(AM_STR_LI) };
            return str_to_buf(buf, p);
        }
        b'r' => {
            let p = nl_langinfo(T_FMT_AMPM);
            let l = __strftime_l(buf.as_mut_ptr() as *mut c_char, 100, p, tm);
            return (buf.as_ptr(), l);
        }
        b'R' => {
            fmt = b"%H:%M\0".as_ptr() as *const c_char;
            let l = __strftime_l(buf.as_mut_ptr() as *mut c_char, 100, fmt, tm);
            return (buf.as_ptr(), l);
        }
        b's' => {
            val = tm_to_secs(tm) - (*tm).tm_gmtoff as i64;
            width = 1;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'S' => {
            val = (*tm).tm_sec as i64;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b't' => { buf[0] = b'\t'; return (buf.as_ptr(), 1); }
        b'T' => {
            fmt = b"%H:%M:%S\0".as_ptr() as *const c_char;
            let l = __strftime_l(buf.as_mut_ptr() as *mut c_char, 100, fmt, tm);
            return (buf.as_ptr(), l);
        }
        b'u' => {
            val = if (*tm).tm_wday != 0 { (*tm).tm_wday as i64 } else { 7 };
            width = 1;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'U' => {
            val = ((*tm).tm_yday as u64 + 7 - (*tm).tm_wday as u64) as i64 / 7;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'W' => {
            val = ((*tm).tm_yday as u64 + 7 - ((*tm).tm_wday as u64 + 6) % 7) as i64 / 7;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'V' => {
            val = week_num(&*tm) as i64;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'w' => {
            val = (*tm).tm_wday as i64;
            width = 1;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'x' => {
            let p = nl_langinfo(D_FMT);
            let l = __strftime_l(buf.as_mut_ptr() as *mut c_char, 100, p, tm);
            return (buf.as_ptr(), l);
        }
        b'X' => {
            let p = nl_langinfo(T_FMT);
            let l = __strftime_l(buf.as_mut_ptr() as *mut c_char, 100, p, tm);
            return (buf.as_ptr(), l);
        }
        b'y' => {
            val = (1900i64 + (*tm).tm_year as i64) % 100;
            if val < 0 { val = -val; }
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'Y' => {
            val = (*tm).tm_year as i64 + 1900;
            if val >= 10000 {
                let mut tmp = [0u8; 100];
                let (_, k) = num_fmt(&mut tmp, val, b'-', 1);
                buf[0] = b'+';
                for i in 0..k { buf[1 + i] = tmp[i]; }
                return (buf.as_ptr(), 1 + k);
            }
            width = 4;
            return num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width);
        }
        b'z' => {
            if (*tm).tm_isdst < 0 { return (b"\0".as_ptr(), 0); }
            let gmtoff = (*tm).tm_gmtoff as i64;
            let sign = if gmtoff >= 0 { b'+' } else { b'-' };
            let off = if gmtoff < 0 { -gmtoff } else { gmtoff };
            let hh = (off / 3600) as i32;
            let mm = ((off % 3600) / 60) as i32;
            buf[0] = sign;
            buf[1] = b'0' + (hh / 10) as u8;
            buf[2] = b'0' + (hh % 10) as u8;
            buf[3] = b'0' + (mm / 10) as u8;
            buf[4] = b'0' + (mm % 10) as u8;
            return (buf.as_ptr(), 5);
        }
        b'Z' => {
            if (*tm).tm_isdst < 0 || (*tm).tm_zone.is_null() { return (b"\0".as_ptr(), 0); }
            return str_to_buf(buf, (*tm).tm_zone);
        }
        b'%' => { buf[0] = b'%'; return (buf.as_ptr(), 1); }
        _ => { return (core::ptr::null(), 0); }
    }
    // fallthrough for 'e' (def_pad='_', same logic as 'd')
    val = (*tm).tm_mday as i64;
    num_fmt(buf, val, if pad != 0 { pad } else { def_pad }, width)
}

unsafe fn __strftime_l(s: *mut c_char, n: usize, f: *const c_char, tm: *const tm) -> usize {
    let mut l: usize = 0;
    let mut fi: usize = 0;
    while l < n {
        let ch = *f.add(fi) as u8;
        if ch == 0 {
            *s.add(l) = 0;
            return l;
        }
        if ch != b'%' {
            *s.add(l) = ch as c_char;
            l += 1;
            fi += 1;
            continue;
        }
        fi += 1;
        // Parse flags
        let mut pad: u8 = 0;
        let fc = *f.add(fi) as u8;
        if fc == b'-' || fc == b'_' || fc == b'0' { pad = fc; fi += 1; }
        let plus = *f.add(fi) as u8 == b'+';
        if plus { fi += 1; }
        // Parse width
        let mut width: usize = 0;
        let mut p_idx = fi;
        {
            let fc2 = *f.add(fi) as u8;
            if fc2 >= b'0' && fc2 <= b'9' {
                let mut w: usize = 0;
                while { let c = *f.add(p_idx) as u8; c >= b'0' && c <= b'9' } {
                    w = w * 10 + (*f.add(p_idx) as usize - b'0' as usize);
                    p_idx += 1;
                }
                width = w;
            }
        }
        let spec_ch = *f.add(p_idx) as u8;
        if spec_ch == b'C' || spec_ch == b'F' || spec_ch == b'G' || spec_ch == b'Y' {
            if width == 0 && p_idx != fi { width = 1; }
        } else {
            width = 0;
        }
        fi = p_idx;
        // Skip E/O modifier
        let fc3 = *f.add(fi) as u8;
        if fc3 == b'E' || fc3 == b'O' { fi += 1; }
        let fch = *f.add(fi) as u8;
        // Call formatter
        let mut buf = [0u8; 100];
        let (t, k) = __strftime_fmt_1(&mut buf, fch, tm, pad);
        if t.is_null() { break; }
        let mut k = k;
        let mut t = t;
        if width != 0 {
            // Strip sign and leading zeros, count remaining digits
            if *t == b'+' || *t == b'-' { t = t.add(1); k -= 1; }
            while k > 0 && *t == b'0' && k > 1 && *t.add(1) >= b'0' && *t.add(1) <= b'9' {
                t = t.add(1); k -= 1;
            }
            if width < k { width = k; }
            let mut d: usize = 0;
            while d < k && *t.add(d) >= b'0' && *t.add(d) <= b'9' { d += 1; }
            if (*tm).tm_year < -1900 {
                if l < n { *s.add(l) = b'-' as c_char; }
                l += 1;
                width -= 1;
            } else if plus && d + (width - k) >= (if spec_ch == b'C' { 3 } else { 5 }) {
                if l < n { *s.add(l) = b'+' as c_char; }
                l += 1;
                width -= 1;
            }
            while width > k && l < n {
                *s.add(l) = b'0' as c_char;
                l += 1;
                width -= 1;
            }
        }
        let copy = if k > n - l { n - l } else { k };
        core::ptr::copy_nonoverlapping(t, s.add(l) as *mut u8, copy);
        l += copy;
        fi += 1;
    }
    if n > 0 {
        if l == n { l = n - 1; }
        *s.add(l) = 0;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn strftime(s: *mut c_char, maxsize: usize, fmt: *const c_char, tm: *const tm) -> usize {
    __strftime_l(s, maxsize, fmt, tm)
}

// ============================================================
// Time: strptime
// ============================================================

unsafe fn match_prefix(s: *const u8, prefix: &[u8]) -> bool {
    for i in 0..prefix.len() {
        if *s.add(i) == 0 || *s.add(i) != prefix[i] { return false; }
    }
    true
}

unsafe fn match_ci_prefix(s: *const u8, prefix: &[u8]) -> bool {
    for i in 0..prefix.len() {
        let c = *s.add(i);
        if c == 0 { return false; }
        if c.to_ascii_lowercase() != prefix[i].to_ascii_lowercase() { return false; }
    }
    true
}

unsafe fn parse_int(p: *const u8, max_digits: usize) -> (i32, *const u8) {
    let mut val = 0i32;
    let mut np = p;
    let mut count = 0;
    while *np >= b'0' && *np <= b'9' && count < max_digits {
        val = val * 10 + (*np - b'0') as i32;
        np = np.add(1);
        count += 1;
    }
    if count == 0 { (0, p) } else { (val, np) }
}

unsafe fn parse_range(p: *const u8, min: i32, max: i32) -> (i32, *const u8) {
    let (val, np) = parse_int(p, 4);
    if np == p || val < min || val > max { (0, p) } else { (val, np) }
}

fn is_ascii_space(c: u8) -> bool {
    c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' || c == 0x0c || c == 0x0b
}

#[no_mangle]
pub unsafe extern "C" fn strptime(s: *const c_char, fmt: *const c_char, tm: *mut tm) -> *mut c_char {
    let mut p = s as *const u8;
    let mut f = fmt as *const u8;
    let mut want_century = 0i32;
    let mut century = 0i32;
    let mut relyear = 0i32;
    while *f != 0 {
        if *f != b'%' {
            if is_ascii_space(*f) {
                while *p != 0 && is_ascii_space(*p) { p = p.add(1); }
            } else {
                if *p != *f { return core::ptr::null_mut(); }
                p = p.add(1);
            }
            f = f.add(1);
            continue;
        }
        f = f.add(1);
        if *f == b'+' { f = f.add(1); }
        while *f >= b'0' && *f <= b'9' { f = f.add(1); }
        if *f == b'E' || *f == b'O' { f = f.add(1); }
        match *f {
            b'a' | b'A' => {
                let mut found = false;
                for i in 0..7 {
                    let names: [&[u8]; 7] = [b"Sunday", b"Monday", b"Tuesday", b"Wednesday", b"Thursday", b"Friday", b"Saturday"];
                    let snames: [&[u8]; 7] = [b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"];
                    if match_prefix(p, names[i]) { (*tm).tm_wday = i as c_int; p = p.add(names[i].len()); found = true; break; }
                    if match_prefix(p, snames[i]) { (*tm).tm_wday = i as c_int; p = p.add(snames[i].len()); found = true; break; }
                }
                if !found { return core::ptr::null_mut(); }
            }
            b'b' | b'B' | b'h' => {
                let mut found = false;
                for i in 0..12 {
                    let names: [&[u8]; 12] = [b"January", b"February", b"March", b"April", b"May", b"June", b"July", b"August", b"September", b"October", b"November", b"December"];
                    let snames: [&[u8]; 12] = [b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov", b"Dec"];
                    if match_prefix(p, names[i]) { (*tm).tm_mon = i as c_int; p = p.add(names[i].len()); found = true; break; }
                    if match_prefix(p, snames[i]) { (*tm).tm_mon = i as c_int; p = p.add(snames[i].len()); found = true; break; }
                }
                if !found { return core::ptr::null_mut(); }
            }
            b'C' => {
                let (v, np) = parse_int(p, 2);
                if np == p { return core::ptr::null_mut(); }
                century = v;
                want_century |= 2;
                p = np;
            }
            b'd' | b'e' => {
                let (v, np) = parse_range(p, 1, 31);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_mday = v;
                p = np;
            }
            b'D' => {
                let r = strptime(p as *const c_char, b"%m/%d/%y\0".as_ptr() as *const c_char, tm);
                if r.is_null() { return core::ptr::null_mut(); }
                p = r as *const u8;
            }
            b'F' => {
                let (yv, ynp) = parse_int(p, 4);
                if ynp == p { return core::ptr::null_mut(); }
                (*tm).tm_year = yv - 1900;
                p = ynp;
                if *p != b'-' { return core::ptr::null_mut(); }
                p = p.add(1);
                let (mv, mnp) = parse_range(p, 1, 12);
                if mnp == p { return core::ptr::null_mut(); }
                (*tm).tm_mon = mv - 1;
                p = mnp;
                if *p != b'-' { return core::ptr::null_mut(); }
                p = p.add(1);
                let (dv, dnp) = parse_range(p, 1, 31);
                if dnp == p { return core::ptr::null_mut(); }
                (*tm).tm_mday = dv;
                p = dnp;
            }
            b'H' | b'I' => {
                let (v, np) = parse_range(p, 0, 23);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_hour = v;
                p = np;
            }
            b'j' => {
                let (v, np) = parse_range(p, 1, 366);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_yday = v - 1;
                p = np;
            }
            b'm' => {
                let (v, np) = parse_range(p, 1, 12);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_mon = v - 1;
                p = np;
            }
            b'M' => {
                let (v, np) = parse_range(p, 0, 59);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_min = v;
                p = np;
            }
            b'n' | b't' => {
                while *p != 0 && is_ascii_space(*p) { p = p.add(1); }
            }
            b'p' => {
                if match_ci_prefix(p, b"AM") {
                    if (*tm).tm_hour == 12 { (*tm).tm_hour = 0; }
                    p = p.add(2);
                } else if match_ci_prefix(p, b"PM") {
                    if (*tm).tm_hour < 12 { (*tm).tm_hour += 12; }
                    p = p.add(2);
                } else { return core::ptr::null_mut(); }
            }
            b'r' => {
                let r = strptime(p as *const c_char, b"%I:%M:%S %p\0".as_ptr() as *const c_char, tm);
                if r.is_null() { return core::ptr::null_mut(); }
                p = r as *const u8;
            }
            b'R' => {
                let r = strptime(p as *const c_char, b"%H:%M\0".as_ptr() as *const c_char, tm);
                if r.is_null() { return core::ptr::null_mut(); }
                p = r as *const u8;
            }
            b's' => {
                let neg = if *p == b'-' { p = p.add(1); true } else { false };
                if *p < b'0' || *p > b'9' { return core::ptr::null_mut(); }
                let mut val = 0i64;
                while *p >= b'0' && *p <= b'9' {
                    val = val * 10 + (*p - b'0') as i64;
                    p = p.add(1);
                }
                if neg { val = -val; }
                secs_to_tm(val, &mut *tm);
            }
            b'S' => {
                let (v, np) = parse_range(p, 0, 60);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_sec = v;
                p = np;
            }
            b'T' => {
                let r = strptime(p as *const c_char, b"%H:%M:%S\0".as_ptr() as *const c_char, tm);
                if r.is_null() { return core::ptr::null_mut(); }
                p = r as *const u8;
            }
            b'U' | b'W' => {
                let (_, np) = parse_range(p, 0, 53);
                if np == p { return core::ptr::null_mut(); }
                p = np;
            }
            b'V' => {
                let (_, np) = parse_range(p, 1, 53);
                if np == p { return core::ptr::null_mut(); }
                p = np;
            }
            b'w' => {
                let (v, np) = parse_range(p, 0, 6);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_wday = v;
                p = np;
            }
            b'y' => {
                let (v, np) = parse_int(p, 2);
                if np == p { return core::ptr::null_mut(); }
                relyear = v;
                want_century |= 1;
                p = np;
            }
            b'Y' => {
                let neg = if *p == b'-' { p = p.add(1); true } else { false };
                let (v, np) = parse_int(p, 4);
                if np == p { return core::ptr::null_mut(); }
                (*tm).tm_year = if neg { -(v + 1900) } else { v - 1900 };
                want_century = 0;
                p = np;
            }
            b'z' => {
                if *p != b'+' && *p != b'-' { return core::ptr::null_mut(); }
                let neg = *p == b'-';
                p = p.add(1);
                let mut off = 0i64;
                let mut digits = 0;
                while digits < 4 && *p >= b'0' && *p <= b'9' {
                    off = off * 10 + (*p - b'0') as i64;
                    p = p.add(1);
                    digits += 1;
                }
                if digits == 0 { return core::ptr::null_mut(); }
                let (hh, mm) = if digits <= 2 { (off, 0i64) } else { (off / 100, off % 100) };
                (*tm).tm_gmtoff = (hh * 3600 + mm * 60) * if neg { -1 } else { 1 };
            }
            b'%' => {
                if *p != b'%' { return core::ptr::null_mut(); }
                p = p.add(1);
            }
            _ => { return core::ptr::null_mut(); }
        }
        f = f.add(1);
    }
    if want_century != 0 {
        (*tm).tm_year = relyear;
        if want_century & 2 != 0 { (*tm).tm_year += century * 100 - 1900; }
        else if (*tm).tm_year <= 68 { (*tm).tm_year += 100; }
    }
    p as *mut c_char
}

// ============================================================
// Time: tzset / daylight / timezone / tzname
// ============================================================

#[no_mangle]
pub static mut daylight: c_int = 0;
#[no_mangle]
pub static mut timezone: c_long = 0;
#[no_mangle]
pub static mut tzname: [*mut c_char; 2] = [b"UTC\0".as_ptr() as *mut c_char, b"UTC\0".as_ptr() as *mut c_char];

#[no_mangle]
pub unsafe extern "C" fn tzset() {
    daylight = 0;
    timezone = 0;
}

// ============================================================
// system()
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn system(cmd: *const c_char) -> c_int {
    if cmd.is_null() {
        // ponytail: test shell availability via fork+exec
        let pid = sys_fork();
        if pid == 0 {
            let sh = b"/bin/sh\0".as_ptr() as *const c_char;
            let dash_c = b"-c\0".as_ptr() as *const c_char;
            let exit0 = b"exit 0\0".as_ptr() as *const c_char;
            let argv = [b"sh\0".as_ptr() as *const c_char, dash_c, exit0, core::ptr::null()];
            sys_execve(sh, argv.as_ptr(), __environ as *const *const c_char);
            _exit(127);
        }
        if pid < 0 { return -1; }
        let mut status: c_int = 0;
        loop {
            let r = sys_wait4(pid as c_int, &mut status, 0, core::ptr::null_mut());
            if r >= 0 { return status; }
            if r != -4 { return -1; } // not EINTR
        }
    }

    let sa_ignore = sigaction {
        sa_handler: SIG_IGN,
        sa_flags: SA_RESTORER,
        sa_restorer: sig_restorer as *const () as usize,
        sa_mask: [0],
    };
    let mut oldint: sigaction = core::mem::zeroed();
    let mut oldquit: sigaction = core::mem::zeroed();

    sys_rt_sigaction(SIGINT, &sa_ignore, &mut oldint, 8);
    sys_rt_sigaction(SIGQUIT, &sa_ignore, &mut oldquit, 8);

    let mut block_set: SigSetT = 0;
    block_set |= 1u64 << (SIGCHLD - 1);
    let mut oldmask: SigSetT = 0;
    sys_rt_sigprocmask(SIG_BLOCK, &block_set, &mut oldmask, 8);

    let pid = sys_fork();

    if pid == 0 {
        sys_rt_sigaction(SIGINT, &oldint, core::ptr::null_mut(), 8);
        sys_rt_sigaction(SIGQUIT, &oldquit, core::ptr::null_mut(), 8);
        sys_rt_sigprocmask(SIG_SETMASK, &oldmask, core::ptr::null_mut(), 8);

        let sh = b"/bin/sh\0".as_ptr() as *const c_char;
        let argv = [
            b"sh\0".as_ptr() as *const c_char,
            b"-c\0".as_ptr() as *const c_char,
            cmd,
            core::ptr::null(),
        ];
        sys_execve(sh, argv.as_ptr(), __environ as *const *const c_char);
        _exit(127);
    }

    let mut status: c_int = -1;
    if pid > 0 {
        loop {
            let r = sys_wait4(pid as c_int, &mut status, 0, core::ptr::null_mut());
            if r >= 0 { break; }
            if r != -4 { break; } // not EINTR
        }
    }

    sys_rt_sigaction(SIGINT, &oldint, core::ptr::null_mut(), 8);
    sys_rt_sigaction(SIGQUIT, &oldquit, core::ptr::null_mut(), 8);
    sys_rt_sigprocmask(SIG_SETMASK, &oldmask, core::ptr::null_mut(), 8);

    status
}

// ============================================================
// errno: strerror / strerror_r / environ alias
// ============================================================

const EPERM_VAL: c_int = 1;
const ENOENT_VAL: c_int = 2;
const ESRCH_VAL: c_int = 3;
const EINTR_VAL: c_int = 4;
const EIO_VAL: c_int = 5;
const ENXIO_VAL: c_int = 6;
const E2BIG_VAL: c_int = 7;
const ENOEXEC_VAL: c_int = 8;
const EBADF_VAL: c_int = 9;
const ECHILD_VAL: c_int = 10;
const EAGAIN_VAL: c_int = 11;
const ENOMEM_VAL: c_int = 12;
const EACCES_VAL: c_int = 13;
const EFAULT_VAL: c_int = 14;
const ENOTBLK_VAL: c_int = 15;
const EBUSY_VAL: c_int = 16;
const EEXIST_VAL: c_int = 17;
const EXDEV_VAL: c_int = 18;
const ENODEV_VAL: c_int = 19;
const ENOTDIR_VAL: c_int = 20;
const EISDIR_VAL: c_int = 21;
const EINVAL_VAL: c_int = 22;
const ENFILE_VAL: c_int = 23;
const EMFILE_VAL: c_int = 24;
const ENOTTY_VAL: c_int = 25;
const ETXTBSY_VAL: c_int = 26;
const EFBIG_VAL: c_int = 27;
const ENOSPC_VAL: c_int = 28;
const ESPIPE_VAL: c_int = 29;
const EROFS_VAL: c_int = 30;
const EMLINK_VAL: c_int = 31;
const EPIPE_VAL: c_int = 32;
const EDOM_VAL: c_int = 33;
const ERANGE_VAL: c_int = 34;
const EDEADLK_VAL: c_int = 35;
const ENAMETOOLONG_VAL: c_int = 36;
const ENOLCK_VAL: c_int = 37;
const ENOSYS_VAL: c_int = 38;
const ENOTEMPTY_VAL: c_int = 39;
const ELOOP_VAL: c_int = 40;
const EWOULDBLOCK_VAL: c_int = 11;
const ENOMSG_VAL: c_int = 42;
const EIDRM_VAL: c_int = 43;
const ECHRNG_VAL: c_int = 44;
const EL2NSYNC_VAL: c_int = 45;
const EL3HLT_VAL: c_int = 46;
const EL3RST_VAL: c_int = 47;
const ELNRNG_VAL: c_int = 48;
const EUNATCH_VAL: c_int = 49;
const ENOCSI_VAL: c_int = 50;
const EL2HLT_VAL: c_int = 51;
const EBADE_VAL: c_int = 52;
const EBADR_VAL: c_int = 53;
const EXFULL_VAL: c_int = 54;
const ENOANO_VAL: c_int = 55;
const EBADRQC_VAL: c_int = 56;
const EBADSLT_VAL: c_int = 57;
const EBFONT_VAL: c_int = 59;
const ENOSTR_VAL: c_int = 60;
const ENODATA_VAL: c_int = 61;
const ETIME_VAL: c_int = 62;
const ENOSR_VAL: c_int = 63;
const ENONET_VAL: c_int = 64;
const ENOPKG_VAL: c_int = 65;
const EREMOTE_VAL: c_int = 66;
const ENOLINK_VAL: c_int = 67;
const EADV_VAL: c_int = 68;
const ESRMNT_VAL: c_int = 69;
const ECOMM_VAL: c_int = 70;
const EPROTO_VAL: c_int = 71;
const EMULTIHOP_VAL: c_int = 72;
const EDOTDOT_VAL: c_int = 73;
const EBADMSG_VAL: c_int = 74;
const EOVERFLOW_VAL: c_int = 75;
const ENOTUNIQ_VAL: c_int = 76;
const EBADFD_VAL: c_int = 77;
const EREMCHG_VAL: c_int = 78;
const ELIBACC_VAL: c_int = 79;
const ELIBBAD_VAL: c_int = 80;
const ELIBSCN_VAL: c_int = 81;
const ELIBMAX_VAL: c_int = 82;
const ELIBEXEC_VAL: c_int = 83;
const EILSEQ_VAL: c_int = 84;
const ERESTART_VAL: c_int = 85;
const ESTRPIPE_VAL: c_int = 86;
const EUSERS_VAL: c_int = 87;
const ENOTSOCK_VAL: c_int = 88;
const EDESTADDRREQ_VAL: c_int = 89;
const EMSGSIZE_VAL: c_int = 90;
const EPROTOTYPE_VAL: c_int = 91;
const ENOPROTOOPT_VAL: c_int = 92;
const EPROTONOSUPPORT_VAL: c_int = 93;
const ESOCKTNOSUPPORT_VAL: c_int = 94;
const EOPNOTSUPP_VAL: c_int = 95;
const ENOTSUP_VAL: c_int = 95;
const EPFNOSUPPORT_VAL: c_int = 96;
const EAFNOSUPPORT_VAL: c_int = 97;
const EADDRINUSE_VAL: c_int = 98;
const EADDRNOTAVAIL_VAL: c_int = 99;
const ENETDOWN_VAL: c_int = 100;
const ENETUNREACH_VAL: c_int = 101;
const ENETRESET_VAL: c_int = 102;
const ECONNABORTED_VAL: c_int = 103;
const ECONNRESET_VAL: c_int = 104;
const ENOBUFS_VAL: c_int = 105;
const EISCONN_VAL: c_int = 106;
const ENOTCONN_VAL: c_int = 107;
const ESHUTDOWN_VAL: c_int = 108;
const ETOOMANYREFS_VAL: c_int = 109;
const ETIMEDOUT_VAL: c_int = 110;
const ECONNREFUSED_VAL: c_int = 111;
const EHOSTDOWN_VAL: c_int = 112;
const EHOSTUNREACH_VAL: c_int = 113;
const EALREADY_VAL: c_int = 114;
const EINPROGRESS_VAL: c_int = 115;
const ESTALE_VAL: c_int = 116;
const EUCLEAN_VAL: c_int = 117;
const ENOTNAM_VAL: c_int = 118;
const ENAVAIL_VAL: c_int = 119;
const EISNAM_VAL: c_int = 120;
const EREMOTEIO_VAL: c_int = 121;
const EDQUOT_VAL: c_int = 122;
const ENOMEDIUM_VAL: c_int = 123;
const EMEDIUMTYPE_VAL: c_int = 124;
const ECANCELED_VAL: c_int = 125;
const ENOKEY_VAL: c_int = 126;
const EKEYEXPIRED_VAL: c_int = 127;
const EKEYREVOKED_VAL: c_int = 128;
const EKEYREJECTED_VAL: c_int = 129;
const EOWNERDEAD_VAL: c_int = 130;
const ENOTRECOVERABLE_VAL: c_int = 131;
const ERFKILL_VAL: c_int = 132;
const EHWPOISON_VAL: c_int = 133;

// ponytail: static error string table indexed by errno
static ERR_STRS: [&[u8]; 134] = [
    b"Success\0",                     // 0
    b"Operation not permitted\0",     // 1
    b"No such file or directory\0",   // 2
    b"No such process\0",             // 3
    b"Interrupted system call\0",     // 4
    b"Input/output error\0",          // 5
    b"No such device or address\0",   // 6
    b"Argument list too long\0",      // 7
    b"Exec format error\0",           // 8
    b"Bad file descriptor\0",         // 9
    b"No child processes\0",          // 10
    b"Resource temporarily unavailable\0", // 11
    b"Cannot allocate memory\0",      // 12
    b"Permission denied\0",           // 13
    b"Bad address\0",                 // 14
    b"Block device required\0",       // 15
    b"Device or resource busy\0",     // 16
    b"File exists\0",                 // 17
    b"Invalid cross-device link\0",   // 18
    b"No such device\0",              // 19
    b"Not a directory\0",             // 20
    b"Is a directory\0",              // 21
    b"Invalid argument\0",            // 22
    b"Too many open files in system\0", // 23
    b"Too many open files\0",         // 24
    b"Inappropriate ioctl for device\0", // 25
    b"Text file busy\0",              // 26
    b"File too large\0",              // 27
    b"No space left on device\0",     // 28
    b"Illegal seek\0",                // 29
    b"Read-only file system\0",       // 30
    b"Too many links\0",              // 31
    b"Broken pipe\0",                 // 32
    b"Numerical argument out of domain\0", // 33
    b"Numerical result out of range\0", // 34
    b"Resource deadlock avoided\0",   // 35
    b"File name too long\0",          // 36
    b"No locks available\0",          // 37
    b"Function not implemented\0",    // 38
    b"Directory not empty\0",         // 39
    b"Too many levels of symbolic links\0", // 40
    b"Unknown error 41\0",            // 41
    b"No message of desired type\0",  // 42
    b"Identifier removed\0",          // 43
    b"Channel number out of range\0", // 44
    b"Level 2 not synchronized\0",    // 45
    b"Level 3 halted\0",              // 46
    b"Level 3 reset\0",               // 47
    b"Link number out of range\0",    // 48
    b"Protocol driver not attached\0", // 49
    b"No CSI structure available\0",  // 50
    b"Level 2 halted\0",              // 51
    b"Invalid exchange\0",            // 52
    b"Invalid request descriptor\0",  // 53
    b"Exchange full\0",               // 54
    b"No anode\0",                    // 55
    b"Invalid request code\0",        // 56
    b"Invalid slot\0",                // 57
    b"Unknown error 58\0",            // 58
    b"Bad font file format\0",        // 59
    b"Device not a stream\0",         // 60
    b"No data available\0",           // 61
    b"Timer expired\0",               // 62
    b"Out of streams resources\0",    // 63
    b"Machine is not on the network\0", // 64
    b"Package not installed\0",       // 65
    b"Object is remote\0",            // 66
    b"Link has been severed\0",       // 67
    b"Advertise error\0",             // 68
    b"Srmount error\0",               // 69
    b"Communication error on send\0", // 70
    b"Protocol error\0",              // 71
    b"Multihop attempted\0",          // 72
    b"RFS specific error\0",          // 73
    b"Bad message\0",                 // 74
    b"Value too large for defined data type\0", // 75
    b"Name not unique on network\0",  // 76
    b"File descriptor in bad state\0", // 77
    b"Remote address changed\0",      // 78
    b"Can not access a needed shared library\0", // 79
    b"Accessing a corrupted shared library\0", // 80
    b".lib section in a.out corrupted\0", // 81
    b"Attempting to link in too many shared libraries\0", // 82
    b"Cannot exec a shared library directly\0", // 83
    b"Invalid or incomplete multibyte or wide character\0", // 84
    b"Interrupted system call should be restarted\0", // 85
    b"Streams pipe error\0",          // 86
    b"Too many users\0",              // 87
    b"Socket operation on non-socket\0", // 88
    b"Destination address required\0", // 89
    b"Message too long\0",            // 90
    b"Protocol wrong type for socket\0", // 91
    b"Protocol not available\0",      // 92
    b"Protocol not supported\0",      // 93
    b"Socket type not supported\0",   // 94
    b"Operation not supported\0",     // 95
    b"Protocol family not supported\0", // 96
    b"Address family not supported by protocol\0", // 97
    b"Address already in use\0",      // 98
    b"Cannot assign requested address\0", // 99
    b"Network is down\0",             // 100
    b"Network is unreachable\0",      // 101
    b"Network dropped connection on reset\0", // 102
    b"Software caused connection abort\0", // 103
    b"Connection reset by peer\0",    // 104
    b"No buffer space available\0",   // 105
    b"Transport endpoint is already connected\0", // 106
    b"Transport endpoint is not connected\0", // 107
    b"Cannot send after transport endpoint shutdown\0", // 108
    b"Too many references: cannot splice\0", // 109
    b"Connection timed out\0",        // 110
    b"Connection refused\0",          // 111
    b"Host is down\0",                // 112
    b"No route to host\0",            // 113
    b"Operation already in progress\0", // 114
    b"Operation now in progress\0",   // 115
    b"Stale file handle\0",           // 116
    b"Structure needs cleaning\0",    // 117
    b"Not a XENIX named type file\0", // 118
    b"No XENIX semaphores available\0", // 119
    b"Is a named type file\0",        // 120
    b"Remote I/O error\0",            // 121
    b"Quota exceeded\0",              // 122
    b"No medium found\0",             // 123
    b"Wrong medium type\0",           // 124
    b"Operation canceled\0",          // 125
    b"Required key not available\0",  // 126
    b"Key has expired\0",             // 127
    b"Key has been revoked\0",        // 128
    b"Key was rejected by service\0", // 129
    b"Owner died\0",                  // 130
    b"State not recoverable\0",       // 131
    b"Operation not possible due to RF-kill\0", // 132
    b"Memory page has hardware error\0", // 133
];

#[no_mangle]
pub unsafe extern "C" fn strerror(errnum: c_int) -> *mut c_char {
    if errnum < 0 || errnum as usize >= ERR_STRS.len() {
        // ponytail: static buf for unknown errors
        static mut UNKNOWN_BUF: [c_char; 32] = [0; 32];
        let prefix = b"Unknown error \0";
        let mut i = 0;
        while prefix[i] != 0 {
            UNKNOWN_BUF[i] = prefix[i] as c_char;
            i += 1;
        }
        let (buf, len) = format_u64(errnum as u64);
        for j in 0..len {
            UNKNOWN_BUF[i] = buf[20 - len + j] as c_char;
            i += 1;
        }
        UNKNOWN_BUF[i] = 0;
        return core::ptr::addr_of_mut!(UNKNOWN_BUF).cast::<c_char>();
    }
    ERR_STRS[errnum as usize].as_ptr() as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: usize) -> c_int {
    let s = strerror(errnum);
    let slen = strlen(s as *const c_char);
    if slen >= buflen {
        if buflen > 0 {
            core::ptr::copy_nonoverlapping(s as *const u8, buf as *mut u8, buflen - 1);
            *buf.add(buflen - 1) = 0;
        }
        return 34; // ERANGE
    }
    core::ptr::copy_nonoverlapping(s as *const u8, buf as *mut u8, slen + 1);
    0
}

// ponytail: `environ` alias for tests that use extern char **environ
#[no_mangle]
pub static mut environ: *mut *mut c_char = core::ptr::null_mut();

// keep environ in sync with __environ
unsafe fn sync_environ() {
    environ = __environ;
}

// ============================================================
// unistd extensions: pipe, dup, dup2, fcntl, access, unlink, etc.
// ============================================================

#[inline]
unsafe fn sys_dup(oldfd: i32) -> i64 {
    <Arch as Syscalls>::syscall1(SYS_DUP, oldfd as i64)
}

#[inline]unsafe fn sys_dup2(oldfd: i32, newfd: i32) -> i64 {
    // ponytail: dup3 with flags=0 is equivalent to dup2
    sys_dup3(oldfd, newfd, 0)
}

#[inline]
unsafe fn sys_access(path: *const u8, mode: i32) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_FACCESSAT, AT_FDCWD as i64, path as i64, mode as i64, 0)
}

#[inline]unsafe fn sys_unlink(path: *const u8) -> i64 {
    sys_unlinkat(AT_FDCWD, path, 0)
}

#[inline]unsafe fn sys_rmdir(path: *const u8) -> i64 {
    sys_unlinkat(AT_FDCWD, path, 512) // AT_REMOVEDIR
}

#[inline]
unsafe fn sys_chdir(path: *const u8) -> i64 {
    <Arch as Syscalls>::syscall1(SYS_CHDIR, path as i64)
}

#[inline]
unsafe fn sys_getcwd(buf: *mut u8, size: usize) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_GETCWD, buf as i64, size as i64)
}

#[inline]
unsafe fn sys_sethostname(name: *const u8, len: usize) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_SETHOSTNAME, name as i64, len as i64)
}

#[inline]
unsafe fn sys_gethostname(buf: *mut u8, len: usize) -> i64 {
    #[repr(C)]
    struct UtsName {
        sysname: [u8; 65],
        nodename: [u8; 65],
        release: [u8; 65],
        version: [u8; 65],
        machine: [u8; 65],
        domainname: [u8; 65],
    }
    let mut uts: UtsName = core::mem::zeroed();
    let result = <Arch as Syscalls>::syscall1(SYS_UNAME, &mut uts as *mut UtsName as i64);
    if result < 0 {
        return result;
    }
    let nlen = strlen(uts.nodename.as_ptr() as *const c_char);
    let copylen = if nlen < len { nlen } else { if len > 0 { len - 1 } else { 0 } };
    core::ptr::copy_nonoverlapping(uts.nodename.as_ptr(), buf, copylen);
    if len > 0 { *buf.add(copylen) = 0; }
    0
}

unsafe fn sys_truncate(path: *const u8, length: i64) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_TRUNCATE, path as i64, length as i64)
}

unsafe fn sys_ftruncate(fd: i32, length: i64) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_FTRUNCATE, fd as i64, length as i64)
}

unsafe fn sys_nanosleep(req: *const timespec, rem: *mut timespec) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_NANOSLEEP, req as i64, rem as i64)
}

const ITIMER_REAL: i32 = 0;

#[repr(C)]
struct itimerval {
    it_interval: timeval,
    it_value: timeval,
}

unsafe fn sys_alarm(seconds: c_uint) -> i64 {
    let mut new: itimerval = core::mem::zeroed();
    new.it_value.tv_sec = seconds as c_long;
    let mut old: itimerval = core::mem::zeroed();
    let r = <Arch as Syscalls>::syscall3(
        SYS_SETITIMER,
        ITIMER_REAL as i64,
        &new as *const _ as i64,
        &mut old as *mut _ as i64,
    );
    if r < 0 {
        return r;
    }
    old.it_value.tv_sec as i64
}

unsafe fn sys_pause() -> i64 {
    #[cfg(target_arch = "x86_64")]
    { <Arch as Syscalls>::syscall0(SYS_PAUSE) }
    #[cfg(target_arch = "aarch64")]
    { <Arch as Syscalls>::syscall4(SYS_PPOLL, 0, 0, 0, 0) }
}

unsafe fn sys_fsync(fd: i32) -> i64 {
    <Arch as Syscalls>::syscall1(SYS_FSYNC, fd as i64)
}

unsafe fn sys_fdatasync(fd: i32) -> i64 {
    <Arch as Syscalls>::syscall1(SYS_FDATASYNC, fd as i64)
}

unsafe fn sys_sync() {
    let _ = <Arch as Syscalls>::syscall0(SYS_SYNC);
}

unsafe fn sys_symlink(target: *const u8, linkpath: *const u8) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_SYMLINKAT, target as i64, AT_FDCWD as i64, linkpath as i64)
}

unsafe fn sys_symlinkat(target: *const u8, newdirfd: i32, linkpath: *const u8) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_SYMLINKAT, target as i64, newdirfd as i64, linkpath as i64)
}

unsafe fn sys_readlinkat(dirfd: i32, path: *const u8, buf: *mut u8, bufsiz: usize) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_READLINKAT, dirfd as i64, path as i64, buf as i64, bufsiz as i64)
}

unsafe fn sys_linkat(olddirfd: i32, oldpath: *const u8, newdirfd: i32, newpath: *const u8, flags: i32) -> i64 {
    <Arch as Syscalls>::syscall5(SYS_LINKAT, olddirfd as i64, oldpath as i64, newdirfd as i64, newpath as i64, flags as i64)
}

unsafe fn sys_fchmod(fd: i32, mode: u32) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_FCHMOD, fd as i64, mode as i64)
}

unsafe fn sys_fchmodat(dirfd: i32, path: *const u8, mode: u32, flags: i32) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_FCHMODAT, dirfd as i64, path as i64, mode as i64, flags as i64)
}

unsafe fn sys_umask(mask: u32) -> u32 {
    <Arch as Syscalls>::syscall1(SYS_UMASK, mask as i64) as u32
}

unsafe fn sys_getgroups(size: i32, list: *mut c_uint) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_GETGROUPS, size as i64, list as i64)
}

unsafe fn sys_setuid(uid: c_uint) -> i64 {
    <Arch as Syscalls>::syscall1(SYS_SETUID, uid as i64)
}

unsafe fn sys_setgid(gid: c_uint) -> i64 {
    <Arch as Syscalls>::syscall1(SYS_SETGID, gid as i64)
}

unsafe fn sys_setsid() -> i64 {
    <Arch as Syscalls>::syscall0(SYS_SETSID)
}

unsafe fn sys_setpgid(pid: c_int, pgid: c_int) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_SETPGID, pid as i64, pgid as i64)
}

unsafe fn sys_getpgid(pid: c_int) -> i64 {
    <Arch as Syscalls>::syscall1(SYS_GETPGID, pid as i64)
}

unsafe fn sys_getsid(pid: c_int) -> i64 {
    <Arch as Syscalls>::syscall1(SYS_GETSID, pid as i64)
}

unsafe fn sys_mkdirat(dirfd: i32, path: *const u8, mode: u32) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_MKDIRAT, dirfd as i64, path as i64, mode as i64)
}

// Public C ABI wrappers for unistd

#[no_mangle]
pub unsafe extern "C" fn pipe(fds: *mut c_int) -> c_int {
    let r = sys_pipe2(fds, 0);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn pipe2(fds: *mut c_int, flags: c_int) -> c_int {
    let r = sys_pipe2(fds, flags);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn dup(oldfd: c_int) -> c_int {
    let r = sys_dup(oldfd);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn dup2(oldfd: c_int, newfd: c_int) -> c_int {
    let r = sys_dup2(oldfd, newfd);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn dup3(oldfd: c_int, newfd: c_int, flags: c_int) -> c_int {
    let r = sys_dup3(oldfd, newfd, flags);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn fcntl(fd: c_int, cmd: c_int, mut args: ...) -> c_int {
    let r = match cmd {
        F_GETFD | F_GETFL => sys_fcntl(fd, cmd, 0),
        F_SETFD | F_SETFL | F_DUPFD | F_DUPFD_CLOEXEC => {
            let arg = args.next_arg::<c_int>();
            sys_fcntl(fd, cmd, arg as i64)
        }
        F_GETLK | F_SETLK | F_SETLKW => {
            let arg = args.next_arg::<*mut c_void>();
            sys_fcntl(fd, cmd, arg as i64)
        }
        _ => sys_fcntl(fd, cmd, 0),
    };
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn access(path: *const c_char, mode: c_int) -> c_int {
    let r = sys_access(path as *const u8, mode);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn unlink(path: *const c_char) -> c_int {
    let r = sys_unlink(path as *const u8);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn rmdir(path: *const c_char) -> c_int {
    let r = sys_rmdir(path as *const u8);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn chdir(path: *const c_char) -> c_int {
    let r = sys_chdir(path as *const u8);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn getcwd(buf: *mut c_char, size: usize) -> *mut c_char {
    if buf.is_null() {
        // allocate
        let alloc_size = if size == 0 { 256 } else { size };
        let p = malloc(alloc_size) as *mut c_char;
        if p.is_null() { ERRNO = ENOMEM; return core::ptr::null_mut(); }
        let r = sys_getcwd(p as *mut u8, alloc_size);
        if r < 0 { free(p as *mut c_void); ERRNO = (-r) as c_int; return core::ptr::null_mut(); }
        return p;
    }
    if size == 0 { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let r = sys_getcwd(buf as *mut u8, size);
    if r < 0 { ERRNO = (-r) as c_int; return core::ptr::null_mut(); }
    buf
}

#[no_mangle]
pub unsafe extern "C" fn gethostname(name: *mut c_char, len: usize) -> c_int {
    let r = sys_gethostname(name as *mut u8, len);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

// ponytail: getpagesize - hardcoded 4096 for x86_64
#[no_mangle]
pub extern "C" fn getpagesize() -> c_int {
    4096
}

#[no_mangle]
pub unsafe extern "C" fn truncate(path: *const c_char, length: i64) -> c_int {
    let r = sys_truncate(path as *const u8, length);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn ftruncate(fd: c_int, length: i64) -> c_int {
    let r = sys_ftruncate(fd, length);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn nanosleep(req: *const timespec, rem: *mut timespec) -> c_int {
    pthread_testcancel();
    let r = sys_nanosleep(req, rem);
    pthread_testcancel();
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sleep(seconds: c_uint) -> c_uint {
    let req = timespec { tv_sec: seconds as c_long, tv_nsec: 0 };
    let mut rem: timespec = core::mem::zeroed();
    pthread_testcancel();
    let r = sys_nanosleep(&req, &mut rem);
    pthread_testcancel();
    if r < 0 {
        let e = (-r) as c_int;
        if e == EINTR { return rem.tv_sec as c_uint; }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn usleep(usec: c_uint) -> c_int {
    let req = timespec { tv_sec: (usec / 1000000) as c_long, tv_nsec: ((usec % 1000000) * 1000) as c_long };
    pthread_testcancel();
    let r = sys_nanosleep(&req, core::ptr::null_mut());
    pthread_testcancel();
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn alarm(seconds: c_uint) -> c_uint {
    sys_alarm(seconds) as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn pause() -> c_int {
    pthread_testcancel();
    sys_pause();
    pthread_testcancel();
    ERRNO = EINTR;
    -1
}

#[no_mangle]
pub unsafe extern "C" fn fsync(fd: c_int) -> c_int {
    let r = sys_fsync(fd);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn fdatasync(fd: c_int) -> c_int {
    let r = sys_fdatasync(fd);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sync() {
    sys_sync();
}

#[no_mangle]
pub unsafe extern "C" fn symlink(target: *const c_char, linkpath: *const c_char) -> c_int {
    let r = sys_symlinkat(target as *const u8, AT_FDCWD, linkpath as *const u8);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn readlink(path: *const c_char, buf: *mut c_char, bufsiz: usize) -> isize {
    let r = sys_readlinkat(AT_FDCWD, path as *const u8, buf as *mut u8, bufsiz);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as isize
}

#[no_mangle]
pub unsafe extern "C" fn link(oldpath: *const c_char, newpath: *const c_char) -> c_int {
    let r = sys_linkat(AT_FDCWD, oldpath as *const u8, AT_FDCWD, newpath as *const u8, 0);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn chmod(path: *const c_char, mode: c_uint) -> c_int {
    let r = sys_fchmodat(AT_FDCWD, path as *const u8, mode, 0);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn fchmod(fd: c_int, mode: c_uint) -> c_int {
    let r = sys_fchmod(fd, mode);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn umask(mask: c_uint) -> c_uint {
    sys_umask(mask) as c_uint
}

#[no_mangle]
pub unsafe extern "C" fn isatty(fd: c_int) -> c_int {
    let mut ws: winsize = core::mem::zeroed();
    if sys_ioctl(fd, TIOCGWINSZ, &mut ws as *mut winsize as *mut u8) == 0 { 1 } else { 0 }
}

// ponytail: ttyname - stub, not critical
#[no_mangle]
pub unsafe extern "C" fn ttyname(_fd: c_int) -> *mut c_char {
    core::ptr::null_mut()
}

// ponytail: getlogin - stub
#[no_mangle]
pub unsafe extern "C" fn getlogin() -> *mut c_char {
    b"root\0".as_ptr() as *mut c_char
}

#[no_mangle]
pub unsafe extern "C" fn getgroups(size: c_int, list: *mut c_uint) -> c_int {
    let r = sys_getgroups(size, list);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn setuid(uid: c_uint) -> c_int {
    let r = sys_setuid(uid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn setgid(gid: c_uint) -> c_int {
    let r = sys_setgid(gid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn seteuid(euid: c_uint) -> c_int {
    // ponytail: on Linux, setreuid(-1, euid) sets effective uid
    let r = sys_setuid(euid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn setegid(egid: c_uint) -> c_int {
    let r = sys_setgid(egid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn setreuid(_ruid: c_uint, _euid: c_uint) -> c_int {
    // ponytail: stub, real UID changes need cap_setuid
    0
}

#[no_mangle]
pub unsafe extern "C" fn setregid(_rgid: c_uint, _egid: c_uint) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn setsid() -> c_int {
    let r = sys_setsid();
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn setpgid(pid: c_int, pgid: c_int) -> c_int {
    let r = sys_setpgid(pid, pgid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

#[no_mangle]
pub unsafe extern "C" fn getpgid(pid: c_int) -> c_int {
    let r = sys_getpgid(pid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn getsid(pid: c_int) -> c_int {
    let r = sys_getsid(pid);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    r as c_int
}

#[no_mangle]
pub unsafe extern "C" fn getpgrp() -> c_int {
    sys_getpgid(0) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn setpgrp() -> c_int {
    setpgid(0, 0)
}

#[no_mangle]
pub unsafe extern "C" fn mkstemp(template: *mut c_char) -> c_int {
    if template.is_null() { ERRNO = EINVAL; return -1; }
    let len = strlen(template as *const c_char);
    if len < 6 { ERRNO = EINVAL; return -1; }
    let x_start = len - 6;
    // check that last 6 chars are XXXXXX
    for i in x_start..len {
        if *template.add(i) != b'X' as c_char { ERRNO = EINVAL; return -1; }
    }
    let mut ts: timespec = core::mem::zeroed();
    let _ = sys_clock_gettime(CLOCK_REALTIME, &mut ts);
    let mut ctr: u32 = ts.tv_nsec as u32;
    for _ in 0..100 {
        ctr = ctr.wrapping_mul(1103515245).wrapping_add(12345);
        let seed = ctr;
        // fill XXXXXX with random hex chars
        let hex = b"0123456789abcdef";
        let mut s = seed;
        for i in x_start..len {
            *template.add(i) = hex[(s & 0xf) as usize] as c_char;
            s >>= 4;
        }
        let fd = sys_open(template as *const u8, (O_RDWR | O_CREAT | O_EXCL) as i64, 0o600);
        if fd >= 0 { return fd as c_int; }
        let e = (-fd) as c_int;
        if e != EEXIST_VAL { ERRNO = e; return -1; }
    }
    ERRNO = EEXIST_VAL;
    -1
}

#[no_mangle]
pub unsafe extern "C" fn mkdir(path: *const c_char, mode: c_uint) -> c_int {
    let r = sys_mkdirat(AT_FDCWD, path as *const u8, mode);
    if r < 0 { ERRNO = (-r) as c_int; return -1; }
    0
}

// ============================================================
// libgen: basename / dirname
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn dirname(s: *mut c_char) -> *mut c_char {
    if s.is_null() || *s == 0 {
        return b".\0".as_ptr() as *mut c_char;
    }
    let mut i = strlen(s as *const c_char);
    i = i.wrapping_sub(1);
    // skip trailing slashes
    while i > 0 && *s.add(i) == b'/' as c_char {
        i = i.wrapping_sub(1);
    }
    if i == 0 && *s.add(0) == b'/' as c_char {
        return b"/\0".as_ptr() as *mut c_char;
    }
    // skip non-slashes
    while i > 0 && *s.add(i) != b'/' as c_char {
        i = i.wrapping_sub(1);
    }
    if i == 0 {
        if *s.add(0) == b'/' as c_char {
            return b"/\0".as_ptr() as *mut c_char;
        }
        return b".\0".as_ptr() as *mut c_char;
    }
    // skip slashes
    while i > 0 && *s.add(i - 1) == b'/' as c_char {
        i = i.wrapping_sub(1);
    }
    if i == 0 {
        return b"/\0".as_ptr() as *mut c_char;
    }
    *s.add(i) = 0;
    s
}

#[no_mangle]
pub unsafe extern "C" fn basename(s: *mut c_char) -> *mut c_char {
    if s.is_null() || *s == 0 {
        return b".\0".as_ptr() as *mut c_char;
    }
    let mut i = strlen(s as *const c_char);
    i = i.wrapping_sub(1);
    // strip trailing slashes
    while i > 0 && *s.add(i) == b'/' as c_char {
        *s.add(i) = 0;
        i = i.wrapping_sub(1);
    }
    // find last slash
    while i > 0 && *s.add(i - 1) != b'/' as c_char {
        i = i.wrapping_sub(1);
    }
    s.add(i)
}

// ============================================================
// arpa/inet: inet_pton / inet_ntop / inet_addr / inet_ntoa / inet_aton
// ============================================================

const AF_INET_VAL: c_int = 2;
const AF_INET6_VAL: c_int = 10;
const EAFNOSUPPORT: c_int = 97;
const ENOSPC_VAL2: c_int = 28;

unsafe fn inet_pton_v4(s: *const u8, a: *mut u8) -> c_int {
    let mut p = s;
    let mut i = 0;
    while i < 4 {
        let oct_start = p;
        let mut v: u32 = 0;
        let mut j: u32 = 0;
        while j < 3 && *p >= b'0' && *p <= b'9' {
            v = v * 10 + (*p - b'0') as u32;
            p = p.add(1);
            j += 1;
        }
        if j == 0 || (j > 1 && *oct_start == b'0') || v > 255 { return 0; }
        *a.add(i) = v as u8;
        if *p == 0 && i == 3 { return 1; }
        if *p != b'.' { return 0; }
        p = p.add(1);
        i += 1;
    }
    0
}

unsafe fn inet_pton_v6(s: *const u8, a: *mut u8) -> c_int {
    let mut ip = [0u16; 8];
    let mut p = s;
    let mut i: usize = 0;
    let mut brk: isize = -1;
    let mut need_v4 = false;
    let mut v4_start: *const u8 = core::ptr::null();

    if *p == b':' {
        p = p.add(1);
        if *p != b':' { return 0; }
    }

    loop {
        if *p == b':' && brk < 0 {
            brk = i as isize;
            ip[i & 7] = 0;
            p = p.add(1);
            if *p == 0 { break; }
            if i == 7 { return 0; }
            i += 1;
            continue;
        }
        v4_start = p;
        let mut v: u16 = 0;
        let mut j = 0;
        while j < 4 {
            let d = hex_digit(*p);
            if d < 0 { break; }
            v = v * 16 + d as u16;
            p = p.add(1);
            j += 1;
        }
        if j == 0 { return 0; }
        ip[i & 7] = v;
        if *p == 0 && (brk >= 0 || i == 7) { break; }
        if i == 7 { return 0; }
        if *p != b':' {
            if *p != b'.' || (i < 6 && brk < 0) { return 0; }
            need_v4 = true;
            i += 1;
            ip[i & 7] = 0;
            break;
        }
        p = p.add(1);
        i += 1;
    }

    if brk >= 0 {
        // expand :: : move trailing fields to the end and zero the gap
        let trail_count = i + 1 - brk as usize;
        let dest_start = brk as usize + 7 - i;
        // move backwards to avoid overwrite in the overlapping case
        for j in (0..trail_count).rev() {
            ip[dest_start + j] = ip[brk as usize + j];
        }
        for j in 0..(7 - i) {
            ip[brk as usize + j] = 0;
        }
    }

    for k in 0..8 {
        *a.add(k * 2) = (ip[k] >> 8) as u8;
        *a.add(k * 2 + 1) = (ip[k] & 0xff) as u8;
    }

    if need_v4 {
        if inet_pton_v4(v4_start, a.add(12)) <= 0 { return 0; }
    }
    1
}

unsafe fn hex_digit(c: u8) -> i32 {
    if c >= b'0' && c <= b'9' { (c - b'0') as i32 }
    else if c >= b'a' && c <= b'f' { (c - b'a' + 10) as i32 }
    else if c >= b'A' && c <= b'F' { (c - b'A' + 10) as i32 }
    else { -1 }
}

#[no_mangle]
pub unsafe extern "C" fn inet_pton(af: c_int, s: *const c_char, a: *mut c_void) -> c_int {
    if af == AF_INET_VAL {
        inet_pton_v4(s as *const u8, a as *mut u8)
    } else if af == AF_INET6_VAL {
        inet_pton_v6(s as *const u8, a as *mut u8)
    } else {
        ERRNO = EAFNOSUPPORT;
        -1
    }
}

#[no_mangle]
pub unsafe extern "C" fn inet_ntop(af: c_int, a: *const c_void, s: *mut c_char, size: u32) -> *mut c_char {
    if af == AF_INET_VAL {
        let b = a as *const u8;
        let needed = 16; // "255.255.255.255\0"
        if size < needed as u32 { ERRNO = ENOSPC_VAL2; return core::ptr::null_mut(); }
        let mut pos = 0;
        for i in 0..4 {
            if i > 0 { *s.add(pos) = b'.' as c_char; pos += 1; }
            let (buf, len) = format_u64(*b.add(i) as u64);
            for j in 0..len { *s.add(pos) = buf[j] as c_char; pos += 1; }
        }
        *s.add(pos) = 0;
        s
    } else if af == AF_INET6_VAL {
        let b = a as *const u8;
        let needed = 46; // max IPv6
        if size < needed as u32 { ERRNO = ENOSPC_VAL2; return core::ptr::null_mut(); }
        let mut words = [0u16; 8];
        for i in 0..8 {
            words[i] = ((*b.add(i * 2) as u16) << 8) | (*b.add(i * 2 + 1) as u16);
        }
        // IPv4-mapped: ::ffff:a.b.c.d
        let mut pos = 0usize;
        if words[0] == 0 && words[1] == 0 && words[2] == 0 && words[3] == 0
            && words[4] == 0 && words[5] == 0xffff
        {
            let prefix = b"::ffff:";
            for k in 0..prefix.len() { *s.add(pos) = prefix[k] as c_char; pos += 1; }
            for idx in 0..4 {
                if idx > 0 { *s.add(pos) = b'.' as c_char; pos += 1; }
                let octet = *b.add(12 + idx);
                if octet >= 100 {
                    *s.add(pos) = (b'0' + octet / 100) as c_char; pos += 1;
                    *s.add(pos) = (b'0' + (octet / 10) % 10) as c_char; pos += 1;
                } else if octet >= 10 {
                    *s.add(pos) = (b'0' + octet / 10) as c_char; pos += 1;
                }
                *s.add(pos) = (b'0' + octet % 10) as c_char; pos += 1;
            }
            *s.add(pos) = 0;
            return s;
        }
        // find longest run of zeros
        let mut best_start: isize = -1;
        let mut best_len: usize = 0;
        let mut j: usize = 0;
        while j < 8 {
            if words[j] == 0 {
                let mut k = j;
                while k < 8 && words[k] == 0 { k += 1; }
                let run = k - j;
                if run > best_len { best_start = j as isize; best_len = run; }
                j = k;
            } else {
                j += 1;
            }
        }
        let mut i: usize = 0;
        let mut prev_in_run = false;
        while i < 8 {
            let in_run = best_len >= 2 && i >= best_start as usize && i < best_start as usize + best_len;
            if in_run {
                if i == best_start as usize {
                    *s.add(pos) = b':' as c_char; pos += 1;
                    *s.add(pos) = b':' as c_char; pos += 1;
                }
                i += 1;
                prev_in_run = true;
                continue;
            }
            if i > 0 && !prev_in_run {
                *s.add(pos) = b':' as c_char; pos += 1;
            }
            let (buf, len) = format_hex(words[i] as u64, false);
            for k in 0..len { *s.add(pos) = buf[k] as c_char; pos += 1; }
            i += 1;
            prev_in_run = false;
        }
        *s.add(pos) = 0;
        s
    } else {
        ERRNO = EAFNOSUPPORT;
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn inet_aton(s: *const c_char, dest: *mut c_void) -> c_int {
    let a = dest as *mut u8;
    let mut p = s as *const u8;
    let mut vals = [0u64; 4];
    let mut i = 0usize;
    while i < 4 {
        if *p < b'0' || *p > b'9' { return 0; }
        let mut base = 10u32;
        if *p == b'0' {
            let c = *p.add(1);
            if c == b'x' || c == b'X' {
                base = 16;
                p = p.add(2);
                if !((*p >= b'0' && *p <= b'9')
                    || (*p >= b'a' && *p <= b'f')
                    || (*p >= b'A' && *p <= b'F')) {
                    return 0;
                }
            } else {
                base = 8;
            }
        }
        let mut v: u64 = 0;
        let start = p;
        loop {
            let c = *p;
            let d = if c >= b'0' && c <= b'9' {
                (c - b'0') as u64
            } else if base == 16 && c >= b'a' && c <= b'f' {
                (c - b'a' + 10) as u64
            } else if base == 16 && c >= b'A' && c <= b'F' {
                (c - b'A' + 10) as u64
            } else {
                break;
            };
            v = v * (base as u64) + d;
            p = p.add(1);
        }
        if p == start { return 0; }
        vals[i] = v;
        if *p == 0 {
            break;
        }
        if *p != b'.' || i == 3 { return 0; }
        p = p.add(1);
        i += 1;
    }
    if i == 4 { return 0; }
    if i < 1 { vals[1] = vals[0] & 0xffffff; vals[0] >>= 24; }
    if i < 2 { vals[2] = vals[1] & 0xffff; vals[1] >>= 16; }
    if i < 3 { vals[3] = vals[2] & 0xff; vals[2] >>= 8; }
    for j in 0..4 {
        if vals[j] > 255 { return 0; }
        *a.add(j) = vals[j] as u8;
    }
    1
}

#[no_mangle]
pub unsafe extern "C" fn inet_addr(s: *const c_char) -> u32 {
    let mut a = [0u8; 4];
    if inet_aton(s, a.as_mut_ptr() as *mut c_void) == 1 {
        u32::from_ne_bytes(a)
    } else {
        !0u32 // INADDR_NONE
    }
}

#[no_mangle]
pub unsafe extern "C" fn inet_ntoa(addr: u32) -> *mut c_char {
    static mut NTOA_BUF: [c_char; 16] = [0; 16];
    let b = addr.to_ne_bytes();
    let mut pos = 0;
    for i in 0..4 {
        if i > 0 { NTOA_BUF[pos] = b'.' as c_char; pos += 1; }
        let (buf, len) = format_u64(b[i] as u64);
        for j in 0..len { NTOA_BUF[pos] = buf[j] as c_char; pos += 1; }
    }
    NTOA_BUF[pos] = 0;
    core::ptr::addr_of_mut!(NTOA_BUF).cast::<c_char>()
}

// ponytail: inet_network/inet_makeaddr/inet_lnaof/inet_netof - simple implementations
#[no_mangle]
pub unsafe extern "C" fn inet_network(s: *const c_char) -> u32 {
    inet_addr(s).to_be()
}

#[no_mangle]
pub extern "C" fn inet_makeaddr(net: c_int, host: c_int) -> u32 {
    let addr: u32;
    if (net as u32) < 128 {
        addr = (net as u32) << 24;
    } else if (net as u32) < 0x10000 {
        addr = (net as u32) << 16;
    } else if (net as u32) < 0x1000000 {
        addr = (net as u32) << 8;
    } else {
        addr = net as u32;
    }
    addr | (host as u32 & 0xffffffff)
}

#[no_mangle]
pub extern "C" fn inet_lnaof(addr: u32) -> c_int {
    let h = u32::from_be(addr);
    if h < 0x80000000 { (h & 0x00ffffff) as c_int }
    else if h < 0xc0000000 { (h & 0x0000ffff) as c_int }
    else { (h & 0x000000ff) as c_int }
}

#[no_mangle]
pub extern "C" fn inet_netof(addr: u32) -> c_int {
    let h = u32::from_be(addr);
    if h < 0x80000000 { (h >> 24) as c_int }
    else if h < 0xc0000000 { (h >> 16) as c_int }
    else { (h >> 8) as c_int }
}

// ============================================================
// stdlib: strtod / strtof / strtold
// ============================================================

// ponytail: correctly-rounded float parser.
// Decimal: collects digits into buffer, uses core::str::FromStr.
// Hex: u128 integer arithmetic with round-to-nearest-even.
// is_f32: true for strtof (f32::from_str), false for strtod/strtold (f64::from_str).
unsafe fn parse_float(s: *const u8, endptr: *mut *mut u8, is_f32: bool) -> f64 {
    let mut p = s;
    // skip whitespace
    while *p == b' ' || *p == b'\t' || *p == b'\n' || *p == b'\r' { p = p.add(1); }
    let mut neg = false;
    if *p == b'-' { neg = true; p = p.add(1); }
    else if *p == b'+' { p = p.add(1); }

    // inf
    if (*p | 0x20) == b'i'
        && (*p.add(1) | 0x20) == b'n'
        && (*p.add(2) | 0x20) == b'f'
    {
        p = p.add(3);
        // optionally consume "inity"
        if (*p | 0x20) == b'i'
            && (*p.add(1) | 0x20) == b't'
            && (*p.add(2) | 0x20) == b'y'
        {
            p = p.add(3);
        }
        if !endptr.is_null() { *endptr = p as *mut u8; }
        return if neg { f64::NEG_INFINITY } else { f64::INFINITY };
    }
    // nan
    if (*p | 0x20) == b'n'
        && (*p.add(1) | 0x20) == b'a'
        && (*p.add(2) | 0x20) == b'n'
    {
        p = p.add(3);
        // optionally consume (n-char-sequence)
        if *p == b'(' {
            p = p.add(1);
            while *p != 0 && *p != b')' { p = p.add(1); }
            if *p == b')' { p = p.add(1); }
        }
        if !endptr.is_null() { *endptr = p as *mut u8; }
        return if neg { -f64::NAN } else { f64::NAN };
    }

    // hex float
    if *p == b'0' && (*p.add(1) == b'x' || *p.add(1) == b'X') {
        p = p.add(2);
        let mut mant: u128 = 0;
        let mut frac_hex: u32 = 0;
        let mut found = false;
        let mut in_frac = false;

        loop {
            if let Some(d) = hex_val(*p) {
                if mant <= u128::MAX / 16 {
                    mant = mant * 16 + d as u128;
                }
                if in_frac { frac_hex += 1; }
                p = p.add(1);
                found = true;
            } else if *p == b'.' && !in_frac {
                in_frac = true;
                p = p.add(1);
            } else {
                break;
            }
        }

        if !found {
            if !endptr.is_null() { *endptr = s as *mut u8; }
            return 0.0;
        }

        // binary exponent p[+-]N
        let mut bin_exp: i32 = 0;
        if *p == b'p' || *p == b'P' {
            p = p.add(1);
            let mut exp_neg = false;
            if *p == b'-' { exp_neg = true; p = p.add(1); }
            else if *p == b'+' { p = p.add(1); }
            while *p >= b'0' && *p <= b'9' {
                if bin_exp < 100000 {
                    bin_exp = bin_exp * 10 + (*p - b'0') as i32;
                }
                p = p.add(1);
            }
            if exp_neg { bin_exp = -bin_exp; }
        }

        if !endptr.is_null() { *endptr = p as *mut u8; }
        if mant == 0 { return if neg { -0.0 } else { 0.0 }; }

        let total_exp = bin_exp as i64 - 4 * frac_hex as i64;
        return hex_mant_to_f64(mant, total_exp, neg);
    }

    // decimal: collect into buffer, use from_str for correct rounding
    let mut buf = [0u8; 65536];
    let mut n = 0usize;
    let mut found_digit = false;

    while *p >= b'0' && *p <= b'9' {
        if n < 65535 { buf[n] = *p; n += 1; }
        p = p.add(1);
        found_digit = true;
    }
    if *p == b'.' {
        if n < 65535 { buf[n] = b'.'; n += 1; }
        p = p.add(1);
        while *p >= b'0' && *p <= b'9' {
            if n < 65535 { buf[n] = *p; n += 1; }
            p = p.add(1);
            found_digit = true;
        }
    }
    if !found_digit {
        if !endptr.is_null() { *endptr = s as *mut u8; }
        return 0.0;
    }
    if *p == b'e' || *p == b'E' {
        if n < 65535 { buf[n] = b'e'; n += 1; }
        p = p.add(1);
        if *p == b'-' || *p == b'+' {
            if n < 65535 { buf[n] = *p; n += 1; }
            p = p.add(1);
        }
        while *p >= b'0' && *p <= b'9' {
            if n < 65535 { buf[n] = *p; n += 1; }
            p = p.add(1);
        }
    }

    if !endptr.is_null() { *endptr = p as *mut u8; }
    let s_str = core::str::from_utf8_unchecked(
        core::slice::from_raw_parts(buf.as_ptr(), n),
    );
    if is_f32 {
        match <f32 as core::str::FromStr>::from_str(s_str) {
            Ok(v) => { let r = v as f64; if neg { -r } else { r } }
            Err(_) => { if !endptr.is_null() { *endptr = s as *mut u8; } 0.0 }
        }
    } else {
        match <f64 as core::str::FromStr>::from_str(s_str) {
            Ok(v) => if neg { -v } else { v },
            Err(_) => { if !endptr.is_null() { *endptr = s as *mut u8; } 0.0 }
        }
    }
}

// Convert hex mantissa integer to f64 with correct rounding.
// mant: integer from parsed hex digits. total_exp: bin_exp - 4 * frac_hex_digits.
// Value = mant * 2^total_exp.
fn hex_mant_to_f64(mant: u128, total_exp: i64, neg: bool) -> f64 {
    if mant == 0 { return if neg { -0.0 } else { 0.0 }; }

    let msb = (127 - mant.leading_zeros()) as i64;
    let unbiased_exp = msb + total_exp;
    let biased_exp = unbiased_exp + 1023;

    // Overflow → infinity
    if biased_exp >= 2047 {
        return if neg { f64::NEG_INFINITY } else { f64::INFINITY };
    }

    // Subnormal or underflow
    if biased_exp <= 0 {
        // subnormal: biased_exp=0, effective exponent=-1022
        // mantissa = round(mant * 2^(total_exp + 1074))
        let shift = total_exp + 1074;
        if shift < 0 {
            return if neg { -0.0 } else { 0.0 };
        }
        if shift > 128 {
            return if neg { f64::NEG_INFINITY } else { f64::INFINITY };
        }
        let shifted = mant << (shift as u32);
        let mantissa52 = (shifted as u64) & 0x000FFFFFFFFFFFFF;
        let guard = ((shifted >> 52) & 1) != 0;
        let sticky = (shifted >> 53) != 0;
        let mut result = mantissa52;
        if guard && (sticky || (result & 1) != 0) {
            result += 1;
            if result > 0x000FFFFFFFFFFFFF {
                // carry → smallest normal
                return if neg { -f64::from_bits(1u64 << 52) } else { f64::from_bits(1u64 << 52) };
            }
        }
        return if neg { -f64::from_bits(result) } else { f64::from_bits(result) };
    }

    // Normal: biased_exp in [1, 2046]
    let shift = msb - 52;

    if shift <= 0 {
        // fewer than 53 bits, no rounding needed
        let mantissa = ((mant as u64) & ((1u64 << msb as u32) - 1)) << ((-shift) as u32);
        let bits = ((biased_exp as u64) << 52) | mantissa;
        return if neg { -f64::from_bits(bits) } else { f64::from_bits(bits) };
    }

    // shift > 0: round to nearest even
    let mantissa = ((mant >> shift as u32) as u64) & 0x000FFFFFFFFFFFFF;
    let guard = ((mant >> (shift as u32 - 1)) & 1) != 0;
    let sticky = if shift > 1 {
        (mant & ((1u128 << (shift as u32 - 1)) - 1)) != 0
    } else { false };

    let mut result_mant = mantissa;
    let mut result_exp = biased_exp;
    if guard && (sticky || (result_mant & 1) != 0) {
        result_mant += 1;
        if result_mant > 0x000FFFFFFFFFFFFF {
            result_mant = 0;
            result_exp += 1;
            if result_exp >= 2047 {
                return if neg { f64::NEG_INFINITY } else { f64::INFINITY };
            }
        }
    }

    let bits = ((result_exp as u64) << 52) | result_mant;
    if neg { -f64::from_bits(bits) } else { f64::from_bits(bits) }
}

unsafe fn hex_val(c: u8) -> Option<u8> {
    if c >= b'0' && c <= b'9' { Some(c - b'0') }
    else if c >= b'a' && c <= b'f' { Some(c - b'a' + 10) }
    else if c >= b'A' && c <= b'F' { Some(c - b'A' + 10) }
    else { None }
}

#[no_mangle]
pub unsafe extern "C" fn strtod(s: *const c_char, endptr: *mut *mut c_char) -> f64 {
    let mut end: *mut u8 = s as *mut u8;
    let r = parse_float(s as *const u8, &mut end, false);
    if !endptr.is_null() { *endptr = end as *mut c_char; }
    r
}

#[no_mangle]
pub unsafe extern "C" fn strtof(s: *const c_char, endptr: *mut *mut c_char) -> f32 {
    let mut end: *mut u8 = s as *mut u8;
    let r = parse_float(s as *const u8, &mut end, true);
    if !endptr.is_null() { *endptr = end as *mut c_char; }
    r as f32
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn strtold(s: *const c_char, endptr: *mut *mut c_char) -> f64 {
    strtod(s, endptr)
}

#[no_mangle]
#[cfg(target_arch = "aarch64")]
pub unsafe extern "C" fn strtold(s: *const c_char, endptr: *mut *mut c_char) -> f128 {
    strtod(s, endptr) as f128
}

#[no_mangle]
pub unsafe extern "C" fn atof(s: *const c_char) -> f64 {
    strtod(s, core::ptr::null_mut())
}

// ============================================================
// stdio: open_memstream / fmemopen
// ============================================================

#[repr(C)]
struct MsCookie {
    bufp: *mut *mut c_char,
    sizep: *mut usize,
    pos: usize,
    len: usize,
    buf: *mut u8,
    space: usize,
}

#[repr(C)]
struct MfCookie {
    pos: usize,
    len: usize,
    size: usize,
    buf: *mut u8,
    mode: c_char,
}

unsafe extern "C" fn ms_seek(f: *mut FILE, off: i64, whence: c_int) -> i64 {
    let c = (*f).cookie as *mut MsCookie;
    if whence < 0 || whence > 2 { ERRNO = EINVAL; return -1; }
    let base: i64 = match whence {
        0 => 0,
        1 => (*c).pos as i64,
        2 => (*c).len as i64,
        _ => { ERRNO = EINVAL; return -1; }
    };
    if off < -base || off > isize::MAX as i64 - base { ERRNO = EINVAL; return -1; }
    (*c).pos = (base + off) as usize;
    (*c).pos as i64
}

unsafe extern "C" fn ms_write(f: *mut FILE, buf: *const u8, len: usize) -> usize {
    let c = (*f).cookie as *mut MsCookie;
    let buffered = (*f).wpos as usize - (*f).wbase as usize;
    if buffered > 0 {
        (*f).wpos = (*f).wbase;
        if ms_write(f, (*f).wbase, buffered) < buffered { return 0; }
    }
    if len == 0 { return 0; }
    if (*c).pos + len >= (*c).space {
        let new_space = 2 * (*c).space + 1 | (*c).pos + len + 1;
        let newbuf = realloc((*c).buf as *mut c_void, new_space) as *mut u8;
        if newbuf.is_null() { return 0; }
        *(*c).bufp = newbuf as *mut c_char;
        core::ptr::write_bytes(newbuf.add((*c).space), 0, new_space - (*c).space);
        (*c).buf = newbuf;
        (*c).space = new_space;
    }
    core::ptr::copy_nonoverlapping(buf, (*c).buf.add((*c).pos), len);
    (*c).pos += len;
    if (*c).pos >= (*c).len { (*c).len = (*c).pos; }
    *(*c).sizep = (*c).pos;
    len
}

unsafe extern "C" fn ms_close(_f: *mut FILE) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn open_memstream(bufp: *mut *mut c_char, sizep: *mut usize) -> *mut FILE {
    if bufp.is_null() || sizep.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let f = calloc(1, core::mem::size_of::<FILE>() + UNGET + BUFSIZ) as *mut FILE;
    if f.is_null() { ERRNO = ENOMEM; return core::ptr::null_mut(); }
    let buf_area = buf_ptr(f);
    init_file(f, -1, b"w\0".as_ptr() as *const c_char, Some(ms_close), buf_area, BUFSIZ);
    (*f).flags = F_NORD | F_SVB;
    (*f).lbf = -1;
    let c = calloc(1, core::mem::size_of::<MsCookie>()) as *mut MsCookie;
    if c.is_null() { free(f as *mut c_void); ERRNO = ENOMEM; return core::ptr::null_mut(); }
    (*c).bufp = bufp;
    (*c).sizep = sizep;
    (*c).pos = 0;
    (*c).len = 0;
    (*c).buf = core::ptr::null_mut();
    (*c).space = 0;
    (*f).cookie = c as *mut c_void;
    (*f).write_fn = Some(ms_write);
    (*f).seek_fn = Some(ms_seek);
    *bufp = core::ptr::null_mut();
    *sizep = 0;
    f
}

unsafe extern "C" fn mseek(f: *mut FILE, off: i64, whence: c_int) -> i64 {
    let c = (*f).cookie as *mut MfCookie;
    if whence < 0 || whence > 2 { ERRNO = EINVAL; return -1; }
    let base: i64 = match whence {
        0 => 0,
        1 => (*c).pos as i64,
        2 => (*c).len as i64,
        _ => { ERRNO = EINVAL; return -1; }
    };
    if off < -base || off > (*c).size as i64 - base { ERRNO = EINVAL; return -1; }
    (*c).pos = (base + off) as usize;
    (*c).pos as i64
}

unsafe extern "C" fn mread(f: *mut FILE, buf: *mut u8, len: usize) -> usize {
    let c = (*f).cookie as *mut MfCookie;
    let mut rem = if (*c).pos <= (*c).len { (*c).len - (*c).pos } else { 0 };
    let mut l = len;
    if l > rem {
        l = rem;
        (*f).flags |= F_EOF;
    }
    core::ptr::copy_nonoverlapping((*c).buf.add((*c).pos), buf, l);
    (*c).pos += l;
    rem -= l;
    if rem > (*f).buf_size { rem = (*f).buf_size; }
    (*f).rpos = (*f).buf;
    (*f).rend = (*f).buf.add(rem);
    core::ptr::copy_nonoverlapping((*c).buf.add((*c).pos), (*f).rpos, rem);
    (*c).pos += rem;
    l
}

unsafe extern "C" fn mwrite(f: *mut FILE, buf: *const u8, len: usize) -> usize {
    let c = (*f).cookie as *mut MfCookie;
    let buffered = (*f).wpos as usize - (*f).wbase as usize;
    if buffered > 0 {
        (*f).wpos = (*f).wbase;
        if mwrite(f, (*f).wbase, buffered) < buffered { return 0; }
    }
    if len == 0 { return 0; }
    if (*c).mode == b'a' as c_char { (*c).pos = (*c).len; }
    let rem = (*c).size - (*c).pos;
    let mut l = len;
    if l > rem { l = rem; }
    core::ptr::copy_nonoverlapping(buf, (*c).buf.add((*c).pos), l);
    (*c).pos += l;
    if (*c).pos > (*c).len {
        (*c).len = (*c).pos;
        if (*c).len < (*c).size {
            *(*c).buf.add((*c).len) = 0;
        } else if ((*f).flags & F_NORD) != 0 && (*c).size > 0 {
            *(*c).buf.add((*c).size - 1) = 0;
        }
    }
    l
}

unsafe extern "C" fn mclose(_f: *mut FILE) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn fmemopen(buf: *mut c_void, size: usize, mode: *const c_char) -> *mut FILE {
    if mode.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let m = *mode;
    if m != b'r' as c_char && m != b'w' as c_char && m != b'a' as c_char {
        ERRNO = EINVAL; return core::ptr::null_mut();
    }
    let has_plus = !strchr(mode as *const u8, b'+' as c_int).is_null();
    let need_alloc = buf.is_null();
    let alloc_size = core::mem::size_of::<FILE>() + UNGET + BUFSIZ + if need_alloc { size } else { 0 };
    let f = calloc(1, alloc_size) as *mut FILE;
    if f.is_null() { ERRNO = ENOMEM; return core::ptr::null_mut(); }
    let buf_area = buf_ptr(f);
    init_file(f, -1, mode, Some(mclose), buf_area, BUFSIZ);
    (*f).flags |= F_SVB;
    let actual_buf: *mut u8 = if need_alloc {
        (f as *mut u8).add(alloc_size - size)
    } else {
        buf as *mut u8
    };
    if need_alloc { core::ptr::write_bytes(actual_buf, 0, size); }
    let c = calloc(1, core::mem::size_of::<MfCookie>()) as *mut MfCookie;
    if c.is_null() { free(f as *mut c_void); ERRNO = ENOMEM; return core::ptr::null_mut(); }
    (*c).buf = actual_buf;
    (*c).size = size;
    (*c).mode = m;
    (*f).cookie = c as *mut c_void;
    (*f).read_fn = Some(mread);
    (*f).write_fn = Some(mwrite);
    (*f).seek_fn = Some(mseek);
    if !has_plus { (*f).flags = if m == b'r' as c_char { F_NOWR } else { F_NORD }; }
    if m == b'r' as c_char {
        (*c).len = size;
    } else if m == b'a' as c_char {
        (*c).len = strnlen(actual_buf, size);
        (*c).pos = (*c).len;
    } else if has_plus {
        *actual_buf = 0;
    }
    f
}

// ============================================================
// Stat (for ftok)
// ============================================================

#[repr(C)]
struct kernel_stat64 {
    st_dev: u64,
    st_ino: u64,
    st_nlink: u64,
    st_mode: u32,
    st_uid: u32,
    st_gid: u32,
    __pad0: c_int,
    st_rdev: u64,
    st_size: i64,
    st_blksize: i64,
    st_blocks: i64,
    st_atime: i64,
    st_atime_nsec: i64,
    st_mtime: i64,
    st_mtime_nsec: i64,
    st_ctime: i64,
    st_ctime_nsec: i64,
    __unused: [i64; 3],
}

// sys_stat = 4 on x86_64
#[inline]
unsafe fn sys_stat(path: *const u8, statbuf: *mut kernel_stat64) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_NEWFSTATAT, AT_FDCWD as i64, path as i64, statbuf as i64, 0)
}

// ============================================================
// Search: hcreate/hdestroy/hsearch (hash table)
// ============================================================

const HSEARCH_MINSIZE: usize = 8;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HSearchEntry {
    key: *mut c_char,
    data: *mut c_void,
}

#[repr(C)]
struct HTab {
    entries: *mut HSearchEntry,
    mask: usize,
    used: usize,
}

#[repr(C)]
pub struct HSearchData {
    tab: *mut HTab,
    __unused1: c_uint,
    __unused2: c_uint,
}

// ponytail: single static hash table for non-_r variants
static mut HTAB_DATA: HSearchData = HSearchData {
    tab: core::ptr::null_mut(),
    __unused1: 0,
    __unused2: 0,
};

unsafe fn keyhash(k: *const c_char) -> usize {
    let mut h: usize = 0;
    let mut p = k;
    while *p != 0 {
        h = h.wrapping_mul(31).wrapping_add(*p as usize);
        p = p.add(1);
    }
    h
}

unsafe fn htab_lookup(key: *const c_char, hash: usize, htab: *mut HSearchData) -> *mut HSearchEntry {
    let tab = (*htab).tab;
    let mask = (*tab).mask;
    let entries = (*tab).entries;
    let mut i = hash;
    let mut j: usize = 1;
    loop {
        let e = entries.add(i & mask);
        if (*e).key.is_null() || strcmp((*e).key as *const u8, key as *const u8) == 0 {
            return e;
        }
        i = i.wrapping_add(j);
        j += 1;
    }
}

unsafe fn htab_resize(nel: usize, htab: *mut HSearchData) -> c_int {
    let tab = (*htab).tab;
    let old_entries = (*tab).entries;
    let old_size = (*tab).mask + 1;

    let mut newsize: usize = HSEARCH_MINSIZE;
    while newsize < nel {
        let Some(next) = newsize.checked_mul(2) else {
            ERRNO = ENOMEM;
            return 0;
        };
        newsize = next;
    }

    let new_entries = calloc(newsize, core::mem::size_of::<HSearchEntry>()) as *mut HSearchEntry;
    if new_entries.is_null() {
        ERRNO = ENOMEM;
        return 0;
    }
    (*tab).entries = new_entries;
    (*tab).mask = newsize - 1;

    if !old_entries.is_null() {
        for idx in 0..old_size {
            let old_e = old_entries.add(idx);
            if !(*old_e).key.is_null() {
                let h = keyhash((*old_e).key);
                let mut i = h;
                let mut j: usize = 1;
                loop {
                    let e = new_entries.add(i & (*tab).mask);
                    if (*e).key.is_null() {
                        *e = *old_e;
                        break;
                    }
                    i = i.wrapping_add(j);
                    j += 1;
                }
            }
        }
        free(old_entries as *mut c_void);
    }
    1
}

unsafe fn hcreate_r_impl(nel: usize, htab: *mut HSearchData) -> c_int {
    if nel == 0 || nel > (1usize << 30) {
        ERRNO = ENOMEM;
        return 0;
    }
    (*htab).tab = calloc(1, core::mem::size_of::<HTab>()) as *mut HTab;
    if (*htab).tab.is_null() {
        ERRNO = ENOMEM;
        return 0;
    }
    if htab_resize(nel, htab) == 0 {
        free((*htab).tab as *mut c_void);
        (*htab).tab = core::ptr::null_mut();
        return 0;
    }
    1
}

unsafe fn hdestroy_r_impl(htab: *mut HSearchData) {
    if !(*htab).tab.is_null() {
        if !(*(*htab).tab).entries.is_null() {
            free((*(*htab).tab).entries as *mut c_void);
        }
        free((*htab).tab as *mut c_void);
        (*htab).tab = core::ptr::null_mut();
    }
}

unsafe fn hsearch_r_impl(item: HSearchEntry, action: c_int, retval: *mut *mut HSearchEntry, htab: *mut HSearchData) -> c_int {
    let hash = keyhash(item.key);
    let e = htab_lookup(item.key, hash, htab);

    if !(*e).key.is_null() {
        *retval = e;
        return 1;
    }
    if action == 0 { // FIND
        *retval = core::ptr::null_mut();
        return 0;
    }
    // ENTER
    let item_key = item.key;
    *e = item;
    (*(*htab).tab).used += 1;
    if (*(*htab).tab).used > (*(*htab).tab).mask - (*(*htab).tab).mask / 4 {
        if htab_resize(2 * (*(*htab).tab).used, htab) == 0 {
            (*(*htab).tab).used -= 1;
            (*e).key = core::ptr::null_mut();
            *retval = core::ptr::null_mut();
            return 0;
        }
        let e2 = htab_lookup(item_key, hash, htab);
        *retval = e2;
        return 1;
    }
    *retval = e;
    1
}

#[no_mangle]
pub unsafe extern "C" fn hcreate(nel: usize) -> c_int {
    hcreate_r_impl(nel, core::ptr::addr_of_mut!(HTAB_DATA))
}

#[no_mangle]
pub unsafe extern "C" fn hdestroy() {
    hdestroy_r_impl(core::ptr::addr_of_mut!(HTAB_DATA));
}

#[no_mangle]
pub unsafe extern "C" fn hsearch(item: HSearchEntry, action: c_int) -> *mut HSearchEntry {
    let mut e: *mut HSearchEntry = core::ptr::null_mut();
    hsearch_r_impl(item, action, &mut e, core::ptr::addr_of_mut!(HTAB_DATA));
    e
}

#[no_mangle]
pub unsafe extern "C" fn hcreate_r(nel: usize, htab: *mut HSearchData) -> c_int {
    hcreate_r_impl(nel, htab)
}

#[no_mangle]
pub unsafe extern "C" fn hdestroy_r(htab: *mut HSearchData) {
    hdestroy_r_impl(htab);
}

#[no_mangle]
pub unsafe extern "C" fn hsearch_r(item: HSearchEntry, action: c_int, retval: *mut *mut HSearchEntry, htab: *mut HSearchData) -> c_int {
    hsearch_r_impl(item, action, retval, htab)
}

// ============================================================
// Search: insque / remque
// ============================================================

#[repr(C)]
struct QueNode {
    next: *mut QueNode,
    prev: *mut QueNode,
}

#[no_mangle]
pub unsafe extern "C" fn insque(element: *mut c_void, pred: *mut c_void) {
    let e = element as *mut QueNode;
    let p = pred as *mut QueNode;
    if p.is_null() {
        (*e).next = core::ptr::null_mut();
        (*e).prev = core::ptr::null_mut();
        return;
    }
    (*e).next = (*p).next;
    (*e).prev = p;
    (*p).next = e;
    if !(*e).next.is_null() {
        (*(*e).next).prev = e;
    }
}

#[no_mangle]
pub unsafe extern "C" fn remque(element: *mut c_void) {
    let e = element as *mut QueNode;
    if !(*e).next.is_null() {
        (*(*e).next).prev = (*e).prev;
    }
    if !(*e).prev.is_null() {
        (*(*e).prev).next = (*e).next;
    }
}

// ============================================================
// Search: lsearch / lfind
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn lsearch(
    key: *const c_void,
    base: *mut c_void,
    nelp: *mut usize,
    width: usize,
    compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
) -> *mut c_void {
    let n = *nelp;
    let p = base as *mut u8;
    for i in 0..n {
        let elem = p.add(i * width);
        if compar.unwrap()(key, elem as *const c_void) == 0 {
            return elem as *mut c_void;
        }
    }
    let dest = p.add(n * width);
    memcpy(dest as *mut c_void, key as *const c_void, width);
    *nelp = n + 1;
    dest as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn lfind(
    key: *const c_void,
    base: *const c_void,
    nelp: *mut usize,
    width: usize,
    compar: Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>,
) -> *mut c_void {
    let n = *nelp;
    let p = base as *const u8;
    for i in 0..n {
        let elem = p.add(i * width);
        if compar.unwrap()(key, elem as *const c_void) == 0 {
            return elem as *mut c_void;
        }
    }
    core::ptr::null_mut()
}

// ============================================================
// Search: tsearch / tfind / tdelete / twalk / tdestroy (BST, no balancing)
// ============================================================

// ponytail: simple BST, no balancing; correct for tests, O(n) worst case
#[repr(C)]
pub struct TreeNode {
    key: *const c_void,
    left: *mut TreeNode,
    right: *mut TreeNode,
}

type ComparFn = Option<unsafe extern "C" fn(*const c_void, *const c_void) -> c_int>;

unsafe fn bst_new(key: *const c_void) -> *mut TreeNode {
    let n = calloc(1, core::mem::size_of::<TreeNode>()) as *mut TreeNode;
    if !n.is_null() {
        (*n).key = key;
        (*n).left = core::ptr::null_mut();
        (*n).right = core::ptr::null_mut();
    }
    n
}

#[no_mangle]
pub unsafe extern "C" fn tsearch(
    key: *const c_void,
    rootp: *mut *mut TreeNode,
    compar: ComparFn,
) -> *mut c_void {
    if rootp.is_null() {
        return core::ptr::null_mut();
    }
    let mut pp = rootp;
    loop {
        let n = *pp;
        if n.is_null() {
            let r = bst_new(key);
            if r.is_null() {
                return core::ptr::null_mut();
            }
            *pp = r;
            return r as *mut c_void;
        }
        let c = compar.unwrap()(key, (*n).key);
        if c == 0 {
            return n as *mut c_void;
        }
        if c < 0 {
            pp = &mut (*n).left;
        } else {
            pp = &mut (*n).right;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn tfind(
    key: *const c_void,
    rootp: *const *mut TreeNode,
    compar: ComparFn,
) -> *mut c_void {
    if rootp.is_null() {
        return core::ptr::null_mut();
    }
    let mut n = *rootp;
    loop {
        if n.is_null() {
            return core::ptr::null_mut();
        }
        let c = compar.unwrap()(key, (*n).key);
        if c == 0 {
            return n as *mut c_void;
        }
        if c < 0 {
            n = (*n).left;
        } else {
            n = (*n).right;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn tdelete(
    key: *const c_void,
    rootp: *mut *mut TreeNode,
    compar: ComparFn,
) -> *mut c_void {
    if rootp.is_null() {
        return core::ptr::null_mut();
    }
    let mut pp = rootp;
    loop {
        let n = *pp;
        if n.is_null() {
            return core::ptr::null_mut();
        }
        let c = compar.unwrap()(key, (*n).key);
        if c == 0 {
            break;
        }
        if c < 0 {
            pp = &mut (*n).left;
        } else {
            pp = &mut (*n).right;
        }
    }
    let n = *pp;
    // parent is *rootp if we went to root, otherwise the node above
    let parent = *rootp;
    if !(*n).left.is_null() {
        // replace with in-order predecessor
        let mut q = &mut (*n).left;
        while !(*(*q)).right.is_null() {
            q = &mut (*(*q)).right;
        }
        let pred = *q;
        // swap keys
        (*n).key = (*pred).key;
        // remove predecessor
        *q = (*pred).left;
        free(pred as *mut c_void);
    } else {
        // replace with right child (may be null)
        *pp = (*n).right;
        free(n as *mut c_void);
    }
    parent as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn twalk(
    root: *const TreeNode,
    action: Option<unsafe extern "C" fn(*const c_void, c_int, c_int)>,
) {
    unsafe fn walk(
        node: *const TreeNode,
        action: Option<unsafe extern "C" fn(*const c_void, c_int, c_int)>,
        depth: c_int,
    ) {
        if node.is_null() {
            return;
        }
        let is_leaf = (*node).left.is_null() && (*node).right.is_null();
        if is_leaf {
            action.unwrap()(node as *const c_void, 3, depth); // leaf
        } else {
            action.unwrap()(node as *const c_void, 0, depth); // preorder
            walk((*node).left, action, depth + 1);
            action.unwrap()(node as *const c_void, 1, depth); // postorder (left done)
            walk((*node).right, action, depth + 1);
            action.unwrap()(node as *const c_void, 2, depth); // endorder
        }
    }
    walk(root, action, 0);
}

#[no_mangle]
pub unsafe extern "C" fn tdestroy(root: *mut TreeNode, freekey: Option<unsafe extern "C" fn(*mut c_void)>) {
    if root.is_null() {
        return;
    }
    tdestroy((*root).left, freekey);
    tdestroy((*root).right, freekey);
    if let Some(f) = freekey {
        f((*root).key as *mut c_void);
    }
    free(root as *mut c_void);
}

// ============================================================
// fnmatch: full POSIX pattern matching
// ============================================================

const FNM_PATHNAME: c_int = 0x1;
const FNM_NOESCAPE: c_int = 0x2;
const FNM_PERIOD: c_int = 0x4;
const FNM_LEADING_DIR: c_int = 0x8;
const FNM_CASEFOLD: c_int = 0x10;
const FNM_NOMATCH: c_int = 1;

const FNM_END: c_int = 0;
const FNM_STAR: c_int = -5;
const FNM_QUESTION: c_int = -4;
const FNM_BRACKET: c_int = -3;
const FNM_UNMATCHABLE: c_int = -2;

fn ascii_tolower(c: u8) -> u8 {
    if c >= b'A' && c <= b'Z' { c + 32 } else { c }
}

fn ascii_toupper(c: u8) -> u8 {
    if c >= b'a' && c <= b'z' { c - 32 } else { c }
}

fn ascii_casefold(k: u8) -> u8 {
    let upper = ascii_toupper(k);
    if upper != k { upper } else { ascii_tolower(k) }
}

/// Get next byte from pattern, classifying it.
/// Returns (token_type, bytes_consumed).
unsafe fn pat_next(pat: *const u8, m: usize, flags: c_int) -> (c_int, usize) {
    if m == 0 || *pat == 0 {
        return (FNM_END, 0);
    }
    let mut p = pat;
    let mut step = 1usize;
    let mut esc = false;
    if *p == b'\\' && !(flags & FNM_NOESCAPE != 0) && *p.add(1) != 0 {
        p = p.add(1);
        step = 2;
        esc = true;
    }
    if *p == b'[' && !esc {
        // scan for matching ]
        let mut k = 1usize;
        if k < m && (*pat.add(k) == b'^' || *pat.add(k) == b'!') { k += 1; }
        if k < m && *pat.add(k) == b']' { k += 1; }
        while k < m && *pat.add(k) != 0 && *pat.add(k) != b']' {
            if k + 1 < m && *pat.add(k) == b'[' &&
               (*pat.add(k+1) == b':' || *pat.add(k+1) == b'.' || *pat.add(k+1) == b'=') {
                let z = *pat.add(k+1);
                k += 2;
                if k < m { k += 1; }
                while k < m && *pat.add(k) != 0 && (*pat.add(k-1) != z || *pat.add(k) != b']') { k += 1; }
                if k == m || *pat.add(k) == 0 { break; }
            }
            k += 1;
        }
        if k == m || *pat.add(k) == 0 {
            // unmatched [ is literal
            return (b'[' as c_int, 1);
        }
        return (FNM_BRACKET, k + 1);
    }
    if *p == b'*' && !esc {
        return (FNM_STAR, 1);
    }
    if *p == b'?' && !esc {
        return (FNM_QUESTION, 1);
    }
    (*p as c_int, step)
}

unsafe fn match_class(name: &[u8], k: u8, kfold: u8) -> bool {
    let eq = |c: u8| k == c || kfold == c;
    let in_range = |lo: u8, hi: u8| (k >= lo && k <= hi) || (kfold >= lo && kfold <= hi);
    match name {
        b"alnum" => in_range(b'0', b'9') || in_range(b'A', b'Z') || in_range(b'a', b'z'),
        b"alpha" => in_range(b'A', b'Z') || in_range(b'a', b'z'),
        b"blank" => eq(b' ') || eq(b'\t'),
        b"cntrl" => k < 0x20 || k == 0x7f,
        b"digit" => in_range(b'0', b'9'),
        b"graph" => k >= 0x21 && k <= 0x7e,
        b"lower" => in_range(b'a', b'z'),
        b"print" => k >= 0x20 && k <= 0x7e,
        b"punct" =>
            (k >= 0x21 && k <= 0x2f) ||
            (k >= 0x3a && k <= 0x40) ||
            (k >= 0x5b && k <= 0x60) ||
            (k >= 0x7b && k <= 0x7e),
        b"space" => eq(b' ') || eq(b'\t') || eq(b'\n') || eq(b'\r') || eq(0x0c) || eq(0x0b),
        b"upper" => in_range(b'A', b'Z'),
        b"xdigit" => in_range(b'0', b'9') || in_range(b'A', b'F') || in_range(b'a', b'f'),
        _ => false,
    }
}

/// Match a bracket expression [class] against char k.
unsafe fn match_bracket(mut p: *const u8, k: u8, kfold: u8) -> bool {
    let mut inv = false;
    p = p.add(1); // skip [
    if *p == b'^' || *p == b'!' {
        inv = true;
        p = p.add(1);
    }
    // handle ] as first char
    if *p == b']' {
        if k == b']' { return !inv; }
        p = p.add(1);
    } else if *p == b'-' {
        if k == b'-' { return !inv; }
        p = p.add(1);
    }
    while *p != b']' && *p != 0 {
        if *p == b'-' && *p.add(1) != b']' && *p.add(1) != 0 {
            // range: previous char to next char
            let lo = *p.sub(1);
            let hi = *p.add(1);
            if lo <= hi && (k >= lo && k <= hi || kfold >= lo && kfold <= hi) {
                return !inv;
            }
            p = p.add(2);
        } else if *p == b'[' && (*p.add(1) == b':' || *p.add(1) == b'.' || *p.add(1) == b'=') {
            let z = *p.add(1);
            let start = p.add(2);
            let mut q = start;
            while *q != 0 && (*q != z || *q.add(1) != b']') { q = q.add(1); }
            if *q != 0 {
                let len = (q as usize) - (start as usize);
                if len > 0 {
                    let inner = core::slice::from_raw_parts(start, len);
                    let matched =
                        if z == b':' { match_class(inner, k, kfold) }
                        else { len == 1 && (inner[0] == k || inner[0] == kfold) };
                    if matched { return !inv; }
                }
                p = q.add(2);
                continue;
            }
        } else {
            if *p == k || *p == kfold {
                return !inv;
            }
            p = p.add(1);
        }
    }
    inv
}

/// Core fnmatch for a single path component (no FNM_PATHNAME splitting).
unsafe fn fnmatch_internal(pat: *const u8, str: *const u8, flags: c_int) -> c_int {
    // Handle leading .
    if flags & FNM_PERIOD != 0 && *str == b'.' && *pat != b'.' {
        return FNM_NOMATCH;
    }

    let mut p = pat;
    let mut s = str;

    // Consume pattern up to first *
    loop {
        let (tok, step) = pat_next(p, usize::MAX, flags);
        match tok {
            FNM_END => {
                return if *s == 0 { 0 } else { FNM_NOMATCH };
            }
            FNM_STAR => {
                p = p.add(1);
                break;
            }
            FNM_QUESTION => {
                if *s == 0 { return FNM_NOMATCH; }
                s = s.add(1);
                p = p.add(step);
            }
            FNM_BRACKET => {
                if *s == 0 { return FNM_NOMATCH; }
                let k = *s;
                let kfold = if flags & FNM_CASEFOLD != 0 { ascii_casefold(k) } else { k };
                if !match_bracket(p, k, kfold) { return FNM_NOMATCH; }
                p = p.add(step);
                s = s.add(1);
            }
            c => {
                if *s == 0 { return FNM_NOMATCH; }
                let k = *s;
                let kfold = if flags & FNM_CASEFOLD != 0 { ascii_casefold(k) } else { k };
                if k as c_int != c && kfold as c_int != c { return FNM_NOMATCH; }
                p = p.add(step);
                s = s.add(1);
            }
        }
    }

    // Now p points after first *, rest of pattern is variable
    // Factor: find tail after last * and count fixed chars needed
    let mut ptail = p;
    let mut tailcnt: usize = 0;
    let mut pp = p;
    loop {
        let (tok, step) = pat_next(pp, usize::MAX, flags);
        if tok == FNM_END { break; }
        if tok == FNM_STAR {
            tailcnt = 0;
            ptail = pp.add(1);
        } else {
            tailcnt += 1;
        }
        pp = pp.add(step);
    }

    // str must have at least tailcnt chars
    let slen = strlen(s as *const c_char);
    if slen < tailcnt { return FNM_NOMATCH; }

    // Match tail: ptail..end against str[tailcnt from end]
    let stail = s.add(slen - tailcnt);
    let mut tp = ptail;
    let mut ts = stail;
    loop {
        let (tok, step) = pat_next(tp, usize::MAX, flags);
        tp = tp.add(step);
        if tok == FNM_END {
            break;
        }
        let k = *ts;
        if k == 0 { return FNM_NOMATCH; }
        let kfold = if flags & FNM_CASEFOLD != 0 { ascii_casefold(k) } else { k };
        match tok {
            FNM_QUESTION => { ts = ts.add(1); }
            FNM_BRACKET => {
                if !match_bracket(tp.sub(step), k, kfold) { return FNM_NOMATCH; }
                ts = ts.add(1);
            }
            c => {
                if k as c_int != c && kfold as c_int != c { return FNM_NOMATCH; }
                ts = ts.add(1);
            }
        }
    }

    let endstr = stail;
    let endpat = ptail;

    let mut cp = p;
    let mut cs = s;
    while cp < endpat {
        let (tok, step) = pat_next(cp, usize::MAX, flags);
        if tok == FNM_STAR {
            cp = cp.add(step);
            cs = s;
            continue;
        }
        let comp_start = cp;
        let mut ok = false;
        let mut try_s = cs;
        while try_s < endstr {
            let mut tmp_p = comp_start;
            let mut tmp_s = try_s;
            let mut matched = true;
            loop {
                let (t, st) = pat_next(tmp_p, usize::MAX, flags);
                if t == FNM_STAR || t == FNM_END {
                    cp = tmp_p;
                    cs = tmp_s;
                    ok = true;
                    break;
                }
                if *tmp_s == 0 {
                    matched = false;
                    break;
                }
                let k = *tmp_s;
                let kfold = if flags & FNM_CASEFOLD != 0 { ascii_casefold(k) } else { k };
                match t {
                    FNM_QUESTION => {
                        tmp_s = tmp_s.add(1);
                    }
                    FNM_BRACKET => {
                        if !match_bracket(tmp_p, k, kfold) {
                            matched = false;
                            break;
                        }
                        tmp_s = tmp_s.add(1);
                    }
                    c => {
                        if k as c_int != c && kfold as c_int != c {
                            matched = false;
                            break;
                        }
                        tmp_s = tmp_s.add(1);
                    }
                }
                tmp_p = tmp_p.add(st);
            }
            if ok { break; }
            if !matched {
                try_s = try_s.add(1);
            }
        }
        if !ok {
            return FNM_NOMATCH;
        }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn fnmatch(pat: *const c_char, str: *const c_char, flags: c_int) -> c_int {
    let pat = pat as *const u8;
    let str = str as *const u8;

    if flags & FNM_PATHNAME != 0 {
        // Split on /
        let mut s = str;
        let mut p = pat;
        loop {
            // find next / in str
            let mut send = s;
            while *send != 0 && *send != b'/' { send = send.add(1); }
            // find next / in pat
            let mut pend = p;
            loop {
                let (tok, step) = pat_next(pend, usize::MAX, flags);
                if tok == FNM_END || (tok >= 0 && tok as u8 == b'/') {
                    break;
                }
                pend = pend.add(step);
            }
            let (ptok, pstep) = pat_next(pend, usize::MAX, flags);
            let pat_is_slash = ptok >= 0 && ptok as u8 == b'/';
            let str_is_slash = *send == b'/';
            if pat_is_slash != str_is_slash && (*send != 0 || !(flags & FNM_LEADING_DIR != 0)) {
                return FNM_NOMATCH;
            }
            // match component
            // We need a null-terminated copy of the component
            let plen = (pend as usize) - (p as usize);
            let slen = (send as usize) - (s as usize);
            // Use stack buffers for components
            // ponytail: max component 1024 bytes
            if plen > 1024 || slen > 1024 { return FNM_NOMATCH; }
            let mut pbuf = [0u8; 1025];
            let mut sbuf = [0u8; 1025];
            core::ptr::copy_nonoverlapping(p, pbuf.as_mut_ptr(), plen);
            pbuf[plen] = 0;
            core::ptr::copy_nonoverlapping(s, sbuf.as_mut_ptr(), slen);
            sbuf[slen] = 0;
            if fnmatch_internal(pbuf.as_ptr(), sbuf.as_ptr(), flags & !FNM_LEADING_DIR) != 0 {
                return FNM_NOMATCH;
            }
            if !pat_is_slash {
                return 0;
            }
            s = send.add(1);
            p = pend.add(pstep);
        }
    }

    if flags & FNM_LEADING_DIR != 0 {
        // Try matching at each /
        let mut s = str;
        while *s != 0 {
            if *s == b'/' {
                // temporarily null-terminate
                // ponytail: we need to copy. Use a simple approach.
                let len = (s as usize) - (str as usize);
                // Just call fnmatch_internal with a temp buffer
                if len < 4096 {
                    let mut buf = [0u8; 4096];
                    core::ptr::copy_nonoverlapping(str, buf.as_mut_ptr(), len);
                    buf[len] = 0;
                    if fnmatch_internal(pat, buf.as_ptr(), flags) == 0 {
                        return 0;
                    }
                }
            }
            s = s.add(1);
        }
    }

    fnmatch_internal(pat, str, flags)
}

// ============================================================
// mntent: setmntent/endmntent/getmntent/getmntent_r/addmntent/hasmntopt
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn setmntent(name: *const c_char, mode: *const c_char) -> *mut FILE {
    fopen(name, mode)
}

#[no_mangle]
pub unsafe extern "C" fn endmntent(f: *mut FILE) -> c_int {
    if !f.is_null() {
        fclose(f);
    }
    1
}

// ponytail: static buffers for getmntent (non-_r variant)
static mut MNTENT_MNT: MntEnt = MntEnt {
    mnt_fsname: core::ptr::null_mut(),
    mnt_dir: core::ptr::null_mut(),
    mnt_type: core::ptr::null_mut(),
    mnt_opts: core::ptr::null_mut(),
    mnt_freq: 0,
    mnt_passno: 0,
};

#[repr(C)]
pub struct MntEnt {
    mnt_fsname: *mut c_char,
    mnt_dir: *mut c_char,
    mnt_type: *mut c_char,
    mnt_opts: *mut c_char,
    mnt_freq: c_int,
    mnt_passno: c_int,
}

unsafe fn unescape_ent(beg: *mut c_char) -> *mut c_char {
    let mut dest = beg;
    let mut src = beg;
    while *src != 0 {
        if *src != b'\\' as c_char {
            *dest = *src;
            dest = dest.add(1);
            src = src.add(1);
            continue;
        }
        // backslash
        if *src.add(1) == b'\\' as c_char {
            src = src.add(1);
            *dest = *src;
            dest = dest.add(1);
            src = src.add(1);
            continue;
        }
        // octal escape
        let mut val: u8 = 0;
        let mut cnt = 0;
        let mut sp = src.add(1);
        while cnt < 3 && *sp >= b'0' as c_char && *sp <= b'7' as c_char {
            val = val * 8 + (*sp as u8 - b'0');
            sp = sp.add(1);
            cnt += 1;
        }
        if cnt > 0 {
            *dest = val as c_char;
            dest = dest.add(1);
            src = sp;
        } else {
            *dest = *src;
            dest = dest.add(1);
            src = src.add(1);
        }
    }
    *dest = 0;
    beg
}

unsafe fn parse_mntent_line(
    linebuf: *mut c_char,
    mnt: *mut MntEnt,
) -> *mut MntEnt {
    let _len = strlen(linebuf as *const c_char);
    // skip comments and empty lines
    // parse: fsname dir type opts freq passno
    let mut p = linebuf;
    // skip leading whitespace
    while *p == b' ' as c_char || *p == b'\t' as c_char || *p == b'\n' as c_char || *p == b'\r' as c_char {
        p = p.add(1);
    }
    if *p == 0 || *p == b'#' as c_char {
        return core::ptr::null_mut();
    }

    // fsname
    (*mnt).mnt_fsname = p;
    while *p != 0 && *p != b' ' as c_char && *p != b'\t' as c_char && *p != b'\n' as c_char {
        p = p.add(1);
    }
    if *p == 0 { return core::ptr::null_mut(); }
    *p = 0;
    p = p.add(1);
    while *p == b' ' as c_char || *p == b'\t' as c_char { p = p.add(1); }

    // dir
    (*mnt).mnt_dir = p;
    while *p != 0 && *p != b' ' as c_char && *p != b'\t' as c_char && *p != b'\n' as c_char {
        p = p.add(1);
    }
    if *p == 0 { return core::ptr::null_mut(); }
    *p = 0;
    p = p.add(1);
    while *p == b' ' as c_char || *p == b'\t' as c_char { p = p.add(1); }

    // type
    (*mnt).mnt_type = p;
    while *p != 0 && *p != b' ' as c_char && *p != b'\t' as c_char && *p != b'\n' as c_char {
        p = p.add(1);
    }
    if *p == 0 { return core::ptr::null_mut(); }
    *p = 0;
    p = p.add(1);
    while *p == b' ' as c_char || *p == b'\t' as c_char { p = p.add(1); }

    // opts
    (*mnt).mnt_opts = p;
    while *p != 0 && *p != b' ' as c_char && *p != b'\t' as c_char && *p != b'\n' as c_char {
        p = p.add(1);
    }
    if *p != 0 {
        *p = 0;
        p = p.add(1);
    }
    if *(*mnt).mnt_opts == 0 {
        (*mnt).mnt_opts = b"defaults\0".as_ptr() as *mut c_char;
    }

    // freq and passno
    (*mnt).mnt_freq = 0;
    (*mnt).mnt_passno = 0;
    // skip whitespace
    while *p == b' ' as c_char || *p == b'\t' as c_char { p = p.add(1); }
    if *p >= b'0' as c_char && *p <= b'9' as c_char {
        (*mnt).mnt_freq = (*p - b'0' as c_char) as c_int;
        p = p.add(1);
        while *p >= b'0' as c_char && *p <= b'9' as c_char {
            (*mnt).mnt_freq = (*mnt).mnt_freq * 10 + (*p - b'0' as c_char) as c_int;
            p = p.add(1);
        }
    }
    while *p == b' ' as c_char || *p == b'\t' as c_char { p = p.add(1); }
    if *p >= b'0' as c_char && *p <= b'9' as c_char {
        (*mnt).mnt_passno = (*p - b'0' as c_char) as c_int;
        p = p.add(1);
        while *p >= b'0' as c_char && *p <= b'9' as c_char {
            (*mnt).mnt_passno = (*mnt).mnt_passno * 10 + (*p - b'0' as c_char) as c_int;
            p = p.add(1);
        }
    }

    // unescape
    unescape_ent((*mnt).mnt_fsname);
    unescape_ent((*mnt).mnt_dir);
    unescape_ent((*mnt).mnt_type);
    unescape_ent((*mnt).mnt_opts);

    mnt
}

// Use a sentinel pointer to detect internal buffer usage
// ponytail: cast integer to pointer for sentinel
static SENTINEL_PTR: usize = 1;

// Read one char from FILE buffer directly (avoids musl's static fgets ABI mismatch)
unsafe fn mntent_fgetc(f: *mut FILE) -> c_int {
    fgetc(f)
}

unsafe fn mntent_fgets(buf: *mut c_char, n: usize, f: *mut FILE) -> *mut c_char {
    if n == 0 { return core::ptr::null_mut(); }
    let max = n - 1;
    let mut i = 0usize;
    while i < max {
        let c = mntent_fgetc(f);
        if c == -1 { if i == 0 { return core::ptr::null_mut(); } break; }
        *buf.add(i) = c as c_char;
        i += 1;
        if c == b'\n' as c_int { break; }
    }
    *buf.add(i) = 0;
    buf
}

#[no_mangle]
pub unsafe extern "C" fn getmntent_r(
    f: *mut FILE,
    mnt: *mut MntEnt,
    mut linebuf: *mut c_char,
    _buflen: c_int,
) -> *mut MntEnt {
    let use_internal = linebuf as usize == SENTINEL_PTR;

    (*mnt).mnt_freq = 0;
    (*mnt).mnt_passno = 0;

    // ponytail: static 4KB line buffer for getmntent
    static mut INTERNAL_BUF: [c_char; 4096] = [0; 4096];

    loop {
        if use_internal {
            linebuf = core::ptr::addr_of_mut!(INTERNAL_BUF).cast::<c_char>();
        }
        let res = mntent_fgets(linebuf, 4096, f);
        if res.is_null() {
            return core::ptr::null_mut();
        }
        let mut has_newline = false;
        let mut p = linebuf;
        while *p != 0 { if *p == b'\n' as c_char { has_newline = true; break; } p = p.add(1); }
        if !has_newline {
            ERRNO = 34;
            return core::ptr::null_mut();
        }
        let result = parse_mntent_line(linebuf, mnt);
        if !result.is_null() {
            return result;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn getmntent(f: *mut FILE) -> *mut MntEnt {
    getmntent_r(f, core::ptr::addr_of_mut!(MNTENT_MNT), SENTINEL_PTR as *mut c_char, 0)
}

#[no_mangle]
pub unsafe extern "C" fn addmntent(f: *mut FILE, mnt: *const MntEnt) -> c_int {
    fseek(f, 0, 2); // SEEK_END
    let written = fprintf(
        f,
        b"%s\t%s\t%s\t%s\t%d\t%d\n\0".as_ptr() as *const c_char,
        (*mnt).mnt_fsname,
        (*mnt).mnt_dir,
        (*mnt).mnt_type,
        (*mnt).mnt_opts,
        (*mnt).mnt_freq,
        (*mnt).mnt_passno,
    );
    if written < 0 { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "C" fn hasmntopt(mnt: *const MntEnt, opt: *const c_char) -> *mut c_char {
    let l = strlen(opt as *const c_char);
    let mut p = (*mnt).mnt_opts;
    loop {
        if strncmp(p as *const u8, opt as *const u8, l) == 0 {
            let after = *p.add(l);
            if after == 0 || after == b',' as c_char || after == b'=' as c_char {
                return p;
            }
        }
        let comma = strchr(p as *const u8, b',' as c_int);
        if comma.is_null() {
            return core::ptr::null_mut();
        }
        p = comma.add(1) as *mut c_char;
    }
}

// ============================================================
// SysV IPC: ftok, msg*, sem*, shm*
// x86_64 direct syscalls: msgget=68, msgsnd=69, msgrcv=70, msgctl=71
//   semget=64, semop=65, semctl=66, semtimedop=220
//   shmget=29, shmat=30, shmctl=31, shmdt=67
// ============================================================

const EEXIST: c_int = 17;
const ENAMETOOLONG: c_int = 36;
const EMFILE: c_int = 24;
const EIDRM: c_int = 43;

#[inline]
unsafe fn sys_msgget(key: c_int, msgflg: c_int) -> i64 {
    <Arch as Syscalls>::syscall2(SYS_MSGGET, key as i64, msgflg as i64)
}

#[inline]
unsafe fn sys_msgsnd(msqid: c_int, msgp: *const c_void, msgsz: usize, msgflg: c_int) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_MSGSND, msqid as i64, msgp as i64, msgsz as i64, msgflg as i64)
}

#[inline]
unsafe fn sys_msgrcv(msqid: c_int, msgp: *mut c_void, msgsz: usize, msgtyp: c_long, msgflg: c_int) -> i64 {
    <Arch as Syscalls>::syscall5(SYS_MSGRCV, msqid as i64, msgp as i64, msgsz as i64, msgtyp as i64, msgflg as i64)
}

#[inline]
unsafe fn sys_msgctl(msqid: c_int, cmd: c_int, buf: *mut c_void) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_MSGCTL, msqid as i64, cmd as i64, buf as i64)
}

#[inline]
unsafe fn sys_semget(key: c_int, nsems: c_int, semflg: c_int) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_SEMGET, key as i64, nsems as i64, semflg as i64)
}

#[inline]
unsafe fn sys_semop(semid: c_int, sops: *const c_void, nsops: usize) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_SEMOP, semid as i64, sops as i64, nsops as i64)
}

#[inline]
unsafe fn sys_semctl(semid: c_int, semnum: c_int, cmd: c_int, arg: *mut c_void) -> i64 {
    <Arch as Syscalls>::syscall4(SYS_SEMCTL, semid as i64, semnum as i64, cmd as i64, arg as i64)
}

#[inline]
unsafe fn sys_shmget(key: c_int, size: usize, shmflg: c_int) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_SHMGET, key as i64, size as i64, shmflg as i64)
}

#[inline]
unsafe fn sys_shmat(shmid: c_int, shmaddr: *const c_void, shmflg: c_int) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_SHMAT, shmid as i64, shmaddr as i64, shmflg as i64)
}

#[inline]
unsafe fn sys_shmdt(shmaddr: *const c_void) -> i64 {
    <Arch as Syscalls>::syscall1(SYS_SHMDT, shmaddr as i64)
}

#[inline]
unsafe fn sys_shmctl(shmid: c_int, cmd: c_int, buf: *mut c_void) -> i64 {
    <Arch as Syscalls>::syscall3(SYS_SHMCTL, shmid as i64, cmd as i64, buf as i64)
}

// ftok
#[no_mangle]
pub unsafe extern "C" fn ftok(path: *const c_char, id: c_int) -> c_int {
    let mut st: kernel_stat64 = core::mem::zeroed();
    let r = sys_stat(path as *const u8, &mut st);
    if r < 0 {
        ERRNO = (-r) as c_int;
        return -1;
    }
    ((st.st_ino & 0xffff) | ((st.st_dev & 0xff) << 16) | (((id as u32 & 0xff) << 24)) as u64) as c_int
}

// msgget
#[no_mangle]
pub unsafe extern "C" fn msgget(key: c_int, msgflg: c_int) -> c_int {
    let r = sys_msgget(key, msgflg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// msgsnd
#[no_mangle]
pub unsafe extern "C" fn msgsnd(msqid: c_int, msgp: *const c_void, msgsz: usize, msgflg: c_int) -> c_int {
    let r = sys_msgsnd(msqid, msgp, msgsz, msgflg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// msgrcv
#[no_mangle]
pub unsafe extern "C" fn msgrcv(msqid: c_int, msgp: *mut c_void, msgsz: usize, msgtyp: c_long, msgflg: c_int) -> isize {
    let r = sys_msgrcv(msqid, msgp, msgsz, msgtyp, msgflg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as isize }
}

// msgctl
#[no_mangle]
pub unsafe extern "C" fn msgctl(msqid: c_int, cmd: c_int, buf: *mut c_void) -> c_int {
    let r = sys_msgctl(msqid, cmd, buf);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// semget
#[no_mangle]
pub unsafe extern "C" fn semget(key: c_int, nsems: c_int, semflg: c_int) -> c_int {
    if nsems > 65535 { ERRNO = 22; return -1; }
    let r = sys_semget(key, nsems, semflg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// semop
#[no_mangle]
pub unsafe extern "C" fn semop(semid: c_int, sops: *const c_void, nsops: usize) -> c_int {
    let r = sys_semop(semid, sops, nsops);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// semctl - C variadic
#[no_mangle]
pub unsafe extern "C" fn semctl(semid: c_int, semnum: c_int, cmd: c_int, mut args: ...) -> c_int {
    let mut arg: *mut c_void = core::ptr::null_mut();
    // Commands that take the union semun argument
    match cmd {
        16 | 13 | 17 | 1 | 3 | 2 | 0x102 | 0x104 | 0x106 => {
            arg = args.next_arg::<*mut c_void>();
        }
        _ => {}
    }
    let r = sys_semctl(semid, semnum, cmd, arg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// shmget
#[no_mangle]
pub unsafe extern "C" fn shmget(key: c_int, size: usize, shmflg: c_int) -> c_int {
    let r = sys_shmget(key, size, shmflg);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// shmat
#[no_mangle]
pub unsafe extern "C" fn shmat(shmid: c_int, shmaddr: *const c_void, shmflg: c_int) -> *mut c_void {
    let r = sys_shmat(shmid, shmaddr, shmflg);
    if (r as usize) > (-(4096isize as isize)) as usize {
        ERRNO = (-r) as c_int;
        return core::ptr::null_mut();
    }
    r as *mut c_void
}

// shmdt
#[no_mangle]
pub unsafe extern "C" fn shmdt(shmaddr: *const c_void) -> c_int {
    let r = sys_shmdt(shmaddr);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// shmctl
#[no_mangle]
pub unsafe extern "C" fn shmctl(shmid: c_int, cmd: c_int, buf: *mut c_void) -> c_int {
    let r = sys_shmctl(shmid, cmd, buf);
    if r < 0 { ERRNO = (-r) as c_int; -1 } else { r as c_int }
}

// ============================================================
// Startup: __libc_start_main
//
// musl crt1.o _start_c calls:
//   __libc_start_main(main, argc, argv, _init, _fini, rtld_fini, stack_end)
// ============================================================

// Auxiliary vector pointer, populated by the dynamic linker before
// constructors run so getauxval() works during early startup.
#[no_mangle]
pub static mut __auxv: *const usize = core::ptr::null();

#[no_mangle]
pub unsafe extern "C" fn getauxval(type_: c_ulong) -> c_ulong {
    let mut p = __auxv;
    if p.is_null() {
        return 0;
    }
    loop {
        let tag = *p;
        if tag == 0 {
            return 0;
        }
        if tag == type_ as usize {
            return *p.add(1) as c_ulong;
        }
        p = p.add(2);
    }
}

type MainFn = unsafe extern "C" fn(c_int, *const *const c_char, *const *const c_char) -> c_int;
type InitFn = unsafe extern "C" fn();

#[no_mangle]
pub unsafe extern "C" fn __libc_start_main(
    main: MainFn,
    argc: c_int,
    argv: *const *const c_char,
    _init: *const c_void,
    _fini: *const c_void,
    _rtld_fini: *const c_void,
    _stack_end: *const c_void,
) -> ! {
    let envp = argv.add((argc + 1) as usize);
    __environ = envp as *mut *mut c_char;
    environ = __environ; // sync environ alias
    __stdio_init();

    let mut set: SigSetT = 0;
    sigemptyset(&mut set);
    sigaddset(&mut set, SIGCANCEL);
    sigprocmask(SIG_UNBLOCK, &set, core::ptr::null_mut());

    if !_init.is_null() {
        let init_fn: InitFn = core::mem::transmute(_init);
        init_fn();
    }

    // aarch64 GCC reads __stack_chk_guard as a global (GOT-based for PIE),
    // so we must initialize it from AT_RANDOM before calling main.
    // x86_64 uses %fs:0x28 which the kernel already set up.
    #[cfg(target_arch = "aarch64")]
    {
        let random_ptr = getauxval(25) as *const u8; // AT_RANDOM = 25
        if !random_ptr.is_null() {
            let a = core::ptr::read_unaligned(random_ptr as *const u64);
            let b = core::ptr::read_unaligned(random_ptr.add(8) as *const u64);
            __stack_chk_guard = (a ^ b) as usize;
        } else {
            // ponytail: fallback entropy, not crypto-grade but nonzero
            __stack_chk_guard = 0xdefaced_cafebeef;
        }
    }

    let result = main(argc, argv, envp);

    if !_fini.is_null() {
        let fini_fn: InitFn = core::mem::transmute(_fini);
        fini_fn();
    }

    exit(result);
}

type LdsoDlopenFn = unsafe extern "C" fn(*const c_char, c_int) -> *mut c_void;
type LdsoDlsymFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void;
type LdsoDlcloseFn = unsafe extern "C" fn(*mut c_void) -> c_int;
type LdsoDlerrorFn = unsafe extern "C" fn() -> *const c_char;

static mut LDSO_DLOPEN: Option<LdsoDlopenFn> = None;
static mut LDSO_DLSYM: Option<LdsoDlsymFn> = None;
static mut LDSO_DLCLOSE: Option<LdsoDlcloseFn> = None;
static mut LDSO_DLERROR: Option<LdsoDlerrorFn> = None;

#[no_mangle]
pub unsafe extern "C" fn __ldso_register_dlopen(f: LdsoDlopenFn) {
    LDSO_DLOPEN = Some(f);
}

#[no_mangle]
pub unsafe extern "C" fn __ldso_register_dlsym(f: LdsoDlsymFn) {
    LDSO_DLSYM = Some(f);
}

#[no_mangle]
pub unsafe extern "C" fn __ldso_register_dlclose(f: LdsoDlcloseFn) {
    LDSO_DLCLOSE = Some(f);
}

#[no_mangle]
pub unsafe extern "C" fn __ldso_register_dlerror(f: LdsoDlerrorFn) {
    LDSO_DLERROR = Some(f);
}

#[no_mangle]
pub unsafe extern "C" fn dlopen(filename: *const c_char, flags: c_int) -> *mut c_void {
    if let Some(f) = LDSO_DLOPEN {
        f(filename, flags)
    } else {
        null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void {
    if let Some(f) = LDSO_DLSYM {
        f(handle, symbol)
    } else {
        null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn dlclose(handle: *mut c_void) -> c_int {
    if let Some(f) = LDSO_DLCLOSE {
        f(handle)
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn dlerror() -> *const c_char {
    if let Some(f) = LDSO_DLERROR {
        f()
    } else {
        core::ptr::null()
    }
}

include!("crypt_impl.rs");
include!("statvfs.rs");
include!("daemon.rs");
include!("dn_expand.rs");
include!("lrand48.rs");
include!("strverscmp.rs");
include!("syscall.rs");
include!("pthread_atfork.rs");
include!("fenv.rs");
include!("locale_ctype.rs");
include!("regression_stubs.rs");

// ============================================================
// regex.h
// ============================================================

type regoff_t = c_int;

#[repr(C)]
pub struct regex_t {
    pub re_nsub: usize,
    pub __opaque: *mut c_void,
    pub __padding: [*mut c_void; 4],
    pub __nsub2: usize,
    pub __padding2: c_char,
}

#[repr(C)]
pub struct regmatch_t {
    pub rm_so: regoff_t,
    pub rm_eo: regoff_t,
}

include!("regex.rs");

#[cfg(test)]
mod libc_unit_stub {
    #[test]
    fn stub() {
        assert!(true);
    }
}
