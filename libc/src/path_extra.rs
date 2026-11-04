// realpath, glob/globfree and nftw/ftw.

const PATH_MAX_V: usize = 4096;

// ============================================================
// realpath
// ============================================================

static mut REALPATH_BUF: [c_char; PATH_MAX_V] = [0; PATH_MAX_V];

unsafe fn rp_is_dir(path: *const c_char) -> bool {
    let mut st: Stat = core::mem::zeroed();
    if crate::lstat(path, &mut st as *mut Stat as *mut c_void) != 0 { return false; }
    st.st_mode & 0o170000 == 0o040000
}

#[no_mangle]
pub unsafe extern "C" fn realpath(
    filename: *const c_char,
    resolved: *mut c_char,
) -> *mut c_char {
    if filename.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    let flen = crate::strlen(filename);
    if flen == 0 { ERRNO = 2; return core::ptr::null_mut(); }
    if flen >= PATH_MAX_V { ERRNO = 36; return core::ptr::null_mut(); }

    let mut out = [0u8; PATH_MAX_V + 1];
    let mut olen = 0usize;
    let mut pend = [0u8; 2 * PATH_MAX_V + 1];
    core::ptr::copy_nonoverlapping(filename as *const u8, pend.as_mut_ptr(), flen);
    pend[flen] = 0;
    let mut pi = 0usize;
    let absolute = pend[0] == b'/';
    if absolute {
        pi = 1;
        out[0] = b'/';
        olen = 1;
    } else {
        let r = crate::getcwd(out.as_mut_ptr() as *mut c_char, PATH_MAX_V);
        if r.is_null() { return core::ptr::null_mut(); }
        olen = crate::strlen(out.as_ptr() as *const c_char);
        if olen == 0 { out[0] = b'/'; olen = 1; }
    }
    let mut symlinks = 0;

    loop {
        while pend[pi] == b'/' { pi += 1; }
        if pend[pi] == 0 { break; }
        let start = pi;
        while pend[pi] != 0 && pend[pi] != b'/' { pi += 1; }
        let comp_len = pi - start;
        let mut q = pi;
        while pend[q] == b'/' { q += 1; }
        let is_last = pend[q] == 0;
        let followed_by_slash = pend[pi] == b'/';
        let needs_dir = !is_last || followed_by_slash;

        if comp_len == 1 && pend[start] == b'.' {
            if needs_dir && is_last { /* "." is a dir */ }
            continue;
        }
        if comp_len == 2 && pend[start] == b'.' && pend[start + 1] == b'.' {
            while olen > 0 && out[olen - 1] != b'/' { olen -= 1; }
            if olen > 0 { olen -= 1; }
            continue;
        }

        if olen == 0 {
            out[0] = b'/';
            olen = 1;
        }
        if olen > 0 && out[olen - 1] != b'/' {
            if olen + 1 >= PATH_MAX_V { ERRNO = 36; return core::ptr::null_mut(); }
            out[olen] = b'/';
            olen += 1;
        }
        if olen + comp_len >= PATH_MAX_V { ERRNO = 36; return core::ptr::null_mut(); }
        core::ptr::copy_nonoverlapping(pend.as_ptr().add(start), out.as_mut_ptr().add(olen), comp_len);
        olen += comp_len;
        out[olen] = 0;

        let mut st: Stat = core::mem::zeroed();
        if crate::lstat(out.as_ptr() as *const c_char, &mut st as *mut Stat as *mut c_void) != 0 {
            if ERRNO == 2 { ERRNO = 2; }
            return core::ptr::null_mut();
        }
        let mode = st.st_mode;
        if mode & 0o170000 == 0o120000 {
            symlinks += 1;
            if symlinks > 40 { ERRNO = 40; return core::ptr::null_mut(); }
            let mut link = [0u8; PATH_MAX_V + 1];
            let n = crate::readlink(out.as_ptr() as *const c_char, link.as_mut_ptr() as *mut c_char, PATH_MAX_V);
            if n < 0 { return core::ptr::null_mut(); }
            let n = n as usize;
            if n == 0 { ERRNO = 2; return core::ptr::null_mut(); }
            link[n] = 0;
            // Remove the component we just appended.
            while olen > 0 && out[olen - 1] != b'/' { olen -= 1; }
            if olen > 0 { olen -= 1; }
            let mut next = [0u8; 2 * PATH_MAX_V + 1];
            let link_abs = link[0] == b'/';
            let mut nl = 0usize;
            if !link_abs && olen > 0 {
                // keep relative target relative to the current directory
            }
            core::ptr::copy_nonoverlapping(link.as_ptr(), next.as_mut_ptr(), n);
            nl = n;
            let rest = &pend[pi..];
            let rest_len = crate::strlen(rest.as_ptr() as *const c_char);
            if rest_len > 0 {
                if nl > 0 && next[nl - 1] != b'/' {
                    next[nl] = b'/';
                    nl += 1;
                }
                if nl + rest_len >= next.len() { ERRNO = 36; return core::ptr::null_mut(); }
                core::ptr::copy_nonoverlapping(rest.as_ptr(), next.as_mut_ptr().add(nl), rest_len + 1);
                nl += rest_len;
            }
            if link_abs { olen = 0; }
            pend = next;
            pi = 0;
            let _ = nl;
            continue;
        }
        if mode & 0o170000 != 0o040000 && needs_dir {
            ERRNO = 20; // ENOTDIR
            return core::ptr::null_mut();
        }
    }

    if olen == 0 {
        out[0] = b'/';
        olen = 1;
    }
    // Collapse a trailing slash except for the root.
    while olen > 1 && out[olen - 1] == b'/' { olen -= 1; }
    out[olen] = 0;
    let dst = if resolved.is_null() { REALPATH_BUF.as_mut_ptr() } else { resolved };
    core::ptr::copy_nonoverlapping(out.as_ptr(), dst as *mut u8, olen + 1);
    dst
}

// ============================================================
// glob
// ============================================================

#[repr(C)]
pub struct GlobT {
    pub gl_pathc: usize,
    pub gl_pathv: *mut *mut c_char,
    pub gl_offs: usize,
    __dummy1: c_int,
    __dummy2: [*mut c_void; 5],
}

const GLOB_ERR_V: c_int = 0x01;
const GLOB_MARK_V: c_int = 0x02;
const GLOB_NOSORT_V: c_int = 0x04;
const GLOB_DOOFFS_V: c_int = 0x08;
const GLOB_NOCHECK_V: c_int = 0x10;
const GLOB_APPEND_V: c_int = 0x20;
const GLOB_NOESCAPE_V: c_int = 0x40;
const GLOB_PERIOD_V: c_int = 0x80;
const GLOB_TILDE_V: c_int = 0x1000;
const GLOB_TILDE_CHECK_V: c_int = 0x4000;

const GLOB_NOSPACE_V: c_int = 1;
const GLOB_ABORTED_V: c_int = 2;
const GLOB_NOMATCH_V: c_int = 3;

struct GlobList {
    items: *mut *mut c_char,
    n: usize,
    cap: usize,
}

unsafe fn glob_list_push(list: *mut GlobList, s: *const c_char) {
    if (*list).n == (*list).cap {
        let ncap = if (*list).cap == 0 { 32 } else { (*list).cap * 2 };
        let na = crate::realloc(
            (*list).items as *mut c_void,
            ncap * core::mem::size_of::<*mut c_char>(),
        ) as *mut *mut c_char;
        if na.is_null() { return; }
        (*list).items = na;
        (*list).cap = ncap;
    }
    let l = crate::strlen(s);
    let copy = crate::malloc(l + 1) as *mut c_char;
    if copy.is_null() { return; }
    core::ptr::copy_nonoverlapping(s as *const u8, copy as *mut u8, l + 1);
    *(*list).items.add((*list).n) = copy;
    (*list).n += 1;
}

unsafe fn glob_list_push_n(list: *mut GlobList, s: *const u8, len: usize) {
    if (*list).n == (*list).cap {
        let ncap = if (*list).cap == 0 { 32 } else { (*list).cap * 2 };
        let na = crate::realloc(
            (*list).items as *mut c_void,
            ncap * core::mem::size_of::<*mut c_char>(),
        ) as *mut *mut c_char;
        if na.is_null() { return; }
        (*list).items = na;
        (*list).cap = ncap;
    }
    let copy = crate::malloc(len + 1) as *mut c_char;
    if copy.is_null() { return; }
    core::ptr::copy_nonoverlapping(s, copy as *mut u8, len);
    *copy.add(len) = 0;
    *(*list).items.add((*list).n) = copy;
    (*list).n += 1;
}

unsafe fn list_dir_names(path: *const c_char) -> (*mut *mut c_char, usize) {
    let fd = crate::open(path, crate::O_DIRECTORY, 0);
    if fd < 0 { return (core::ptr::null_mut(), 0); }
    let mut cap = 32usize;
    let mut count = 0usize;
    let mut arr = crate::malloc(cap * core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
    if arr.is_null() { crate::close(fd); return (core::ptr::null_mut(), 0); }
    #[repr(align(8))]
    struct Buf([u8; 8192]);
    let mut buf = Buf([0u8; 8192]);
    let name_off = core::mem::offset_of!(linux_dirent64, d_name);
    loop {
        let n = <Arch as Syscalls>::syscall3(
            SYS_GETDENTS64,
            fd as i64,
            buf.0.as_mut_ptr() as i64,
            buf.0.len() as i64,
        ) as isize;
        if n <= 0 { break; }
        let mut off = 0usize;
        while off + name_off < n as usize {
            let ent = buf.0.as_ptr().add(off) as *const linux_dirent64;
            let reclen = (*ent).d_reclen as usize;
            if reclen == 0 { break; }
            let name = buf.0.as_ptr().add(off + name_off) as *const c_char;
            let nl = crate::strlen(name);
            if count + 2 >= cap {
                cap *= 2;
                let na = crate::realloc(arr as *mut c_void, cap * core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
                if na.is_null() { break; }
                arr = na;
            }
            let copy = crate::malloc(nl + 1) as *mut c_char;
            if copy.is_null() { break; }
            core::ptr::copy_nonoverlapping(name as *const u8, copy as *mut u8, nl + 1);
            *arr.add(count) = copy;
            count += 1;
            off += reclen;
        }
    }
    crate::close(fd);
    (arr, count)
}

unsafe fn free_names(arr: *mut *mut c_char, n: usize) {
    if arr.is_null() { return; }
    for i in 0..n {
        if !(*arr.add(i)).is_null() { crate::free(*arr.add(i) as *mut c_void); }
    }
    crate::free(arr as *mut c_void);
}

fn glob_has_magic(comp: &[u8], noescape: bool) -> bool {
    let mut i = 0;
    while i < comp.len() {
        match comp[i] {
            b'\\' if !noescape => i += 1,
            b'*' | b'?' | b'[' => return true,
            _ => {}
        }
        i += 1;
    }
    false
}

unsafe fn glob_unescape(dst: &mut [u8], comp: &[u8], noescape: bool) -> usize {
    let mut o = 0usize;
    let mut i = 0usize;
    while i < comp.len() {
        if comp[i] == b'\\' && !noescape && i + 1 < comp.len() {
            i += 1;
        }
        if o < dst.len() { dst[o] = comp[i]; o += 1; }
        i += 1;
    }
    o
}

#[derive(Copy, Clone)]
struct Comp {
    start: usize,
    end: usize,
}

unsafe fn glob_split(pat: &[u8], noescape: bool, comps: &mut [Comp; 128]) -> (usize, bool) {
    let mut n = 0usize;
    let mut i = 0usize;
    let absolute = !pat.is_empty() && pat[0] == b'/';
    let mut trailing = false;
    while i < pat.len() {
        while i < pat.len() && pat[i] == b'/' { i += 1; }
        if i >= pat.len() { if i > 0 { trailing = pat[i - 1] == b'/'; } break; }
        let start = i;
        while i < pat.len() {
            if pat[i] == b'\\' && !noescape && i + 1 < pat.len() { i += 2; continue; }
            if pat[i] == b'/' { break; }
            i += 1;
        }
        if n < comps.len() {
            comps[n] = Comp { start, end: i };
            n += 1;
        }
    }
    let _ = trailing;
    (n, absolute)
}

unsafe fn glob_walk(
    base: &mut [u8; PATH_MAX_V + 1],
    blen: &mut usize,
    pat: &[u8],
    comps: &[Comp],
    ci: usize,
    trailing: bool,
    flags: c_int,
    errfunc: Option<unsafe extern "C" fn(*const c_char, c_int) -> c_int>,
    list: *mut GlobList,
) -> c_int {
    if ci >= comps.len() {
        glob_list_push_n(list, base.as_ptr(), *blen);
        return 0;
    }
    let comp = &pat[comps[ci].start..comps[ci].end];
    let last = ci + 1 == comps.len();
    let noescape = flags & GLOB_NOESCAPE_V != 0;

    if !glob_has_magic(comp, noescape) {
        let mut name = [0u8; PATH_MAX_V + 1];
        let nlen = glob_unescape(&mut name, comp, noescape);
        let mut newlen = *blen;
        if newlen > 0 && base[newlen - 1] != b'/' {
            if newlen + 1 >= base.len() { return GLOB_NOSPACE_V; }
            base[newlen] = b'/';
            newlen += 1;
        }
        if newlen + nlen >= base.len() { return GLOB_NOSPACE_V; }
        core::ptr::copy_nonoverlapping(name.as_ptr(), base.as_mut_ptr().add(newlen), nlen);
        newlen += nlen;
        base[newlen] = 0;
        let mut st: Stat = core::mem::zeroed();
        let exists = crate::lstat(base.as_ptr() as *const c_char, &mut st as *mut Stat as *mut c_void) == 0;
        let is_dir = exists && st.st_mode & 0o170000 == 0o040000;
        if last {
            if exists && (!trailing || is_dir) {
                if flags & GLOB_ONLYDIR_V != 0 && !is_dir {
                    *blen = newlen;
                    return 0;
                }
                let mut end = newlen;
                if (flags & GLOB_MARK_V != 0) && is_dir {
                    if end + 1 < base.len() {
                        base[end] = b'/';
                        end += 1;
                    }
                }
                glob_list_push_n(list, base.as_ptr(), end);
            }
        } else {
            if !is_dir {
                if flags & GLOB_ERR_V != 0 {
                    *blen = newlen;
                    return GLOB_ABORTED_V;
                }
            } else {
                let old = *blen;
                *blen = newlen;
                let r = glob_walk(base, blen, pat, comps, ci + 1, trailing, flags, errfunc, list);
                *blen = old;
                if r != 0 { return r; }
            }
        }
        *blen = newlen;
        return 0;
    }

    // Wildcard component: enumerate the base directory.
    let mut dirc = [0 as c_char; PATH_MAX_V + 1];
    let dp: *const c_char;
    if *blen == 0 {
        dirc[0] = b'.' as c_char;
        dirc[1] = 0;
        dp = dirc.as_ptr();
    } else {
        core::ptr::copy_nonoverlapping(base.as_ptr(), dirc.as_mut_ptr() as *mut u8, *blen);
        dirc[*blen] = 0;
        dp = dirc.as_ptr();
    }
    let (names, count) = list_dir_names(dp);
    if names.is_null() {
        if flags & GLOB_ERR_V != 0 {
            if let Some(f) = errfunc { let _ = f(dp, ERRNO); }
            return GLOB_ABORTED_V;
        }
        return 0;
    }
    let mut patc = [0 as c_char; PATH_MAX_V + 1];
    core::ptr::copy_nonoverlapping(comp.as_ptr(), patc.as_mut_ptr() as *mut u8, comp.len());
    patc[comp.len()] = 0;
    let mut fnm_flags = 0;
    if noescape { fnm_flags |= FNM_NOESCAPE; }
    if flags & GLOB_PERIOD_V == 0 { fnm_flags |= FNM_PERIOD; }
    let old = *blen;
    for i in 0..count {
        let name = *names.add(i);
        if flags & GLOB_PERIOD_V == 0 {
            // FNM_PERIOD already covers this, but skip "." / ".." here.
            if *name == b'.' as c_char {
                let second = *name.add(1);
                if second == 0 || (second == b'.' as c_char && *name.add(2) == 0) { continue; }
            }
        }
        if crate::fnmatch(patc.as_ptr(), name, fnm_flags) != 0 { continue; }
        let nlen = crate::strlen(name);
        let mut newlen = old;
        if newlen > 0 && base[newlen - 1] != b'/' {
            if newlen + 1 >= base.len() { free_names(names, count); return GLOB_NOSPACE_V; }
            base[newlen] = b'/';
            newlen += 1;
        }
        if newlen + nlen >= base.len() { free_names(names, count); return GLOB_NOSPACE_V; }
        core::ptr::copy_nonoverlapping(name as *const u8, base.as_mut_ptr().add(newlen), nlen);
        newlen += nlen;
        base[newlen] = 0;
        let mut st: Stat = core::mem::zeroed();
        let exists = crate::lstat(base.as_ptr() as *const c_char, &mut st as *mut Stat as *mut c_void) == 0;
        let is_dir = exists && st.st_mode & 0o170000 == 0o040000;
        if last {
            if flags & GLOB_ONLYDIR_V != 0 && !is_dir { continue; }
            let mut end = newlen;
            if (flags & GLOB_MARK_V != 0) && is_dir && end + 1 < base.len() {
                base[end] = b'/';
                end += 1;
            }
            glob_list_push_n(list, base.as_ptr(), end);
        } else if is_dir {
            *blen = newlen;
            let r = glob_walk(base, blen, pat, comps, ci + 1, trailing, flags, errfunc, list);
            *blen = old;
            if r != 0 { free_names(names, count); return r; }
        }
    }
    free_names(names, count);
    *blen = old;
    0
}

const GLOB_ONLYDIR_V: c_int = 0x200; // same value as musl/glibc

unsafe fn glob_expand_tilde(pat: *const c_char, buf: *mut u8, cap: usize) -> Option<usize> {
    let s = pat as *const u8;
    if *s != b'~' { return None; }
    let mut i = 1usize;
    while *s.add(i) != 0 && *s.add(i) != b'/' { i += 1; }
    let mut name = [0 as c_char; 256];
    let nlen = i - 1;
    if nlen >= name.len() { return None; }
    core::ptr::copy_nonoverlapping(s.add(1), name.as_mut_ptr() as *mut u8, nlen);
    name[nlen] = 0;
    let pw = if nlen == 0 {
        crate::getpwuid(crate::getuid())
    } else {
        crate::getpwnam(name.as_ptr())
    };
    if pw.is_null() { return None; }
    let home = (*pw).pw_dir;
    let hl = crate::strlen(home);
    let rest = crate::strlen(s.add(i) as *const c_char);
    if hl + rest + 1 > cap { return None; }
    core::ptr::copy_nonoverlapping(home as *const u8, buf, hl);
    core::ptr::copy_nonoverlapping(s.add(i), buf.add(hl), rest + 1);
    Some(hl + rest)
}

#[no_mangle]
pub unsafe extern "C" fn glob(
    pattern: *const c_char,
    flags: c_int,
    errfunc: Option<unsafe extern "C" fn(*const c_char, c_int) -> c_int>,
    pglob: *mut GlobT,
) -> c_int {
    if pattern.is_null() || pglob.is_null() { ERRNO = EINVAL; return GLOB_ABORTED_V; }

    let mut expanded = [0u8; PATH_MAX_V + 1];
    let mut pat_ptr = pattern as *const u8;
    let mut pat_len = crate::strlen(pattern);
    if flags & (GLOB_TILDE_V | GLOB_TILDE_CHECK_V) != 0 {
        if let Some(n) = glob_expand_tilde(pattern, expanded.as_mut_ptr(), PATH_MAX_V) {
            pat_ptr = expanded.as_ptr();
            pat_len = n;
        } else if flags & GLOB_TILDE_CHECK_V != 0 {
            return GLOB_NOMATCH_V;
        }
    }
    let pat = &core::slice::from_raw_parts(pat_ptr, pat_len)[..];

    let mut comps = [Comp { start: 0, end: 0 }; 128];
    let (ncomp, absolute) = glob_split(pat, flags & GLOB_NOESCAPE_V != 0, &mut comps);

    let mut list = GlobList { items: core::ptr::null_mut(), n: 0, cap: 0 };
    let mut base = [0u8; PATH_MAX_V + 1];
    let mut blen = 0usize;
    if absolute {
        base[0] = b'/';
        blen = 1;
    }
    let trailing = !pat.is_empty() && pat[pat.len() - 1] == b'/';
    let r = if ncomp == 0 {
        if absolute {
            glob_list_push_n(&mut list, b"/".as_ptr(), 1);
            0
        } else {
            GLOB_NOMATCH_V
        }
    } else {
        glob_walk(&mut base, &mut blen, pat, &comps[..ncomp], 0, trailing, flags, errfunc, &mut list)
    };
    if r == GLOB_ABORTED_V {
        for i in 0..list.n { crate::free(*list.items.add(i) as *mut c_void); }
        crate::free(list.items as *mut c_void);
        return r;
    }
    let mut matches = list.n;
    if matches == 0 && flags & GLOB_NOCHECK_V != 0 {
        glob_list_push(&mut list, pattern);
        matches = list.n;
    }
    if matches == 0 && flags & GLOB_APPEND_V == 0 {
        crate::free(list.items as *mut c_void);
        return GLOB_NOMATCH_V;
    }
    if flags & GLOB_NOSORT_V == 0 && matches > 1 {
        crate::qsort(
            list.items as *mut c_void,
            matches,
            core::mem::size_of::<*mut c_char>(),
            glob_cmp,
        );
    }

    let old_n = if flags & GLOB_APPEND_V != 0 { (*pglob).gl_pathc } else { 0 };
    let offs = if flags & GLOB_DOOFFS_V != 0 { (*pglob).gl_offs } else { 0 };
    if old_n != 0 && !(*pglob).gl_pathv.is_null() && flags & GLOB_APPEND_V == 0 {
        // Nothing to free; append is off.
    }
    let total = offs + old_n + matches + 1;
    let pathv = crate::calloc(total, core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
    if pathv.is_null() {
        for i in 0..list.n { crate::free(*list.items.add(i) as *mut c_void); }
        crate::free(list.items as *mut c_void);
        return GLOB_NOSPACE_V;
    }
    for i in 0..offs { *pathv.add(i) = core::ptr::null_mut(); }
    if flags & GLOB_APPEND_V != 0 && old_n != 0 && !(*pglob).gl_pathv.is_null() {
        for i in 0..old_n {
            *pathv.add(offs + i) = *(*pglob).gl_pathv.add(offs + i);
        }
        crate::free((*pglob).gl_pathv as *mut c_void);
    }
    for i in 0..list.n {
        *pathv.add(offs + old_n + i) = *list.items.add(i);
    }
    *pathv.add(offs + old_n + matches) = core::ptr::null_mut();
    crate::free(list.items as *mut c_void);

    (*pglob).gl_pathc = old_n + matches;
    (*pglob).gl_pathv = pathv;
    (*pglob).gl_offs = offs;
    (*pglob).__dummy1 = 0;
    0
}

unsafe extern "C" fn glob_cmp(a: *const c_void, b: *const c_void) -> c_int {
    let sa = *(a as *const *const c_char);
    let sb = *(b as *const *const c_char);
    crate::strcmp(sa as *const u8, sb as *const u8)
}

#[no_mangle]
pub unsafe extern "C" fn globfree(pglob: *mut GlobT) {
    if pglob.is_null() || (*pglob).gl_pathv.is_null() { return; }
    let offs = (*pglob).gl_offs;
    for i in 0..(*pglob).gl_pathc {
        let p = *(*pglob).gl_pathv.add(offs + i);
        if !p.is_null() { crate::free(p as *mut c_void); }
    }
    crate::free((*pglob).gl_pathv as *mut c_void);
    (*pglob).gl_pathv = core::ptr::null_mut();
    (*pglob).gl_pathc = 0;
}

// ============================================================
// nftw / ftw
// ============================================================

#[repr(C)]
pub struct Ftw {
    pub base: c_int,
    pub level: c_int,
}

const FTW_F_V: c_int = 1;
const FTW_D_V: c_int = 2;
const FTW_DNR_V: c_int = 3;
const FTW_NS_V: c_int = 4;
const FTW_SL_V: c_int = 5;
const FTW_DP_V: c_int = 6;
const FTW_SLN_V: c_int = 7;

const FTW_PHYS_V: c_int = 1;
const FTW_MOUNT_V: c_int = 2;
const FTW_DEPTH_V: c_int = 8;

struct History {
    chain: *mut History,
    dev: u64,
    ino: u64,
    level: c_int,
    base: c_int,
}

#[derive(Copy, Clone)]
enum WalkCb {
    N(Option<unsafe extern "C" fn(*const c_char, *const Stat, c_int, *mut Ftw) -> c_int>),
    F(Option<unsafe extern "C" fn(*const c_char, *const Stat, c_int) -> c_int>),
}

unsafe fn walk_call(cb: WalkCb, path: *const c_char, st: *const Stat, type_: c_int, ftw: *mut Ftw) -> c_int {
    match cb {
        WalkCb::N(Some(f)) => f(path, st, type_, ftw),
        WalkCb::F(Some(f)) => f(path, st, type_),
        _ => 0,
    }
}

unsafe fn nftw_do(
    buf: *mut u8,
    cap: usize,
    len: usize,
    cb: WalkCb,
    fd_limit: c_int,
    flags: c_int,
    h: *mut History,
    base: c_int,
    level: c_int,
) -> c_int {
    let path = buf as *const c_char;
    let mut st: Stat = core::mem::zeroed();
    let phys = flags & FTW_PHYS_V != 0;
    let mut type_: c_int;
    let mut stat_ok = true;
    let rc = if phys { crate::lstat(path, &mut st as *mut Stat as *mut c_void) } else { crate::stat(path, &mut st) };
    if rc != 0 {
        if !phys && ERRNO == 2 && crate::lstat(path, &mut st as *mut Stat as *mut c_void) == 0 {
            type_ = FTW_SLN_V;
        } else if ERRNO != 13 {
            return -1;
        } else {
            type_ = FTW_NS_V;
            st = core::mem::zeroed();
            stat_ok = false;
        }
    } else {
        let mode = st.st_mode;
        if mode & 0o170000 == 0o040000 {
            type_ = if flags & FTW_DEPTH_V != 0 { FTW_DP_V } else { FTW_D_V };
        } else if mode & 0o170000 == 0o120000 {
            type_ = if phys { FTW_SL_V } else { FTW_SLN_V };
        } else {
            type_ = FTW_F_V;
        }
    }
    if flags & FTW_MOUNT_V != 0 && !h.is_null() && stat_ok && type_ != FTW_NS_V && st.st_dev != (*h).dev {
        return 0;
    }
    let parent_base = if h.is_null() {
        let mut k = len;
        while k > 0 && *buf.add(k - 1) == b'/' { k -= 1; }
        while k > 0 && *buf.add(k - 1) != b'/' { k -= 1; }
        k as c_int
    } else {
        (*h).base
    };
    let mut new = History {
        chain: h,
        dev: st.st_dev,
        ino: st.st_ino,
        level: level + 1,
        base: parent_base,
    };
    let mut ftw = Ftw { base: parent_base, level: level };

    let mut dfd: c_int = -1;
    let mut open_err: c_int = 0;
    if type_ == FTW_D_V || type_ == FTW_DP_V {
        dfd = crate::open(path, 0, 0);
        open_err = ERRNO;
        if dfd < 0 && open_err == 13 { type_ = FTW_DNR_V; }
        if fd_limit == 0 && dfd >= 0 { crate::close(dfd); dfd = -1; }
    }

    if flags & FTW_DEPTH_V == 0 {
        let r = walk_call(cb, path, &st, type_, &mut ftw);
        if r != 0 { return r; }
    }

    let mut p = h;
    while !p.is_null() {
        if (*p).dev == st.st_dev && (*p).ino == st.st_ino { return 0; }
        p = (*p).chain;
    }

    if (type_ == FTW_D_V || type_ == FTW_DP_V) && fd_limit > 0 {
        if dfd < 0 {
            ERRNO = open_err;
            return -1;
        }
        // Enumerate with getdents64 so nested walks are independent.
        #[repr(align(8))]
        struct Buf([u8; 8192]);
        let mut dbuf = Buf([0u8; 8192]);
        let name_off = core::mem::offset_of!(linux_dirent64, d_name);
        loop {
            let n = <Arch as Syscalls>::syscall3(
                SYS_GETDENTS64,
                dfd as i64,
                dbuf.0.as_mut_ptr() as i64,
                dbuf.0.len() as i64,
            ) as isize;
            if n <= 0 { break; }
            let mut off = 0usize;
            while off + name_off < n as usize {
                let ent = dbuf.0.as_ptr().add(off) as *const linux_dirent64;
                let reclen = (*ent).d_reclen as usize;
                if reclen == 0 { break; }
                let name = dbuf.0.as_ptr().add(off + name_off) as *const c_char;
                off += reclen;
                if *name == b'.' as c_char {
                    let s = *name.add(1);
                    if s == 0 || (s == b'.' as c_char && *name.add(2) == 0) { continue; }
                }
                let nl = crate::strlen(name);
                if len + 1 + nl >= cap {
                    ERRNO = 36;
                    crate::close(dfd);
                    return -1;
                }
                *buf.add(len) = b'/';
                core::ptr::copy_nonoverlapping(name as *const u8, buf.add(len + 1), nl + 1);
                let r = nftw_do(buf, cap, len + 1 + nl, cb, fd_limit - 1, flags, &mut new, parent_base, level + 1);
                if r != 0 { crate::close(dfd); return r; }
            }
        }
        crate::close(dfd);
    } else if dfd >= 0 {
        crate::close(dfd);
    }
    *buf.add(len) = 0;

    if flags & FTW_DEPTH_V != 0 {
        let mut f2 = Ftw { base: parent_base, level: level };
        let r = walk_call(cb, path, &st, type_, &mut f2);
        if r != 0 { return r; }
    }
    let _ = &mut new;
    0
}

#[no_mangle]
pub unsafe extern "C" fn nftw(
    path: *const c_char,
    f: Option<unsafe extern "C" fn(*const c_char, *const Stat, c_int, *mut Ftw) -> c_int>,
    fd_limit: c_int,
    flags: c_int,
) -> c_int {
    if path.is_null() || f.is_none() { ERRNO = EINVAL; return -1; }
    if fd_limit <= 0 { return 0; }
    let l = crate::strlen(path);
    if l >= PATH_MAX_V { ERRNO = 36; return -1; }
    let mut buf = [0u8; PATH_MAX_V + 1];
    core::ptr::copy_nonoverlapping(path as *const u8, buf.as_mut_ptr(), l + 1);
    nftw_do(buf.as_mut_ptr(), buf.len(), l, WalkCb::N(f), fd_limit, flags, core::ptr::null_mut(), 0, 0)
}

#[no_mangle]
pub unsafe extern "C" fn ftw(
    path: *const c_char,
    f: Option<unsafe extern "C" fn(*const c_char, *const Stat, c_int) -> c_int>,
    fd_limit: c_int,
) -> c_int {
    if path.is_null() || f.is_none() { ERRNO = EINVAL; return -1; }
    if fd_limit <= 0 { return 0; }
    let l = crate::strlen(path);
    if l >= PATH_MAX_V { ERRNO = 36; return -1; }
    let mut buf = [0u8; PATH_MAX_V + 1];
    core::ptr::copy_nonoverlapping(path as *const u8, buf.as_mut_ptr(), l + 1);
    nftw_do(buf.as_mut_ptr(), buf.len(), l, WalkCb::F(f), fd_limit, FTW_PHYS_V, core::ptr::null_mut(), 0, 0)
}
