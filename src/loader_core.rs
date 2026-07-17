#![allow(dead_code)]

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Ehdr {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Phdr {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

pub const ET_DYN: u16 = 3;
pub const ET_EXEC: u16 = 2;
pub const EM_X86_64: u16 = 62;
pub const EM_AARCH64: u16 = 183;
pub const EM_RISCV: u16 = 243;

pub const PT_LOAD: u32 = 1;
pub const PT_DYNAMIC: u32 = 2;
pub const PT_INTERP: u32 = 3;

pub const PF_R: u32 = 4;
pub const PF_W: u32 = 2;
pub const PF_X: u32 = 1;

pub const DT_NULL: u64 = 0;
pub const DT_RELA: u64 = 7;
pub const DT_RELAENT: u64 = 9;
pub const DT_RELASZ: u64 = 8;
pub const DT_JMPREL: u64 = 23;
pub const DT_PLTRELSZ: u64 = 2;

pub const R_X86_64_RELATIVE: u64 = 8;
pub const R_AARCH64_RELATIVE: u64 = 1027;
pub const R_RISCV_RELATIVE: u64 = 3;

#[cfg(target_arch = "x86_64")]
const R_RELATIVE: u64 = R_X86_64_RELATIVE;
#[cfg(target_arch = "aarch64")]
const R_RELATIVE: u64 = R_AARCH64_RELATIVE;
#[cfg(target_arch = "riscv64")]
const R_RELATIVE: u64 = R_RISCV_RELATIVE;

pub const AT_NULL: u64 = 0;
pub const AT_PHDR: u64 = 3;
pub const AT_PHNUM: u64 = 5;
pub const AT_PAGESZ: u64 = 6;
pub const AT_ENTRY: u64 = 9;
pub const AT_BASE: u64 = 7;

pub const PROT_READ: i32 = 1;
pub const PROT_WRITE: i32 = 2;
pub const PROT_EXEC: i32 = 4;
pub const PROT_NONE: i32 = 0;

pub const MAP_PRIVATE: i32 = 0x02;
pub const MAP_FIXED: i32 = 0x10;
pub const MAP_ANONYMOUS: i32 = 0x20;

pub const MAP_FAILED: *mut u8 = !0 as *mut u8;

pub const PAGE_SIZE: usize = 4096;

pub trait Syscalls {
    unsafe fn syscall0(n: i64) -> i64;
    unsafe fn syscall1(n: i64, a1: i64) -> i64;
    unsafe fn syscall2(n: i64, a1: i64, a2: i64) -> i64;
    unsafe fn syscall3(n: i64, a1: i64, a2: i64, a3: i64) -> i64;
    unsafe fn syscall4(n: i64, a1: i64, a2: i64, a3: i64, a4: i64) -> i64;
    unsafe fn syscall5(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64) -> i64;
    unsafe fn syscall6(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64, a6: i64) -> i64;
    unsafe fn syscall_noreturn1(n: i64, a1: i64) -> !;
}

pub struct X86_64;

#[cfg(target_arch = "x86_64")]
impl Syscalls for X86_64 {
    #[inline]
    unsafe fn syscall0(n: i64) -> i64 {
        let result: i64;
        unsafe {
            core::arch::asm!(
                "syscall",
                inlateout("rax") n => result,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        result
    }

    #[inline]
    unsafe fn syscall1(n: i64, a1: i64) -> i64 {
        let result: i64;
        unsafe {
            core::arch::asm!(
                "syscall",
                inlateout("rax") n => result,
                in("rdi") a1,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        result
    }

    #[inline]
    unsafe fn syscall2(n: i64, a1: i64, a2: i64) -> i64 {
        let result: i64;
        unsafe {
            core::arch::asm!(
                "syscall",
                inlateout("rax") n => result,
                in("rdi") a1,
                in("rsi") a2,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        result
    }

    #[inline]
    unsafe fn syscall3(n: i64, a1: i64, a2: i64, a3: i64) -> i64 {
        let result: i64;
        unsafe {
            core::arch::asm!(
                "syscall",
                inlateout("rax") n => result,
                in("rdi") a1,
                in("rsi") a2,
                in("rdx") a3,
                lateout("rcx") _,
                lateout("r11") _,
            );
        }
        result
    }

    #[inline]
    unsafe fn syscall4(n: i64, a1: i64, a2: i64, a3: i64, a4: i64) -> i64 {
        let result: i64;
        unsafe {
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
        }
        result
    }

    #[inline]
    unsafe fn syscall5(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64) -> i64 {
        let result: i64;
        unsafe {
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
        }
        result
    }

    #[inline]
    unsafe fn syscall6(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64, a6: i64) -> i64 {
        let result: i64;
        unsafe {
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
        }
        result
    }

    #[inline]
    unsafe fn syscall_noreturn1(n: i64, a1: i64) -> ! {
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") n,
                in("rdi") a1,
                options(noreturn)
            );
        }
    }
}

pub struct Aarch64;

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

pub struct Riscv64;

#[cfg(target_arch = "riscv64")]
impl Syscalls for Riscv64 {
    #[inline(always)]
    unsafe fn syscall0(n: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "ecall",
            inlateout("a7") n => _,
            lateout("a0") result,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall1(n: i64, a1: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "ecall",
            inlateout("a7") n => _,
            inlateout("a0") a1 => result,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall2(n: i64, a1: i64, a2: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "ecall",
            inlateout("a7") n => _,
            inlateout("a0") a1 => result,
            inlateout("a1") a2 => _,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall3(n: i64, a1: i64, a2: i64, a3: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "ecall",
            inlateout("a7") n => _,
            inlateout("a0") a1 => result,
            inlateout("a1") a2 => _,
            inlateout("a2") a3 => _,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall4(n: i64, a1: i64, a2: i64, a3: i64, a4: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "ecall",
            inlateout("a7") n => _,
            inlateout("a0") a1 => result,
            inlateout("a1") a2 => _,
            inlateout("a2") a3 => _,
            inlateout("a3") a4 => _,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall5(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "ecall",
            inlateout("a7") n => _,
            inlateout("a0") a1 => result,
            inlateout("a1") a2 => _,
            inlateout("a2") a3 => _,
            inlateout("a3") a4 => _,
            inlateout("a4") a5 => _,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall6(n: i64, a1: i64, a2: i64, a3: i64, a4: i64, a5: i64, a6: i64) -> i64 {
        let result: i64;
        core::arch::asm!(
            "ecall",
            inlateout("a7") n => _,
            inlateout("a0") a1 => result,
            inlateout("a1") a2 => _,
            inlateout("a2") a3 => _,
            inlateout("a3") a4 => _,
            inlateout("a4") a5 => _,
            inlateout("a5") a6 => _,
            options(nostack),
        );
        result
    }
    #[inline(always)]
    unsafe fn syscall_noreturn1(n: i64, a1: i64) -> ! {
        core::arch::asm!(
            "ecall",
            in("a7") n,
            in("a0") a1,
            options(noreturn, nostack),
        );
    }
}

#[cfg(target_arch = "x86_64")]
pub type Arch = X86_64;
#[cfg(target_arch = "aarch64")]
pub type Arch = Aarch64;
#[cfg(target_arch = "riscv64")]
pub type Arch = Riscv64;



// Architecture-specific syscall numbers
#[cfg(target_arch = "x86_64")]
mod sysnr {
    pub const SYS_MMAP: i64 = 9;
}
#[cfg(target_arch = "aarch64")]
mod sysnr {
    pub const SYS_MMAP: i64 = 222;
}
#[cfg(target_arch = "riscv64")]
mod sysnr {
    pub const SYS_MMAP: i64 = 222;
}
pub use sysnr::*;

#[inline]
pub unsafe fn sys_mmap(
    addr: *mut u8,
    length: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: i64,
) -> *mut u8 {
    unsafe { <Arch as Syscalls>::syscall6(SYS_MMAP, addr as i64, length as i64, prot as i64, flags as i64, fd as i64, offset) as *mut u8 }
}

pub fn parse_ehdr(data: &[u8]) -> Result<Ehdr, &'static str> {
    if data.len() < 64 {
        return Err("file too small for ELF header");
    }
    if &data[..4] != b"\x7fELF" {
        return Err("bad ELF magic");
    }
    if data[4] != 2 {
        return Err("not ELFCLASS64");
    }
    let ehdr = unsafe { &*(data.as_ptr() as *const Ehdr) };
    if ehdr.e_machine != EM_X86_64 && ehdr.e_machine != EM_AARCH64 && ehdr.e_machine != EM_RISCV {
        return Err("not a supported EM machine");
    }
    Ok(*ehdr)
}

pub fn parse_phdrs<'a>(data: &'a [u8], ehdr: &Ehdr) -> Result<&'a [Phdr], &'static str> {
    let end = (ehdr.e_phoff as usize)
        .checked_add(ehdr.e_phnum as usize * ehdr.e_phentsize as usize)
        .ok_or("phdr table overflow")?;
    if end > data.len() {
        return Err("phdr table extends past EOF");
    }
    let base = unsafe { data.as_ptr().add(ehdr.e_phoff as usize) as *const Phdr };
    Ok(unsafe { core::slice::from_raw_parts(base, ehdr.e_phnum as usize) })
}

pub fn map_segments(phdrs: &[Phdr], fd: i32, e_type: u16) -> Result<u64, &'static str> {
    let page = PAGE_SIZE as u64;

    let mut min = u64::MAX;
    let mut max = 0u64;
    for p in phdrs.iter().filter(|p| p.p_type == PT_LOAD) {
        let seg_start = p.p_vaddr & !(page - 1);
        let seg_end = (p.p_vaddr + p.p_memsz + page - 1) & !(page - 1);
        min = min.min(seg_start);
        max = max.max(seg_end);
    }
    if min >= max {
        return Err("no PT_LOAD segments");
    }

    let mut load_bias = 0u64;
    if e_type == ET_DYN {
        let span = max - min;
        let reserve = unsafe {
            sys_mmap(
                core::ptr::null_mut(),
                span as usize,
                PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if reserve == MAP_FAILED {
            return Err("mmap failed while reserving ET_DYN span");
        }
        load_bias = reserve as u64 - min;
    }

    for ph in phdrs.iter().filter(|p| p.p_type == PT_LOAD) {
        let adj = ph.p_vaddr & (page - 1);
        let map_addr = load_bias + ph.p_vaddr - adj;
        let map_off = ph.p_offset - adj;
        let map_len = (ph.p_filesz + adj + page - 1) & !(page - 1);

        let prot = prot_from_flags(ph.p_flags);
        let ptr = unsafe {
            sys_mmap(
                map_addr as *mut u8,
                map_len as usize,
                prot,
                MAP_PRIVATE | MAP_FIXED,
                fd,
                map_off as i64,
            )
        };
        if ptr == MAP_FAILED {
            return Err("mmap failed for PT_LOAD segment");
        }

        if ph.p_memsz > ph.p_filesz {
            let bss_start = (load_bias + ph.p_vaddr + ph.p_filesz) as *mut u8;
            let bss_len = (ph.p_memsz - ph.p_filesz) as usize;
            unsafe { core::ptr::write_bytes(bss_start, 0, bss_len) };
        }
    }

    Ok(load_bias)
}

pub fn apply_relocations(data: &[u8], phdrs: &[Phdr], base_addr: u64) -> Result<(), &'static str> {
    let dyn_ph = match phdrs.iter().find(|p| p.p_type == PT_DYNAMIC) {
        Some(p) => p,
        None => return Ok(()),
    };

    apply_relocations_from_dynamic(
        data,
        dyn_ph.p_offset as usize,
        dyn_ph.p_filesz as usize,
        base_addr,
    )
}

pub fn apply_relocations_from_dynamic(
    data: &[u8],
    dyn_off: usize,
    dyn_filesz: usize,
    base_addr: u64,
) -> Result<(), &'static str> {
    let dyn_end = dyn_off + dyn_filesz;
    if dyn_end > data.len() {
        return Err("PT_DYNAMIC extends past EOF");
    }

    let mut rela_off = 0u64;
    let mut rela_sz = 0u64;
    let mut rela_ent = 24u64;

    let mut pos = dyn_off;
    loop {
        if pos + 16 > data.len() {
            break;
        }
        let d_tag = u64::from_le_bytes(data[pos..pos + 8].try_into().unwrap());
        let d_val = u64::from_le_bytes(data[pos + 8..pos + 16].try_into().unwrap());
        if d_tag == DT_NULL {
            break;
        }
        match d_tag {
            DT_RELA => rela_off = d_val,
            DT_RELASZ => rela_sz = d_val,
            DT_RELAENT => rela_ent = d_val,
            _ => {}
        }
        pos += 16;
    }

    if rela_sz == 0 {
        return Ok(());
    }

    let count = rela_sz / rela_ent;
    for i in 0..count {
        let off = (rela_off as usize) + (i as usize * rela_ent as usize);
        let end = off + 24;
        if end > data.len() {
            return Err("relocation entry extends past EOF");
        }
        let r_offset = u64::from_le_bytes(data[off..off + 8].try_into().unwrap());
        let r_info = u64::from_le_bytes(data[off + 8..off + 16].try_into().unwrap());
        let r_addend = i64::from_le_bytes(data[off + 16..off + 24].try_into().unwrap());

        if r_info & 0xffffffff == R_RELATIVE {
            let ptr = (base_addr + r_offset) as *mut u64;
            let val = (base_addr as i64 + r_addend) as u64;
            unsafe { *ptr = val };
        }
    }

    Ok(())
}

// Self-relocate: read DT_RELA/DT_RELASZ from in-memory PT_DYNAMIC, apply RELATIVE entries.
// Only stack-local variables and raw pointer writes before this returns.
pub unsafe fn self_relocate(base: usize, phdrs: &[Phdr]) {
    for ph in phdrs.iter().filter(|p| p.p_type == PT_DYNAMIC) {
        let dyn_start = base + ph.p_vaddr as usize;
        let dyn_end = dyn_start + ph.p_memsz as usize;
        let mut pos = dyn_start;

        let mut rela_ptr: *const u8 = core::ptr::null();
        let mut rela_sz: usize = 0;

        while pos + 16 <= dyn_end {
            let d_tag = u64::from_le_bytes(unsafe { *(pos as *const [u8; 8]) });
            let d_val = u64::from_le_bytes(unsafe { *(((pos + 8) as *const [u8; 8])) });
            if d_tag == DT_NULL {
                break;
            }
            match d_tag {
                DT_RELA => rela_ptr = (base + d_val as usize) as *const u8,
                DT_RELASZ => rela_sz = d_val as usize,
                _ => {}
            }
            pos += 16;
        }

        if rela_ptr.is_null() || rela_sz == 0 {
            continue;
        }

        let count = rela_sz / 24;
        for i in 0..count {
            let entry = unsafe { rela_ptr.add(i * 24) };
            let r_offset = u64::from_le_bytes(unsafe { *(entry as *const [u8; 8]) });
            let r_info = u64::from_le_bytes(unsafe { *((entry.add(8)) as *const [u8; 8]) });
            let r_addend = i64::from_le_bytes(unsafe { *((entry.add(16)) as *const [u8; 8]) });

            if r_info & 0xffffffff == R_RELATIVE {
                let ptr = (base as u64 + r_offset) as *mut u64;
                let val = (base as i64 + r_addend) as u64;
                unsafe { *ptr = val };
            }
        }
    }
}

pub fn prot_from_flags(flags: u32) -> i32 {
    let mut prot = 0;
    if flags & PF_R != 0 {
        prot |= PROT_READ;
    }
    if flags & PF_W != 0 {
        prot |= PROT_WRITE;
    }
    if flags & PF_X != 0 {
        prot |= PROT_EXEC;
    }
    prot
}
