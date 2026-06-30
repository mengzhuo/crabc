use core::ffi::{c_char, c_int, c_uint, c_ulong};

#[repr(C)]
struct kernel_statfs {
    f_type: c_ulong,
    f_bsize: c_ulong,
    f_blocks: u64,
    f_bfree: u64,
    f_bavail: u64,
    f_files: u64,
    f_ffree: u64,
    f_fsid: [c_int; 2],
    f_namelen: c_ulong,
    f_frsize: c_ulong,
    f_flags: c_ulong,
    f_spare: [c_ulong; 4],
}

#[repr(C)]
pub struct statvfs {
    pub f_bsize: c_ulong,
    pub f_frsize: c_ulong,
    pub f_blocks: u64,
    pub f_bfree: u64,
    pub f_bavail: u64,
    pub f_files: u64,
    pub f_ffree: u64,
    pub f_favail: u64,
    pub f_fsid: c_ulong,
    pub f_flag: c_ulong,
    pub f_namemax: c_ulong,
    pub f_type: c_uint,
    __reserved: [c_int; 5],
}

pub const ST_RDONLY: c_ulong = 1;
pub const ST_NOSUID: c_ulong = 2;
pub const ST_NODEV: c_ulong = 4;
pub const ST_NOEXEC: c_ulong = 8;
pub const ST_SYNCHRONOUS: c_ulong = 16;
pub const ST_MANDLOCK: c_ulong = 64;
pub const ST_WRITE: c_ulong = 128;
pub const ST_APPEND: c_ulong = 256;
pub const ST_IMMUTABLE: c_ulong = 512;
pub const ST_NOATIME: c_ulong = 1024;
pub const ST_NODIRATIME: c_ulong = 2048;
pub const ST_RELATIME: c_ulong = 4096;

// ponytail: SYS_statfs(137) vs SYS_statfs64(305) identical on x86_64
#[inline]
unsafe fn sys_statfs(path: *const c_char, buf: *mut kernel_statfs) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 137i64 => result,
        in("rdi") path,
        in("rsi") buf,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

#[inline]
unsafe fn sys_fstatfs(fd: c_int, buf: *mut kernel_statfs) -> i64 {
    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") 138i64 => result,
        in("rdi") fd as i64,
        in("rsi") buf,
        lateout("rcx") _,
        lateout("r11") _,
    );
    result
}

unsafe fn fixup(out: *mut statvfs, kbuf: *const kernel_statfs) {
    core::ptr::write_bytes(out, 0, 1);
    (*out).f_bsize = (*kbuf).f_bsize;
    (*out).f_frsize = if (*kbuf).f_frsize != 0 { (*kbuf).f_frsize } else { (*kbuf).f_bsize };
    (*out).f_blocks = (*kbuf).f_blocks;
    (*out).f_bfree = (*kbuf).f_bfree;
    (*out).f_bavail = (*kbuf).f_bavail;
    (*out).f_files = (*kbuf).f_files;
    (*out).f_ffree = (*kbuf).f_ffree;
    (*out).f_favail = (*kbuf).f_ffree;
    (*out).f_fsid = (*kbuf).f_fsid[0] as c_ulong;
    (*out).f_flag = (*kbuf).f_flags;
    (*out).f_namemax = (*kbuf).f_namelen;
    (*out).f_type = (*kbuf).f_type as c_uint;
}

extern "C" {
    fn __errno_location() -> *mut c_int;
}

#[no_mangle]
pub unsafe extern "C" fn statvfs(path: *const c_char, buf: *mut statvfs) -> c_int {
    let mut kbuf: kernel_statfs = core::mem::zeroed();
    let r = sys_statfs(path, &mut kbuf);
    if r < 0 {
        *(*__errno_location()) = (-r) as c_int;
        return -1;
    }
    fixup(buf, &kbuf);
    0
}

#[no_mangle]
pub unsafe extern "C" fn fstatvfs(fd: c_int, buf: *mut statvfs) -> c_int {
    let mut kbuf: kernel_statfs = core::mem::zeroed();
    let r = sys_fstatfs(fd, &mut kbuf);
    if r < 0 {
        *(*__errno_location()) = (-r) as c_int;
        return -1;
    }
    fixup(buf, &kbuf);
    0
}
