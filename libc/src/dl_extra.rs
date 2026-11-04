// dladdr / dl_iterate_phdr / dlinfo.

#[repr(C)]
pub struct DlInfo {
    pub dli_fname: *const c_char,
    pub dli_fbase: *mut c_void,
    pub dli_sname: *const c_char,
    pub dli_saddr: *mut c_void,
}

#[repr(C)]
pub struct DlPhdrInfo {
    pub dlpi_addr: u64,
    pub dlpi_name: *const c_char,
    pub dlpi_phdr: *const u8,
    pub dlpi_phnum: u16,
    __pad0: u16,
    __pad1: u32,
    pub dlpi_adds: u64,
    pub dlpi_subs: u64,
    pub dlpi_tls_modid: usize,
    pub dlpi_tls_data: *mut c_void,
}

type DlIterateCb = unsafe extern "C" fn(*mut DlPhdrInfo, usize, *mut c_void) -> c_int;
type LdsoIterateFn = unsafe extern "C" fn(Option<DlIterateCb>, *mut c_void) -> c_int;
type LdsoDlinfoFn = unsafe extern "C" fn(*mut c_void, c_int, *mut c_void) -> c_int;

static mut LDSO_ITERATE: Option<LdsoIterateFn> = None;
static mut LDSO_DLINFO: Option<LdsoDlinfoFn> = None;

#[no_mangle]
pub unsafe extern "C" fn __ldso_register_dl_iterate(f: LdsoIterateFn) {
    LDSO_ITERATE = Some(f);
}

#[no_mangle]
pub unsafe extern "C" fn __ldso_register_dlinfo(f: LdsoDlinfoFn) {
    LDSO_DLINFO = Some(f);
}

// ------------------------------------------------------------------
// /proc/self/maps
// ------------------------------------------------------------------

static mut MAPS_PATH: [c_char; 512] = [0; 512];

unsafe fn maps_parse_hex(p: *const u8) -> (u64, *const u8) {
    let mut v: u64 = 0;
    let mut q = p;
    loop {
        let c = *q;
        let d = if c >= b'0' && c <= b'9' {
            (c - b'0') as u64
        } else if c >= b'a' && c <= b'f' {
            (c - b'a' + 10) as u64
        } else if c >= b'A' && c <= b'F' {
            (c - b'A' + 10) as u64
        } else {
            break;
        };
        v = (v << 4) | d;
        q = q.add(1);
    }
    (v, q)
}

/// Find the mapping containing `addr`: (base, path). The path is copied into
/// MAPS_PATH and must be consumed before the next call.
unsafe fn maps_lookup(addr: usize) -> (u64, u64, *const c_char, bool) {
    MAPS_PATH[0] = 0;
    let f = crate::fopen(b"/proc/self/maps\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    if f.is_null() { return (0, 0, MAPS_PATH.as_ptr(), false); }
    let mut line = [0 as c_char; 1024];
    let mut start_out = 0u64;
    let mut off_out = 0u64;
    let mut found = false;
    while !crate::fgets(line.as_mut_ptr(), line.len() as c_int, f).is_null() {
        let p = line.as_ptr() as *const u8;
        let (start, after) = maps_parse_hex(p);
        if *after != b'-' { continue; }
        let (end, after2) = maps_parse_hex(after.add(1));
        if (addr as u64) < start || (addr as u64) >= end { continue; }
        // Field order after the range: perms offset dev inode path.
        let mut q = after2;
        while *q == b' ' { q = q.add(1); }
        while *q != 0 && *q != b' ' { q = q.add(1); }
        while *q == b' ' { q = q.add(1); }
        let (off, after_off) = maps_parse_hex(q);
        start_out = start;
        off_out = off;
        let mut q2 = after_off;
        for _ in 0..2 {
            while *q2 == b' ' { q2 = q2.add(1); }
            while *q2 != 0 && *q2 != b' ' { q2 = q2.add(1); }
        }
        while *q2 == b' ' { q2 = q2.add(1); }
        if *q2 != 0 {
            let mut i = 0usize;
            while *q2 != 0 && *q2 != b'\n' && i + 1 < MAPS_PATH.len() {
                MAPS_PATH[i] = *q2 as c_char;
                i += 1;
                q2 = q2.add(1);
            }
            MAPS_PATH[i] = 0;
        }
        found = true;
        break;
    }
    crate::fclose(f);
    (start_out, off_out, MAPS_PATH.as_ptr(), found)
}

/// Compute the load bias for a mapping given the mapping start and file offset.
/// For a PT_LOAD segment, start = bias + p_vaddr - adj and off = p_offset - adj,
/// so bias = (start - off) - (p_vaddr - p_offset).
unsafe fn elf_bias_for(path: *const c_char, start: u64, off: u64) -> u64 {
    let fd = crate::open(path, 0, 0);
    if fd < 0 { return start.wrapping_sub(off); }
    let mut eh = [0u8; 64];
    if crate::pread(fd, eh.as_mut_ptr() as *mut c_void, 64, 0) != 64 || eh[0] != 0x7f {
        crate::close(fd);
        return start.wrapping_sub(off);
    }
    let e_phoff = rd64(eh.as_ptr().add(0x20));
    let e_phentsize = rd16(eh.as_ptr().add(0x36)) as usize;
    let e_phnum = rd16(eh.as_ptr().add(0x38)) as usize;
    if e_phentsize != 56 || e_phnum == 0 || e_phnum > 512 {
        crate::close(fd);
        return start.wrapping_sub(off);
    }
    let total = e_phnum * e_phentsize;
    let ph = crate::malloc(total) as *mut u8;
    if ph.is_null() {
        crate::close(fd);
        return start.wrapping_sub(off);
    }
    let got = crate::pread(fd, ph as *mut c_void, total, e_phoff as i64);
    crate::close(fd);
    if got != total as isize {
        crate::free(ph as *mut c_void);
        return start.wrapping_sub(off);
    }
    let mut bias = start.wrapping_sub(off);
    for i in 0..e_phnum {
        let p = ph.add(i * e_phentsize);
        let p_type = rd32(p);
        if p_type != 1 { continue; }
        let p_offset = rd64(p.add(8));
        let p_vaddr = rd64(p.add(0x10));
        // The loader maps each segment at file offset p_offset - (p_vaddr&0xfff).
        let map_off = p_offset.wrapping_sub(p_vaddr & 0xfff);
        if off == map_off {
            bias = start.wrapping_sub(off).wrapping_sub(p_vaddr.wrapping_sub(p_offset));
            break;
        }
    }
    crate::free(ph as *mut c_void);
    bias
}

// ------------------------------------------------------------------
// ELF symbol lookup (for dladdr)
// ------------------------------------------------------------------

static mut DLADDR_NAME: [c_char; 512] = [0; 512];

unsafe fn rd16(p: *const u8) -> u16 { core::ptr::read_unaligned(p as *const u16) }
unsafe fn rd32(p: *const u8) -> u32 { core::ptr::read_unaligned(p as *const u32) }
unsafe fn rd64(p: *const u8) -> u64 { core::ptr::read_unaligned(p as *const u64) }

unsafe fn elf_symbol_lookup(path: *const c_char, addr: u64) -> (bool, u64) {
    let fd = crate::open(path, 0, 0);
    if fd < 0 { return (false, 0); }
    let mut eh = [0u8; 64];
    if crate::pread(fd, eh.as_mut_ptr() as *mut c_void, 64, 0) != 64 {
        crate::close(fd);
        return (false, 0);
    }
    if eh[0] != 0x7f || eh[1] != b'E' || eh[2] != b'L' || eh[3] != b'F' {
        crate::close(fd);
        return (false, 0);
    }
    let e_shoff = rd64(eh.as_ptr().add(0x28));
    let e_shentsize = rd16(eh.as_ptr().add(0x3a)) as usize;
    let e_shnum = rd16(eh.as_ptr().add(0x3c)) as usize;
    if e_shoff == 0 || e_shentsize < 64 || e_shnum == 0 || e_shnum > 4096 {
        crate::close(fd);
        return (false, 0);
    }
    let shtotal = e_shnum * e_shentsize;
    let sh = crate::malloc(shtotal) as *mut u8;
    if sh.is_null() { crate::close(fd); return (false, 0); }
    if crate::pread(fd, sh as *mut c_void, shtotal, e_shoff as i64) != shtotal as isize {
        crate::free(sh as *mut c_void);
        crate::close(fd);
        return (false, 0);
    }
    let mut best: u64 = 0;
    let mut have = false;
    for i in 0..e_shnum {
        let sec = sh.add(i * e_shentsize);
        let sh_type = rd32(sec.add(4));
        if sh_type != 2 && sh_type != 11 { continue; }
        let sh_offset = rd64(sec.add(0x18));
        let sh_size = rd64(sec.add(0x20)) as usize;
        let sh_link = rd32(sec.add(0x28)) as usize;
        let sh_entsize = rd64(sec.add(0x38)) as usize;
        if sh_size == 0 || sh_entsize < 24 || sh_link >= e_shnum { continue; }
        let syms = crate::malloc(sh_size) as *mut u8;
        if syms.is_null() { continue; }
        let got = crate::pread(fd, syms as *mut c_void, sh_size, sh_offset as i64);
        if got != sh_size as isize {
            crate::free(syms as *mut c_void);
            continue;
        }
        let strsec = sh.add(sh_link * e_shentsize);
        let str_off = rd64(strsec.add(0x18));
        let str_size = rd64(strsec.add(0x20)) as usize;
        if str_size == 0 || str_size > (1 << 24) {
            crate::free(syms as *mut c_void);
            continue;
        }
        let strs = crate::malloc(str_size) as *mut u8;
        if strs.is_null() {
            crate::free(syms as *mut c_void);
            continue;
        }
        if crate::pread(fd, strs as *mut c_void, str_size, str_off as i64) != str_size as isize {
            crate::free(strs as *mut c_void);
            crate::free(syms as *mut c_void);
            continue;
        }
        let count = sh_size / sh_entsize;
        for j in 0..count {
            let sym = syms.add(j * sh_entsize);
            let st_name = rd32(sym) as usize;
            let st_value = rd64(sym.add(8));
            let st_size = rd64(sym.add(16));
            let st_shndx = rd16(sym.add(6));
            if st_name == 0 || st_name >= str_size || st_shndx == 0 { continue; }
            if st_value > addr { continue; }
            if st_size != 0 && addr >= st_value + st_size { continue; }
            if !have || st_value > best {
                best = st_value;
                have = true;
                let s = strs.add(st_name);
                let mut k = 0usize;
                while *s.add(k) != 0 && k + 1 < DLADDR_NAME.len() {
                    DLADDR_NAME[k] = *s.add(k) as c_char;
                    k += 1;
                }
                DLADDR_NAME[k] = 0;
            }
        }
        crate::free(strs as *mut c_void);
        crate::free(syms as *mut c_void);
    }
    crate::free(sh as *mut c_void);
    crate::close(fd);
    (have, best)
}

#[no_mangle]
pub unsafe extern "C" fn dladdr(addr: *const c_void, info: *mut DlInfo) -> c_int {
    if info.is_null() { ERRNO = EINVAL; return 0; }
    (*info).dli_fname = core::ptr::null();
    (*info).dli_fbase = core::ptr::null_mut();
    (*info).dli_sname = core::ptr::null();
    (*info).dli_saddr = core::ptr::null_mut();
    if addr.is_null() { return 0; }
    let (start, off, path, found) = maps_lookup(addr as usize);
    if !found { return 0; }
    let base = if MAPS_PATH[0] != 0 && MAPS_PATH[0] != b'[' as c_char {
        elf_bias_for(path, start, off)
    } else {
        start.wrapping_sub(off)
    };
    (*info).dli_fname = path;
    (*info).dli_fbase = base as *mut c_void;
    if MAPS_PATH[0] == 0 || MAPS_PATH[0] == b'[' as c_char { return 1; }
    let off = addr as u64 - base;
    let (have, value) = elf_symbol_lookup(path, off);
    if have {
        (*info).dli_sname = DLADDR_NAME.as_ptr();
        (*info).dli_saddr = (base + value) as *mut c_void;
    }
    1
}

// ------------------------------------------------------------------
// dl_iterate_phdr
// ------------------------------------------------------------------

static mut EXE_PATH: [c_char; 512] = [0; 512];

unsafe fn read_exe_path() -> *const c_char {
    let n = crate::readlink(
        b"/proc/self/exe\0".as_ptr() as *const c_char,
        EXE_PATH.as_mut_ptr(),
        EXE_PATH.len() - 1,
    );
    if n < 0 {
        EXE_PATH[0] = 0;
    } else {
        EXE_PATH[n as usize] = 0;
    }
    EXE_PATH.as_ptr()
}

/// Returns (load bias, phdr pointer, phnum) for the main executable.
unsafe fn main_program_info() -> (u64, *const u8, u16) {
    let exe = read_exe_path();
    if *exe != 0 {
        let f = crate::fopen(b"/proc/self/maps\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
        if !f.is_null() {
            let mut line = [0 as c_char; 1024];
            while !crate::fgets(line.as_mut_ptr(), line.len() as c_int, f).is_null() {
                let p = line.as_ptr() as *const u8;
                let (start, after) = maps_parse_hex(p);
                if *after != b'-' { continue; }
                let (_end, after2) = maps_parse_hex(after.add(1));
                let mut q = after2;
                while *q == b' ' { q = q.add(1); }
                while *q != 0 && *q != b' ' { q = q.add(1); }
                while *q == b' ' { q = q.add(1); }
                let (off, after_off) = maps_parse_hex(q);
                if off != 0 { continue; }
                let mut q2 = after_off;
                for _ in 0..2 {
                    while *q2 == b' ' { q2 = q2.add(1); }
                    while *q2 != 0 && *q2 != b' ' { q2 = q2.add(1); }
                }
                while *q2 == b' ' { q2 = q2.add(1); }
                if *q2 == 0 { continue; }
                let mut path = [0 as c_char; 512];
                let mut i = 0usize;
                while *q2 != 0 && *q2 != b'\n' && i + 1 < path.len() {
                    path[i] = *q2 as c_char;
                    i += 1;
                    q2 = q2.add(1);
                }
                path[i] = 0;
                if crate::strcmp(path.as_ptr() as *const u8, exe as *const u8) != 0 { continue; }
                let base = start;
                let eh = base as *const u8;
                if *eh == 0x7f && *eh.add(1) == b'E' && *eh.add(2) == b'L' && *eh.add(3) == b'F' {
                    let e_phoff = rd64(eh.add(32));
                    let e_phentsize = rd16(eh.add(54));
                    let e_phnum = rd16(eh.add(56));
                    if e_phentsize as usize == 56 {
                        crate::fclose(f);
                        return (base, (base + e_phoff) as *const u8, e_phnum);
                    }
                }
                crate::fclose(f);
                return (base, core::ptr::null(), 0);
            }
            crate::fclose(f);
        }
    }
    let phdr = crate::getauxval(3);
    let phnum = crate::getauxval(5) as u16;
    if phdr != 0 && phnum != 0 {
        let (start, off, path, _f) = maps_lookup(phdr as usize);
        let base = elf_bias_for(path, start, off);
        return (base, phdr as *const u8, phnum);
    }
    (0, core::ptr::null(), 0)
}

#[no_mangle]
pub unsafe extern "C" fn dl_iterate_phdr(
    callback: Option<unsafe extern "C" fn(*mut DlPhdrInfo, usize, *mut c_void) -> c_int>,
    data: *mut c_void,
) -> c_int {
    let cb = match callback { Some(c) => c, None => return 0 };
    let (base, phdr, phnum) = main_program_info();
    if !phdr.is_null() && phnum != 0 {
        let info = DlPhdrInfo {
            dlpi_addr: base,
            dlpi_name: b"\0".as_ptr() as *const c_char,
            dlpi_phdr: phdr,
            dlpi_phnum: phnum,
            __pad0: 0,
            __pad1: 0,
            dlpi_adds: 0,
            dlpi_subs: 0,
            dlpi_tls_modid: 0,
            dlpi_tls_data: core::ptr::null_mut(),
        };
        let r = cb(&info as *const DlPhdrInfo as *mut DlPhdrInfo, core::mem::size_of::<DlPhdrInfo>(), data);
        if r != 0 { return r; }
    }
    if let Some(f) = LDSO_ITERATE {
        return f(Some(cb), data);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn dlinfo(handle: *mut c_void, request: c_int, info: *mut c_void) -> c_int {
    if let Some(f) = LDSO_DLINFO {
        return f(handle, request, info);
    }
    ERRNO = 95; // ENOTSUP
    -1
}
