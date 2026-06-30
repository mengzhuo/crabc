// daemon() - ported from musl/src/legacy/daemon.c
// Include in libc/src/lib.rs in the "unistd.h: process primitives" section.

const ROOT_PATH: &[u8] = b"/\0";
const DEV_NULL: &[u8] = b"/dev/null\0";

#[no_mangle]
pub unsafe extern "C" fn daemon(nochdir: c_int, noclose: c_int) -> c_int {
    if nochdir == 0 {
        let r = sys_chdir(ROOT_PATH.as_ptr());
        if r < 0 {
            ERRNO = (-r) as c_int;
            return -1;
        }
    }
    if noclose == 0 {
        let fd = sys_open(DEV_NULL.as_ptr(), O_RDWR as i64, 0);
        if fd < 0 {
            ERRNO = (-fd) as c_int;
            return -1;
        }
        let mut failed = false;
        let mut err = 0i64;
        let r0 = sys_dup2(fd as i32, 0);
        if r0 < 0 {
            failed = true;
            err = -r0;
        }
        let r1 = sys_dup2(fd as i32, 1);
        if r1 < 0 && !failed {
            failed = true;
            err = -r1;
        }
        let r2 = sys_dup2(fd as i32, 2);
        if r2 < 0 && !failed {
            failed = true;
            err = -r2;
        }
        if fd > 2 {
            sys_close(fd);
        }
        if failed {
            ERRNO = err as c_int;
            return -1;
        }
    }
    let ret = sys_fork();
    match ret {
        0 => {}
        -1 => {
            ERRNO = (-ret) as c_int;
            return -1;
        }
        _ => _exit(0),
    }
    let r = sys_setsid();
    if r < 0 {
        ERRNO = (-r) as c_int;
        return -1;
    }
    let ret = sys_fork();
    match ret {
        0 => {}
        -1 => {
            ERRNO = (-ret) as c_int;
            return -1;
        }
        _ => _exit(0),
    }
    0
}
