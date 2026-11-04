// getopt / getopt_long / getopt_long_only (port of musl's implementation).

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LongOption {
    pub name: *const c_char,
    pub has_arg: c_int,
    pub flag: *mut c_int,
    pub val: c_int,
}

#[no_mangle]
pub static mut __optpos: c_int = 0;
#[no_mangle]
pub static mut __optreset: c_int = 0;

unsafe fn getopt_reset_if_needed() {
    if optind == 0 || __optreset != 0 || optreset != 0 {
        __optreset = 0;
        optreset = 0;
        __optpos = 0;
        optind = 1;
    }
}

unsafe fn opt_char_len(s: *const c_char) -> usize {
    // Option characters are bytes; UTF-8 continuation bytes are part of the
    // same option character for the purposes of matching.
    let b = *s as u8;
    if b < 0x80 { 1 }
    else if b >= 0xf0 { 4 }
    else if b >= 0xe0 { 3 }
    else if b >= 0xc0 { 2 }
    else { 1 }
}

#[no_mangle]
pub unsafe extern "C" fn getopt(
    argc: c_int,
    argv: *const *const c_char,
    optstring_in: *const c_char,
) -> c_int {
    getopt_reset_if_needed();
    if argc <= 0 || argv.is_null() || optstring_in.is_null() { return -1; }
    if optind >= argc { return -1; }
    let av = argv.add(optind as usize);
    if (*av).is_null() { return -1; }
    let arg = *av;
    if *arg != b'-' as c_char {
        if *optstring_in == b'-' as c_char {
            optarg = arg as *mut c_char;
            optind += 1;
            return 1;
        }
        return -1;
    }
    if *arg.add(1) == 0 { return -1; }
    if *arg.add(1) == b'-' as c_char && *arg.add(2) == 0 {
        optind += 1;
        return -1;
    }
    if __optpos == 0 { __optpos = 1; }
    let clen = opt_char_len(arg.add(__optpos as usize));
    let c = *arg.add(__optpos as usize) as u8;
    let optchar = arg.add(__optpos as usize);
    __optpos += clen as c_int;
    if *arg.add(__optpos as usize) == 0 {
        optind += 1;
        __optpos = 0;
    }

    let mut spec = optstring_in;
    if *spec == b'-' as c_char || *spec == b'+' as c_char { spec = spec.add(1); }

    let mut i = 0usize;
    let mut found = false;
    while *spec.add(i) != 0 {
        if *spec.add(i) as u8 == c { found = true; break; }
        i += 1;
    }
    if !found || c == b':' {
        optopt = c as c_int;
        if *spec != b':' as c_char && opterr != 0 {
            let mut msg = [0 as c_char; 256];
            let prog = if !(*argv).is_null() { *argv } else { b"\0".as_ptr() as *const c_char };
            crate::fprintf(
                crate::stderr,
                b"%s: unrecognized option: %.*s\n\0".as_ptr() as *const c_char,
                prog,
                clen as c_int,
                optchar,
            );
            let _ = &mut msg;
        }
        return b'?' as c_int;
    }
    if *spec.add(i + 1) == b':' as c_char {
        optarg = core::ptr::null_mut();
        if *spec.add(i + 2) != b':' as c_char || __optpos != 0 {
            if optind >= argc {
                optopt = c as c_int;
                if *spec == b':' as c_char { return b':' as c_int; }
                if opterr != 0 {
                    let prog = if !(*argv).is_null() { *argv } else { b"\0".as_ptr() as *const c_char };
                    crate::fprintf(
                        crate::stderr,
                        b"%s: option requires an argument: %.*s\n\0".as_ptr() as *const c_char,
                        prog,
                        clen as c_int,
                        optchar,
                    );
                }
                return b'?' as c_int;
            }
            optarg = *argv.add(optind as usize) as *mut c_char;
            optind += 1;
            if __optpos != 0 { optarg = optarg.add(__optpos as usize); }
            __optpos = 0;
        }
    }
    c as c_int
}

unsafe fn getopt_permute(argv: *mut *mut c_char, dest: isize, src: isize) {
    let tmp = *argv.offset(src);
    let mut i = src;
    while i > dest {
        *argv.offset(i) = *argv.offset(i - 1);
        i -= 1;
    }
    *argv.offset(dest) = tmp;
}

unsafe fn getopt_long_core(
    argc: c_int,
    argv: *const *const c_char,
    optstring: *const c_char,
    longopts: *const LongOption,
    idx: *mut c_int,
    longonly: bool,
) -> c_int {
    optarg = core::ptr::null_mut();
    let cur = *argv.add(optind as usize);
    let is_long = !longopts.is_null()
        && *cur == b'-' as c_char
        && ((longonly && *cur.add(1) != 0 && *cur.add(1) != b'-' as c_char)
            || (*cur.add(1) == b'-' as c_char && *cur.add(2) != 0));
    if is_long {
        let colon = *optstring.add(if *optstring == b'+' as c_char || *optstring == b'-' as c_char { 1 } else { 0 }) == b':' as c_char;
        let start = cur.add(1);
        let mut cnt = 0;
        let mut match_idx = 0usize;
        let mut arg: *const c_char = core::ptr::null();
        let mut i = 0usize;
        loop {
            let opt = (*longopts.add(i)).name;
            if opt.is_null() { break; }
            let mut o = start;
            if *o == b'-' as c_char { o = o.add(1); }
            let mut n = opt;
            while *o != 0 && *o != b'=' as c_char && *o == *n {
                o = o.add(1);
                n = n.add(1);
            }
            if *o != 0 && *o != b'=' as c_char {
                i += 1;
                continue;
            }
            arg = o;
            match_idx = i;
            if *n == 0 { cnt = 1; break; }
            cnt += 1;
            i += 1;
        }
        let _ = arg;
        if cnt == 1 {
            let opt = arg;
            optind += 1;
            let entry = *longopts.add(match_idx);
            if *opt == b'=' as c_char {
                if entry.has_arg == 0 {
                    optopt = entry.val;
                    if colon || opterr == 0 { return b'?' as c_int; }
                    crate::fprintf(
                        crate::stderr,
                        b"%s: option does not take an argument: %s\n\0".as_ptr() as *const c_char,
                        *argv,
                        entry.name,
                    );
                    return b'?' as c_int;
                }
                optarg = opt.add(1) as *mut c_char;
            } else if entry.has_arg == 1 {
                if optind >= argc {
                    optopt = entry.val;
                    if colon { return b':' as c_int; }
                    if opterr == 0 { return b'?' as c_int; }
                    crate::fprintf(
                        crate::stderr,
                        b"%s: option requires an argument: %s\n\0".as_ptr() as *const c_char,
                        *argv,
                        entry.name,
                    );
                    return b'?' as c_int;
                }
                optarg = *argv.add(optind as usize) as *mut c_char;
                optind += 1;
            }
            if !idx.is_null() { *idx = match_idx as c_int; }
            if !entry.flag.is_null() {
                *entry.flag = entry.val;
                return 0;
            }
            return entry.val;
        }
        if *cur.add(1) == b'-' as c_char {
            optopt = 0;
            if !colon && opterr != 0 {
                crate::fprintf(
                    crate::stderr,
                    if cnt != 0 {
                        b"%s: option is ambiguous: %s\n\0".as_ptr() as *const c_char
                    } else {
                        b"%s: unrecognized option: %s\n\0".as_ptr() as *const c_char
                    },
                    *argv,
                    cur.add(2),
                );
            }
            optind += 1;
            return b'?' as c_int;
        }
    }
    getopt(argc, argv, optstring)
}

#[no_mangle]
pub unsafe extern "C" fn getopt_long(
    argc: c_int,
    argv: *const *const c_char,
    optstring: *const c_char,
    longopts: *const LongOption,
    idx: *mut c_int,
) -> c_int {
    getopt_reset_if_needed();
    if argc <= 0 || argv.is_null() { return -1; }
    if optind >= argc || (*argv.add(optind as usize)).is_null() { return -1; }
    let skipped = optind;
    if *optstring != b'+' as c_char && *optstring != b'-' as c_char {
        let mut i = optind;
        loop {
            if i >= argc || (*argv.add(i as usize)).is_null() { return -1; }
            let a = *argv.add(i as usize);
            if *a == b'-' as c_char && *a.add(1) != 0 { break; }
            i += 1;
        }
        optind = i;
    }
    let resumed = optind;
    let ret = getopt_long_core(argc, argv, optstring, longopts, idx, false);
    if resumed > skipped {
        let cnt = optind - resumed;
        let mut i = 0;
        while i < cnt {
            getopt_permute(argv as *mut *mut c_char, skipped as isize, (optind - 1) as isize);
            i += 1;
        }
        optind = skipped + cnt;
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn getopt_long_only(
    argc: c_int,
    argv: *const *const c_char,
    optstring: *const c_char,
    longopts: *const LongOption,
    idx: *mut c_int,
) -> c_int {
    getopt_reset_if_needed();
    if argc <= 0 || argv.is_null() { return -1; }
    if optind >= argc || (*argv.add(optind as usize)).is_null() { return -1; }
    let skipped = optind;
    if *optstring != b'+' as c_char && *optstring != b'-' as c_char {
        let mut i = optind;
        loop {
            if i >= argc || (*argv.add(i as usize)).is_null() { return -1; }
            let a = *argv.add(i as usize);
            if *a == b'-' as c_char && *a.add(1) != 0 { break; }
            i += 1;
        }
        optind = i;
    }
    let resumed = optind;
    let ret = getopt_long_core(argc, argv, optstring, longopts, idx, true);
    if resumed > skipped {
        let cnt = optind - resumed;
        let mut i = 0;
        while i < cnt {
            getopt_permute(argv as *mut *mut c_char, skipped as isize, (optind - 1) as isize);
            i += 1;
        }
        optind = skipped + cnt;
    }
    ret
}
