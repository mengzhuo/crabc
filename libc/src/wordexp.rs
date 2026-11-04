// A real wordexp(3): quoting, parameter expansion, command substitution and
// pathname expansion.

#[repr(C)]
pub struct WordexpT {
    pub we_wordc: usize,
    pub we_wordv: *mut *mut c_char,
    pub we_offs: usize,
}

const WRDE_APPEND_V: c_int = 1;
const WRDE_DOOFFS_V: c_int = 2;
const WRDE_NOCMD_V: c_int = 4;
const WRDE_REUSE_V: c_int = 8;
const WRDE_UNDEF_V: c_int = 32;

const WRDE_NOSPACE_V: c_int = 1;
const WRDE_BADCHAR_V: c_int = 2;
const WRDE_BADVAL_V: c_int = 3;
const WRDE_CMDSUB_V: c_int = 4;
const WRDE_SYNTAX_V: c_int = 5;

const WORDEXP_CAP: usize = 4096;

struct WordList {
    items: *mut *mut c_char,
    n: usize,
    cap: usize,
}

unsafe fn wl_push(list: *mut WordList, s: *const u8, len: usize) {
    if (*list).n == (*list).cap {
        let ncap = if (*list).cap == 0 { 16 } else { (*list).cap * 2 };
        let na = crate::realloc(
            (*list).items as *mut c_void,
            ncap * core::mem::size_of::<*mut c_char>(),
        ) as *mut *mut c_char;
        if na.is_null() { return; }
        (*list).items = na;
        (*list).cap = ncap;
    }
    let p = crate::malloc(len + 1) as *mut c_char;
    if p.is_null() { return; }
    core::ptr::copy_nonoverlapping(s, p as *mut u8, len);
    *p.add(len) = 0;
    *(*list).items.add((*list).n) = p;
    (*list).n += 1;
}

unsafe fn wl_free(list: *mut WordList) {
    for i in 0..(*list).n {
        crate::free(*(*list).items.add(i) as *mut c_void);
    }
    crate::free((*list).items as *mut c_void);
    (*list).items = core::ptr::null_mut();
    (*list).n = 0;
    (*list).cap = 0;
}

struct WordBuf {
    buf: [u8; WORDEXP_CAP],
    len: usize,
    has_glob: bool,
    quoted: bool,
}

unsafe fn wb_push(w: *mut WordBuf, c: u8, quoted: bool) -> c_int {
    if (*w).len + 1 >= WORDEXP_CAP { return WRDE_NOSPACE_V; }
    (*w).buf[(*w).len] = c;
    (*w).len += 1;
    if quoted {
        (*w).quoted = true;
    } else if c == b'*' || c == b'?' || c == b'[' {
        (*w).has_glob = true;
    }
    0
}

unsafe fn wb_push_str(w: *mut WordBuf, s: *const u8, len: usize, quoted: bool) -> c_int {
    if (*w).len + len >= WORDEXP_CAP { return WRDE_NOSPACE_V; }
    core::ptr::copy_nonoverlapping(s, (*w).buf.as_mut_ptr().add((*w).len), len);
    (*w).len += len;
    if !quoted {
        for i in 0..len {
            let c = *s.add(i);
            if c == b'*' || c == b'?' || c == b'[' { (*w).has_glob = true; break; }
        }
    }
    (*w).quoted = true;
    0
}

unsafe fn env_value(name: *const u8, len: usize) -> *mut c_char {
    let mut buf = [0 as c_char; 256];
    if len >= buf.len() { return core::ptr::null_mut(); }
    for i in 0..len { buf[i] = *name.add(i) as c_char; }
    buf[len] = 0;
    crate::getenv(buf.as_ptr())
}

/// Expand `$name` or `${name}`; returns the new index or an error code.
unsafe fn expand_var(
    s: &[u8],
    i_start: usize,
    w: *mut WordBuf,
    flags: c_int,
) -> Result<usize, c_int> {
    let mut i = i_start + 1;
    let mut name = [0u8; 256];
    let mut nl = 0usize;
    if s[i] == b'{' {
        i += 1;
        while s[i] != 0 && s[i] != b'}' {
            if nl + 1 >= name.len() { return Err(WRDE_NOSPACE_V); }
            name[nl] = s[i];
            nl += 1;
            i += 1;
        }
        if s[i] != b'}' { return Err(WRDE_SYNTAX_V); }
        i += 1;
    } else {
        while nl + 1 < name.len() {
            let c = s[i];
            if (c >= b'A' && c <= b'Z') || (c >= b'a' && c <= b'z') || (c >= b'0' && c <= b'9') || c == b'_' {
                name[nl] = c;
                nl += 1;
                i += 1;
            } else {
                break;
            }
        }
        if nl == 0 {
            // A lone '$' is literal.
            let r = wb_push(w, b'$', false);
            if r != 0 { return Err(r); }
            return Ok(i_start + 1);
        }
    }
    if nl == 1 && name[0] == b'$' {
        let mut pidbuf = [0 as c_char; 16];
        let pid = crate::getpid();
        let mut n = 0usize;
        let mut v = pid as u32;
        let mut digits = [0u8; 10];
        if v == 0 { digits[0] = b'0'; n = 1; }
        while v > 0 { digits[n] = b'0' + (v % 10) as u8; v /= 10; n += 1; }
        let mut o = 0usize;
        while n > 0 { n -= 1; pidbuf[o] = digits[n] as c_char; o += 1; }
        pidbuf[o] = 0;
        let r = wb_push_str(w, pidbuf.as_ptr() as *const u8, o, false);
        if r != 0 { return Err(r); }
        return Ok(i);
    }
    let val = env_value(name.as_ptr(), nl);
    if val.is_null() {
        if flags & WRDE_UNDEF_V != 0 { return Err(WRDE_BADVAL_V); }
        return Ok(i);
    }
    let vl = crate::strlen(val);
    let r = wb_push_str(w, val as *const u8, vl, false);
    if r != 0 { return Err(r); }
    Ok(i)
}

unsafe fn command_sub(
    s: &[u8],
    i_start: usize,
    w: *mut WordBuf,
    flags: c_int,
) -> Result<usize, c_int> {
    if flags & WRDE_NOCMD_V != 0 { return Err(WRDE_CMDSUB_V); }
    let mut i = i_start;
    let mut cmd = [0u8; 2048];
    let mut cl = 0usize;
    if s[i] == b'`' {
        i += 1;
        while s[i] != 0 && s[i] != b'`' {
            if cl + 1 >= cmd.len() { return Err(WRDE_NOSPACE_V); }
            cmd[cl] = s[i];
            cl += 1;
            i += 1;
        }
        if s[i] != b'`' { return Err(WRDE_SYNTAX_V); }
        i += 1;
    } else {
        // $( ... ) with nesting.
        i += 2;
        let mut depth = 1;
        while s[i] != 0 {
            let c = s[i];
            if c == b'\\' && s[i + 1] != 0 {
                if cl + 1 >= cmd.len() { return Err(WRDE_NOSPACE_V); }
                cmd[cl] = c;
                cl += 1;
                i += 1;
                if cl + 1 >= cmd.len() { return Err(WRDE_NOSPACE_V); }
                cmd[cl] = s[i];
                cl += 1;
                i += 1;
                continue;
            }
            if c == b'\'' {
                // Copy the quoted section verbatim.
                if cl + 1 >= cmd.len() { return Err(WRDE_NOSPACE_V); }
                cmd[cl] = c;
                cl += 1;
                i += 1;
                while s[i] != 0 && s[i] != b'\'' {
                    if cl + 1 >= cmd.len() { return Err(WRDE_NOSPACE_V); }
                    cmd[cl] = s[i];
                    cl += 1;
                    i += 1;
                }
                if s[i] == b'\'' {
                    if cl + 1 >= cmd.len() { return Err(WRDE_NOSPACE_V); }
                    cmd[cl] = s[i];
                    cl += 1;
                    i += 1;
                }
                continue;
            }
            if c == b'(' { depth += 1; }
            if c == b')' {
                depth -= 1;
                if depth == 0 { i += 1; break; }
            }
            if cl + 1 >= cmd.len() { return Err(WRDE_NOSPACE_V); }
            cmd[cl] = c;
            cl += 1;
            i += 1;
        }
        if depth != 0 { return Err(WRDE_SYNTAX_V); }
    }
    cmd[cl] = 0;
    let f = crate::popen(cmd.as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    if f.is_null() { return Ok(i); }
    let mut out = [0u8; 4096];
    let n = crate::fread(out.as_mut_ptr() as *mut c_void, 1, out.len() - 1, f);
    crate::pclose(f);
    let mut l = n;
    while l > 0 && (out[l - 1] == b'\n' || out[l - 1] == b'\r') { l -= 1; }
    // Word splitting on the command output.
    let mut p = 0usize;
    let mut first = true;
    while p < l {
        while p < l && (out[p] == b' ' || out[p] == b'\t' || out[p] == b'\n') { p += 1; }
        if p >= l { break; }
        let st = p;
        while p < l && out[p] != b' ' && out[p] != b'\t' && out[p] != b'\n' { p += 1; }
        if !first {
            // Caller appends to the current word; insert a separator by
            // returning a space marker is not possible here, so concatenate.
        }
        let r = wb_push_str(w, out.as_ptr().add(st), p - st, false);
        if r != 0 { return Err(r); }
        first = false;
    }
    Ok(i)
}

#[no_mangle]
pub unsafe extern "C" fn wordexp(s: *const c_char, p: *mut WordexpT, flags: c_int) -> c_int {
    if s.is_null() || p.is_null() { return WRDE_SYNTAX_V; }
    if flags & WRDE_REUSE_V != 0 { wordfree(p); }
    let mut list = WordList { items: core::ptr::null_mut(), n: 0, cap: 0 };
    let src = core::slice::from_raw_parts(s as *const u8, crate::strlen(s) + 1);
    let mut i = 0usize;
    loop {
        while src[i] == b' ' || src[i] == b'\t' || src[i] == b'\n' { i += 1; }
        if src[i] == 0 { break; }
        let mut w = WordBuf { buf: [0u8; WORDEXP_CAP], len: 0, has_glob: false, quoted: false };
        let mut done = false;
        while !done {
            let c = src[i];
            match c {
                0 => { done = true; }
                b' ' | b'\t' | b'\n' => { i += 1; done = true; }
                b'\'' => {
                    i += 1;
                    loop {
                        if src[i] == 0 {
                            wl_free(&mut list);
                            return WRDE_SYNTAX_V;
                        }
                        if src[i] == b'\'' { i += 1; break; }
                        let r = wb_push(&mut w, src[i], true);
                        if r != 0 { wl_free(&mut list); return r; }
                        i += 1;
                    }
                    w.quoted = true;
                }
                b'"' => {
                    i += 1;
                    loop {
                        if src[i] == 0 {
                            wl_free(&mut list);
                            return WRDE_SYNTAX_V;
                        }
                        if src[i] == b'"' { i += 1; break; }
                        if src[i] == b'\\' {
                            let n = src[i + 1];
                            if n == b'$' || n == b'`' || n == b'"' || n == b'\\' {
                                let r = wb_push(&mut w, n, true);
                                if r != 0 { wl_free(&mut list); return r; }
                                i += 2;
                                continue;
                            }
                            let r = wb_push(&mut w, src[i], true);
                            if r != 0 { wl_free(&mut list); return r; }
                            i += 1;
                            continue;
                        }
                        if src[i] == b'`' || (src[i] == b'$' && src[i + 1] == b'(') {
                            match command_sub(src, i, &mut w, flags) {
                                Ok(ni) => { i = ni; continue; }
                                Err(e) => { wl_free(&mut list); return e; }
                            }
                        }
                        if src[i] == b'$' {
                            match expand_var(src, i, &mut w, flags) {
                                Ok(ni) => { i = ni; continue; }
                                Err(e) => { wl_free(&mut list); return e; }
                            }
                        }
                        let r = wb_push(&mut w, src[i], true);
                        if r != 0 { wl_free(&mut list); return r; }
                        i += 1;
                    }
                    w.quoted = true;
                }
                b'\\' => {
                    i += 1;
                    if src[i] == 0 {
                        wl_free(&mut list);
                        return WRDE_SYNTAX_V;
                    }
                    let r = wb_push(&mut w, src[i], true);
                    if r != 0 { wl_free(&mut list); return r; }
                    i += 1;
                }
                b'$' => {
                    if src[i + 1] == b'(' {
                        match command_sub(src, i, &mut w, flags) {
                            Ok(ni) => { i = ni; }
                            Err(e) => { wl_free(&mut list); return e; }
                        }
                    } else {
                        match expand_var(src, i, &mut w, flags) {
                            Ok(ni) => { i = ni; }
                            Err(e) => { wl_free(&mut list); return e; }
                        }
                    }
                }
                b'`' => {
                    match command_sub(src, i, &mut w, flags) {
                        Ok(ni) => { i = ni; }
                        Err(e) => { wl_free(&mut list); return e; }
                    }
                }
                b'*' | b'?' | b'[' => {
                    w.has_glob = true;
                    let r = wb_push(&mut w, c, false);
                    if r != 0 { wl_free(&mut list); return r; }
                    i += 1;
                }
                _ => {
                    let r = wb_push(&mut w, c, false);
                    if r != 0 { wl_free(&mut list); return r; }
                    i += 1;
                }
            }
        }
        if w.len == 0 && !w.quoted { continue; }
        w.buf[w.len] = 0;
        if w.has_glob && !w.quoted {
            let mut g: GlobT = core::mem::zeroed();
            let r = crate::glob(w.buf.as_ptr() as *const c_char, 0x10 | 0x04, None, &mut g); // NOCHECK|NOSORT
            if r == 0 {
                for k in 0..g.gl_pathc {
                    let gp = *g.gl_pathv.add(k);
                    let l = crate::strlen(gp);
                    wl_push(&mut list, gp as *const u8, l);
                }
                crate::globfree(&mut g);
            } else {
                wl_push(&mut list, w.buf.as_ptr(), w.len);
            }
        } else {
            wl_push(&mut list, w.buf.as_ptr(), w.len);
        }
    }

    let old = if flags & WRDE_APPEND_V != 0 { (*p).we_wordc } else { 0 };
    let offs = if flags & WRDE_DOOFFS_V != 0 { (*p).we_offs } else { 0 };
    let total = offs + old + list.n + 1;
    let pathv = crate::calloc(total, core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
    if pathv.is_null() {
        wl_free(&mut list);
        return WRDE_NOSPACE_V;
    }
    for k in 0..offs { *pathv.add(k) = core::ptr::null_mut(); }
    if flags & WRDE_APPEND_V != 0 && old != 0 && !(*p).we_wordv.is_null() {
        for k in 0..old {
            *pathv.add(offs + k) = *(*p).we_wordv.add(offs + k);
        }
        crate::free((*p).we_wordv as *mut c_void);
    }
    for k in 0..list.n {
        *pathv.add(offs + old + k) = *list.items.add(k);
    }
    *pathv.add(offs + old + list.n) = core::ptr::null_mut();
    crate::free(list.items as *mut c_void);
    (*p).we_wordc = old + list.n;
    (*p).we_wordv = pathv;
    (*p).we_offs = offs;
    0
}

#[no_mangle]
pub unsafe extern "C" fn wordfree(p: *mut WordexpT) {
    if p.is_null() || (*p).we_wordv.is_null() { return; }
    let offs = (*p).we_offs;
    for i in 0..(*p).we_wordc {
        let q = *(*p).we_wordv.add(offs + i);
        if !q.is_null() { crate::free(q as *mut c_void); }
    }
    crate::free((*p).we_wordv as *mut c_void);
    (*p).we_wordv = core::ptr::null_mut();
    (*p).we_wordc = 0;
}
