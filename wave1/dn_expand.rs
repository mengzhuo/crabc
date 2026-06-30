// musl src/network/dn_expand.c
#[no_mangle]
pub unsafe extern "C" fn dn_expand(
    base: *const u8,
    end: *const u8,
    src: *const u8,
    dest: *mut c_char,
    space: c_int,
) -> c_int {
    let mut p = src;
    let dbegin = dest as *mut u8;
    let mut d = dbegin;
    let dend = d.add(if space > 254 { 254 } else { space as usize });
    let mut len: c_int = -1;
    let blen = (end as usize).wrapping_sub(base as usize);
    let mut i: usize = 0;
    while i < blen {
        if *p & 0xc0 != 0 {
            if p.add(1) == end {
                return -1;
            }
            let j = (((*p & 0x3f) as usize) << 8) | (*p.add(1) as usize);
            if len < 0 {
                len = ((p.add(2) as usize).wrapping_sub(src as usize)) as c_int;
            }
            if j >= blen {
                return -1;
            }
            p = base.add(j);
        } else if *p != 0 {
            if d != dbegin {
                *d = b'.';
                d = d.add(1);
            }
            let j = *p as usize;
            p = p.add(1);
            if j > (end as usize).wrapping_sub(p as usize)
                || j > (dend as usize).wrapping_sub(d as usize)
            {
                return -1;
            }
            core::ptr::copy_nonoverlapping(p, d, j);
            d = d.add(j);
            p = p.add(j);
        } else {
            *d = 0;
            if len < 0 {
                len = ((p.add(1) as usize).wrapping_sub(src as usize)) as c_int;
            }
            return len;
        }
        i += 2;
    }
    -1
}
