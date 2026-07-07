pub type fexcept_t = u16;

// x87 fnstenv/fldenv (28 bytes) + stmxcsr/ldmxcsr (4 bytes) = 32 bytes
#[repr(C)]
pub struct fenv_t {
    __x87_env: [u8; 28],
    __mxcsr: u32,
}

const FE_INVALID: c_int = 1;
const FE_DIVBYZERO: c_int = 4;
const FE_OVERFLOW: c_int = 8;
const FE_UNDERFLOW: c_int = 16;
const FE_INEXACT: c_int = 32;
const FE_ALL_EXCEPT: c_int = 63;

const FE_TONEAREST: c_int = 0;
const FE_DOWNWARD: c_int = 0x400;
const FE_UPWARD: c_int = 0x800;
const FE_TOWARDZERO: c_int = 0xc00;

// musl: ((const fenv_t *) -1)
const FE_DFL_ENV: *const fenv_t = -1isize as *const fenv_t;

// CW=0x037f (all masked, nearest, extended), tags=0xffff (all empty)
const DEFAULT_X87_ENV: [u8; 28] = {
    let mut env = [0u8; 28];
    env[0] = 0x7f;
    env[1] = 0x03;
    env[8] = 0xff;
    env[9] = 0xff;
    env
};
const DEFAULT_MXCSR: u32 = 0x1f80;

// musl semantics: clears x87, merges old x87 into MXCSR then clears MXCSR
#[no_mangle]
pub unsafe extern "C" fn feclearexcept(excepts: c_int) -> c_int {
    let mask = excepts & 0x3f;
    let sw: u16;
    core::arch::asm!("fnstsw ax", out("ax") sw);
    if (sw as c_int) & mask != 0 {
        core::arch::asm!("fnclex");
    }
    let mut mxcsr: u32 = 0;
    let p = &mut mxcsr as *mut u32;
    core::arch::asm!("stmxcsr [{p}]", p = in(reg) p);
    let mxcsr_val = core::ptr::read(p) | ((sw as u32) & 0x3f);
    if mxcsr_val & (mask as u32) != 0 {
        core::ptr::write(p, mxcsr_val & !(mask as u32));
        core::arch::asm!("ldmxcsr [{p}]", p = in(reg) p);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn feraiseexcept(excepts: c_int) -> c_int {
    let mask = (excepts & 0x3f) as u32;
    let mut mxcsr: u32 = 0;
    let p = &mut mxcsr as *mut u32;
    core::arch::asm!("stmxcsr [{p}]", p = in(reg) p);
    core::ptr::write(p, core::ptr::read(p) | mask);
    core::arch::asm!("ldmxcsr [{p}]", p = in(reg) p);
    0
}

#[no_mangle]
pub unsafe extern "C" fn fetestexcept(excepts: c_int) -> c_int {
    let mask = excepts & 0x3f;
    let mut mxcsr: u32 = 0;
    let p = &mut mxcsr as *mut u32;
    core::arch::asm!("stmxcsr [{p}]", p = in(reg) p);
    let sw: u16;
    core::arch::asm!("fnstsw ax", out("ax") sw);
    ((sw as c_int) | (core::ptr::read(p) as c_int)) & mask
}

#[no_mangle]
pub unsafe extern "C" fn fegetexceptflag(fp: *mut fexcept_t, mask: c_int) -> c_int {
    *fp = fetestexcept(mask) as fexcept_t;
    0
}

#[no_mangle]
pub unsafe extern "C" fn fesetexceptflag(fp: *const fexcept_t, mask: c_int) -> c_int {
    let flags = *fp as c_int;
    feclearexcept((!flags) & mask);
    feraiseexcept(flags & mask);
    0
}

#[no_mangle]
pub unsafe extern "C" fn fegetround() -> c_int {
    let mut mxcsr: u32 = 0;
    let p = &mut mxcsr as *mut u32;
    core::arch::asm!("stmxcsr [{p}]", p = in(reg) p);
    // MXCSR bits 13-14 map to x87 CW bits 10-11
    ((core::ptr::read(p) >> 3) & 0xc00) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn fesetround(r: c_int) -> c_int {
    if r != FE_TONEAREST && r != FE_DOWNWARD && r != FE_UPWARD && r != FE_TOWARDZERO {
        return -1;
    }
    let mut cw: u16 = 0;
    let p_cw = &mut cw as *mut u16;
    core::arch::asm!("fnstcw [{p}]", p = in(reg) p_cw);
    core::ptr::write(p_cw, (core::ptr::read(p_cw) & 0xf3ff) | (r as u16 & 0x0c00));
    core::arch::asm!("fldcw [{p}]", p = in(reg) p_cw);

    let mut mxcsr: u32 = 0;
    let p_mx = &mut mxcsr as *mut u32;
    core::arch::asm!("stmxcsr [{p}]", p = in(reg) p_mx);
    core::ptr::write(p_mx, (core::ptr::read(p_mx) & 0xffff9fff) | ((r as u32 & 0xc00) << 3));
    core::arch::asm!("ldmxcsr [{p}]", p = in(reg) p_mx);
    0
}

#[no_mangle]
pub unsafe extern "C" fn fegetenv(envp: *mut fenv_t) -> c_int {
    core::arch::asm!("fnstenv [{p}]", p = in(reg) envp);
    let mxcsr_p = (envp as *mut u8).add(28) as *mut u32;
    core::arch::asm!("stmxcsr [{p}]", p = in(reg) mxcsr_p);
    0
}

#[no_mangle]
pub unsafe extern "C" fn fesetenv(envp: *const fenv_t) -> c_int {
    if envp == FE_DFL_ENV {
        core::arch::asm!("fldenv [{p}]", p = in(reg) DEFAULT_X87_ENV.as_ptr());
        let p = &DEFAULT_MXCSR as *const u32;
        core::arch::asm!("ldmxcsr [{p}]", p = in(reg) p);
    } else {
        core::arch::asm!("fldenv [{p}]", p = in(reg) envp);
        let mxcsr_p = (envp as *const u8).add(28) as *const u32;
        core::arch::asm!("ldmxcsr [{p}]", p = in(reg) mxcsr_p);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn feholdexcept(envp: *mut fenv_t) -> c_int {
    fegetenv(envp);
    feclearexcept(FE_ALL_EXCEPT);
    0
}

#[no_mangle]
pub unsafe extern "C" fn feupdateenv(envp: *const fenv_t) -> c_int {
    let ex = fetestexcept(FE_ALL_EXCEPT);
    fesetenv(envp);
    feraiseexcept(ex);
    0
}

// C99 FLT_ROUNDS: 0=towardzero, 1=nearest, 2=upward, 3=downward
#[no_mangle]
pub extern "C" fn __flt_rounds() -> c_int {
    match unsafe { fegetround() } {
        FE_TOWARDZERO => 0,
        FE_TONEAREST => 1,
        FE_UPWARD => 2,
        FE_DOWNWARD => 3,
        _ => -1,
    }
}
