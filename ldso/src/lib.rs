#![no_std]
#![no_main]
#![allow(dead_code, deref_nullptr)]

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
const DT_STRTAB: u64 = 5;
const DT_SYMTAB: u64 = 6;
const DT_RELA: u64 = 7;
const DT_RELASZ: u64 = 8;
const DT_STRSZ: u64 = 10;
const DT_JMPREL: u64 = 23;

const R_X86_64_64: u64 = 1;
const R_X86_64_GLOB_DAT: u64 = 6;
const R_X86_64_JUMP_SLOT: u64 = 7;
const R_X86_64_RELATIVE: u64 = 8;

const AT_NULL: u64 = 0;
const AT_PHDR: u64 = 3;
const AT_PHNUM: u64 = 5;
const AT_PAGESZ: u64 = 6;
const AT_BASE: u64 = 7;
const AT_ENTRY: u64 = 9;

const PROT_READ: i32 = 1;
const PROT_WRITE: i32 = 2;
const PROT_EXEC: i32 = 4;
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
}

const EMPTY_OBJ: LoadedObject = LoadedObject {
    base: 0,
    symtab: core::ptr::null(),
    sym_count: 0,
    strtab: core::ptr::null(),
    strsz: 0,
    dyn_addr: 0,
    dyn_memsz: 0,
};

// Safety: only accessed from single-threaded _start -> run_main
static mut LOADED: [LoadedObject; MAX_LOADED] = [EMPTY_OBJ; MAX_LOADED];
static mut LOADED_COUNT: usize = 0;

// ============================================================
// _start: self-relocate ldso, then call run_main(sp)
// ============================================================

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

// ============================================================
// Entry point
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn run_main(sp: usize) -> ! {
    unsafe { load_and_jump(sp) }
}

// ============================================================
// String helpers (no_std)
// ============================================================

unsafe fn strlen(s: *const u8) -> usize {
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
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

fn sys_open(path: *const u8) -> i64 {
    let result: i64;
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") 2i64 => result,
            in("rdi") path,
            in("rsi") 0i64,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    result
}

fn sys_read(fd: i64, buf: *mut u8, count: usize) -> i64 {
    let result: i64;
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") 0i64 => result,
            in("rdi") fd,
            in("rsi") buf,
            in("rdx") count,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    result
}

fn sys_close(fd: i64) {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 3i64,
            in("rdi") fd,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
}

fn sys_mmap(
    addr: *mut u8,
    length: usize,
    prot: i32,
    flags: i32,
    fd: i32,
    offset: i64,
) -> *mut u8 {
    let result: i64;
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") 9i64 => result,
            in("rdi") addr,
            in("rsi") length,
            in("rdx") prot,
            in("r10") flags,
            in("r8") fd,
            in("r9") offset,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    result as *mut u8
}

fn sys_exit(code: i32) -> ! {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 60i64,
            in("rdi") code,
            options(noreturn)
        );
    }
}

fn sys_lseek(fd: i64, offset: i64) -> i64 {
    let result: i64;
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") 8i64 => result,
            in("rdi") fd,
            in("rsi") offset,
            in("rdx") 0i64, // SEEK_SET
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    result
}

fn sys_arch_prctl(code: i64, addr: u64) -> i64 {
    let result: i64;
    unsafe {
        core::arch::asm!(
            "syscall",
            inlateout("rax") 158i64 => result,
            in("rdi") code,
            in("rsi") addr,
            lateout("rcx") _,
            lateout("r11") _,
        );
    }
    result
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

/// Try to open `lib_name` (null-terminated, length known) by searching:
///   LD_LIBRARY_PATH, then /lib, /usr/lib, /usr/local/lib.
/// Returns fd >= 0 on success, -1 on failure.
unsafe fn find_library_fd(
    lib_name: *const u8,
    lib_name_len: usize,
    ld_path: Option<*const u8>,
) -> i64 {
    let mut path_buf = [0u8; 512];

    // Helper: try to open dir/lib_name.  Returns fd or -1.
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

    // 1. LD_LIBRARY_PATH (colon-separated)
    if let Some(ldp) = ld_path {
        let ldp_len = strlen(ldp);
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

    // 2. Default paths
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
unsafe fn load_dso_from_fd(fd: i64, base: u64) -> bool {
    let mut buf = [0u8; 4096];
    let n = sys_read(fd, buf.as_mut_ptr(), buf.len());
    if n < 64 {
        return false;
    }
    if buf[0] != 0x7f || buf[1] != b'E' {
        return false;
    }

    let e_phoff = u64::from_le_bytes(buf[32..40].try_into().unwrap());
    let e_phnum = u16::from_le_bytes(buf[56..58].try_into().unwrap()) as usize;
    let phdr_end = e_phoff as usize + e_phnum * PHDR_SIZE;
    if phdr_end > n as usize {
        return false;
    }

    // Map PT_LOAD segments at base + p_vaddr
    for i in 0..e_phnum {
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

        let page = 4096u64;
        let adj = p_vaddr & (page - 1);
        let map_addr = base + p_vaddr - adj;
        let map_off = p_offset - adj;
        let map_len = ((p_filesz + adj + page - 1) & !(page - 1)) as usize;
        let prot = prot_from_flags(p_flags);

        let ptr = sys_mmap(
            map_addr as *mut u8,
            map_len,
            prot,
            MAP_PRIVATE | MAP_FIXED,
            fd as i32,
            map_off as i64,
        );
        if ptr as usize == MAP_FAILED {
            return false;
        }

        if p_memsz > p_filesz {
            let bss_start = (base + p_vaddr + p_filesz) as *mut u8;
            let bss_len = (p_memsz - p_filesz) as usize;
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
        return false;
    }

    let dyn_addr = (base + dyn_vaddr) as usize;
    let dyn_end = dyn_addr + dyn_memsz as usize;

    // Parse DT_SYMTAB, DT_STRTAB, DT_STRSZ
    let mut dt_symtab: u64 = 0;
    let mut dt_strtab: u64 = 0;
    let mut dt_strsz: u64 = 0;
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
            _ => {}
        }
        dp += 16;
    }

    let symtab_ptr = (base + dt_symtab) as *const u8;
    let strtab_ptr = (base + dt_strtab) as *const u8;
    let strsz = dt_strsz as usize;

    // sym_count: entries between symtab and strtab (adjacent in typical ELF layout)
    let sym_count = if dt_strtab > dt_symtab && dt_strtab - dt_symtab >= SYMTAB_ENT_SIZE as u64 {
        ((dt_strtab - dt_symtab) / SYMTAB_ENT_SIZE as u64) as usize
    } else {
        0
    };

    if LOADED_COUNT < MAX_LOADED {
        LOADED[LOADED_COUNT] = LoadedObject {
            base,
            symtab: symtab_ptr,
            sym_count,
            strtab: strtab_ptr,
            strsz,
            dyn_addr,
            dyn_memsz: dyn_memsz as usize,
        };
        LOADED_COUNT += 1;
    }

    true
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
    if sym_idx * SYMTAB_ENT_SIZE >= obj.sym_count * SYMTAB_ENT_SIZE {
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
    let name_len = strlen(name);
    if name_len == 0 {
        return 0;
    }

    for i in 0..LOADED_COUNT {
        let obj = &LOADED[i];
        if obj.symtab.is_null() || obj.strtab.is_null() {
            continue;
        }
        for j in 0..obj.sym_count {
            let sym_entry = obj.symtab.add(j * SYMTAB_ENT_SIZE);
            let st_name_off =
                u32::from_le_bytes(core::ptr::read_unaligned(sym_entry as *const [u8; 4])) as usize;
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
                return obj.base + st_value;
            }
        }
    }
    0
}

// ============================================================
// Relocation processing
// ============================================================

/// Process all relocations for every loaded object.
unsafe fn process_all_relocations() {
    for i in 0..LOADED_COUNT {
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

        // .rela.dyn
        apply_rela_table(i, base, rela_off, rela_sz);
        // .rela.plt
        apply_rela_table(i, base, jmprel_off, jmprel_sz);
    }
}

/// Apply entries from one relocation table.
unsafe fn apply_rela_table(obj_idx: usize, base: u64, table_off: u64, table_sz: u64) {
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

        match r_type {
            R_X86_64_RELATIVE => {
                *slot = (base as i64 + r_addend) as u64;
            }
            R_X86_64_64 => {
                let sym_value = resolve_symbol_from_index(obj_idx, r_sym_idx);
                *slot = (sym_value as i64 + r_addend) as u64;
            }
            R_X86_64_GLOB_DAT | R_X86_64_JUMP_SLOT => {
                let sym_value = resolve_symbol_from_index(obj_idx, r_sym_idx);
                *slot = sym_value;
            }
            _ => {}
        }
    }
}

// ============================================================
// Main flow: load executable + dependencies, relocate, jump
// ============================================================

unsafe fn load_and_jump(sp: usize) -> ! {
    // 1. Find LD_LIBRARY_PATH from kernel envp
    let ld_path = find_env(sp, b"LD_LIBRARY_PATH=");

    // 2. Open and read the executable (the PIE that invoked us as PT_INTERP)
    let proc_exe = b"/proc/self/exe\0";
    let fd = sys_open(proc_exe.as_ptr());
    if fd < 0 {
        sys_exit(99);
    }

    let mut buf = [0u8; 4096];
    let n = sys_read(fd, buf.as_mut_ptr(), buf.len());
    if n < 64 {
        sys_exit(98);
    }

    if buf[0] != 0x7f || buf[1] != b'E' {
        sys_exit(97);
    }

    let e_phoff = u64::from_le_bytes(buf[32..40].try_into().unwrap());
    let e_phnum = u16::from_le_bytes(buf[56..58].try_into().unwrap());
    let e_entry = u64::from_le_bytes(buf[24..32].try_into().unwrap());

    // 3. Map executable's PT_LOAD segments at p_vaddr (base = 0 for PIE)
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

        let page = 4096u64;
        let adj = p_vaddr & (page - 1);
        let map_addr = p_vaddr - adj;
        let map_off = p_offset - adj;
        let map_len = ((p_filesz + adj + page - 1) & !(page - 1)) as usize;
        let prot = prot_from_flags(p_flags);

        let ptr = sys_mmap(
            map_addr as *mut u8,
            map_len,
            prot,
            MAP_PRIVATE | MAP_FIXED,
            fd as i32,
            map_off as i64,
        );
        if ptr as usize == MAP_FAILED {
            sys_exit(95);
        }

        if p_memsz > p_filesz {
            let bss_start = (p_vaddr + p_filesz) as *mut u8;
            let bss_len = (p_memsz - p_filesz) as usize;
            core::ptr::write_bytes(bss_start, 0, bss_len);
        }
    }

    // Static TLS setup for the main executable
    {
        let mut tls_offset: u64 = 0;
        let mut tls_filesz: u64 = 0;
        let mut tls_memsz: u64 = 0;
        let mut tls_align: u64 = 0;
        for i in 0..e_phnum as usize {
            let ph = buf.as_ptr().add(e_phoff as usize + i * PHDR_SIZE);
            let p_type = u32::from_le_bytes(core::ptr::read_unaligned(ph as *const [u8; 4]));
            if p_type == PT_TLS {
                tls_offset = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_OFFSET) as *const [u8; 8]));
                tls_filesz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_FILESZ) as *const [u8; 8]));
                tls_memsz = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_MEMSZ) as *const [u8; 8]));
                tls_align = u64::from_le_bytes(core::ptr::read_unaligned(ph.add(PH_ALIGN) as *const [u8; 8]));
                break;
            }
        }
        if tls_memsz > 0 {
            let align = if tls_align > 0 { tls_align } else { 4096 };
            let total_tls_size = ((tls_memsz + align - 1) & !(align - 1)) as usize;
            let alloc_size = total_tls_size + TCB_SIZE;

            let tls_block = sys_mmap(
                core::ptr::null_mut(),
                alloc_size,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            );
            if tls_block as usize == MAP_FAILED {
                sys_exit(93);
            }

            sys_lseek(fd, tls_offset as i64);
            let n = sys_read(fd, tls_block, tls_filesz as usize);
            if n < tls_filesz as i64 {
                sys_exit(92);
            }

            if tls_memsz > tls_filesz {
                let bss_start = tls_block.add(tls_filesz as usize);
                let bss_len = (tls_memsz - tls_filesz) as usize;
                core::ptr::write_bytes(bss_start, 0, bss_len);
            }

            let tcb_addr = tls_block.add(total_tls_size);
            core::ptr::write_unaligned(tcb_addr as *mut u64, tcb_addr as u64);

            // ARCH_SET_FS = 0x1002
            sys_arch_prctl(0x1002, tcb_addr as u64);
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

    // ponytail: max 32 DT_NEEDED, enough for any realistic binary
    let mut needed_offsets: [u64; 32] = [0; 32];
    let mut needed_count: usize = 0;

    if dyn_vaddr != 0 {
        let dyn_start = dyn_vaddr as usize;
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
                _ => {}
            }
            dp += 16;
        }
    }

    // Resolve DT_NEEDED name pointers (offsets into strtab)
    let mut needed_names: [(*const u8, usize); 32] = [(core::ptr::null(), 0); 32];
    for i in 0..needed_count {
        let name_ptr = (dt_strtab + needed_offsets[i]) as *const u8;
        let name_len = strlen(name_ptr);
        needed_names[i] = (name_ptr, name_len);
    }

    // Register executable as LOADED[0]
    let exec_sym_count = if dt_strtab > dt_symtab && dt_strtab - dt_symtab >= SYMTAB_ENT_SIZE as u64 {
        ((dt_strtab - dt_symtab) / SYMTAB_ENT_SIZE as u64) as usize
    } else {
        0
    };
    LOADED[0] = LoadedObject {
        base: 0,
        symtab: dt_symtab as *const u8,
        sym_count: exec_sym_count,
        strtab: dt_strtab as *const u8,
        strsz: dt_strsz as usize,
        dyn_addr: dyn_vaddr as usize,
        dyn_memsz: dyn_memsz as usize,
    };
    LOADED_COUNT = 1;

    // 5. Load each DT_NEEDED DSO
    for i in 0..needed_count {
        let (name_ptr, name_len) = needed_names[i];
        let lib_fd = find_library_fd(name_ptr, name_len, ld_path);
        if lib_fd < 0 {
            sys_exit(89);
        }
        let base = DSO_BASE_START + (i as u64) * DSO_BASE_STRIDE;
        if !load_dso_from_fd(lib_fd, base) {
            sys_close(lib_fd);
            sys_exit(88);
        }
        sys_close(lib_fd);
    }

    // 6. Process all relocations (symbolic + RELATIVE) for all objects
    process_all_relocations();

    // 7. Compute phdr address in mapped memory and jump
    let phdr_addr = e_phoff; // base=0 for PIE, so phdr table is at e_phoff
    build_and_jump(e_entry, phdr_addr, e_phnum)
}

// ============================================================
// Build a fresh stack for the target program and jump
// ============================================================

unsafe fn build_and_jump(entry: u64, phdr_addr: u64, phnum: u16) -> ! {
    let stack_size = 64 * 1024usize;
    let stack_base = sys_mmap(
        core::ptr::null_mut(),
        stack_size,
        PROT_READ | PROT_WRITE,
        MAP_PRIVATE | MAP_ANONYMOUS,
        -1,
        0,
    );
    if stack_base as usize == MAP_FAILED {
        sys_exit(94);
    }

    let mut sp = stack_base as usize + stack_size;

    // argv[0] = "tiny\0"
    let argv0 = b"tiny\0";
    sp -= argv0.len();
    let src = argv0.as_ptr();
    let dst = sp as *mut u8;
    *dst = *src;
    *dst.add(1) = *src.add(1);
    *dst.add(2) = *src.add(2);
    *dst.add(3) = *src.add(3);
    *dst.add(4) = *src.add(4);
    let argv0_ptr = sp;

    sp &= !7usize;

    // auxv
    sp -= 5 * 16;
    let auxv_ptr = sp as *mut u64;
    *auxv_ptr.add(0) = AT_PHDR;
    *auxv_ptr.add(1) = phdr_addr;
    *auxv_ptr.add(2) = AT_PHNUM;
    *auxv_ptr.add(3) = phnum as u64;
    *auxv_ptr.add(4) = AT_PAGESZ;
    *auxv_ptr.add(5) = 4096;
    *auxv_ptr.add(6) = AT_ENTRY;
    *auxv_ptr.add(7) = entry;
    *auxv_ptr.add(8) = AT_NULL;
    *auxv_ptr.add(9) = 0;

    // envp = NULL
    sp -= 8;
    *(sp as *mut u64) = 0;
    // argv terminator = NULL
    sp -= 8;
    *(sp as *mut u64) = 0;
    // argv[0]
    sp -= 8;
    *(sp as *mut u64) = argv0_ptr as u64;
    // argc = 1
    sp -= 8;
    *(sp as *mut u64) = 1;

    sp &= !15usize;

    core::arch::asm!(
        "mov rsp, {sp}",
        "jmp {entry}",
        sp = in(reg) sp,
        entry = in(reg) entry,
        in("rdi") 1usize,
        in("rsi") sp + 8,
        options(noreturn)
    );
}

// ============================================================
// Memory functions (required by no_std linker)
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut p = s;
    let mut i = 0;
    while i < n {
        unsafe {
            *p = c as u8;
        }
        p = unsafe { p.add(1) };
        i += 1;
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        unsafe {
            *dst.add(i) = *src.add(i);
        }
        i += 1;
    }
    dst
}

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
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
    dst
}
