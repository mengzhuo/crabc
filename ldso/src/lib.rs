#![no_std]
#![no_main]
#![allow(dead_code, deref_nullptr)]

use core::ffi::{c_char, c_void};
use core::sync::atomic::{AtomicBool, Ordering};

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// ============================================================
// Constants
// ============================================================

const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const PT_TLS: u32 = 7;
const PF_R: u32 = 4;
const PF_W: u32 = 2;
const PF_X: u32 = 1;

const DT_NULL: u64 = 0;
const DT_NEEDED: u64 = 1;
const DT_PLTRELSZ: u64 = 2;
const DT_HASH: u64 = 4;
const DT_STRTAB: u64 = 5;
const DT_SYMTAB: u64 = 6;
const DT_RELA: u64 = 7;
const DT_RELASZ: u64 = 8;
const DT_STRSZ: u64 = 10;
const DT_INIT: u64 = 12;
const DT_FINI: u64 = 13;
const DT_RPATH: u64 = 15;
const DT_JMPREL: u64 = 23;
const DT_INIT_ARRAY: u64 = 25;
const DT_FINI_ARRAY: u64 = 26;
const DT_INIT_ARRAYSZ: u64 = 27;
const DT_FINI_ARRAYSZ: u64 = 28;
const DT_RUNPATH: u64 = 29;
const DT_GNU_HASH: u64 = 0x6ffffef5;

const R_X86_64_64: u64 = 1;
const R_X86_64_COPY: u64 = 5;
const R_X86_64_GLOB_DAT: u64 = 6;
const R_X86_64_JUMP_SLOT: u64 = 7;
const R_X86_64_RELATIVE: u64 = 8;
const R_X86_64_DTPMOD64: u64 = 16;
const R_X86_64_DTPOFF64: u64 = 17;
const R_X86_64_TPOFF64: u64 = 18;

const R_AARCH64_NONE: u64 = 0;
const R_AARCH64_ABS64: u64 = 257;
const R_AARCH64_GLOB_DAT: u64 = 1025;
const R_AARCH64_JUMP_SLOT: u64 = 1026;
const R_AARCH64_RELATIVE: u64 = 1027;
const R_AARCH64_TLS_DTPMOD64: u64 = 1029;
const R_AARCH64_TLS_DTPREL64: u64 = 1030;
const R_AARCH64_TLS_TPREL64: u64 = 1031;
const R_AARCH64_TLSLE_ADD_TPREL_HI12: u64 = 549;
const R_AARCH64_TLSLE_ADD_TPREL_LO12: u64 = 550;
const R_AARCH64_TLSLE_ADD_TPREL_LO12_NC: u64 = 551;

const RTLD_LAZY: i32 = 1;
const RTLD_NOW: i32 = 2;
const RTLD_LOCAL: i32 = 0;
const RTLD_GLOBAL: i32 = 0x100;

const AT_NULL: u64 = 0;
const AT_PHDR: u64 = 3;
const AT_PHENT: u64 = 4;
const AT_PHNUM: u64 = 5;
const AT_PAGESZ: u64 = 6;
const AT_BASE: u64 = 7;
const AT_ENTRY: u64 = 9;
const AT_UID: u64 = 11;
const AT_EUID: u64 = 12;
const AT_GID: u64 = 13;
const AT_EGID: u64 = 14;
const AT_SECURE: u64 = 23;
const AT_RANDOM: u64 = 25;

const PROT_READ: i32 = 1;
const PROT_WRITE: i32 = 2;
const PROT_EXEC: i32 = 4;
const PROT_NONE: i32 = 0;
const MAP_PRIVATE: i32 = 0x02;
const MAP_FIXED: i32 = 0x10;
const MAP_ANONYMOUS: i32 = 0x20;
const MAP_FAILED: usize = !0usize;

const PHDR_SIZE: usize = 56;
const PH_TYPE: usize = 0;
const PH_FLAGS: usize = 4;
const PH_OFFSET: usize = 8;
const PH_VADDR: usize = 16;
const PH_FILESZ: usize = 32;
const PH_MEMSZ: usize = 40;
const PH_ALIGN: usize = 48;

const SYMTAB_ENT_SIZE: usize = 24;
const MAX_LOADED: usize = 16;
const TCB_SIZE: usize = 256;
const DSO_BASE_START: u64 = 0x200000000;
const DSO_BASE_STRIDE: u64 = 0x100000000;

// ============================================================
// Loaded object tracking
// ============================================================

struct LoadedObject {
    base: u64,
    symtab: *const u8,
    sym_count: usize,
    strtab: *const u8,
    strsz: usize,
    dyn_addr: usize,
    dyn_memsz: usize,
    tls_image: *const u8,
    tls_filesz: u64,
    tls_memsz: u64,
    tls_align: u64,
    init: u64,
    init_array: u64,
    init_array_sz: u64,
    init_present: bool,
    init_array_present: bool,
    global: bool,
    name: [u8; 256],
}

const EMPTY_OBJ: LoadedObject = LoadedObject {
    base: 0,
    symtab: core::ptr::null(),
    sym_count: 0,
    strtab: core::ptr::null(),
    strsz: 0,
    dyn_addr: 0,
    dyn_memsz: 0,
    tls_image: core::ptr::null(),
    tls_filesz: 0,
    tls_memsz: 0,
    tls_align: 0,
    init: 0,
    init_array: 0,
    init_array_sz: 0,
    init_present: false,
    init_array_present: false,
    global: false,
    name: [0; 256],
};

// Safety: only accessed from single-threaded _start -> run_main
static mut LOADED: [LoadedObject; MAX_LOADED] = [EMPTY_OBJ; MAX_LOADED];
static mut LOADED_COUNT: usize = 0;

static mut TLS_TOTAL_SIZE: usize = 0;
static mut TLS_LAYOUT_OFFSET: [usize; MAX_LOADED] = [0; MAX_LOADED];
static mut TLS_FILESZ: [u64; MAX_LOADED] = [0; MAX_LOADED];
static mut TLS_MEMSZ: [u64; MAX_LOADED] = [0; MAX_LOADED];
static mut TLS_IMAGE: [*const u8; MAX_LOADED] = [core::ptr::null(); MAX_LOADED];
static mut TLS_MODULE_COUNT: usize = 0;
static mut TLS_GENERATION: u64 = 1;
static mut TLS_OLD_TOTAL: usize = 0;
static mut TLS_OLD_MODULE_COUNT: usize = 0;
static TLS_LOCK: AtomicBool = AtomicBool::new(false);
static mut LD_LIBRARY_PATH: *const u8 = core::ptr::null();
static mut RUNPATH: *const u8 = core::ptr::null();
static mut RUNPATH_LEN: usize = 0;
static mut ORIGIN_DIR: [u8; 256] = [0; 256];
static mut ORIGIN_LEN: usize = 0;

// ============================================================
// _start: self-relocate ldso, then call run_main(sp)
// ============================================================

#[cfg(not(test))]
#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".global _start",
    ".type _start, @function",
    "_start:",
    "mov rdi, rsp",
    "mov rax, [rsp]",
    "lea rbx, [rsp + 8]",
    "lea rcx, [rbx + rax*8]",
    "add rcx, 8",
    "2:",
    "cmp qword ptr [rcx], 0",
    "je 3f",
    "add rcx, 8",
    "jmp 2b",
    "3:",
    "add rcx, 8",
    "xor rsi, rsi",
    "4:",
    "mov rax, [rcx]",
    "cmp rax, 0",
    "je 5f",
    "cmp rax, 7",
    "jne 6f",
    "mov rsi, [rcx + 8]",
    "6:",
    "add rcx, 16",
    "jmp 4b",
    "5:",
    "mov rax, [rsi + 32]",
    "movzx rcx, word ptr [rsi + 56]",
    "lea r8, [rsi + rax]",
    "xor r9, r9",
    "7:",
    "cmp r9, rcx",
    "jge 8f",
    "mov eax, [r8]",
    "cmp eax, 2",
    "je 9f",
    "add r8, 56",
    "inc r9",
    "jmp 7b",
    "9:",
    "mov r10, [r8 + 16]",
    "mov r11, [r8 + 40]",
    "add r10, rsi",
    "xor rax, rax",
    "xor rbx, rbx",
    "mov rcx, r10",
    "lea rdx, [r10 + r11]",
    "10:",
    "cmp rcx, rdx",
    "jge 11f",
    "mov r12, [rcx]",
    "mov r13, [rcx + 8]",
    "cmp r12, 0",
    "je 11f",
    "cmp r12, 7",
    "jne 12f",
    "lea rax, [rsi + r13]",
    "12:",
    "cmp r12, 8",
    "jne 13f",
    "mov rbx, r13",
    "13:",
    "add rcx, 16",
    "jmp 10b",
    "11:",
    "test rbx, rbx",
    "jz 8f",
    "test rax, rax",
    "jz 8f",
    "xor rcx, rcx",
    "14:",
    "cmp rcx, rbx",
    "jge 8f",
    "mov r12, [rax + rcx]",
    "mov r13, [rax + rcx + 8]",
    "mov r14, [rax + rcx + 16]",
    "and r13d, 0xffffffff",
    "cmp r13d, 8",
    "jne 15f",
    "add r12, rsi",
    "add r14, rsi",
    "mov [r12], r14",
    "15:",
    "add rcx, 24",
    "jmp 14b",
    "8:",
    ".hidden {run_main}",
    "call {run_main}",
    "ud2",
    run_main = sym run_main,
);

// aarch64 _start: self-relocate ldso, then call run_main(sp, ldso_base)
#[cfg(not(test))]
#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    ".global _start",
    ".type _start, @function",
    "_start:",
    // Save sp into x29 (frame pointer, callee-saved)
    "mov x29, sp",
    // Walk stack: argc, argv[], NULL, envp[], NULL, auxv[]
    "ldr x0, [sp]",              // argc
    "add x1, sp, #8",            // &argv[0]
    "add x2, x1, x0, lsl #3",   // skip argv[]
    "add x2, x2, #8",            // skip NULL after argv -> &envp[0]
    "2:",
    "ldr x3, [x2]",
    "cbz x3, 3f",
    "add x2, x2, #8",
    "b 2b",
    "3:",
    "add x2, x2, #8",            // &auxv[0]
    "mov x20, #0",                // ldso_base = 0
    "4:",
    "ldr x3, [x2]",              // auxv tag
    "cbz x3, 5f",                // AT_NULL -> done
    "cmp x3, #7",                // AT_BASE
    "bne 6f",
    "ldr x20, [x2, #8]",         // ldso_base
    "6:",
    "add x2, x2, #16",
    "b 4b",
    "5:",
    // x20 = ldso_base. Walk ldso's ELF phdrs to find PT_DYNAMIC.
    "ldr x0, [x20, #32]",        // e_phoff
    "ldrh w1, [x20, #56]",       // e_phnum
    "add x2, x20, x0",           // phdr table
    "mov x3, #0",                 // i
    "7:",
    "cmp x3, x1",
    "bge 8f",
    "ldr w4, [x2]",              // p_type
    "cmp w4, #2",                // PT_DYNAMIC
    "beq 9f",
    "add x2, x2, #56",           // next phdr (PHDR_SIZE=56)
    "add x3, x3, #1",
    "b 7b",
    "9:",
    // Found PT_DYNAMIC. Read DT_RELA and DT_RELASZ from dynamic section.
    "ldr x4, [x2, #16]",         // p_vaddr
    "ldr x5, [x2, #40]",         // p_memsz
    "add x4, x4, x20",           // dyn_addr = base + p_vaddr
    "add x5, x4, x5",            // dyn_end
    "mov x6, #0",                 // rela = 0
    "mov x7, #0",                 // relasz = 0
    "10:",
    "cmp x4, x5",
    "bge 11f",
    "ldr x8, [x4]",              // d_tag
    "ldr x9, [x4, #8]",          // d_val
    "cbz x8, 11f",               // DT_NULL
    "cmp x8, #7",                // DT_RELA
    "bne 12f",
    "add x6, x20, x9",           // rela = base + d_val
    "12:",
    "cmp x8, #8",                // DT_RELASZ
    "bne 13f",
    "mov x7, x9",                // relasz = d_val
    "13:",
    "add x4, x4, #16",
    "b 10b",
    "11:",
    // Apply R_AARCH64_RELATIVE (type 1027) relocations.
    "cbz x7, 8f",
    "cbz x6, 8f",
    "add x8, x6, x7",            // table_end
    "14:",
    "cmp x6, x8",
    "bge 8f",
    "ldr x9, [x6]",              // r_offset
    "ldr x10, [x6, #8]",         // r_info
    "ldr x11, [x6, #16]",        // r_addend
    "cmp w10, #1027",             // R_AARCH64_RELATIVE
    "bne 15f",
    "add x9, x9, x20",           // slot = base + r_offset
    "add x11, x11, x20",         // val = base + r_addend
    "str x11, [x9]",
    "15:",
    "add x6, x6, #24",
    "b 14b",
    "8:",
    ".hidden {run_main}",
    "mov x0, x29",               // sp
    "mov x1, x20",               // ldso_base
    "bl {run_main}",
    "brk #1",
    run_main = sym run_main,
);

// ============================================================
// Entry point
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn run_main(sp: usize, ldso_base: u64) -> ! {
    unsafe { load_and_jump(sp, ldso_base) }
}

// ============================================================
// String helpers (no_std)
// ============================================================

unsafe fn str_len(s: *const u8) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

unsafe fn sym_count_from_gnu_hash(gh: usize) -> usize {
    let nb = u32::from_le_bytes(core::ptr::read_unaligned(gh as *const [u8; 4])) as usize;
    let symoffset = u32::from_le_bytes(core::ptr::read_unaligned((gh + 4) as *const [u8; 4])) as usize;
    let bloom_size = u32::from_le_bytes(core::ptr::read_unaligned((gh + 8) as *const [u8; 4])) as usize;
    let buckets = gh + 16 + bloom_size * 8;
    let chain = buckets + nb * 4;
    let mut max_idx = 0usize;
    let mut has_any = false;
    for i in 0..nb {
        let symidx = u32::from_le_bytes(core::ptr::read_unaligned((buckets + i * 4) as *const [u8; 4])) as usize;
        if symidx == 0 || symidx < symoffset {
            continue;
        }
        let mut idx = symidx;
        loop {
            let cidx = idx - symoffset;
            if cidx > max_idx {
                max_idx = cidx;
            }
            has_any = true;
            let entry = u32::from_le_bytes(core::ptr::read_unaligned((chain + cidx * 4) as *const [u8; 4]));
            if entry & 1 != 0 {
                break;
            }
            idx += 1;
        }
    }
    if has_any {
        symoffset + max_idx + 1
    } else {
        symoffset
    }
}

unsafe fn sym_count_from_hash(h: usize) -> usize {
    let nchain = u32::from_le_bytes(core::ptr::read_unaligned((h + 4) as *const [u8; 4])) as usize;
    nchain
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> usize {
    str_len(s as *const u8)
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32 {
    let a = a as *const u8;
    let b = b as *const u8;
    let mut i = 0;
    while i < n {
        let va = *a.add(i);
        let vb = *b.add(i);
        if va != vb {
            return va as i32 - vb as i32;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn bcmp(a: *const c_void, b: *const c_void, n: usize) -> i32 {
    memcmp(a, b, n)
}

/// Compare null-terminated `a` (with known length) against null-terminated `b`.
unsafe fn str_eq(a: *const u8, a_len: usize, b: *const u8) -> bool {
    let mut i = 0;
    while i < a_len {
        if *a.add(i) != *b.add(i) {
            return false;
        }
        i += 1;
    }
    *b.add(a_len) == 0
}

/// Walk kernel-stack envp for a var starting with `prefix` (e.g. b"LD_LIBRARY_PATH=").
/// Returns pointer to the value part (after the '=') or None.
unsafe fn find_env(sp: usize, prefix: &[u8]) -> Option<*const u8> {
    let argc = *(sp as *const u64) as usize;
    // skip: argc + argv[0..argc] + NULL
    let mut p = sp + 8 + (argc + 1) * 8;
    loop {
        let env_ptr = *(p as *const u64) as *const u8;
        if env_ptr.is_null() {
            break;
        }
        let mut matches = true;
        let mut i = 0;
        while i < prefix.len() {
            if *env_ptr.add(i) != prefix[i] {
                matches = false;
                break;
            }
            i += 1;
        }
        if matches {
            return Some(env_ptr.add(prefix.len()));
        }
        p += 8;
    }
    None
}

// ============================================================
// Syscall wrappers (raw, no_std)
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
    pub const SYS_OPENAT: i64 = 257;
    pub const SYS_CLOSE: i64 = 3;
    pub const SYS_LSEEK: i64 = 8;
    pub const SYS_MMAP: i64 = 9;
    pub const SYS_MUNMAP: i64 = 11;
    pub const SYS_READLINKAT: i64 = 267;
    pub const SYS_ARCH_PRCTL: i64 = 158;
    pub const SYS_EXIT: i64 = 60;
}
#[cfg(target_arch = "aarch64")]
mod sysnr {
    pub const SYS_READ: i64 = 63;
    pub const SYS_WRITE: i64 = 64;
    pub const SYS_OPENAT: i64 = 56;
    pub const SYS_CLOSE: i64 = 57;
    pub const SYS_LSEEK: i64 = 62;
    pub const SYS_MMAP: i64 = 222;
    pub const SYS_MUNMAP: i64 = 215;
    pub const SYS_READLINKAT: i64 = 78;
    pub const SYS_EXIT: i64 = 93;
}
pub use sysnr::*;

const AT_FDCWD: i64 = -100;

// ============================================================
// Syscall wrappers (raw, no_std)
// ============================================================

fn sys_open(path: *const u8) -> i64 {
    unsafe { <Arch as Syscalls>::syscall3(SYS_OPENAT, AT_FDCWD, path as i64, 0) }
}

fn sys_readlink(path: *const u8, buf: *mut u8, bufsz: usize) -> i64 {
    unsafe { <Arch as Syscalls>::syscall4(SYS_READLINKAT, AT_FDCWD, path as i64, buf as i64, bufsz as i64) }
}

fn sys_read(fd: i64, buf: *mut u8, count: usize) -> i64 {
    unsafe { <Arch as Syscalls>::syscall3(SYS_READ, fd, buf as i64, count as i64) }
}

fn sys_write(fd: i64, buf: *const u8, count: usize) -> i64 {
    unsafe { <Arch as Syscalls>::syscall3(SYS_WRITE, fd, buf as i64, count as i64) }
}

fn sys_close(fd: i64) {
    unsafe { <Arch as Syscalls>::syscall1(SYS_CLOSE, fd); }
}

fn sys_mmap(
    addr: *mut u8,
    length: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: i64,
) -> *mut u8 {
    let result = unsafe { <Arch as Syscalls>::syscall6(SYS_MMAP, addr as i64, length as i64, prot as i64, flags as i64, fd as i64, offset) };
    if result < 0 && result > -4096 {
        return MAP_FAILED as *mut u8;
    }
    result as *mut u8
}

fn sys_exit(code: i32) -> ! {
    unsafe { <Arch as Syscalls>::syscall_noreturn1(SYS_EXIT, code as i64) }
}

fn sys_lseek(fd: i64, offset: i64) -> i64 {
    unsafe { <Arch as Syscalls>::syscall3(SYS_LSEEK, fd, offset, 0) }
}

#[cfg(target_arch = "x86_64")]
fn sys_arch_prctl(code: i64, addr: u64) -> i64 {
    unsafe { <Arch as Syscalls>::syscall2(SYS_ARCH_PRCTL, code, addr as i64) }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn read_tp() -> usize {
    let tp: usize;
    core::arch::asm!("mov {}, fs:[0]", out(reg) tp);
    tp
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn read_tp() -> usize {
    let tp: usize;
    core::arch::asm!("mrs {}, tpidr_el0", out(reg) tp);
    tp
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn write_tp(addr: usize) {
    sys_arch_prctl(0x1002, addr as u64);
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn write_tp(addr: usize) {
    core::arch::asm!("msr tpidr_el0, {}", in(reg) addr);
}

unsafe fn write_stderr(msg: &[u8]) {
    let _ = sys_write(2, msg.as_ptr(), msg.len());
}

unsafe fn write_hex_stderr(v: usize) {
    let mut buf = [0u8; 18];
    buf[0] = b'0';
    buf[1] = b'x';
    for i in 0..16 {
        let nibble = ((v >> (60 - i * 4)) & 0xf) as u8;
        buf[2 + i] = if nibble < 10 { b'0' + nibble } else { b'a' + nibble - 10 };
    }
    write_stderr(&buf);
}

unsafe fn die(code: i32, label: &[u8], detail: usize) -> ! {
    write_stderr(b"[ldso fatal ");
    write_stderr(label);
    write_stderr(b" ");
    write_hex_stderr(detail);
    write_stderr(b"]\n");
    sys_exit(code)
}

// ============================================================
// ELF helpers
// ============================================================

fn prot_from_flags(flags: u32) -> i32 {
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

// ============================================================
// Library search
// ============================================================

unsafe fn try_open(
    path_buf: &mut [u8; 512],
    dir: *const u8,
    dir_len: usize,
    lib_name: *const u8,
    lib_name_len: usize,
) -> i64 {
    if dir_len + 1 + lib_name_len >= 512 {
        return -1;
    }
    let mut pos = 0;
    let mut i = 0;
    while i < dir_len {
        path_buf[pos] = *dir.add(i);
        pos += 1;
        i += 1;
    }
    path_buf[pos] = b'/';
    pos += 1;
    let mut i = 0;
    while i < lib_name_len {
        path_buf[pos] = *lib_name.add(i);
        pos += 1;
        i += 1;
    }
    path_buf[pos] = 0;
    sys_open(path_buf.as_ptr())
}

unsafe fn try_open_expanded(
    path_buf: &mut [u8; 512],
    dir: *const u8,
    dir_len: usize,
    lib_name: *const u8,
    lib_name_len: usize,
) -> i64 {
    if dir_len >= 7 {
        let origin = b"$ORIGIN";
        let mut matches = true;
        let mut i = 0;
        while i < 7 {
            if *dir.add(i) != origin[i] {
                matches = false;
                break;
            }
            i += 1;
        }
        if matches {
            let rem_len = dir_len - 7;
            if ORIGIN_LEN + rem_len + 1 + lib_name_len >= 512 {
                return -1;
            }
            let mut pos = 0;
            let mut i = 0;
            while i < ORIGIN_LEN {
                path_buf[pos] = ORIGIN_DIR[i];
                pos += 1;
                i += 1;
            }
            let mut i = 0;
            while i < rem_len {
                path_buf[pos] = *dir.add(7 + i);
                pos += 1;
                i += 1;
            }
            path_buf[pos] = b'/';
            pos += 1;
            let mut i = 0;
            while i < lib_name_len {
                path_buf[pos] = *lib_name.add(i);
                pos += 1;
                i += 1;
            }
            path_buf[pos] = 0;
            return sys_open(path_buf.as_ptr());
        }
    }
    try_open(path_buf, dir, dir_len, lib_name, lib_name_len)
}

unsafe fn find_library_fd(
    lib_name: *const u8,
    lib_name_len: usize,
    ld_path: Option<*const u8>,
) -> i64 {
    let mut path_buf = [0u8; 512];

    if lib_name_len > 0 {
        let fd = sys_open(lib_name);
        if fd >= 0 {
            return fd;
        }
    }

    if let Some(ldp) = ld_path {
        let ldp_len = str_len(ldp);
        let mut start = 0usize;
        while start < ldp_len {
            let mut end = start;
            while end < ldp_len && *ldp.add(end) != b':' {
                end += 1;
            }
            if end > start {
                let fd = try_open(&mut path_buf, ldp.add(start), end - start, lib_name, lib_name_len);
                if fd >= 0 {
                    return fd;
                }
            }
            if end >= ldp_len {
                break;
            }
            start = end + 1;
        }
    }

    if RUNPATH_LEN > 0 {
        let rp = RUNPATH;
        let rp_len = RUNPATH_LEN;
        let mut start = 0usize;
        while start < rp_len {
            let mut end = start;
            while end < rp_len && *rp.add(end) != b':' {
                end += 1;
            }
            if end > start {
                let fd = try_open_expanded(&mut path_buf, rp.add(start), end - start, lib_name, lib_name_len);
                if fd >= 0 {
                    return fd;
                }
            }
            if end >= rp_len {
                break;
            }
            start = end + 1;
        }
    }

    let defaults: &[(&[u8], usize)] = &[
        (b"/lib", 4),
        (b"/usr/lib", 8),
        (b"/usr/local/lib", 14),
    ];
    for &(dir_bytes, dir_len) in defaults {
        let fd = try_open(&mut path_buf, dir_bytes.as_ptr(), dir_len, lib_name, lib_name_len);
        if fd >= 0 {
            return fd;
        }
    }

    -1
}

// ============================================================
// DSO loading
// ============================================================

/// Load a shared object from an already-open fd at the given base address.
/// Registers it in the LOADED array. Returns true on success.
fn sys_munmap(addr: *mut u8, length: usize) -> i64 {
    unsafe { <Arch as Syscalls>::syscall2(SYS_MUNMAP, addr as i64, length as i64) }
}

unsafe fn load_dso_from_fd(fd: i64, desired_base: u64) -> Option<u64> {
    let mut buf = [0u8; 4096];
    let n = sys_read(fd, buf.as_mut_ptr(), buf.len());
    if n < 64 {
        return None;
    }
    if buf[0] != 0x7f || buf[1] != b'E' {
        return None;
    }

    let e_phoff = u64::from_le_bytes(buf[32..40].try_into().unwrap());
    let e_phnum = u16::from_le_bytes(buf[56..58].try_into().unwrap()) as usize;
    let phdr_end = e_phoff as usize + e_phnum * PHDR_SIZE;
    if phdr_end > n as usize {
        return None;
    }

    #[derive(Copy, Clone)]
    struct LoadSeg {
        p_offset: u64,
        p_vaddr: u64,
        p_filesz: u64,
        p_memsz: u64,
        p_flags: u32,
    }

    let mut segs: [LoadSeg; 8] = [LoadSeg { p_offset: 0, p_vaddr: 0, p_filesz: 0, p_memsz: 0, p_flags: 0 }; 8];
    let mut seg_count: usize = 0;
    let mut min_vaddr = u64::MAX;
    let mut max_vaddr_end = 0u64;

    let mut tls_vaddr: u64 = 0;
    let mut tls_filesz: u64 = 0;
    let mut tls_memsz: u64 = 0;
    let mut tls_align: u64 = 0;

    for i in 0..e_phnum {
        let ph = buf.as_ptr().add(e_phoff as usize + i * PHDR_SIZE);
        let p_type = u32::from_le_bytes(core::ptr::read_unaligned(ph as *const [u8; 4]));
        if p_type == PT_TLS {
            tls_vaddr = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_VADDR) as *const [u8; 8]));
            tls_filesz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_FILESZ) as *const [u8; 8]));
            tls_memsz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_MEMSZ) as *const [u8; 8]));
            tls_align = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_ALIGN) as *const [u8; 8]));
            continue;
        }
        if p_type != PT_LOAD {
            continue;
        }
        if seg_count >= segs.len() {
            return None;
        }
        let p_flags = u32::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_FLAGS) as *const [u8; 4]));
        let p_offset = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_OFFSET) as *const [u8; 8]));
        let p_vaddr = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_VADDR) as *const [u8; 8]));
        let p_filesz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_FILESZ) as *const [u8; 8]));
        let p_memsz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_MEMSZ) as *const [u8; 8]));
        segs[seg_count] = LoadSeg { p_offset, p_vaddr, p_filesz, p_memsz, p_flags };
        seg_count += 1;
        if p_vaddr < min_vaddr { min_vaddr = p_vaddr; }
        let end = p_vaddr + p_memsz;
        if end > max_vaddr_end { max_vaddr_end = end; }
    }

    if seg_count == 0 || min_vaddr == u64::MAX {
        return None;
    }

    const PAGE: u64 = 4096;
    let total_size = ((max_vaddr_end + PAGE - 1) & !(PAGE - 1)) as usize;

    // ponytail: ASLR may place our own stack or the executable at the
    // desired base. Probe for a free span before committing with MAP_FIXED.
    let mut base = desired_base;
    let actual_base = loop {
        let probe = sys_mmap(
            base as *mut u8,
            total_size,
            PROT_NONE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        );
        if probe as usize == base as usize {
            sys_munmap(probe, total_size);
            break base;
        }
        if probe as usize != MAP_FAILED {
            sys_munmap(probe, total_size);
        }
        base += DSO_BASE_STRIDE;
        if base > desired_base + DSO_BASE_STRIDE * 16 {
            return None;
        }
    };

    let tls_image = (actual_base + tls_vaddr) as *const u8;

    for i in 0..seg_count {
        let seg = segs[i];
        let adj = seg.p_vaddr & (PAGE - 1);
        let map_addr = actual_base + seg.p_vaddr - adj;
        let map_off = seg.p_offset - adj;
        let map_len = ((seg.p_memsz + adj + PAGE - 1) & !(PAGE - 1)) as usize;
        let prot = prot_from_flags(seg.p_flags);

        // Map the whole segment anonymously first so the tail (bss) is backed
        // by zeroed anonymous pages, then overlay the file-backed portion.
        let ptr = sys_mmap(
            map_addr as *mut u8,
            map_len,
            prot,
            MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS,
            -1,
            0,
        );
        if ptr as usize == MAP_FAILED {
            return None;
        }

        let file_map_len = ((seg.p_filesz + adj + PAGE - 1) & !(PAGE - 1)) as usize;
        if file_map_len > 0 {
            let fptr = sys_mmap(
                map_addr as *mut u8,
                file_map_len,
                prot,
                MAP_PRIVATE | MAP_FIXED,
                fd as i32,
                map_off as i64,
            );
            if fptr as usize == MAP_FAILED {
                return None;
            }
        }

        if seg.p_memsz > seg.p_filesz {
            let bss_start = (actual_base + seg.p_vaddr + seg.p_filesz) as *mut u8;
            let bss_len = (seg.p_memsz - seg.p_filesz) as usize;
            core::ptr::write_bytes(bss_start, 0, bss_len);
        }
    }

    // Find PT_DYNAMIC
    let mut dyn_vaddr: u64 = 0;
    let mut dyn_memsz: u64 = 0;
    for i in 0..e_phnum {
        let ph = buf.as_ptr().add(e_phoff as usize + i * PHDR_SIZE);
        let p_type = u32::from_le_bytes(core::ptr::read_unaligned(ph as *const [u8; 4]));
        if p_type == PT_DYNAMIC {
            dyn_vaddr = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_VADDR) as *const [u8; 8]));
            dyn_memsz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_MEMSZ) as *const [u8; 8]));
            break;
        }
    }
    if dyn_vaddr == 0 {
        return None;
    }

    let dyn_addr = (actual_base + dyn_vaddr) as usize;
    let dyn_end = dyn_addr + dyn_memsz as usize;

    // Parse DT_SYMTAB, DT_STRTAB, DT_STRSZ
    let mut dt_symtab: u64 = 0;
    let mut dt_strtab: u64 = 0;
    let mut dt_strsz: u64 = 0;
    let mut dt_init: u64 = 0;
    let mut dt_init_array: u64 = 0;
    let mut dt_init_array_sz: u64 = 0;
    let mut dt_init_present = false;
    let mut dt_init_array_present = false;
    let mut dt_gnu_hash: u64 = 0;
    let mut dt_hash: u64 = 0;
    let mut dp = dyn_addr;
    while dp + 16 <= dyn_end {
        let d_tag = u64::from_le_bytes(core::ptr::read_unaligned(dp as *const [u8; 8]));
        let d_val = u64::from_le_bytes(core::ptr::read_unaligned((dp + 8) as *const [u8; 8]));
        if d_tag == DT_NULL {
            break;
        }
        match d_tag {
            DT_SYMTAB => dt_symtab = d_val,
            DT_STRTAB => dt_strtab = d_val,
            DT_STRSZ => dt_strsz = d_val,
            DT_GNU_HASH => dt_gnu_hash = d_val,
            DT_HASH => dt_hash = d_val,
            DT_INIT => { dt_init = d_val; dt_init_present = true; }
            DT_INIT_ARRAY => { dt_init_array = d_val; dt_init_array_present = true; }
            DT_INIT_ARRAYSZ => dt_init_array_sz = d_val,
            _ => {}
        }
        dp += 16;
    }

    let symtab_ptr = (actual_base + dt_symtab) as *const u8;
    let strtab_ptr = (actual_base + dt_strtab) as *const u8;
    let strsz = dt_strsz as usize;

    let mut sym_count: usize = 0;
    if dt_gnu_hash != 0 {
        sym_count = sym_count_from_gnu_hash((actual_base + dt_gnu_hash) as usize);
    } else if dt_hash != 0 {
        sym_count = sym_count_from_hash((actual_base + dt_hash) as usize);
    } else if dt_strtab > dt_symtab && dt_strtab - dt_symtab >= SYMTAB_ENT_SIZE as u64 {
        sym_count = ((dt_strtab - dt_symtab) / SYMTAB_ENT_SIZE as u64) as usize;
    }

    if LOADED_COUNT < MAX_LOADED {
        LOADED[LOADED_COUNT] = LoadedObject {
            base: actual_base,
            symtab: symtab_ptr,
            sym_count,
            strtab: strtab_ptr,
            strsz,
            dyn_addr,
            dyn_memsz: dyn_memsz as usize,
            tls_image,
            tls_filesz,
            tls_memsz,
            tls_align,
            init: actual_base + dt_init,
            init_array: actual_base + dt_init_array,
            init_array_sz: dt_init_array_sz,
            init_present: dt_init_present,
            init_array_present: dt_init_array_present,
            global: false,
            name: [0; 256],
        };
        LOADED_COUNT += 1;
    }

    Some(actual_base)
}

// ============================================================
// Symbol resolution
// ============================================================

/// Look up symbol name from object's own symtab, then search all loaded objects.
unsafe fn resolve_symbol_from_index(obj_idx: usize, sym_idx: usize) -> u64 {
    let obj = &LOADED[obj_idx];
    if sym_idx == 0 || obj.symtab.is_null() || obj.strtab.is_null() {
        return 0;
    }
    let sym_entry = obj.symtab.add(sym_idx * SYMTAB_ENT_SIZE);
    let st_name = u32::from_le_bytes(core::ptr::read_unaligned(sym_entry as *const [u8; 4])) as usize;
    if st_name >= obj.strsz {
        return 0;
    }
    let name = obj.strtab.add(st_name);
    resolve_symbol(name)
}

/// Search all loaded objects for a symbol with the given null-terminated name.
/// Returns the resolved address (base + st_value) or 0 if not found.
unsafe fn resolve_symbol(name: *const u8) -> u64 {
    resolve_symbol_with_size(name, usize::MAX).0
}

/// Same as resolve_symbol but also returns the defining symbol's st_size.
/// `exclude` is an object index to skip (use usize::MAX to skip nothing).
unsafe fn resolve_symbol_with_size(name: *const u8, exclude: usize) -> (u64, usize) {
    let name_len = str_len(name);
    if name_len == 0 {
        return (0, 0);
    }
    if str_eq(name, name_len, b"__tls_get_addr\0".as_ptr()) {
        return ((__tls_get_addr as *const () as usize) as u64, 0);
    }
    if str_eq(name, name_len, b"__rc_create_thread_tls\0".as_ptr()) {
        return ((__rc_create_thread_tls as *const () as usize) as u64, 0);
    }
    if str_eq(name, name_len, b"__rc_tls_block_size\0".as_ptr()) {
        return ((__rc_tls_block_size as *const () as usize) as u64, 0);
    }

    for i in 0..LOADED_COUNT {
        if i == exclude {
            continue;
        }
        let obj = &LOADED[i];
        if obj.symtab.is_null() || obj.strtab.is_null() {
            continue;
        }
        for j in 0..obj.sym_count {
            let sym_entry = obj.symtab.add(j * SYMTAB_ENT_SIZE);
            let st_name_off =
                u32::from_le_bytes(core::ptr::read_unaligned(sym_entry as *const [u8; 4])) as usize;
            let st_info = *sym_entry.add(4);
            if st_info >> 4 == 0 {
                continue;
            }
            let st_value = u64::from_le_bytes(core::ptr::read_unaligned(
                sym_entry.add(8) as *const [u8; 8],
            ));
            if st_value == 0 {
                continue;
            }
            if st_name_off >= obj.strsz {
                continue;
            }
            let sym_name = obj.strtab.add(st_name_off);
            if str_eq(name, name_len, sym_name) {
                let st_size = u64::from_le_bytes(core::ptr::read_unaligned(
                    sym_entry.add(16) as *const [u8; 8],
                ));
                return (obj.base + st_value, st_size as usize);
            }
        }
    }
    (0, 0)
}

unsafe fn resolve_copy_source(obj_idx: usize, sym_idx: usize) -> (u64, usize) {
    let obj = &LOADED[obj_idx];
    if sym_idx == 0 || obj.symtab.is_null() || obj.strtab.is_null() {
        return (0, 0);
    }
    if sym_idx * SYMTAB_ENT_SIZE >= obj.sym_count * SYMTAB_ENT_SIZE {
        return (0, 0);
    }
    let sym_entry = obj.symtab.add(sym_idx * SYMTAB_ENT_SIZE);
    let st_name =
        u32::from_le_bytes(core::ptr::read_unaligned(sym_entry as *const [u8; 4])) as usize;
    if st_name >= obj.strsz {
        return (0, 0);
    }
    let name = obj.strtab.add(st_name);
    resolve_symbol_with_size(name, obj_idx)
}

unsafe fn resolve_symbol_module(obj_idx: usize, sym_idx: usize) -> usize {
    let obj = &LOADED[obj_idx];
    if sym_idx == 0 || obj.symtab.is_null() || obj.strtab.is_null() {
        return obj_idx;
    }
    if sym_idx * SYMTAB_ENT_SIZE >= obj.sym_count * SYMTAB_ENT_SIZE {
        return obj_idx;
    }
    let sym_entry = obj.symtab.add(sym_idx * SYMTAB_ENT_SIZE);
    let st_name =
        u32::from_le_bytes(core::ptr::read_unaligned(sym_entry as *const [u8; 4])) as usize;
    if st_name == 0 || st_name >= obj.strsz {
        return obj_idx;
    }
    let st_info = *sym_entry.add(4);
    if (st_info & 0xf) == 6 {
        return obj_idx;
    }
    let name = obj.strtab.add(st_name);
    let name_len = str_len(name);
    for i in 0..LOADED_COUNT {
        let o = &LOADED[i];
        if o.symtab.is_null() || o.strtab.is_null() {
            continue;
        }
        for j in 0..o.sym_count {
            let se = o.symtab.add(j * SYMTAB_ENT_SIZE);
            let s_name =
                u32::from_le_bytes(core::ptr::read_unaligned(se as *const [u8; 4])) as usize;
            let s_value = u64::from_le_bytes(core::ptr::read_unaligned(
                se.add(8) as *const [u8; 8],
            ));
            if s_value == 0 {
                continue;
            }
            if s_name >= o.strsz {
                continue;
            }
            let sym_name = o.strtab.add(s_name);
            if str_eq(name, name_len, sym_name) {
                return i;
            }
        }
    }
    obj_idx
}

unsafe fn tls_sym_offset(obj_idx: usize, sym_idx: usize) -> u64 {
    let obj = &LOADED[obj_idx];
    if sym_idx == 0 || obj.symtab.is_null() {
        return 0;
    }
    if sym_idx * SYMTAB_ENT_SIZE >= obj.sym_count * SYMTAB_ENT_SIZE {
        return 0;
    }
    let sym_entry = obj.symtab.add(sym_idx * SYMTAB_ENT_SIZE);
    u64::from_le_bytes(core::ptr::read_unaligned(sym_entry.add(8) as *const [u8; 8]))
}

unsafe fn tls_tprel_offset(obj_idx: usize, sym_idx: usize, addend: i64) -> i64 {
    let module = if sym_idx == 0 {
        obj_idx
    } else {
        resolve_symbol_module(obj_idx, sym_idx)
    };
    let off_in_mod = tls_sym_offset(obj_idx, sym_idx) as i64 + addend;
    (TLS_LAYOUT_OFFSET[module] as i64) + off_in_mod - (tls_var_area_offset_from_tp() as i64)
}

unsafe fn tls_var_area_offset_from_block() -> usize {
    #[cfg(target_arch = "x86_64")]
    { 0 }
    #[cfg(target_arch = "aarch64")]
    { TCB_SIZE }
}

unsafe fn tls_tcb_offset_from_block() -> usize {
    #[cfg(target_arch = "x86_64")]
    { TLS_TOTAL_SIZE }
    #[cfg(target_arch = "aarch64")]
    { 0 }
}

unsafe fn tls_tp_offset_from_block() -> usize {
    #[cfg(target_arch = "x86_64")]
    { TLS_TOTAL_SIZE }
    #[cfg(target_arch = "aarch64")]
    { TCB_SIZE }
}

unsafe fn tls_var_area_offset_from_tp() -> usize {
    #[cfg(target_arch = "x86_64")]
    { TLS_TOTAL_SIZE }
    #[cfg(target_arch = "aarch64")]
    { 0 }
}

unsafe fn tls_tcb_offset_from_tp() -> isize {
    #[cfg(target_arch = "x86_64")]
    { 0 }
    #[cfg(target_arch = "aarch64")]
    { -(TCB_SIZE as isize) }
}

// ============================================================
// Relocation processing
// ============================================================

/// Process all relocations for every loaded object.
unsafe fn process_all_relocations() {
    // First pass: non-COPY relocations so source symbols have final values.
    for i in 0..LOADED_COUNT {
        let (base, rela_off, rela_sz, jmprel_off, jmprel_sz) = relocation_info(i);
        apply_rela_table(i, base, rela_off, rela_sz, false);
        apply_rela_table(i, base, jmprel_off, jmprel_sz, false);
    }
    // Second pass: COPY relocations copy initialized data into the executable.
    for i in 0..LOADED_COUNT {
        let (base, rela_off, rela_sz, _, _) = relocation_info(i);
        apply_rela_table(i, base, rela_off, rela_sz, true);
    }
}

unsafe fn relocation_info(i: usize) -> (u64, u64, u64, u64, u64) {
    let obj = &LOADED[i];
    let base = obj.base;
    let dp = obj.dyn_addr;
    let dyn_end = dp + obj.dyn_memsz;

    let mut rela_off: u64 = 0;
    let mut rela_sz: u64 = 0;
    let mut jmprel_off: u64 = 0;
    let mut jmprel_sz: u64 = 0;

    let mut pos = dp;
    while pos + 16 <= dyn_end {
        let d_tag = u64::from_le_bytes(core::ptr::read_unaligned(pos as *const [u8; 8]));
        let d_val = u64::from_le_bytes(core::ptr::read_unaligned((pos + 8) as *const [u8; 8]));
        if d_tag == DT_NULL {
            break;
        }
        match d_tag {
            DT_RELA => rela_off = d_val,
            DT_RELASZ => rela_sz = d_val,
            DT_JMPREL => jmprel_off = d_val,
            DT_PLTRELSZ => jmprel_sz = d_val,
            _ => {}
        }
        pos += 16;
    }
    (base, rela_off, rela_sz, jmprel_off, jmprel_sz)
}

/// Apply entries from one relocation table.
unsafe fn apply_rela_table(
    obj_idx: usize,
    base: u64,
    table_off: u64,
    table_sz: u64,
    copy_only: bool,
) {
    if table_sz == 0 {
        return;
    }
    let table = (base + table_off) as *const u8;
    let count = table_sz as usize / 24;

    for i in 0..count {
        let entry = table.add(i * 24);
        let r_offset = u64::from_le_bytes(core::ptr::read_unaligned(entry as *const [u8; 8]));
        let r_info = u64::from_le_bytes(core::ptr::read_unaligned(entry.add(8) as *const [u8; 8]));
        let r_addend =
            i64::from_le_bytes(core::ptr::read_unaligned(entry.add(16) as *const [u8; 8]));

        let r_type = r_info & 0xffffffff;
        let r_sym_idx = (r_info >> 32) as usize;
        let slot = (base + r_offset) as *mut u64;

        if r_type == R_X86_64_COPY {
            if !copy_only {
                continue;
            }
            let (src, sym_size) = resolve_copy_source(obj_idx, r_sym_idx);
            if src != 0 && sym_size != 0 {
                let dst = (base + r_offset) as *mut u8;
                core::ptr::copy_nonoverlapping(src as *const u8, dst, sym_size);
            }
            continue;
        }
        if copy_only {
            continue;
        }

        match r_type {
            R_X86_64_RELATIVE | R_AARCH64_RELATIVE => {
                *slot = (base as i64 + r_addend) as u64;
            }
            R_X86_64_64 | R_AARCH64_ABS64 => {
                let sym_value = resolve_symbol_from_index(obj_idx, r_sym_idx);
                *slot = (sym_value as i64 + r_addend) as u64;
            }
            R_X86_64_GLOB_DAT | R_X86_64_JUMP_SLOT
            | R_AARCH64_GLOB_DAT | R_AARCH64_JUMP_SLOT => {
                let sym_value = resolve_symbol_from_index(obj_idx, r_sym_idx);
                *slot = sym_value;
            }
            R_X86_64_DTPMOD64 | R_AARCH64_TLS_DTPMOD64 => {
                let module = if r_sym_idx == 0 {
                    obj_idx
                } else {
                    resolve_symbol_module(obj_idx, r_sym_idx)
                };
                *slot = module as u64;
            }
            R_X86_64_DTPOFF64 | R_AARCH64_TLS_DTPREL64 => {
                let off = (tls_sym_offset(obj_idx, r_sym_idx) as i64 + r_addend) as u64;
                *slot = off;
            }
            R_X86_64_TPOFF64 | R_AARCH64_TLS_TPREL64 => {
                let fs_off = tls_tprel_offset(obj_idx, r_sym_idx, r_addend);
                *slot = fs_off as u64;
            }
            R_AARCH64_TLSLE_ADD_TPREL_HI12 => {
                let fs_off = tls_tprel_offset(obj_idx, r_sym_idx, r_addend);
                let insn = core::ptr::read_unaligned(slot as *const u32);
                let imm = ((fs_off >> 12) & 0xFFF) as u32;
                let new_insn = (insn & !(0xFFFu32 << 10)) | (imm << 10);
                core::ptr::write_unaligned(slot as *mut u32, new_insn);
            }
            R_AARCH64_TLSLE_ADD_TPREL_LO12 | R_AARCH64_TLSLE_ADD_TPREL_LO12_NC => {
                let fs_off = tls_tprel_offset(obj_idx, r_sym_idx, r_addend);
                let insn = core::ptr::read_unaligned(slot as *const u32);
                let imm = (fs_off & 0xFFF) as u32;
                let new_insn = (insn & !(0xFFFu32 << 10)) | (imm << 10);
                core::ptr::write_unaligned(slot as *mut u32, new_insn);
            }
            _ => {}
        }
    }
}

unsafe fn run_constructors() {
    for i in 0..LOADED_COUNT {
        run_constructors_for(i);
    }
}

unsafe fn run_constructors_for(idx: usize) {
    let obj = &LOADED[idx];
    if obj.init_present && obj.init != 0 {
        let f: extern "C" fn() = core::mem::transmute(obj.init);
        f();
    }
    if obj.init_array_present && obj.init_array != 0 && obj.init_array_sz >= 8 {
        let count = (obj.init_array_sz / 8) as usize;
        for j in 0..count {
            let entry = (obj.init_array as *const u8).add(j * 8);
            let fp = u64::from_le_bytes(core::ptr::read_unaligned(entry as *const [u8; 8]));
            if fp != 0 {
                let f: extern "C" fn() = core::mem::transmute(fp);
                f();
            }
        }
    }
}

unsafe fn tls_lock() {
    while TLS_LOCK.swap(true, Ordering::Acquire) {}
}

unsafe fn tls_unlock() {
    TLS_LOCK.store(false, Ordering::Release);
}

unsafe fn expand_thread_tls(old_total: usize, old_module_count: usize) {
    let total = TLS_TOTAL_SIZE + TCB_SIZE;
    let block = sys_mmap(
        core::ptr::null_mut(),
        total,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );
    if block as usize == MAP_FAILED {
        return;
    }
    let old_fs = read_tp();
    let old_var_base = (old_fs as isize - tls_var_area_offset_from_tp() as isize) as *mut u8;
    if old_total > 0 {
        core::ptr::copy_nonoverlapping(old_var_base, block.add(tls_var_area_offset_from_block()), old_total);
    }
    for i in old_module_count..TLS_MODULE_COUNT {
        if TLS_MEMSZ[i] == 0 {
            continue;
        }
        let dst = block.add(tls_var_area_offset_from_block()).add(TLS_LAYOUT_OFFSET[i]);
        let src = TLS_IMAGE[i];
        let filesz = TLS_FILESZ[i] as usize;
        let memsz = TLS_MEMSZ[i] as usize;
        if filesz > 0 {
            core::ptr::copy_nonoverlapping(src, dst, filesz);
        }
        if memsz > filesz {
            core::ptr::write_bytes(dst.add(filesz), 0, memsz - filesz);
        }
    }
    let tcb = block.add(tls_tcb_offset_from_block());
    core::ptr::write_unaligned(tcb as *mut u64, tcb as u64);
    core::ptr::write_unaligned((tcb as *mut u64).add(1), TLS_GENERATION);
    write_tp(block.add(tls_tp_offset_from_block()) as usize);
}

unsafe fn update_tls_for_new_module(idx: usize) {
    let obj = &LOADED[idx];
    if obj.tls_memsz == 0 {
        return;
    }
    tls_lock();
    let old_total = TLS_TOTAL_SIZE;
    let old_module_count = TLS_MODULE_COUNT;
    let align = if obj.tls_align > 0 { obj.tls_align as usize } else { 1 };

    let mut highest_end: usize = 0;
    for i in 0..old_module_count {
        let block_size = ((TLS_MEMSZ[i] as usize + align - 1) / align) * align;
        let end = TLS_LAYOUT_OFFSET[i] + block_size;
        if end > highest_end {
            highest_end = end;
        }
    }

    let new_offset = (highest_end + align - 1) & !(align - 1);
    TLS_LAYOUT_OFFSET[idx] = new_offset;
    TLS_FILESZ[idx] = obj.tls_filesz;
    TLS_MEMSZ[idx] = obj.tls_memsz;
    TLS_IMAGE[idx] = obj.tls_image;
    TLS_MODULE_COUNT = LOADED_COUNT;
    TLS_GENERATION = TLS_GENERATION.wrapping_add(1);
    if TLS_GENERATION == 0 {
        TLS_GENERATION = 1;
    }

    expand_thread_tls(old_total, old_module_count);

    TLS_OLD_TOTAL = old_total;
    TLS_OLD_MODULE_COUNT = old_module_count;
    TLS_OLD_MODULE_COUNT = old_module_count;
    tls_unlock();
}

unsafe fn lookup_symbol_in_object(obj_idx: usize, name: *const u8, name_len: usize) -> u64 {
    let obj = &LOADED[obj_idx];
    if obj.symtab.is_null() || obj.strtab.is_null() {
        return 0;
    }
    for j in 0..obj.sym_count {
        let sym_entry = obj.symtab.add(j * SYMTAB_ENT_SIZE);
        let st_name_off = u32::from_le_bytes(core::ptr::read_unaligned(sym_entry as *const [u8; 4])) as usize;
        let st_value = u64::from_le_bytes(core::ptr::read_unaligned(sym_entry.add(8) as *const [u8; 8]));
        if st_value == 0 {
            continue;
        }
        if st_name_off >= obj.strsz {
            continue;
        }
        let sym_name = obj.strtab.add(st_name_off);
        if str_eq(name, name_len, sym_name) {
            return obj.base + st_value;
        }
    }
    0
}

static mut DLERROR_BUF: [u8; 128] = [0; 128];
static mut DLERROR_SET: bool = false;
const DLERROR_BUF_SIZE: usize = 128;

unsafe fn set_dlerror(msg: &[u8]) {
    let len = if msg.len() >= DLERROR_BUF_SIZE {
        DLERROR_BUF_SIZE - 1
    } else {
        msg.len()
    };
    core::ptr::copy_nonoverlapping(
        msg.as_ptr(),
        core::ptr::addr_of_mut!(DLERROR_BUF).cast::<u8>(),
        len,
    );
    DLERROR_BUF[len] = 0;
    DLERROR_SET = true;
}

const DL_GLOBAL_SENTINEL: *mut u8 = 1usize as *mut u8;

type LdsoDlopenFn = unsafe extern "C" fn(*const u8, i32) -> *mut u8;
type LdsoDlsymFn = unsafe extern "C" fn(*mut u8, *const u8) -> *mut u8;
type LdsoDlcloseFn = unsafe extern "C" fn(*mut u8) -> i32;
type LdsoDlerrorFn = unsafe extern "C" fn() -> *const u8;

unsafe fn register_dlopen_callbacks() {
    let reg_open = resolve_symbol(b"__ldso_register_dlopen\0".as_ptr());
    if reg_open != 0 {
        let f: extern "C" fn(LdsoDlopenFn) = core::mem::transmute(reg_open);
        f(__ldso_dlopen as LdsoDlopenFn);
    }
    let reg_sym = resolve_symbol(b"__ldso_register_dlsym\0".as_ptr());
    if reg_sym != 0 {
        let f: extern "C" fn(LdsoDlsymFn) = core::mem::transmute(reg_sym);
        f(__ldso_dlsym as LdsoDlsymFn);
    }
    let reg_close = resolve_symbol(b"__ldso_register_dlclose\0".as_ptr());
    if reg_close != 0 {
        let f: extern "C" fn(LdsoDlcloseFn) = core::mem::transmute(reg_close);
        f(__ldso_dlclose as LdsoDlcloseFn);
    }
    let reg_error = resolve_symbol(b"__ldso_register_dlerror\0".as_ptr());
    if reg_error != 0 {
        let f: extern "C" fn(LdsoDlerrorFn) = core::mem::transmute(reg_error);
        f(__ldso_dlerror as LdsoDlerrorFn);
    }
}

#[no_mangle]
pub unsafe extern "C" fn __ldso_dlopen(filename: *const u8, flags: i32) -> *mut u8 {
    DLERROR_SET = false;
    if filename.is_null() {
        return DL_GLOBAL_SENTINEL;
    }
    let name_len = str_len(filename);
    if let Some(idx) = loaded_object_by_name(filename, name_len) {
        LOADED[idx].global = LOADED[idx].global || (flags & RTLD_GLOBAL) != 0;
        return &mut LOADED[idx] as *mut LoadedObject as *mut u8;
    }
    let ld = if LD_LIBRARY_PATH.is_null() {
        None
    } else {
        Some(LD_LIBRARY_PATH)
    };
    let fd = find_library_fd(filename, name_len, ld);
    if fd < 0 {
        set_dlerror(b"dlopen: cannot open file\0");
        return core::ptr::null_mut();
    }
    let desired = DSO_BASE_START + (LOADED_COUNT as u64) * DSO_BASE_STRIDE;
    let _base = match load_dso_from_fd(fd, desired) {
        Some(b) => b,
        None => {
            sys_close(fd);
            set_dlerror(b"dlopen: failed to load\0");
            return core::ptr::null_mut();
        }
    };
    sys_close(fd);
    let idx = LOADED_COUNT - 1;
    set_loaded_name(idx, filename, name_len);
    LOADED[idx].global = (flags & RTLD_GLOBAL) != 0;
    process_all_relocations();
    update_tls_for_new_module(idx);
    run_constructors_for(idx);
    &mut LOADED[idx] as *mut LoadedObject as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn __ldso_dlsym(handle: *mut u8, symbol: *const u8) -> *mut u8 {
    DLERROR_SET = false;
    if symbol.is_null() {
        set_dlerror(b"dlsym: null symbol\0");
        return core::ptr::null_mut();
    }
    let name_len = str_len(symbol);
    let mut result: u64 = 0;
    if handle == DL_GLOBAL_SENTINEL {
        for i in 0..LOADED_COUNT {
            if i == 0 || LOADED[i].global {
                result = lookup_symbol_in_object(i, symbol, name_len);
                if result != 0 {
                    break;
                }
            }
        }
    } else {
        let idx = ((handle as usize - core::ptr::addr_of_mut!(LOADED) as usize)
            / core::mem::size_of::<LoadedObject>()) as usize;
        if idx < LOADED_COUNT {
            result = lookup_symbol_in_object(idx, symbol, name_len);
        }
        if result == 0 {
            for i in 0..LOADED_COUNT {
                if i == 0 || LOADED[i].global {
                    result = lookup_symbol_in_object(i, symbol, name_len);
                    if result != 0 {
                        break;
                    }
                }
            }
        }
    }
    if result == 0 {
        set_dlerror(b"dlsym: symbol not found\0");
    }
    result as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn __ldso_dlclose(_handle: *mut u8) -> i32 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn __ldso_dlerror() -> *const u8 {
    if DLERROR_SET {
        DLERROR_SET = false;
        core::ptr::addr_of_mut!(DLERROR_BUF).cast::<u8>()
    } else {
        core::ptr::null()
    }
}

unsafe fn compute_tls_layout() {
    let mut total: usize = 0;
    for i in 0..LOADED_COUNT {
        let obj = &LOADED[i];
        let align = if obj.tls_align > 0 { obj.tls_align as usize } else { 1 };
        let block_size = ((obj.tls_memsz as usize + align - 1) / align) * align;
        total += block_size;
    }
    if total < 4096 {
        total = 4096;
    }
    total += total;
    TLS_TOTAL_SIZE = (total + 4095) & !4095;

    let mut end = TLS_TOTAL_SIZE;
    for i in 0..LOADED_COUNT {
        let obj = &LOADED[i];
        if obj.tls_memsz == 0 {
            TLS_LAYOUT_OFFSET[i] = 0;
            TLS_FILESZ[i] = 0;
            TLS_MEMSZ[i] = 0;
            TLS_IMAGE[i] = core::ptr::null();
            continue;
        }
        let align = if obj.tls_align > 0 { obj.tls_align as usize } else { 1 };
        let block_size = ((obj.tls_memsz as usize + align - 1) / align) * align;
        end -= block_size;
        end &= !(align - 1);
        TLS_LAYOUT_OFFSET[i] = end;
        TLS_FILESZ[i] = obj.tls_filesz;
        TLS_MEMSZ[i] = obj.tls_memsz;
        TLS_IMAGE[i] = obj.tls_image;
    }
    TLS_MODULE_COUNT = LOADED_COUNT;
}

unsafe fn init_tls_block(block: *mut u8) -> *mut u8 {
    let var_base = block.add(tls_var_area_offset_from_block());
    for i in 0..TLS_MODULE_COUNT {
        if TLS_MEMSZ[i] == 0 {
            continue;
        }
        let dst = var_base.add(TLS_LAYOUT_OFFSET[i]);
        let src = TLS_IMAGE[i];
        let filesz = TLS_FILESZ[i] as usize;
        let memsz = TLS_MEMSZ[i] as usize;
        if filesz > 0 {
            core::ptr::copy_nonoverlapping(src, dst, filesz);
        }
        if memsz > filesz {
            core::ptr::write_bytes(dst.add(filesz), 0, memsz - filesz);
        }
    }
    let tcb = block.add(tls_tcb_offset_from_block());
    core::ptr::write_unaligned(tcb as *mut u64, tcb as u64);
    core::ptr::write_unaligned((tcb as *mut u64).add(1), TLS_GENERATION);
    block.add(tls_tp_offset_from_block())
}

#[repr(C)]
pub struct TlsIndex {
    ti_module: usize,
    ti_offset: usize,
}

#[no_mangle]
pub unsafe extern "C" fn __tls_get_addr(ti: *const TlsIndex) -> *mut u8 {
    let module = (*ti).ti_module;
    let offset = (*ti).ti_offset;
    let fs_base = read_tp();
    let tcb = (fs_base as isize + tls_tcb_offset_from_tp()) as *mut u8;
    let thread_gen = core::ptr::read_unaligned((tcb as *const u64).add(1));
    if thread_gen != TLS_GENERATION {
        tls_lock();
        let thread_gen2 = core::ptr::read_unaligned((tcb as *const u64).add(1));
        if thread_gen2 != TLS_GENERATION {
            expand_thread_tls(TLS_OLD_TOTAL, TLS_OLD_MODULE_COUNT);
        }
        tls_unlock();
    }
    let fs_base2 = read_tp();
    let tls_base = fs_base2 - tls_var_area_offset_from_tp();
    (tls_base as *mut u8).add(TLS_LAYOUT_OFFSET[module]).add(offset) as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn __rc_create_thread_tls() -> *mut u8 {
    let total = TLS_TOTAL_SIZE + TCB_SIZE;
    if total == 0 {
        return core::ptr::null_mut();
    }
    let block = sys_mmap(
        core::ptr::null_mut(),
        total,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );
    if block as usize == MAP_FAILED {
        return core::ptr::null_mut();
    }
    init_tls_block(block)
}

#[no_mangle]
pub unsafe extern "C" fn __rc_tls_block_size() -> usize {
    TLS_TOTAL_SIZE + TCB_SIZE
}

#[no_mangle]
pub unsafe extern "C" fn __rc_tls_base_offset() -> usize {
    tls_var_area_offset_from_tp() + tls_var_area_offset_from_block()
}

unsafe fn register_self(ldso_base: u64) {
    let ehdr = ldso_base as *const u8;
    if *ehdr != 0x7f || *ehdr.add(1) != b'E' {
        return;
    }
    let e_phoff = u64::from_le_bytes(core::ptr::read_unaligned(ehdr.add(32) as *const [u8; 8]));
    let e_phnum = u16::from_le_bytes(core::ptr::read_unaligned(ehdr.add(56) as *const [u8; 2])) as usize;
    let mut dyn_vaddr: u64 = 0;
    let mut dyn_memsz: u64 = 0;
    for i in 0..e_phnum {
        let ph = ehdr.add(e_phoff as usize + i * PHDR_SIZE);
        let p_type = u32::from_le_bytes(core::ptr::read_unaligned(ph as *const [u8; 4]));
        if p_type == PT_DYNAMIC {
            dyn_vaddr = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_VADDR) as *const [u8; 8]));
            dyn_memsz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_MEMSZ) as *const [u8; 8]));
            break;
        }
    }
    if dyn_vaddr == 0 {
        return;
    }
    let dyn_addr = (ldso_base + dyn_vaddr) as usize;
    let dyn_end = dyn_addr + dyn_memsz as usize;
    let mut dt_symtab: u64 = 0;
    let mut dt_strtab: u64 = 0;
    let mut dt_strsz: u64 = 0;
    let mut pos = dyn_addr;
    while pos + 16 <= dyn_end {
        let d_tag = u64::from_le_bytes(core::ptr::read_unaligned(pos as *const [u8; 8]));
        let d_val = u64::from_le_bytes(core::ptr::read_unaligned((pos + 8) as *const [u8; 8]));
        if d_tag == DT_NULL {
            break;
        }
        match d_tag {
            DT_SYMTAB => dt_symtab = d_val,
            DT_STRTAB => dt_strtab = d_val,
            DT_STRSZ => dt_strsz = d_val,
            _ => {}
        }
        pos += 16;
    }
    if dt_symtab == 0 || dt_strtab == 0 {
        return;
    }
    let symtab_ptr = (ldso_base + dt_symtab) as *const u8;
    let strtab_ptr = (ldso_base + dt_strtab) as *const u8;
    let sym_count = ((dt_strtab - dt_symtab) / SYMTAB_ENT_SIZE as u64) as usize;
    if LOADED_COUNT < MAX_LOADED {
        LOADED[LOADED_COUNT] = LoadedObject {
            base: ldso_base,
            symtab: symtab_ptr,
            sym_count,
            strtab: strtab_ptr,
            strsz: dt_strsz as usize,
            dyn_addr,
            dyn_memsz: dyn_memsz as usize,
            tls_image: core::ptr::null(),
            tls_filesz: 0,
            tls_memsz: 0,
            tls_align: 0,
            init: 0,
            init_array: 0,
            init_array_sz: 0,
            init_present: false,
            init_array_present: false,
            global: false,
            name: [0; 256],
        };
        LOADED_COUNT += 1;
    }
}

unsafe fn set_loaded_name(idx: usize, name: *const u8, name_len: usize) {
    if idx >= MAX_LOADED {
        return;
    }
    let len = if name_len >= 255 { 255 } else { name_len };
    let buf = &mut LOADED[idx].name;
    for i in 0..len {
        buf[i] = *name.add(i);
    }
    buf[len] = 0;
}

unsafe fn loaded_object_by_name(name: *const u8, name_len: usize) -> Option<usize> {
    if name_len == 0 {
        return None;
    }
    for i in 0..LOADED_COUNT {
        if LOADED[i].name[0] == 0 {
            continue;
        }
        if str_eq(name, name_len, LOADED[i].name.as_ptr()) {
            return Some(i);
        }
    }
    None
}

// ============================================================
// Main flow: load executable + dependencies, relocate, jump
// ============================================================

unsafe fn load_and_jump(sp: usize, ldso_base: u64) -> ! {
    // 1. Find LD_LIBRARY_PATH from kernel envp
    let ld_path = find_env(sp, b"LD_LIBRARY_PATH=");
    LD_LIBRARY_PATH = ld_path.unwrap_or(core::ptr::null());

    // 2. Open and read the executable (the PIE that invoked us as PT_INTERP)
    let proc_exe = b"/proc/self/exe\0";
    let fd = sys_open(proc_exe.as_ptr());
    if fd < 0 {
        die(99, b"open_exe", fd as usize);
    }
    {
        let mut exe_path = [0u8; 256];
        let r = sys_readlink(proc_exe.as_ptr(), exe_path.as_mut_ptr(), exe_path.len());
        if r > 0 {
            let len = r as usize;
            let mut slash = len;
            while slash > 0 {
                slash -= 1;
                if exe_path[slash] == b'/' {
                    break;
                }
            }
            ORIGIN_LEN = slash;
            let mut i = 0;
            while i < slash {
                ORIGIN_DIR[i] = exe_path[i];
                i += 1;
            }
        }
    }

    let mut buf = [0u8; 4096];
    let n = sys_read(fd, buf.as_mut_ptr(), buf.len());
    if n < 64 {
        die(98, b"read_exe", n as usize);
    }

    if buf[0] != 0x7f || buf[1] != b'E' {
        die(97, b"elf_magic", u16::from_le_bytes([buf[0], buf[1]]) as usize);
    }

    let e_phoff = u64::from_le_bytes(buf[32..40].try_into().unwrap());
    let e_phnum = u16::from_le_bytes(buf[56..58].try_into().unwrap());
    let e_entry = u64::from_le_bytes(buf[24..32].try_into().unwrap());

    // 3. Map executable's PT_LOAD segments at a safe base address.
    //    PIE p_vaddr often starts at 0 which is below mmap_min_addr on CI.
    //    Pre-scan to find span, probe for free region, then MAP_FIXED there.
    let page = 4096u64;
    let mut min_vaddr = u64::MAX;
    let mut max_vaddr_end = 0u64;
    for i in 0..e_phnum as usize {
        let ph = buf.as_ptr().add(e_phoff as usize + i * PHDR_SIZE);
        let p_type = u32::from_le_bytes(core::ptr::read_unaligned(ph as *const [u8; 4]));
        if p_type != PT_LOAD {
            continue;
        }
        let p_vaddr = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_VADDR) as *const [u8; 8]));
        let p_memsz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_MEMSZ) as *const [u8; 8]));
        if p_vaddr < min_vaddr { min_vaddr = p_vaddr; }
        let end = p_vaddr + p_memsz;
        if end > max_vaddr_end { max_vaddr_end = end; }
    }
    let total_size = ((max_vaddr_end - min_vaddr + page - 1) & !(page - 1)) as usize;
    // ponytail: mmap_min_addr is typically 65536; probe upward to find free span
    let desired = if min_vaddr < 0x10000 { 0x10000 } else { min_vaddr };
    let mut probe_addr = desired;
    let load_start = loop {
        let probe = sys_mmap(
            probe_addr as *mut u8, total_size,
            PROT_NONE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0,
        );
        if probe as usize == probe_addr as usize {
            sys_munmap(probe, total_size);
            break probe_addr;
        }
        if probe as usize != MAP_FAILED {
            sys_munmap(probe, total_size);
        }
        probe_addr += DSO_BASE_STRIDE;
        if probe_addr > desired + DSO_BASE_STRIDE * 16 {
            die(95, b"map_exec", probe_addr as usize);
        }
    };
    let exec_base = load_start - min_vaddr;

    for i in 0..e_phnum as usize {
        let ph = buf.as_ptr().add(e_phoff as usize + i * PHDR_SIZE);
        let p_type = u32::from_le_bytes(core::ptr::read_unaligned(ph as *const [u8; 4]));
        if p_type != PT_LOAD {
            continue;
        }
        let p_flags = u32::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_FLAGS) as *const [u8; 4]));
        let p_offset = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_OFFSET) as *const [u8; 8]));
        let p_vaddr = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_VADDR) as *const [u8; 8]));
        let p_filesz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_FILESZ) as *const [u8; 8]));
        let p_memsz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_MEMSZ) as *const [u8; 8]));

        let adj = p_vaddr & (page - 1);
        let map_addr = exec_base + p_vaddr - adj;
        let map_off = p_offset - adj;
        let map_len = ((p_memsz + adj + page - 1) & !(page - 1)) as usize;
        let prot = prot_from_flags(p_flags);

        // Map the whole segment anonymously first so the tail (bss) is backed
        // by zeroed anonymous pages, then overlay the file-backed portion.
        let ptr = sys_mmap(
            map_addr as *mut u8,
            map_len,
            prot,
            MAP_PRIVATE | MAP_FIXED | MAP_ANONYMOUS,
            -1,
            0,
        );
        if ptr as usize == MAP_FAILED {
            die(95, b"map_exec", map_addr as usize);
        }

        let file_map_len = ((p_filesz + adj + page - 1) & !(page - 1)) as usize;
        if file_map_len > 0 {
            let fptr = sys_mmap(
                map_addr as *mut u8,
                file_map_len,
                prot,
                MAP_PRIVATE | MAP_FIXED,
                fd as i32,
                map_off as i64,
            );
            if fptr as usize == MAP_FAILED {
                die(95, b"map_exec_file", map_addr as usize);
            }
        }

        if p_memsz > p_filesz {
            let bss_start = (exec_base + p_vaddr + p_filesz) as *mut u8;
            let bss_len = (p_memsz - p_filesz) as usize;
            core::ptr::write_bytes(bss_start, 0, bss_len);
        }
    }

    let mut exec_tls_image: *const u8 = core::ptr::null();
    let mut exec_tls_filesz: u64 = 0;
    let mut exec_tls_memsz: u64 = 0;
    let mut exec_tls_align: u64 = 0;
    for i in 0..e_phnum as usize {
        let ph = buf.as_ptr().add(e_phoff as usize + i * PHDR_SIZE);
        let p_type = u32::from_le_bytes(core::ptr::read_unaligned(ph as *const [u8; 4]));
        if p_type == PT_TLS {
            let p_vaddr = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_VADDR) as *const [u8; 8]));
            exec_tls_filesz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_FILESZ) as *const [u8; 8]));
            exec_tls_memsz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_MEMSZ) as *const [u8; 8]));
            exec_tls_align = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_ALIGN) as *const [u8; 8]));
            exec_tls_image = (exec_base + p_vaddr) as *const u8;
            break;
        }
    }

    sys_close(fd);

    // 4. Parse executable's PT_DYNAMIC (base = 0)
    let mut dyn_vaddr: u64 = 0;
    let mut dyn_memsz: u64 = 0;
    for i in 0..e_phnum as usize {
        let ph = buf.as_ptr().add(e_phoff as usize + i * PHDR_SIZE);
        let p_type = u32::from_le_bytes(core::ptr::read_unaligned(ph as *const [u8; 4]));
        if p_type == PT_DYNAMIC {
            dyn_vaddr = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_VADDR) as *const [u8; 8]));
            dyn_memsz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_MEMSZ) as *const [u8; 8]));
            break;
        }
    }

    let mut dt_symtab: u64 = 0;
    let mut dt_strtab: u64 = 0;
    let mut dt_strsz: u64 = 0;
    let mut dt_init: u64 = 0;
    let mut dt_init_array: u64 = 0;
    let mut dt_init_array_sz: u64 = 0;
    let mut dt_init_present = false;
    let mut dt_init_array_present = false;
    let mut dt_runpath_off: u64 = 0;
    let mut dt_runpath_present = false;
    let mut dt_rpath_off: u64 = 0;
    let mut dt_rpath_present = false;
    let mut dt_gnu_hash: u64 = 0;
    let mut dt_hash: u64 = 0;

    // ponytail: max 32 DT_NEEDED, enough for any realistic binary
    let mut needed_offsets: [u64; 32] = [0; 32];
    let mut needed_count: usize = 0;

    if dyn_vaddr != 0 {
        let dyn_start = (exec_base + dyn_vaddr) as usize;
        let dyn_end = dyn_start + dyn_memsz as usize;
        let mut dp = dyn_start;
        while dp + 16 <= dyn_end {
            let d_tag = u64::from_le_bytes(core::ptr::read_unaligned(dp as *const [u8; 8]));
            let d_val = u64::from_le_bytes(core::ptr::read_unaligned((dp + 8) as *const [u8; 8]));
            if d_tag == DT_NULL {
                break;
            }
            match d_tag {
                DT_NEEDED => {
                    if needed_count < 32 {
                        needed_offsets[needed_count] = d_val;
                        needed_count += 1;
                    }
                }
                DT_SYMTAB => dt_symtab = d_val,
                DT_STRTAB => dt_strtab = d_val,
                DT_STRSZ => dt_strsz = d_val,
                DT_GNU_HASH => dt_gnu_hash = d_val,
                DT_HASH => dt_hash = d_val,
                DT_INIT => { dt_init = d_val; dt_init_present = true; }
                DT_INIT_ARRAY => { dt_init_array = d_val; dt_init_array_present = true; }
                DT_INIT_ARRAYSZ => dt_init_array_sz = d_val,
                DT_RUNPATH => { dt_runpath_off = d_val; dt_runpath_present = true; }
                DT_RPATH => { dt_rpath_off = d_val; dt_rpath_present = true; }
                _ => {}
            }
            dp += 16;
        }
    }

    // Resolve DT_NEEDED name pointers (offsets into strtab)
    let mut needed_names: [(*const u8, usize); 32] = [(core::ptr::null(), 0); 32];
    for i in 0..needed_count {
        let name_ptr = (exec_base + dt_strtab + needed_offsets[i]) as *const u8;
        let name_len = str_len(name_ptr);
        needed_names[i] = (name_ptr, name_len);
    }

    if dt_runpath_present {
        RUNPATH = (exec_base + dt_strtab + dt_runpath_off) as *const u8;
        RUNPATH_LEN = str_len(RUNPATH);
    } else if dt_rpath_present {
        RUNPATH = (exec_base + dt_strtab + dt_rpath_off) as *const u8;
        RUNPATH_LEN = str_len(RUNPATH);
    }

    // Register executable as LOADED[0]
    let mut exec_sym_count: usize = 0;
    if dt_gnu_hash != 0 {
        exec_sym_count = sym_count_from_gnu_hash((exec_base + dt_gnu_hash) as usize);
    } else if dt_hash != 0 {
        exec_sym_count = sym_count_from_hash((exec_base + dt_hash) as usize);
    } else if dt_strtab > dt_symtab && dt_strtab - dt_symtab >= SYMTAB_ENT_SIZE as u64 {
        exec_sym_count = ((dt_strtab - dt_symtab) / SYMTAB_ENT_SIZE as u64) as usize;
    }
    LOADED[0] = LoadedObject {
        base: exec_base,
        symtab: (exec_base + dt_symtab) as *const u8,
        sym_count: exec_sym_count,
        strtab: (exec_base + dt_strtab) as *const u8,
        strsz: dt_strsz as usize,
        dyn_addr: (exec_base + dyn_vaddr) as usize,
        dyn_memsz: dyn_memsz as usize,
        tls_image: exec_tls_image,
        tls_filesz: exec_tls_filesz,
        tls_memsz: exec_tls_memsz,
        tls_align: exec_tls_align,
        init: exec_base + dt_init,
        init_array: exec_base + dt_init_array,
        init_array_sz: dt_init_array_sz,
        init_present: dt_init_present,
        init_array_present: dt_init_array_present,
        global: true,
        name: [0; 256],
    };
    LOADED_COUNT = 1;
    register_self(ldso_base);

    // 5. Load each DT_NEEDED DSO
    for i in 0..needed_count {
        let (name_ptr, name_len) = needed_names[i];
        let lib_fd = find_library_fd(name_ptr, name_len, ld_path);
        if lib_fd < 0 {
            die(89, b"needed_fd", i);
        }
        let desired_base = DSO_BASE_START + (i as u64) * DSO_BASE_STRIDE;
        if load_dso_from_fd(lib_fd, desired_base).is_none() {
            sys_close(lib_fd);
            die(88, b"load_dso", desired_base as usize);
        }
        sys_close(lib_fd);
        set_loaded_name(LOADED_COUNT - 1, name_ptr, name_len);
    }

    compute_tls_layout();
    TLS_OLD_TOTAL = TLS_TOTAL_SIZE;
    TLS_OLD_MODULE_COUNT = TLS_MODULE_COUNT;

    process_all_relocations();
    register_dlopen_callbacks();

    // Always allocate a TCB so that %fs-relative accesses (e.g. stack canary
    // at %fs:0x28) work even when there is no TLS data in the binary.
    {
        let alloc_size = TLS_TOTAL_SIZE + TCB_SIZE;
        let tls_block = sys_mmap(
            core::ptr::null_mut(),
            alloc_size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        );
        if tls_block as usize == MAP_FAILED {
            die(93, b"tls_mmap", alloc_size);
        }
        let _tcb = init_tls_block(tls_block);
        write_tp(_tcb as usize);
    }

    // Set libc.so's __auxv so constructors (e.g. compiler_builtins CPU feature
    // detection) can call getauxval before __libc_start_main runs.
    let argc = *(sp as *const u64) as usize;
    let argv_start = sp + 8;
    let envp_start = argv_start + (argc + 1) * 8;
    let mut envc = 0usize;
    while *((envp_start + envc * 8) as *const u64) != 0 {
        envc += 1;
    }
    let auxv = (envp_start + (envc + 1) * 8) as *const usize;
    let auxv_sym = resolve_symbol(b"__auxv\0".as_ptr());
    if auxv_sym != 0 {
        core::ptr::write(auxv_sym as *mut *const usize, auxv);
    }

    run_constructors();

    let phdr_addr = exec_base + e_phoff;
    build_and_jump(exec_base + e_entry, phdr_addr, e_phnum, sp)
}

// ============================================================
// Build a fresh stack for the target program and jump
// ============================================================

struct OrigAuxv {
    at_base: u64,
    at_random: *const u8,
    at_secure: u64,
    at_uid: u64,
    at_euid: u64,
    at_gid: u64,
    at_egid: u64,
}

unsafe fn read_orig_auxv(orig_sp: usize, envc: usize) -> OrigAuxv {
    let mut out = OrigAuxv {
        at_base: 0,
        at_random: core::ptr::null(),
        at_secure: 0,
        at_uid: 0,
        at_euid: 0,
        at_gid: 0,
        at_egid: 0,
    };
    let argc = *(orig_sp as *const u64) as usize;
    let argv_start = orig_sp + 8;
    let envp_start = argv_start + (argc + 1) * 8;
    let auxv = (envp_start + (envc + 1) * 8) as *const u64;
    let mut i = 0;
    loop {
        let tag = *auxv.add(i * 2);
        let val = *auxv.add(i * 2 + 1);
        if tag == AT_NULL {
            break;
        }
        match tag {
            AT_BASE => out.at_base = val,
            AT_RANDOM => out.at_random = val as *const u8,
            AT_SECURE => out.at_secure = val,
            AT_UID => out.at_uid = val,
            AT_EUID => out.at_euid = val,
            AT_GID => out.at_gid = val,
            AT_EGID => out.at_egid = val,
            _ => {}
        }
        i += 1;
    }
    out
}

unsafe fn build_and_jump(entry: u64, phdr_addr: u64, phnum: u16, orig_sp: usize) -> ! {
    let argc = *(orig_sp as *const u64) as usize;
    let argv_start = orig_sp + 8;
    let envp_start = argv_start + (argc + 1) * 8;

    let mut envc: usize = 0;
    while *((envp_start + envc * 8) as *const u64) != 0 {
        envc += 1;
    }

    // ponytail: max 128 args, 512 env vars — covers any realistic binary
    let max_args = if argc > 128 { 128 } else { argc };
    let max_env = if envc > 512 { 512 } else { envc };

    let orig_auxv = read_orig_auxv(orig_sp, envc);

    let stack_size = 256 * 1024usize;
    let stack_base = sys_mmap(
        core::ptr::null_mut(),
        stack_size,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );
    if stack_base as usize == MAP_FAILED {
        die(94, b"stack_mmap", stack_size);
    }
    let mut sp = stack_base as usize + stack_size;

    let mut new_argv: [usize; 128] = [0; 128];
    for i in 0..max_args {
        let s = *((argv_start + i * 8) as *const u64) as *const u8;
        let len = str_len(s);
        sp -= len + 1;
        core::ptr::copy_nonoverlapping(s, sp as *mut u8, len + 1);
        new_argv[i] = sp;
    }

    let mut new_envp: [usize; 512] = [0; 512];
    for i in 0..max_env {
        let s = *((envp_start + i * 8) as *const u64) as *const u8;
        let len = str_len(s);
        sp -= len + 1;
        core::ptr::copy_nonoverlapping(s, sp as *mut u8, len + 1);
        new_envp[i] = sp;
    }

    // musl's crt1 needs AT_RANDOM for the stack canary; preserve it on the new stack.
    let mut random_bytes = [0u8; 16];
    if !orig_auxv.at_random.is_null() {
        core::ptr::copy_nonoverlapping(orig_auxv.at_random, random_bytes.as_mut_ptr(), 16);
    } else {
        for i in 0..16 {
            random_bytes[i] = (i as u8).wrapping_add(1);
        }
    }
    sp -= 16;
    sp &= !15usize;
    let random_ptr = sp;
    core::ptr::copy_nonoverlapping(random_bytes.as_ptr(), random_ptr as *mut u8, 16);

    // 16-byte align before structured data so argc lands on a boundary
    sp &= !15usize;

    // Pad before auxv so argc lands on 16-byte boundary (no gap between argv[] and argc)
    if (max_env + max_args) % 2 == 0 {
        sp -= 8;
        *(sp as *mut u64) = 0;
    }

    const AUXV_ENTRIES: usize = 13;
    sp -= AUXV_ENTRIES * 16;
    let aux = sp as *mut u64;
    *aux.add(0) = AT_PHDR;
    *aux.add(1) = phdr_addr;
    *aux.add(2) = AT_PHENT;
    *aux.add(3) = PHDR_SIZE as u64;
    *aux.add(4) = AT_PHNUM;
    *aux.add(5) = phnum as u64;
    *aux.add(6) = AT_PAGESZ;
    *aux.add(7) = 4096;
    *aux.add(8) = AT_ENTRY;
    *aux.add(9) = entry;
    *aux.add(10) = AT_BASE;
    *aux.add(11) = orig_auxv.at_base;
    *aux.add(12) = AT_SECURE;
    *aux.add(13) = orig_auxv.at_secure;
    *aux.add(14) = AT_RANDOM;
    *aux.add(15) = random_ptr as u64;
    *aux.add(16) = AT_UID;
    *aux.add(17) = orig_auxv.at_uid;
    *aux.add(18) = AT_EUID;
    *aux.add(19) = orig_auxv.at_euid;
    *aux.add(20) = AT_GID;
    *aux.add(21) = orig_auxv.at_gid;
    *aux.add(22) = AT_EGID;
    *aux.add(23) = orig_auxv.at_egid;
    *aux.add(24) = AT_NULL;
    *aux.add(25) = 0;

    sp -= (max_env + 1) * 8;
    for i in 0..max_env {
        *((sp + i * 8) as *mut u64) = new_envp[i] as u64;
    }
    *((sp + max_env * 8) as *mut u64) = 0;

    sp -= (max_args + 1) * 8;
    for i in 0..max_args {
        *((sp + i * 8) as *mut u64) = new_argv[i] as u64;
    }
    *((sp + max_args * 8) as *mut u64) = 0;

    sp -= 8;
    *(sp as *mut u64) = argc as u64;

    #[cfg(target_arch = "x86_64")]
    core::arch::asm!(
        "mov rsp, {sp}",
        "jmp {entry}",
        sp = in(reg) sp,
        entry = in(reg) entry,
        options(noreturn)
    );

    #[cfg(target_arch = "aarch64")]
    core::arch::asm!(
        "mov sp, {sp}",
        "br {entry}",
        sp = in(reg) sp,
        entry = in(reg) entry,
        options(noreturn)
    );
}

// ============================================================
// Memory functions (required by no_std linker)
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void {
    let s = s as *mut u8;
    let mut p = s;
    let mut i = 0;
    while i < n {
        unsafe {
            *p = c as u8;
        }
        p = unsafe { p.add(1) };
        i += 1;
    }
    s as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let dst = dst as *mut u8;
    let src = src as *const u8;
    let mut i = 0;
    while i < n {
        unsafe {
            *dst.add(i) = *src.add(i);
        }
        i += 1;
    }
    dst as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let dst = dst as *mut u8;
    let src = src as *const u8;
    if (dst as usize) < (src as usize) {
        let mut i = 0;
        while i < n {
            unsafe {
                *dst.add(i) = *src.add(i);
            }
            i += 1;
        }
    } else {
        let mut i = n;
        while i > 0 {
            i -= 1;
            unsafe {
                *dst.add(i) = *src.add(i);
            }
        }
    }
    dst as *mut c_void
}
