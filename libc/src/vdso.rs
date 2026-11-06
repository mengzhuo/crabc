// ============================================================
// vDSO clock_gettime
// ============================================================
//
// musl resolves __vdso_clock_gettime from the ELF image the kernel passes in
// AT_SYSINFO_EHDR and calls it directly; the syscall is only a fallback.
// Without this, every clock_gettime costs a ring transition and timer-heavy
// programs are an order of magnitude slower than on musl.

const AT_SYSINFO_EHDR: c_ulong = 33;

const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const DT_HASH: u64 = 4;
const DT_STRTAB: u64 = 5;
const DT_SYMTAB: u64 = 6;

const STT_FUNC: u8 = 2;
const STB_GLOBAL: u8 = 1;
const STB_WEAK: u8 = 2;

// usize::MAX means "not resolved yet"; 0 means "no vDSO".
static CLOCK_GETTIME_VDSO: AtomicUsize = AtomicUsize::new(usize::MAX);

#[inline]
unsafe fn read_u16(p: *const u8) -> u16 {
    u16::from_le(core::ptr::read_unaligned(p as *const u16))
}

#[inline]
unsafe fn read_u32(p: *const u8) -> u32 {
    u32::from_le(core::ptr::read_unaligned(p as *const u32))
}

#[inline]
unsafe fn read_u64(p: *const u8) -> u64 {
    u64::from_le(core::ptr::read_unaligned(p as *const u64))
}

unsafe fn cstr_eq(s: *const u8, want: &[u8]) -> bool {
    let mut i = 0usize;
    while i < want.len() {
        if *s.add(i) != want[i] {
            return false;
        }
        i += 1;
    }
    *s.add(want.len()) == 0
}

/// Look up a symbol in the vDSO by name.  Versions are ignored: the vDSO
/// exports exactly one definition of each entry point.
unsafe fn vdso_lookup(want: &[u8]) -> usize {
    let eh = crate::getauxval(AT_SYSINFO_EHDR) as usize;
    if eh == 0 {
        return 0;
    }
    let image = eh as *const u8;
    if read_u32(image) != 0x464c_457f || *image.add(4) != 2 {
        return 0; // not a 64-bit ELF image
    }
    let phoff = read_u64(image.add(32)) as usize;
    let phentsize = read_u16(image.add(54)) as usize;
    let phnum = read_u16(image.add(56)) as usize;

    let mut load_base = usize::MAX;
    let mut dynamic = 0usize;
    let mut i = 0usize;
    while i < phnum {
        let ph = image.add(phoff + i * phentsize);
        match read_u32(ph) {
            PT_LOAD => {
                let off = read_u64(ph.add(8)) as usize;
                let vaddr = read_u64(ph.add(16)) as usize;
                load_base = eh.wrapping_add(off).wrapping_sub(vaddr);
            }
            PT_DYNAMIC => {
                dynamic = eh.wrapping_add(read_u64(ph.add(8)) as usize);
            }
            _ => {}
        }
        i += 1;
    }
    if load_base == usize::MAX || dynamic == 0 {
        return 0;
    }

    let mut strtab = 0usize;
    let mut symtab = 0usize;
    let mut hash = 0usize;
    let mut dp = dynamic as *const u8;
    loop {
        let tag = read_u64(dp);
        if tag == 0 {
            break;
        }
        let val = read_u64(dp.add(8)) as usize;
        match tag {
            DT_STRTAB => strtab = load_base.wrapping_add(val),
            DT_SYMTAB => symtab = load_base.wrapping_add(val),
            DT_HASH => hash = load_base.wrapping_add(val),
            _ => {}
        }
        dp = dp.add(16);
    }
    if strtab == 0 || symtab == 0 || hash == 0 {
        return 0;
    }

    // DT_HASH starts with nbucket/nchain; the vDSO always carries one.
    let nsym = read_u32((hash + 4) as *const u8) as usize;
    let mut i = 1usize;
    while i < nsym {
        let sym = (symtab + i * 24) as *const u8;
        let info = *sym.add(4);
        if info & 0xf != STT_FUNC {
            i += 1;
            continue;
        }
        let bind = info >> 4;
        if bind != STB_GLOBAL && bind != STB_WEAK {
            i += 1;
            continue;
        }
        if read_u16(sym.add(6)) == 0 {
            i += 1;
            continue;
        }
        let name = strtab.wrapping_add(read_u32(sym) as usize);
        if cstr_eq(name as *const u8, want) {
            return load_base.wrapping_add(read_u64(sym.add(8)) as usize);
        }
        i += 1;
    }
    0
}

#[inline]
unsafe fn vdso_clock_gettime() -> usize {
    let p = CLOCK_GETTIME_VDSO.load(Ordering::Acquire);
    if p != usize::MAX {
        return p;
    }
    let p = vdso_lookup(b"__vdso_clock_gettime");
    CLOCK_GETTIME_VDSO.store(p, Ordering::Release);
    p
}

#[no_mangle]
pub unsafe extern "C" fn clock_gettime(clockid: c_int, ts: *mut timespec) -> c_int {
    let p = vdso_clock_gettime();
    if p != 0 {
        let f: unsafe extern "C" fn(c_int, *mut timespec) -> c_int = core::mem::transmute(p);
        let r = f(clockid, ts);
        if r == 0 {
            return 0;
        }
        if r == -EINVAL {
            crate::ERRNO = EINVAL;
            return -1;
        }
        // Unsupported clock or an old vDSO: fall back to the syscall.
    }
    let r = sys_clock_gettime(clockid, ts);
    if r < 0 {
        crate::ERRNO = (-r) as c_int;
        -1
    } else {
        0
    }
}
