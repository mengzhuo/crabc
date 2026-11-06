// fopencookie, open_wmemstream, strfmon and getdate.

// ============================================================
// fopencookie
// ============================================================

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CookieIo {
    pub read: usize,
    pub write: usize,
    pub seek: usize,
    pub close: usize,
}

#[repr(C)]
struct CookieFile {
    cookie: *mut c_void,
    io: CookieIo,
}

type CookieReadFn = unsafe extern "C" fn(*mut c_void, *mut c_char, usize) -> isize;
type CookieWriteFn = unsafe extern "C" fn(*mut c_void, *const c_char, usize) -> isize;
type CookieSeekFn = unsafe extern "C" fn(*mut c_void, *mut i64, c_int) -> c_int;
type CookieCloseFn = unsafe extern "C" fn(*mut c_void) -> c_int;

unsafe extern "C" fn cookie_read(f: *mut FILE, buf: *mut u8, len: usize) -> usize {
    let cf = (*f).cookie as *mut CookieFile;
    if cf.is_null() || (*cf).io.read == 0 {
        (*f).flags |= F_ERR;
        (*f)._err = 1;
        return 0;
    }
    let func: CookieReadFn = core::mem::transmute((*cf).io.read);
    let n = func((*cf).cookie, buf as *mut c_char, len);
    if n <= 0 {
        if n == 0 {
            (*f).flags |= F_EOF;
        } else {
            (*f).flags |= F_ERR;
            (*f)._err = 1;
        }
        return 0;
    }
    n as usize
}

unsafe extern "C" fn cookie_write(f: *mut FILE, buf: *const u8, len: usize) -> usize {
    let cf = (*f).cookie as *mut CookieFile;
    if cf.is_null() || (*cf).io.write == 0 { return len; }
    let func: CookieWriteFn = core::mem::transmute((*cf).io.write);
    let buffered = if (*f).wbase.is_null() { 0 } else { (*f).wpos as usize - (*f).wbase as usize };
    if buffered > 0 {
        let n = func((*cf).cookie, (*f).wbase as *const c_char, buffered);
        if n != buffered as isize {
            (*f).wpos = core::ptr::null_mut();
            (*f).wbase = core::ptr::null_mut();
            (*f).wend = core::ptr::null_mut();
            (*f).flags |= F_ERR;
            (*f).flags |= F_ERR;
            return 0;
        }
        (*f).wbase = (*f).buf;
        (*f).wpos = (*f).buf;
        (*f).wend = (*f).buf.add((*f).buf_size);
    }
    if len > 0 {
        let n = func((*cf).cookie, buf as *const c_char, len);
        if n < 0 { (*f).flags |= F_ERR; return 0; }
        return n as usize;
    }
    0
}

unsafe extern "C" fn cookie_seek(f: *mut FILE, off: i64, whence: c_int) -> i64 {
    let cf = (*f).cookie as *mut CookieFile;
    if cf.is_null() || (*cf).io.seek == 0 {
        ERRNO!() = 95; // ENOTSUP
        return -1;
    }
    if whence < 0 || whence > 2 {
        ERRNO!() = EINVAL;
        return -1;
    }
    let func: CookieSeekFn = core::mem::transmute((*cf).io.seek);
    let mut o = off;
    let r = func((*cf).cookie, &mut o as *mut i64, whence);
    if r < 0 { return -1; }
    o
}

unsafe extern "C" fn cookie_close(f: *mut FILE) -> c_int {
    let cf = (*f).cookie as *mut CookieFile;
    if cf.is_null() { return 0; }
    let mut r = 0;
    if (*cf).io.close != 0 {
        let func: CookieCloseFn = core::mem::transmute((*cf).io.close);
        r = func((*cf).cookie);
    }
    crate::free(cf as *mut c_void);
    (*f).cookie = core::ptr::null_mut();
    r
}

#[no_mangle]
pub unsafe extern "C" fn fopencookie(cookie: *mut c_void, mode: *const c_char, io_funcs: CookieIo) -> *mut FILE {
    if mode.is_null() { ERRNO!() = EINVAL; return core::ptr::null_mut(); }
    let m = *mode;
    if m != b'r' as c_char && m != b'w' as c_char && m != b'a' as c_char {
        ERRNO!() = EINVAL;
        return core::ptr::null_mut();
    }
    let f = crate::calloc(1, core::mem::size_of::<FILE>() + UNGET + BUFSIZ) as *mut FILE;
    if f.is_null() { ERRNO!() = ENOMEM; return core::ptr::null_mut(); }
    let cf = crate::calloc(1, core::mem::size_of::<CookieFile>()) as *mut CookieFile;
    if cf.is_null() { crate::free(f as *mut c_void); ERRNO!() = ENOMEM; return core::ptr::null_mut(); }
    (*cf).cookie = cookie;
    (*cf).io = io_funcs;

    (*f).fd = -1;
    (*f).cookie = cf as *mut c_void;
    (*f).buf = buf_ptr(f);
    (*f).buf_size = BUFSIZ;
    (*f).lbf = -1;
    (*f).read_fn = Some(cookie_read);
    (*f).write_fn = Some(cookie_write);
    (*f).seek_fn = Some(cookie_seek);
    (*f).close = Some(cookie_close);
    if !crate::strchr(mode as *const u8, b'+' as c_int).is_null() {
        (*f).flags = 0;
    } else if m == b'r' as c_char {
        (*f).flags = F_NOWR;
    } else {
        (*f).flags = F_NORD;
    }
    if m == b'a' as c_char { (*f).flags |= F_APP; }
    f
}

// ============================================================
// open_wmemstream
// ============================================================

#[repr(C)]
struct WmsCookie {
    bufp: *mut *mut wchar_t,
    sizep: *mut usize,
    pos: usize,
    buf: *mut wchar_t,
    len: usize,
    space: usize,
    // Incremental UTF-8 decoder.
    need: u8,
    have: u8,
    cp: u32,
}

unsafe fn wms_emit(c: *mut WmsCookie, wc: u32) {
    if (*c).pos >= (*c).space {
        let len2 = ((*c).space * 2 + 1) | ((*c).pos + 1);
        if len2 > (isize::MAX as usize) / 4 { return; }
        let newbuf = crate::realloc((*c).buf as *mut c_void, len2 * 4) as *mut wchar_t;
        if newbuf.is_null() { ERRNO!() = ENOMEM; return; }
        core::ptr::write_bytes((newbuf as *mut u8).add((*c).space * 4), 0, (len2 - (*c).space) * 4);
        (*c).buf = newbuf;
        (*c).space = len2;
        if !(*c).bufp.is_null() { *(*c).bufp = newbuf; }
    }
    *(*c).buf.add((*c).pos) = wc as wchar_t;
    (*c).pos += 1;
    if (*c).pos > (*c).len { (*c).len = (*c).pos; }
    *(*c).buf.add((*c).pos) = 0;
    if !(*c).sizep.is_null() { *(*c).sizep = (*c).pos; }
}

unsafe extern "C" fn wms_write(f: *mut FILE, buf: *const u8, len: usize) -> usize {
    let c = (*f).cookie as *mut WmsCookie;
    if c.is_null() { return 0; }
    for i in 0..len {
        let b = *buf.add(i);
        if (*c).need == 0 {
            if b < 0x80 {
                wms_emit(c, b as u32);
            } else if b >= 0xc2 && b <= 0xdf {
                (*c).need = 1;
                (*c).have = 0;
                (*c).cp = (b & 0x1f) as u32;
            } else if b >= 0xe0 && b <= 0xef {
                (*c).need = 2;
                (*c).have = 0;
                (*c).cp = (b & 0x0f) as u32;
            } else if b >= 0xf0 && b <= 0xf4 {
                (*c).need = 3;
                (*c).have = 0;
                (*c).cp = (b & 0x07) as u32;
            } else {
                wms_emit(c, 0xfffd);
            }
        } else if b & 0xc0 == 0x80 {
            (*c).cp = ((*c).cp << 6) | (b & 0x3f) as u32;
            (*c).have += 1;
            if (*c).have == (*c).need {
                wms_emit(c, (*c).cp);
                (*c).need = 0;
                (*c).have = 0;
                (*c).cp = 0;
            }
        } else {
            // Invalid continuation: emit a replacement and treat this byte as
            // the start of a new sequence.
            wms_emit(c, 0xfffd);
            (*c).need = 0;
            (*c).have = 0;
            (*c).cp = 0;
            if b < 0x80 {
                wms_emit(c, b as u32);
            } else if b >= 0xc2 && b <= 0xdf {
                (*c).need = 1;
                (*c).cp = (b & 0x1f) as u32;
            } else if b >= 0xe0 && b <= 0xef {
                (*c).need = 2;
                (*c).cp = (b & 0x0f) as u32;
            } else if b >= 0xf0 && b <= 0xf4 {
                (*c).need = 3;
                (*c).cp = (b & 0x07) as u32;
            } else {
                wms_emit(c, 0xfffd);
            }
        }
    }
    len
}

unsafe extern "C" fn wms_seek(f: *mut FILE, off: i64, whence: c_int) -> i64 {
    let c = (*f).cookie as *mut WmsCookie;
    if c.is_null() || whence < 0 || whence > 2 {
        ERRNO!() = EINVAL;
        return -1;
    }
    let base: i64 = match whence {
        0 => 0,
        1 => (*c).pos as i64,
        _ => (*c).len as i64,
    };
    if off < -base || off > (isize::MAX as i64) / 4 - base {
        ERRNO!() = EINVAL;
        return -1;
    }
    (*c).pos = (base + off) as usize;
    (*c).need = 0;
    (*c).have = 0;
    (*c).cp = 0;
    (*c).pos as i64
}

unsafe extern "C" fn wms_close(_f: *mut FILE) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn open_wmemstream(bufp: *mut *mut wchar_t, sizep: *mut usize) -> *mut FILE {
    if bufp.is_null() || sizep.is_null() { ERRNO!() = EINVAL; return core::ptr::null_mut(); }
    let f = crate::calloc(1, core::mem::size_of::<FILE>() + 1) as *mut FILE;
    if f.is_null() { ERRNO!() = ENOMEM; return core::ptr::null_mut(); }
    let c = crate::calloc(1, core::mem::size_of::<WmsCookie>()) as *mut WmsCookie;
    if c.is_null() { crate::free(f as *mut c_void); ERRNO!() = ENOMEM; return core::ptr::null_mut(); }
    let buf = crate::calloc(1, 4) as *mut wchar_t;
    if buf.is_null() {
        crate::free(c as *mut c_void);
        crate::free(f as *mut c_void);
        ERRNO!() = ENOMEM;
        return core::ptr::null_mut();
    }
    (*c).bufp = bufp;
    (*c).sizep = sizep;
    (*c).buf = buf;
    (*c).space = 1;
    *bufp = buf;
    *sizep = 0;
    (*f).fd = -1;
    (*f).cookie = c as *mut c_void;
    let base = (f as *mut u8).add(core::mem::size_of::<FILE>());
    (*f).buf = base;
    (*f).buf_size = 0;
    (*f).lbf = -1;
    (*f).flags = F_NORD;
    (*f).write_fn = Some(wms_write);
    (*f).seek_fn = Some(wms_seek);
    (*f).close = Some(wms_close);
    (*f).mode = 1;
    f
}

// ============================================================
// strfmon / strfmon_l
// ============================================================

unsafe fn strfmon_impl(mut s: *mut c_char, n: usize, fmt: *const c_char, mut ap: VaList) -> isize {
    let start = s;
    let mut cap = n;
    let mut f = fmt;
    while cap > 0 && *f != 0 {
        if *f as u8 != b'%' {
            *s = *f;
            s = s.add(1);
            f = f.add(1);
            cap -= 1;
            continue;
        }
        f = f.add(1);
        if *f as u8 == b'%' {
            *s = *f;
            s = s.add(1);
            f = f.add(1);
            cap -= 1;
            continue;
        }
        let mut fill = b' ' as c_char;
        let mut nogrp = true; // C locale has no digit grouping
        let mut negpar = false;
        let mut nosym = false;
        let mut left = false;
        let mut plus = false;
        loop {
            match *f as u8 {
                b'=' => { f = f.add(1); fill = *f; f = f.add(1); }
                b'^' => { nogrp = true; f = f.add(1); }
                b'(' => { negpar = true; f = f.add(1); }
                b'+' => { plus = true; f = f.add(1); }
                b'!' => { nosym = true; f = f.add(1); }
                b'-' => { left = true; f = f.add(1); }
                _ => break,
            }
        }
        let mut fw = 0usize;
        while *f as u8 >= b'0' && *f as u8 <= b'9' {
            fw = fw * 10 + (*f as u8 - b'0') as usize;
            f = f.add(1);
        }
        let mut lp = 1usize;
        if *f as u8 == b'#' {
            f = f.add(1);
            lp = 0;
            while *f as u8 >= b'0' && *f as u8 <= b'9' {
                lp = lp * 10 + (*f as u8 - b'0') as usize;
                f = f.add(1);
            }
            if lp == 0 { lp = 1; }
        }
        let mut rp = 2usize;
        if *f as u8 == b'.' {
            rp = 0;
            f = f.add(1);
            while *f as u8 >= b'0' && *f as u8 <= b'9' {
                rp = rp * 10 + (*f as u8 - b'0') as usize;
                f = f.add(1);
            }
        }
        let _intl = *f as u8 == b'i';
        if *f != 0 { f = f.add(1); }
        let x = ap.next_arg::<f64>();
        let neg = x < 0.0;
        let ax = if neg { -x } else { x };

        // Render the absolute number without width.
        let mut num = [0 as c_char; 256];
        let rn = crate::snprintf(
            num.as_mut_ptr(),
            num.len(),
            b"%.*f\0".as_ptr() as *const c_char,
            rp as c_int,
            ax,
        );
        let mut nlen = core::cmp::min(rn.max(0) as usize, num.len() - 1);

        // Split integer/fraction parts.
        let mut dot = nlen;
        for i in 0..nlen {
            if num[i] as u8 == b'.' { dot = i; break; }
        }
        let mut int_digits = dot;
        // Minimum integer digits requested with '#<n>'.
        let zero_pad = if int_digits < lp { lp - int_digits } else { 0 };

        // Group integer digits (only when explicitly enabled; C locale has none).
        let mut grouped = [0 as c_char; 320];
        let mut glen = 0usize;
        if !nogrp && int_digits > 3 {
            let first = int_digits % 3;
            let mut i = 0usize;
            while i < int_digits {
                if i > 0 && (i - first) % 3 == 0 { grouped[glen] = b',' as c_char; glen += 1; }
                grouped[glen] = num[i]; glen += 1; i += 1;
            }
        } else {
            for i in 0..int_digits { grouped[glen] = num[i]; glen += 1; }
        }
        let _ = &mut int_digits;
        let _ = &mut nlen;

        // Compose sign/parens + zeros + grouped integer + fraction.
        let mut core = [0 as c_char; 640];
        let mut clen = 0usize;
        let parens = negpar && neg;
        if parens { core[clen] = b'(' as c_char; clen += 1; }
        else if neg { core[clen] = b'-' as c_char; clen += 1; }
        else if plus { core[clen] = b'+' as c_char; clen += 1; }
        for _ in 0..zero_pad {
            core[clen] = b'0' as c_char; clen += 1;
        }
        for i in 0..glen { core[clen] = grouped[i]; clen += 1; }
        for i in dot..nlen { core[clen] = num[i]; clen += 1; }
        if parens { core[clen] = b')' as c_char; clen += 1; }

        let width = if fw > clen { fw } else { clen };
        let pad = width - clen;
        if left {
            for i in 0..clen {
                if cap <= 1 { break; }
                *s = core[i]; s = s.add(1); cap -= 1;
            }
            for _ in 0..pad {
                if cap <= 1 { break; }
                *s = b' ' as c_char; s = s.add(1); cap -= 1;
            }
        } else {
            for _ in 0..pad {
                if cap <= 1 { break; }
                *s = fill; s = s.add(1); cap -= 1;
            }
            for i in 0..clen {
                if cap <= 1 { break; }
                *s = core[i]; s = s.add(1); cap -= 1;
            }
        }
        let _ = nosym;
    }
    *s = 0;
    (s as usize - start as usize) as isize
}

#[no_mangle]
pub unsafe extern "C" fn strfmon(s: *mut c_char, n: usize, fmt: *const c_char, args: ...) -> isize {
    strfmon_impl(s, n, fmt, args)
}

#[no_mangle]
pub unsafe extern "C" fn strfmon_l(
    s: *mut c_char,
    n: usize,
    _loc: locale_t,
    fmt: *const c_char,
    args: ...,
) -> isize {
    strfmon_impl(s, n, fmt, args)
}

// ============================================================
// getdate
// ============================================================

#[no_mangle]
pub static mut getdate_err: c_int = 0;

static mut GETDATE_TM: crate::tm = crate::tm {
    tm_sec: 0,
    tm_min: 0,
    tm_hour: 0,
    tm_mday: 0,
    tm_mon: 0,
    tm_year: 0,
    tm_wday: 0,
    tm_yday: 0,
    tm_isdst: 0,
    tm_gmtoff: 0,
    tm_zone: core::ptr::null(),
};

#[no_mangle]
pub unsafe extern "C" fn getdate(s: *const c_char) -> *mut crate::tm {
    if s.is_null() {
        getdate_err = 1;
        return core::ptr::null_mut();
    }
    let datemsk = crate::getenv(b"DATEMSK\0".as_ptr() as *const c_char);
    if datemsk.is_null() {
        getdate_err = 1;
        return core::ptr::null_mut();
    }
    let f = crate::fopen(datemsk, b"r\0".as_ptr() as *const c_char);
    if f.is_null() {
        getdate_err = if ERRNO!() == ENOMEM { 6 } else { 2 };
        return core::ptr::null_mut();
    }
    let mut fmt = [0 as c_char; 128];
    let mut found = false;
    while !crate::fgets(fmt.as_mut_ptr(), fmt.len() as c_int, f).is_null() {
        let mut end: *mut c_char = core::ptr::null_mut();
        let p = crate::strptime(s, fmt.as_ptr(), &raw mut GETDATE_TM);
        if !p.is_null() && *p == 0 {
            found = true;
            break;
        }
        let _ = &mut end;
    }
    if !found && crate::ferror(f) != 0 { getdate_err = 5; }
    crate::fclose(f);
    if found {
        getdate_err = 0;
        return &raw mut GETDATE_TM;
    }
    if getdate_err == 0 { getdate_err = 7; }
    core::ptr::null_mut()
}
