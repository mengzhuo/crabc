#[cfg(target_arch = "x86_64")]
pub type fexcept_t = u16;

#[cfg(target_arch = "aarch64")]
pub type fexcept_t = u32;

#[cfg(target_arch = "riscv64")]
pub type fexcept_t = u32;

// x87 fnstenv/fldenv (28 bytes) + stmxcsr/ldmxcsr (4 bytes) = 32 bytes
#[cfg(target_arch = "x86_64")]
#[repr(C)]
pub struct fenv_t {
    __x87_env: [u8; 28],
    __mxcsr: u32,
}

#[cfg(target_arch = "aarch64")]
#[repr(C)]
pub struct fenv_t {
    __fpcr: u32,
    __fpsr: u32,
}

#[cfg(target_arch = "riscv64")]
pub type fenv_t = u32;

#[cfg(target_arch = "x86_64")]
const FE_DFL_ENV: *const fenv_t = -1isize as *const fenv_t;
#[cfg(target_arch = "aarch64")]
const FE_DFL_ENV: *const fenv_t = -1isize as *const fenv_t;
#[cfg(target_arch = "riscv64")]
const FE_DFL_ENV: *const fenv_t = -1isize as *const fenv_t;

#[cfg(target_arch = "x86_64")]
mod feconst {
    use super::*;
    pub const FE_INVALID: c_int = 1;
    pub const FE_DIVBYZERO: c_int = 4;
    pub const FE_OVERFLOW: c_int = 8;
    pub const FE_UNDERFLOW: c_int = 16;
    pub const FE_INEXACT: c_int = 32;
    pub const FE_ALL_EXCEPT: c_int = 63;

    pub const FE_TONEAREST: c_int = 0;
    pub const FE_DOWNWARD: c_int = 0x400;
    pub const FE_UPWARD: c_int = 0x800;
    pub const FE_TOWARDZERO: c_int = 0xc00;
}

#[cfg(target_arch = "aarch64")]
mod feconst {
    use super::*;
    pub const FE_INVALID: c_int = 1;
    pub const FE_DIVBYZERO: c_int = 2;
    pub const FE_OVERFLOW: c_int = 4;
    pub const FE_UNDERFLOW: c_int = 8;
    pub const FE_INEXACT: c_int = 16;
    pub const FE_ALL_EXCEPT: c_int = 31;

    pub const FE_TONEAREST: c_int = 0;
    pub const FE_DOWNWARD: c_int = 0x800000;
    pub const FE_UPWARD: c_int = 0x400000;
    pub const FE_TOWARDZERO: c_int = 0xc00000;
}

#[cfg(target_arch = "riscv64")]
mod feconst {
    use super::*;
    pub const FE_INVALID: c_int = 16;
    pub const FE_DIVBYZERO: c_int = 8;
    pub const FE_OVERFLOW: c_int = 4;
    pub const FE_UNDERFLOW: c_int = 2;
    pub const FE_INEXACT: c_int = 1;
    pub const FE_ALL_EXCEPT: c_int = 31;

    pub const FE_TONEAREST: c_int = 0;
    pub const FE_DOWNWARD: c_int = 2;
    pub const FE_UPWARD: c_int = 3;
    pub const FE_TOWARDZERO: c_int = 1;
}

use feconst::*;

// x86_64 implementation using x87/MXCSR
#[cfg(target_arch = "x86_64")]
mod x86_64_imp {
    use super::*;

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
}

#[cfg(target_arch = "x86_64")]
use x86_64_imp::*;

// aarch64 implementation using FPCR/FPSR
#[cfg(target_arch = "aarch64")]
mod aarch64_imp {
    use super::*;

    #[no_mangle]
    pub unsafe extern "C" fn feclearexcept(excepts: c_int) -> c_int {
        let mask = (excepts & 0x1f) as u64;
        let mut fpsr: u64;
        core::arch::asm!("mrs {fpsr}, fpsr", fpsr = out(reg) fpsr);
        let new_fpsr = fpsr & !mask;
        core::arch::asm!("msr fpsr, {fpsr}", fpsr = in(reg) new_fpsr);
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn feraiseexcept(excepts: c_int) -> c_int {
        let mask = (excepts & 0x1f) as u64;
        let mut fpsr: u64;
        core::arch::asm!("mrs {fpsr}, fpsr", fpsr = out(reg) fpsr);
        let new_fpsr = fpsr | mask;
        core::arch::asm!("msr fpsr, {fpsr}", fpsr = in(reg) new_fpsr);
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn fetestexcept(excepts: c_int) -> c_int {
        let mask = (excepts & 0x1f) as u64;
        let mut fpsr: u64;
        core::arch::asm!("mrs {fpsr}, fpsr", fpsr = out(reg) fpsr);
        (fpsr & mask) as c_int
    }

    #[no_mangle]
    pub unsafe extern "C" fn fegetround() -> c_int {
        let mut fpcr: u64;
        core::arch::asm!("mrs {fpcr}, fpcr", fpcr = out(reg) fpcr);
        (fpcr as c_int) & 0xc00000
    }

    #[no_mangle]
    pub unsafe extern "C" fn fesetround(r: c_int) -> c_int {
        if r != FE_TONEAREST && r != FE_DOWNWARD && r != FE_UPWARD && r != FE_TOWARDZERO {
            return -1;
        }
        let mut fpcr: u64;
        core::arch::asm!("mrs {fpcr}, fpcr", fpcr = out(reg) fpcr);
        let new_fpcr = (fpcr & !0xc00000u64) | ((r as u64) & 0xc00000);
        core::arch::asm!("msr fpcr, {fpcr}", fpcr = in(reg) new_fpcr);
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn fegetenv(envp: *mut fenv_t) -> c_int {
        let mut fpcr: u64;
        let mut fpsr: u64;
        core::arch::asm!("mrs {fpcr}, fpcr", fpcr = out(reg) fpcr);
        core::arch::asm!("mrs {fpsr}, fpsr", fpsr = out(reg) fpsr);
        (*envp).__fpcr = fpcr as u32;
        (*envp).__fpsr = fpsr as u32;
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn fesetenv(envp: *const fenv_t) -> c_int {
        let (fpcr, fpsr) = if envp == FE_DFL_ENV {
            (0u64, 0u64)
        } else {
            ((*envp).__fpcr as u64, (*envp).__fpsr as u64)
        };
        core::arch::asm!("msr fpcr, {fpcr}", fpcr = in(reg) fpcr);
        core::arch::asm!("msr fpsr, {fpsr}", fpsr = in(reg) fpsr);
        0
    }
}

#[cfg(target_arch = "aarch64")]
use aarch64_imp::*;

// riscv64 implementation using fflags/frm/fcsr CSRs
#[cfg(target_arch = "riscv64")]
mod riscv64_imp {
    use super::*;

    #[no_mangle]
    pub unsafe extern "C" fn feclearexcept(excepts: c_int) -> c_int {
        let mask = (excepts & 0x1f) as u64;
        // csrc fflags, a0 — clear bits in fflags
        core::arch::asm!(
            "csrc fflags, {mask}",
            mask = in(reg) mask,
            options(nostack),
        );
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn feraiseexcept(excepts: c_int) -> c_int {
        let mask = (excepts & 0x1f) as u64;
        // csrs fflags, a0 — set bits in fflags
        core::arch::asm!(
            "csrs fflags, {mask}",
            mask = in(reg) mask,
            options(nostack),
        );
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn fetestexcept(excepts: c_int) -> c_int {
        let fflags: u64;
        // frflags t0 — read fflags
        core::arch::asm!(
            "frflags {fflags}",
            fflags = out(reg) fflags,
            options(nostack),
        );
        (fflags as c_int) & (excepts & 0x1f)
    }

    #[no_mangle]
    pub unsafe extern "C" fn fegetround() -> c_int {
        let frm: u64;
        // frrm a0 — read rounding mode
        core::arch::asm!(
            "frrm {frm}",
            frm = out(reg) frm,
            options(nostack),
        );
        frm as c_int
    }

    #[no_mangle]
    pub unsafe extern "C" fn fesetround(r: c_int) -> c_int {
        if r != FE_TONEAREST && r != FE_DOWNWARD && r != FE_UPWARD && r != FE_TOWARDZERO {
            return -1;
        }
        // fsrm t0, a0 — set rounding mode
        core::arch::asm!(
            "fsrm zero, {r}",
            r = in(reg) r as u64,
            options(nostack),
        );
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn fegetenv(envp: *mut fenv_t) -> c_int {
        let fcsr: u64;
        // frcsr t0 — read full CSR
        core::arch::asm!(
            "frcsr {fcsr}",
            fcsr = out(reg) fcsr,
            options(nostack),
        );
        *envp = fcsr as u32;
        0
    }

    #[no_mangle]
    pub unsafe extern "C" fn fesetenv(envp: *const fenv_t) -> c_int {
        let val = if envp == FE_DFL_ENV {
            0u64
        } else {
            *envp as u64
        };
        // fscsr t1 — set full CSR
        core::arch::asm!(
            "fscsr zero, {val}",
            val = in(reg) val,
            options(nostack),
        );
        0
    }
}

#[cfg(target_arch = "riscv64")]
use riscv64_imp::*;

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
