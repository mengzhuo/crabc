// ============================================================
// String/memory fast paths
// ============================================================
//
// These are hot enough that the generic byte loops in the rest of the libc
// were measurably slower than musl.  The x86_64 versions use SSE2 (part of
// the x86_64 baseline) and dispatch to AVX2 when the CPU and kernel support
// it; other architectures use word-at-a-time loops with an unaligned first
// block, which is what the musl C implementations compile to.

#[cfg(target_arch = "x86_64")]
mod fastmem_x86 {
    use core::sync::atomic::{AtomicU8, Ordering};

    const UNKNOWN: u8 = 0;
    const NO: u8 = 1;
    const YES: u8 = 2;

    static AVX2: AtomicU8 = AtomicU8::new(UNKNOWN);
    static ERMS: AtomicU8 = AtomicU8::new(UNKNOWN);

    #[inline]
    fn xgetbv0() -> u64 {
        let lo: u32;
        let hi: u32;
        unsafe {
            core::arch::asm!("xgetbv", in("ecx") 0u32, out("eax") lo, out("edx") hi,
                             options(nostack, nomem));
        }
        ((hi as u64) << 32) | lo as u64
    }

    pub fn avx2() -> bool {
        match AVX2.load(Ordering::Relaxed) {
            YES => return true,
            NO => return false,
            _ => {}
        }
        let ok = {
            let max = core::arch::x86_64::__cpuid(0).eax;
            if max < 7 {
                false
            } else {
                let c1 = core::arch::x86_64::__cpuid(1);
                let osxsave = c1.ecx & (1 << 27) != 0;
                let avx = c1.ecx & (1 << 28) != 0;
                osxsave
                    && avx
                    && (xgetbv0() & 0x6) == 0x6
                    && core::arch::x86_64::__cpuid_count(7, 0).ebx & (1 << 5) != 0
            }
        };
        AVX2.store(if ok { YES } else { NO }, Ordering::Relaxed);
        ok
    }

    pub fn erms() -> bool {
        match ERMS.load(Ordering::Relaxed) {
            YES => return true,
            NO => return false,
            _ => {}
        }
        let ok = {
            let max = core::arch::x86_64::__cpuid(0).eax;
            max >= 7 && core::arch::x86_64::__cpuid_count(7, 0).ebx & (1 << 9) != 0
        };
        ERMS.store(if ok { YES } else { NO }, Ordering::Relaxed);
        ok
    }
}

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

// ------------------------------------------------------------
// strlen
// ------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
unsafe fn strlen_sse2(s: *const u8) -> usize {
    let start = s as usize;
    let mut p = start & !15usize;
    let zero = _mm_setzero_si128();
    let first = _mm_load_si128(p as *const __m128i);
    let mut mask = _mm_movemask_epi8(_mm_cmpeq_epi8(first, zero)) as u32;
    mask &= !0u32 << (start - p);
    if mask != 0 {
        return p - start + mask.trailing_zeros() as usize;
    }
    p += 16;
    loop {
        let a = _mm_load_si128(p as *const __m128i);
        let ma = _mm_movemask_epi8(_mm_cmpeq_epi8(a, zero)) as u32;
        if ma != 0 {
            return p + ma.trailing_zeros() as usize - start;
        }
        let b = _mm_load_si128((p + 16) as *const __m128i);
        let mb = _mm_movemask_epi8(_mm_cmpeq_epi8(b, zero)) as u32;
        if mb != 0 {
            return p + 16 + mb.trailing_zeros() as usize - start;
        }
        p += 32;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn strlen_avx2(s: *const u8) -> usize {
    let start = s as usize;
    let mut p = start & !31usize;
    let zero = _mm256_setzero_si256();
    let first = _mm256_load_si256(p as *const __m256i);
    let mut mask = _mm256_movemask_epi8(_mm256_cmpeq_epi8(first, zero)) as u32;
    mask &= !0u32 << (start - p);
    if mask != 0 {
        return p - start + mask.trailing_zeros() as usize;
    }
    p += 32;
    loop {
        let a = _mm256_load_si256(p as *const __m256i);
        let ma = _mm256_movemask_epi8(_mm256_cmpeq_epi8(a, zero)) as u32;
        if ma != 0 {
            return p + ma.trailing_zeros() as usize - start;
        }
        let b = _mm256_load_si256((p + 32) as *const __m256i);
        let mb = _mm256_movemask_epi8(_mm256_cmpeq_epi8(b, zero)) as u32;
        if mb != 0 {
            return p + 32 + mb.trailing_zeros() as usize - start;
        }
        p += 64;
    }
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn strlen_words(s: *const u8) -> usize {
    let start = s as usize;
    let mut p = s;
    while (p as usize) & 7 != 0 {
        if *p == 0 {
            return p as usize - start;
        }
        p = p.add(1);
    }
    const ONES: u64 = 0x0101_0101_0101_0101;
    const HIGHS: u64 = 0x8080_8080_8080_8080;
    // Four words per branch: fewer loop-carried branches than the one-word
    // form, and the cold path still locates the first zero byte exactly.
    loop {
        let w0 = core::ptr::read(p as *const u64);
        let w1 = core::ptr::read(p.add(8) as *const u64);
        let w2 = core::ptr::read(p.add(16) as *const u64);
        let w3 = core::ptr::read(p.add(24) as *const u64);
        let z0 = w0.wrapping_sub(ONES) & !w0 & HIGHS;
        let z1 = w1.wrapping_sub(ONES) & !w1 & HIGHS;
        let z2 = w2.wrapping_sub(ONES) & !w2 & HIGHS;
        let z3 = w3.wrapping_sub(ONES) & !w3 & HIGHS;
        if z0 | z1 | z2 | z3 != 0 {
            let (z, base) = if z0 != 0 {
                (z0, 0usize)
            } else if z1 != 0 {
                (z1, 8)
            } else if z2 != 0 {
                (z2, 16)
            } else {
                (z3, 24)
            };
            return p as usize - start + base + (z.trailing_zeros() as usize / 8);
        }
        p = p.add(32);
    }
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> usize {
    #[cfg(target_arch = "x86_64")]
    {
        if fastmem_x86::avx2() {
            return strlen_avx2(s as *const u8);
        }
        strlen_sse2(s as *const u8)
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        strlen_words(s as *const u8)
    }
}

// ------------------------------------------------------------
// memcpy
// ------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn store16(dst: *mut u8, src: *const u8) {
    let v = _mm_loadu_si128(src as *const __m128i);
    _mm_storeu_si128(dst as *mut __m128i, v);
}

// RISC-V ABIs allow misaligned loads/stores, but LLVM lowers unaligned
// `u64` accesses to eight byte operations because the psABI does not
// guarantee them. Force a single `ld`/`sd`; the kernel emulates the rare
// case where the hardware cannot do it in one cycle.
#[cfg(target_arch = "riscv64")]
#[inline(always)]
unsafe fn load_u64(p: *const u8) -> u64 {
    let v: u64;
    core::arch::asm!("ld {}, 0({})", out(reg) v, in(reg) p, options(nostack));
    v
}

#[cfg(not(target_arch = "riscv64"))]
#[inline(always)]
unsafe fn load_u64(p: *const u8) -> u64 {
    core::ptr::read_unaligned(p as *const u64)
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
unsafe fn store_u64(p: *mut u8, v: u64) {
    core::arch::asm!("sd {}, 0({})", in(reg) v, in(reg) p, options(nostack));
}

#[cfg(not(target_arch = "riscv64"))]
#[inline(always)]
unsafe fn store_u64(p: *mut u8, v: u64) {
    core::ptr::write_unaligned(p as *mut u64, v);
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
unsafe fn copy32(d: *mut u8, s: *const u8) {
    core::arch::asm!(
        "ld t0, 0({s})", "ld t1, 8({s})", "ld t2, 16({s})", "ld t3, 24({s})",
        "sd t0, 0({d})", "sd t1, 8({d})", "sd t2, 16({d})", "sd t3, 24({d})",
        s = in(reg) s, d = in(reg) d,
        out("t0") _, out("t1") _, out("t2") _, out("t3") _,
        options(nostack),
    );
}

#[cfg(target_arch = "riscv64")]
#[inline(always)]
unsafe fn store32(d: *mut u8, v: u64) {
    core::arch::asm!(
        "sd {v}, 0({d})", "sd {v}, 8({d})", "sd {v}, 16({d})", "sd {v}, 24({d})",
        v = in(reg) v, d = in(reg) d,
        options(nostack),
    );
}

#[inline]
unsafe fn copy8(dst: *mut u8, src: *const u8) {
    store_u64(dst, load_u64(src));
}

#[cfg(target_arch = "x86_64")]
unsafe fn memcpy_sse2(dst: *mut u8, src: *const u8, n: usize) {
    if n >= 16 {
        let mut i = 0usize;
        while i + 16 <= n {
            store16(dst.add(i), src.add(i));
            i += 16;
        }
        if i < n {
            store16(dst.add(n - 16), src.add(n - 16));
        }
    } else if n >= 8 {
        copy8(dst, src);
        copy8(dst.add(n - 8), src.add(n - 8));
    } else if n >= 4 {
        core::ptr::write_unaligned(
            dst as *mut u32,
            core::ptr::read_unaligned(src as *const u32),
        );
        core::ptr::write_unaligned(
            dst.add(n - 4) as *mut u32,
            core::ptr::read_unaligned(src.add(n - 4) as *const u32),
        );
    } else if n >= 2 {
        core::ptr::write_unaligned(
            dst as *mut u16,
            core::ptr::read_unaligned(src as *const u16),
        );
        core::ptr::write_unaligned(
            dst.add(n - 2) as *mut u16,
            core::ptr::read_unaligned(src.add(n - 2) as *const u16),
        );
    } else if n == 1 {
        *dst = *src;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn memcpy_avx2(dst: *mut u8, src: *const u8, n: usize) {
    let mut i = 0usize;
    let head = (32 - (dst as usize & 31)) & 31;
    while i < head {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    while i + 128 <= n {
        let a = _mm256_loadu_si256(src.add(i) as *const __m256i);
        let b = _mm256_loadu_si256(src.add(i + 32) as *const __m256i);
        let c = _mm256_loadu_si256(src.add(i + 64) as *const __m256i);
        let d = _mm256_loadu_si256(src.add(i + 96) as *const __m256i);
        _mm256_store_si256(dst.add(i) as *mut __m256i, a);
        _mm256_store_si256(dst.add(i + 32) as *mut __m256i, b);
        _mm256_store_si256(dst.add(i + 64) as *mut __m256i, c);
        _mm256_store_si256(dst.add(i + 96) as *mut __m256i, d);
        i += 128;
    }
    while i + 32 <= n {
        let a = _mm256_loadu_si256(src.add(i) as *const __m256i);
        _mm256_store_si256(dst.add(i) as *mut __m256i, a);
        i += 32;
    }
    if i < n {
        let off = n - 32;
        let a = _mm256_loadu_si256(src.add(off) as *const __m256i);
        _mm256_storeu_si256(dst.add(off) as *mut __m256i, a);
    }
    _mm256_zeroupper();
}

#[cfg(target_arch = "x86_64")]
unsafe fn memcpy_rep(dst: *mut u8, src: *const u8, n: usize) {
    core::arch::asm!("rep movsb", inout("rdi") dst => _, inout("rsi") src => _,
                     inout("rcx") n => _, options(nostack, preserves_flags));
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn memcpy_words(dst: *mut u8, src: *const u8, n: usize) {
    let mut d = dst;
    let mut s = src;
    let all = dst.add(n);
    while d < all && (d as usize) & 7 != 0 {
        *d = *s;
        d = d.add(1);
        s = s.add(1);
    }
    let mut end = d.add((all as usize - d as usize) & !31);
    while d < end {
        #[cfg(target_arch = "riscv64")]
        copy32(d, s);
        #[cfg(not(target_arch = "riscv64"))]
        {
            copy8(d, s);
            copy8(d.add(8), s.add(8));
            copy8(d.add(16), s.add(16));
            copy8(d.add(24), s.add(24));
        }
        d = d.add(32);
        s = s.add(32);
    }
    end = d.add((all as usize - d as usize) & !7);
    while d < end {
        copy8(d, s);
        d = d.add(8);
        s = s.add(8);
    }
    while d < all {
        *d = *s;
        d = d.add(1);
        s = s.add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let d = dst as *mut u8;
    let s = src as *const u8;
    if n != 0 {
        #[cfg(target_arch = "x86_64")]
        {
            if n >= 128 {
                if fastmem_x86::avx2() {
                    memcpy_avx2(d, s, n);
                } else if fastmem_x86::erms() {
                    memcpy_rep(d, s, n);
                } else {
                    memcpy_sse2(d, s, n);
                }
            } else {
                memcpy_sse2(d, s, n);
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        memcpy_words(d, s, n);
    }
    dst
}

// ------------------------------------------------------------
// memset
// ------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
unsafe fn memset_sse2(dst: *mut u8, c: u8, n: usize) {
    let v16 = _mm_set1_epi8(c as i8);
    if n >= 16 {
        let mut i = 0usize;
        while i + 16 <= n {
            _mm_storeu_si128(dst.add(i) as *mut __m128i, v16);
            i += 16;
        }
        if i < n {
            _mm_storeu_si128(dst.add(n - 16) as *mut __m128i, v16);
        }
    } else if n >= 8 {
        let v64 = 0x0101_0101_0101_0101u64 * c as u64;
        core::ptr::write_unaligned(dst as *mut u64, v64);
        core::ptr::write_unaligned(dst.add(n - 8) as *mut u64, v64);
    } else if n >= 4 {
        let v32 = 0x0101_0101u32 * c as u32;
        core::ptr::write_unaligned(dst as *mut u32, v32);
        core::ptr::write_unaligned(dst.add(n - 4) as *mut u32, v32);
    } else if n >= 2 {
        let v16 = 0x0101u16 * c as u16;
        core::ptr::write_unaligned(dst as *mut u16, v16);
        core::ptr::write_unaligned(dst.add(n - 2) as *mut u16, v16);
    } else if n == 1 {
        *dst = c;
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn memset_avx2(dst: *mut u8, c: u8, n: usize) {
    let v = _mm256_set1_epi8(c as i8);
    let mut i = 0usize;
    let head = (32 - (dst as usize & 31)) & 31;
    while i < head {
        *dst.add(i) = c;
        i += 1;
    }
    while i + 128 <= n {
        _mm256_store_si256(dst.add(i) as *mut __m256i, v);
        _mm256_store_si256(dst.add(i + 32) as *mut __m256i, v);
        _mm256_store_si256(dst.add(i + 64) as *mut __m256i, v);
        _mm256_store_si256(dst.add(i + 96) as *mut __m256i, v);
        i += 128;
    }
    while i + 32 <= n {
        _mm256_store_si256(dst.add(i) as *mut __m256i, v);
        i += 32;
    }
    if i < n {
        _mm256_storeu_si256(dst.add(n - 32) as *mut __m256i, v);
    }
    _mm256_zeroupper();
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn memset_words(dst: *mut u8, c: u8, n: usize) {
    let word = 0x0101_0101_0101_0101u64 * c as u64;
    let all = dst.add(n);
    let mut d = dst;
    while d < all && (d as usize) & 7 != 0 {
        *d = c;
        d = d.add(1);
    }
    let mut end = d.add((all as usize - d as usize) & !31);
    while d < end {
        #[cfg(target_arch = "riscv64")]
        store32(d, word);
        #[cfg(not(target_arch = "riscv64"))]
        {
            store_u64(d, word);
            store_u64(d.add(8), word);
            store_u64(d.add(16), word);
            store_u64(d.add(24), word);
        }
        d = d.add(32);
    }
    end = d.add((all as usize - d as usize) & !7);
    while d < end {
        store_u64(d, word);
        d = d.add(8);
    }
    while d < all {
        *d = c;
        d = d.add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut c_void, c: c_int, n: usize) -> *mut c_void {
    let d = s as *mut u8;
    let b = c as u8;
    if n != 0 {
        #[cfg(target_arch = "x86_64")]
        {
            if n >= 128 && fastmem_x86::avx2() {
                memset_avx2(d, b, n);
            } else {
                memset_sse2(d, b, n);
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        memset_words(d, b, n);
    }
    s
}

// ------------------------------------------------------------
// memmove
// ------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let d = dst as *mut u8;
    let s = src as *const u8;
    let du = d as usize;
    let su = s as usize;
    if n == 0 || du == su {
        return dst;
    }
    if du < su {
        if du + n <= su {
            return memcpy(dst, src, n);
        }
        // Overlapping, moving down: copy forwards. A chunk may only be as
        // wide as the overlap, otherwise its store would clobber the source
        // of the next chunk.
        let delta = su - du;
        let mut i = 0usize;
        if delta >= 32 {
            while i + 32 <= n {
                copy8(d.add(i), s.add(i));
                copy8(d.add(i + 8), s.add(i + 8));
                copy8(d.add(i + 16), s.add(i + 16));
                copy8(d.add(i + 24), s.add(i + 24));
                i += 32;
            }
        }
        if delta >= 8 {
            while i + 8 <= n {
                copy8(d.add(i), s.add(i));
                i += 8;
            }
        }
        while i < n {
            *d.add(i) = *s.add(i);
            i += 1;
        }
    } else {
        if su + n <= du {
            return memcpy(dst, src, n);
        }
        // Overlapping, moving up: copy backwards. The sub-chunks inside a
        // block must also go high to low so a store cannot clobber bytes the
        // same block has not read yet.
        let mut i = n;
        while i >= 32 {
            i -= 32;
            copy8(d.add(i + 24), s.add(i + 24));
            copy8(d.add(i + 16), s.add(i + 16));
            copy8(d.add(i + 8), s.add(i + 8));
            copy8(d.add(i), s.add(i));
        }
        while i >= 8 {
            i -= 8;
            copy8(d.add(i), s.add(i));
        }
        while i > 0 {
            i -= 1;
            *d.add(i) = *s.add(i);
        }
    }
    dst
}

// ------------------------------------------------------------
// memcmp
// ------------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn memcmp_avx2(a: *const u8, b: *const u8, n: usize) -> c_int {
    let mut i = 0usize;
    while i + 32 <= n {
        let va = _mm256_loadu_si256(a.add(i) as *const __m256i);
        let vb = _mm256_loadu_si256(b.add(i) as *const __m256i);
        let mask = _mm256_movemask_epi8(_mm256_cmpeq_epi8(va, vb)) as u32;
        if mask != u32::MAX {
            let k = (!mask).trailing_zeros() as usize;
            _mm256_zeroupper();
            return *a.add(i + k) as c_int - *b.add(i + k) as c_int;
        }
        i += 32;
    }
    _mm256_zeroupper();
    while i + 8 <= n {
        let wa = load_u64(a.add(i));
        let wb = load_u64(b.add(i));
        if wa != wb {
            break;
        }
        i += 8;
    }
    while i < n {
        let x = *a.add(i);
        let y = *b.add(i);
        if x != y {
            return x as c_int - y as c_int;
        }
        i += 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> c_int {
    let a = s1 as *const u8;
    let b = s2 as *const u8;
    if n == 0 {
        return 0;
    }
    // Most unequal comparisons differ early; resolve that without setting up
    // a vector/word loop.
    if *a != *b {
        return *a as c_int - *b as c_int;
    }
    #[cfg(target_arch = "x86_64")]
    {
        if n >= 32 && fastmem_x86::avx2() {
            return memcmp_avx2(a, b, n);
        }
    }
    let mut i = 0usize;
    while i + 8 <= n {
        let wa = core::ptr::read_unaligned(a.add(i) as *const u64);
        let wb = core::ptr::read_unaligned(b.add(i) as *const u64);
        if wa != wb {
            break;
        }
        i += 8;
    }
    while i < n {
        let x = *a.add(i);
        let y = *b.add(i);
        if x != y {
            return x as c_int - y as c_int;
        }
        i += 1;
    }
    0
}

// ------------------------------------------------------------
// strcmp
// ------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> c_int {
    let mut a = s1;
    let mut b = s2;
    const ONES: u64 = 0x0101_0101_0101_0101;
    const HIGHS: u64 = 0x8080_8080_8080_8080;
    loop {
        // Walk to the next page end (or as far as the shorter string allows)
        // with a tight word loop; the room computation runs once per page,
        // not once per word.
        let room_a = 4096 - ((a as usize) & 4095);
        let room_b = 4096 - ((b as usize) & 4095);
        let room = if room_a < room_b { room_a } else { room_b };
        let words_end = a.add(room & !7usize);
        while a < words_end {
            let wa = load_u64(a);
            let wb = load_u64(b);
            if wa == wb {
                if wa.wrapping_sub(ONES) & !wa & HIGHS != 0 {
                    return 0;
                }
                a = a.add(8);
                b = b.add(8);
                continue;
            }
            let mut i = 0usize;
            while i < 8 {
                let x = *a.add(i);
                let y = *b.add(i);
                if x != y {
                    return x as c_int - y as c_int;
                }
                if x == 0 {
                    return 0;
                }
                i += 1;
            }
        }
        let tail = room & 7;
        let mut i = 0usize;
        while i < tail {
            let x = *a;
            let y = *b;
            if x != y {
                return x as c_int - y as c_int;
            }
            if x == 0 {
                return 0;
            }
            a = a.add(1);
            b = b.add(1);
            i += 1;
        }
    }
}
