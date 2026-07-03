// syscall() - variadic syscall forwarding, musl-compatible.
// Add to libc/src/lib.rs in the "syscall wrappers" section.
// Matches musl x86_64 convention: rax=num, rdi,rsi,rdx,r10,r8,r9; rcx,r11 clobbered.

#[no_mangle]
pub unsafe extern "C" fn syscall(num: c_long, mut args: ...) -> c_long {
    let a: c_long = args.next_arg();
    let b: c_long = args.next_arg();
    let c: c_long = args.next_arg();
    let d: c_long = args.next_arg();
    let e: c_long = args.next_arg();
    let f: c_long = args.next_arg();

    let result: i64;
    core::arch::asm!(
        "syscall",
        inlateout("rax") num as i64 => result,
        in("rdi") a as i64,
        in("rsi") b as i64,
        in("rdx") c as i64,
        in("r10") d as i64,
        in("r8") e as i64,
        in("r9") f as i64,
        lateout("rcx") _,
        lateout("r11") _,
    );

    // ponytail: kernel returns -errno in low 4096 range; mirror musl __syscall_ret
    if (result as u64) > (-4096i64 as u64) {
        ERRNO = (-result) as c_int;
        return -1;
    }
    result as c_long
}
