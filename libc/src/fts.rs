// A real fts(3) implementation. The tree is scanned depth-first on the first
// fts_read() call into a preorder array; fts_read() then walks that array.

pub const FTS_COMFOLLOW_V: c_int = 0x001;
pub const FTS_LOGICAL_V: c_int = 0x002;
pub const FTS_NOCHDIR_V: c_int = 0x004;
pub const FTS_NOSTAT_V: c_int = 0x008;
pub const FTS_PHYSICAL_V: c_int = 0x010;
pub const FTS_SEEDOT_V: c_int = 0x020;
pub const FTS_XDEV_V: c_int = 0x040;

pub const FTS_D_V: u16 = 1;
pub const FTS_DC_V: u16 = 2;
pub const FTS_DEFAULT_V: u16 = 3;
pub const FTS_DNR_V: u16 = 4;
pub const FTS_DOT_V: u16 = 5;
pub const FTS_DP_V: u16 = 6;
pub const FTS_ERR_V: u16 = 7;
pub const FTS_F_V: u16 = 8;
pub const FTS_INIT_V: u16 = 9;
pub const FTS_NS_V: u16 = 10;
pub const FTS_NSOK_V: u16 = 11;
pub const FTS_SL_V: u16 = 12;
pub const FTS_SLNONE_V: u16 = 13;
pub const FTS_W_V: u16 = 14;

pub const FTS_SKIP_V: c_int = 1;
pub const FTS_FOLLOW_V: c_int = 2;
pub const FTS_NOINSTR_V: c_int = 3;

const FTS_NAME_OFF: usize = 112;

#[repr(C)]
pub struct FtsEnt {
    pub fts_cycle: *mut FtsEnt,
    pub fts_parent: *mut FtsEnt,
    pub fts_link: *mut FtsEnt,
    pub fts_number: c_long,
    pub fts_pointer: *mut c_void,
    pub fts_accpath: *mut c_char,
    pub fts_path: *mut c_char,
    pub fts_errno: c_int,
    pub fts_symfd: c_int,
    pub fts_pathlen: u16,
    pub fts_namelen: u16,
    pub fts_ino: u64,
    pub fts_dev: u64,
    pub fts_nlink: u64,
    pub fts_level: u16,
    pub fts_info: u16,
    pub fts_flags: u16,
    pub fts_instr: u16,
    pub fts_statp: *mut Stat,
    pub fts_name: [c_char; 1],
}

const _: [(); 120] = [(); core::mem::size_of::<FtsEnt>()];

pub struct Fts {
    roots: *mut *mut c_char,
    nroots: usize,
    options: c_int,
    compar: Option<unsafe extern "C" fn(*const *const FtsEnt, *const *const FtsEnt) -> c_int>,
    ents: *mut *mut FtsEnt,
    nents: usize,
    ents_cap: usize,
    idx: usize,
    built: bool,
}

unsafe fn fts_name_ptr(ent: *mut FtsEnt) -> *mut c_char {
    (ent as *mut u8).add(FTS_NAME_OFF) as *mut c_char
}

unsafe fn fts_ent_alloc(
    path: *const c_char,
    name: *const c_char,
    level: u16,
    parent: *mut FtsEnt,
) -> *mut FtsEnt {
    let plen = crate::strlen(path);
    let nlen = crate::strlen(name);
    let namelen = nlen + 1;
    let stat_off = (core::mem::size_of::<FtsEnt>() + namelen + 7) & !7usize;
    let total = stat_off + core::mem::size_of::<Stat>();
    let raw = crate::calloc(1, total) as *mut u8;
    if raw.is_null() { return core::ptr::null_mut(); }
    let ent = raw as *mut FtsEnt;
    (*ent).fts_parent = parent;
    (*ent).fts_level = level;
    (*ent).fts_pathlen = plen as u16;
    (*ent).fts_namelen = nlen as u16;
    let pathbuf = crate::malloc(plen + 1) as *mut c_char;
    if pathbuf.is_null() { crate::free(raw as *mut c_void); return core::ptr::null_mut(); }
    core::ptr::copy_nonoverlapping(path, pathbuf, plen + 1);
    (*ent).fts_path = pathbuf;
    (*ent).fts_accpath = pathbuf;
    core::ptr::copy_nonoverlapping(name, fts_name_ptr(ent), namelen);
    (*ent).fts_statp = raw.add(stat_off) as *mut Stat;
    ent
}

unsafe fn fts_push(fts: *mut Fts, ent: *mut FtsEnt) {
    if ent.is_null() { return; }
    if (*fts).nents == (*fts).ents_cap {
        let ncap = if (*fts).ents_cap == 0 { 64 } else { (*fts).ents_cap * 2 };
        let na = crate::realloc(
            (*fts).ents as *mut c_void,
            ncap * core::mem::size_of::<*mut FtsEnt>(),
        ) as *mut *mut FtsEnt;
        if na.is_null() { return; }
        (*fts).ents = na;
        (*fts).ents_cap = ncap;
    }
    *(*fts).ents.add((*fts).nents) = ent;
    (*fts).nents += 1;
    (*ent).fts_link = core::ptr::null_mut();
}

unsafe fn fts_free_ent(ent: *mut FtsEnt) {
    if ent.is_null() { return; }
    if !(*ent).fts_path.is_null() { crate::free((*ent).fts_path as *mut c_void); }
    crate::free(ent as *mut c_void);
}

unsafe fn fts_name_of(path: *const c_char) -> *const c_char {
    let n = crate::strlen(path);
    let mut i = n;
    while i > 0 {
        if *path.add(i - 1) == b'/' as c_char { break; }
        i -= 1;
    }
    path.add(i)
}

unsafe fn fts_join(dir: *const c_char, name: *const c_char) -> *mut c_char {
    let dl = crate::strlen(dir);
    let nl = crate::strlen(name);
    let need_sep = dl > 0 && *dir.add(dl - 1) != b'/' as c_char;
    let total = dl + if need_sep { 1 } else { 0 } + nl + 1;
    let p = crate::malloc(total) as *mut c_char;
    if p.is_null() { return core::ptr::null_mut(); }
    core::ptr::copy_nonoverlapping(dir, p, dl);
    let mut o = dl;
    if need_sep { *p.add(o) = b'/' as c_char; o += 1; }
    core::ptr::copy_nonoverlapping(name, p.add(o), nl + 1);
    p
}

unsafe fn fts_is_dir(mode: u32) -> bool { mode & 0o170000 == 0o040000 }
unsafe fn fts_is_link(mode: u32) -> bool { mode & 0o170000 == 0o120000 }

unsafe fn fts_stat_path(path: *const c_char, follow: bool, st: *mut Stat) -> c_int {
    if follow { crate::stat(path, st) } else { crate::lstat(path, st as *mut c_void) }
}

unsafe fn fts_collect_names(path: *const c_char) -> (*mut *mut c_char, usize) {
    let fd = crate::open(path, crate::O_DIRECTORY, 0); // O_RDONLY|O_DIRECTORY
    if fd < 0 { return (core::ptr::null_mut(), 0); }
    let mut cap = 16usize;
    let mut count = 0usize;
    let mut arr = crate::malloc(cap * core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
    if arr.is_null() { crate::close(fd); return (core::ptr::null_mut(), 0); }
    let mut buf = [0u8; 4096];
    loop {
        let n = <Arch as Syscalls>::syscall3(
            SYS_GETDENTS64,
            fd as i64,
            buf.as_mut_ptr() as i64,
            buf.len() as i64,
        ) as isize;
        if n <= 0 { break; }
        let mut pos = 0usize;
        while pos < n as usize {
            let ent = buf.as_ptr().add(pos) as *const linux_dirent64;
            let reclen = (*ent).d_reclen as usize;
            if reclen == 0 { break; }
            let namep = (ent as *const u8).add(core::mem::offset_of!(linux_dirent64, d_name)) as *const c_char;
            let nl = crate::strlen(namep);
            if count + 2 >= cap {
                cap *= 2;
                let na = crate::realloc(arr as *mut c_void, cap * core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
                if na.is_null() { break; }
                arr = na;
            }
            let copy = crate::malloc(nl + 1) as *mut c_char;
            if copy.is_null() { break; }
            core::ptr::copy_nonoverlapping(namep, copy, nl + 1);
            *arr.add(count) = copy;
            count += 1;
            pos += reclen;
        }
    }
    crate::close(fd);
    (arr, count)
}

unsafe fn fts_free_names(arr: *mut *mut c_char, count: usize) {
    if arr.is_null() { return; }
    for i in 0..count {
        if !(*arr.add(i)).is_null() { crate::free(*arr.add(i) as *mut c_void); }
    }
    crate::free(arr as *mut c_void);
}

unsafe fn fts_sort_entries(fts: *mut Fts, arr: *mut *mut FtsEnt, count: usize) {
    let cmp = match (*fts).compar { Some(c) => c, None => return };
    for i in 1..count {
        let v = *arr.add(i);
        let mut j = i;
        while j > 0 {
            let prev = *arr.add(j - 1);
            let a = prev as *const FtsEnt;
            let b = v as *const FtsEnt;
            if cmp(&a, &b) <= 0 { break; }
            *arr.add(j) = prev;
            j -= 1;
        }
        *arr.add(j) = v;
    }
}

unsafe fn fts_fill_info(ent: *mut FtsEnt, st: *mut Stat) -> u16 {
    if (*ent).fts_errno != 0 { return FTS_NS_V; }
    let mode = (*st).st_mode;
    if fts_is_dir(mode) { FTS_D_V }
    else if fts_is_link(mode) { FTS_SL_V }
    else if mode & 0o170000 == 0o100000 { FTS_F_V }
    else { FTS_DEFAULT_V }
}

unsafe fn fts_walk(fts: *mut Fts, ent: *mut FtsEnt) {
    fts_push(fts, ent);
    if (*ent).fts_level >= 64 || (*ent).fts_info != FTS_D_V { return; }
    let mut p = (*ent).fts_parent;
    while !p.is_null() {
        if (*p).fts_info == FTS_D_V && (*p).fts_dev == (*ent).fts_dev && (*p).fts_ino == (*ent).fts_ino {
            (*ent).fts_info = FTS_DC_V;
            return;
        }
        p = (*p).fts_parent;
    }

    let (names, count) = fts_collect_names((*ent).fts_path);
    if !names.is_null() {
        let ptrsz = core::mem::size_of::<*mut c_char>();
        let kids = crate::malloc((count + 1) * ptrsz) as *mut *mut FtsEnt;
        if !kids.is_null() {
            let mut nkids = 0usize;
            for i in 0..count {
                let name = *names.add(i);
                if ((*fts).options & FTS_SEEDOT_V) == 0 {
                    if crate::strcmp(name as *const u8, b".\0".as_ptr()) == 0 { continue; }
                    if crate::strcmp(name as *const u8, b"..\0".as_ptr()) == 0 { continue; }
                }
                let full = fts_join((*ent).fts_path, name);
                if full.is_null() { continue; }
                let child = fts_ent_alloc(full, name, (*ent).fts_level + 1, ent);
                crate::free(full as *mut c_void);
                if child.is_null() { continue; }
                let mut st: Stat = core::mem::zeroed();
                let follow = (*fts).options & FTS_LOGICAL_V != 0;
                let mut rc = fts_stat_path((*child).fts_path, follow, &mut st);
                if rc != 0 && !follow {
                    rc = fts_stat_path((*child).fts_path, true, &mut st);
                }
                if rc != 0 {
                    (*child).fts_errno = ERRNO;
                    (*child).fts_info = FTS_NS_V;
                } else {
                    core::ptr::copy_nonoverlapping(&st as *const Stat as *const u8, (*child).fts_statp as *mut u8, core::mem::size_of::<Stat>());
                    (*child).fts_ino = st.st_ino;
                    (*child).fts_dev = st.st_dev;
                    (*child).fts_nlink = st.st_nlink as u64;
                    (*child).fts_info = fts_fill_info(child, &mut st);
                }
                *kids.add(nkids) = child;
                nkids += 1;
            }
            fts_sort_entries(fts, kids, nkids);
            for i in 0..nkids { fts_walk(fts, *kids.add(i)); }
            crate::free(kids as *mut c_void);
        }
        fts_free_names(names, count);
    }

    let dp = fts_ent_alloc((*ent).fts_path, fts_name_ptr(ent), (*ent).fts_level, (*ent).fts_parent);
    if !dp.is_null() {
        (*dp).fts_info = FTS_DP_V;
        (*dp).fts_dev = (*ent).fts_dev;
        (*dp).fts_ino = (*ent).fts_ino;
        fts_push(fts, dp);
    }
}

unsafe fn fts_build(fts: *mut Fts) {
    (*fts).built = true;
    for i in 0..(*fts).nroots {
        let root = *(*fts).roots.add(i);
        if root.is_null() { continue; }
        let mut st: Stat = core::mem::zeroed();
        let follow = (*fts).options & (FTS_LOGICAL_V | FTS_COMFOLLOW_V) != 0;
        let rc = fts_stat_path(root, follow, &mut st);
        let ent = fts_ent_alloc(root, fts_name_of(root), 0, core::ptr::null_mut());
        if ent.is_null() { continue; }
        if rc != 0 {
            (*ent).fts_errno = ERRNO;
            (*ent).fts_info = FTS_NS_V;
            fts_push(fts, ent);
            continue;
        }
        core::ptr::copy_nonoverlapping(&st as *const Stat as *const u8, (*ent).fts_statp as *mut u8, core::mem::size_of::<Stat>());
        (*ent).fts_ino = st.st_ino;
        (*ent).fts_dev = st.st_dev;
        (*ent).fts_nlink = st.st_nlink as u64;
        (*ent).fts_info = fts_fill_info(ent, &mut st);
        fts_walk(fts, ent);
    }
}

#[no_mangle]
pub unsafe extern "C" fn fts_open(
    path_argv: *mut *mut c_char,
    options: c_int,
    compar: Option<unsafe extern "C" fn(*const *const FtsEnt, *const *const FtsEnt) -> c_int>,
) -> *mut Fts {
    if path_argv.is_null() || (*path_argv).is_null() {
        ERRNO = EINVAL;
        return core::ptr::null_mut();
    }
    let fts = crate::calloc(1, core::mem::size_of::<Fts>()) as *mut Fts;
    if fts.is_null() { ERRNO = ENOMEM; return core::ptr::null_mut(); }
    let mut n = 0usize;
    while !(*path_argv.add(n)).is_null() { n += 1; }
    let roots = crate::malloc((n + 1) * core::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
    if roots.is_null() { crate::free(fts as *mut c_void); ERRNO = ENOMEM; return core::ptr::null_mut(); }
    for i in 0..n {
        let s = *path_argv.add(i);
        let l = crate::strlen(s);
        let copy = crate::malloc(l + 1) as *mut c_char;
        if copy.is_null() { ERRNO = ENOMEM; crate::free(roots as *mut c_void); crate::free(fts as *mut c_void); return core::ptr::null_mut(); }
        core::ptr::copy_nonoverlapping(s, copy, l + 1);
        *roots.add(i) = copy;
    }
    *roots.add(n) = core::ptr::null_mut();
    (*fts).roots = roots;
    (*fts).nroots = n;
    (*fts).options = options;
    (*fts).compar = compar;
    (*fts).ents = core::ptr::null_mut();
    (*fts).nents = 0;
    (*fts).ents_cap = 0;
    (*fts).idx = 0;
    (*fts).built = false;
    fts
}

#[no_mangle]
pub unsafe extern "C" fn fts_read(fts: *mut Fts) -> *mut FtsEnt {
    if fts.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    if !(*fts).built { fts_build(fts); }
    if (*fts).idx < (*fts).nents {
        let e = *(*fts).ents.add((*fts).idx);
        (*fts).idx += 1;
        return e;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn fts_set(fts: *mut Fts, ent: *mut FtsEnt, instr: c_int) -> c_int {
    if fts.is_null() || ent.is_null() { ERRNO = EINVAL; return -1; }
    match instr {
        FTS_SKIP_V => {
            (*ent).fts_instr = FTS_SKIP_V as u16;
            if (*ent).fts_info == FTS_D_V {
                let lvl = (*ent).fts_level;
                while (*fts).idx < (*fts).nents {
                    let e = *(*fts).ents.add((*fts).idx);
                    if (*e).fts_level < lvl { break; }
                    (*fts).idx += 1;
                    if (*e).fts_level == lvl && (*e).fts_info == FTS_DP_V { break; }
                }
                (*ent).fts_instr = FTS_SKIP_V as u16;
            }
        }
        FTS_FOLLOW_V => {
            (*ent).fts_instr = FTS_FOLLOW_V as u16;
            if (*ent).fts_info == FTS_SL_V || (*ent).fts_info == FTS_SLNONE_V {
                let mut st: Stat = core::mem::zeroed();
                if crate::stat((*ent).fts_path, &mut st) == 0 {
                    core::ptr::copy_nonoverlapping(&st as *const Stat as *const u8, (*ent).fts_statp as *mut u8, core::mem::size_of::<Stat>());
                    (*ent).fts_ino = st.st_ino;
                    (*ent).fts_dev = st.st_dev;
                    (*ent).fts_nlink = st.st_nlink as u64;
                    (*ent).fts_info = fts_fill_info(ent, &mut st);
                }
            }
        }
        FTS_NOINSTR_V => { (*ent).fts_instr = FTS_NOINSTR_V as u16; }
        _ => { ERRNO = EINVAL; return -1; }
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn fts_children(fts: *mut Fts, _instr: c_int) -> *mut FtsEnt {
    if fts.is_null() { ERRNO = EINVAL; return core::ptr::null_mut(); }
    if !(*fts).built { fts_build(fts); }
    if (*fts).idx == 0 { return core::ptr::null_mut(); }
    let parent = *(*fts).ents.add((*fts).idx - 1);
    if (*parent).fts_info != FTS_D_V { return core::ptr::null_mut(); }
    let level = (*parent).fts_level + 1;
    let mut head: *mut FtsEnt = core::ptr::null_mut();
    let mut tail: *mut FtsEnt = core::ptr::null_mut();
    let mut i = (*fts).idx;
    while i < (*fts).nents {
        let q = *(*fts).ents.add(i);
        if (*q).fts_level < level { break; }
        if (*q).fts_level == level && (*q).fts_info != FTS_DP_V {
            (*q).fts_link = core::ptr::null_mut();
            if head.is_null() { head = q; } else { (*tail).fts_link = q; }
            tail = q;
        }
        i += 1;
    }
    head
}

#[no_mangle]
pub unsafe extern "C" fn fts_close(fts: *mut Fts) -> c_int {
    if fts.is_null() { ERRNO = EINVAL; return -1; }
    for i in 0..(*fts).nents {
        fts_free_ent(*(*fts).ents.add(i));
    }
    if !(*fts).ents.is_null() { crate::free((*fts).ents as *mut c_void); }
    for i in 0..(*fts).nroots {
        if !(*(*fts).roots.add(i)).is_null() { crate::free(*(*fts).roots.add(i) as *mut c_void); }
    }
    crate::free((*fts).roots as *mut c_void);
    crate::free(fts as *mut c_void);
    0
}
