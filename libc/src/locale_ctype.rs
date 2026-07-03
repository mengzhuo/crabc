#[no_mangle]
pub extern "C" fn toascii(c: c_int) -> c_int {
    c & 0x7f
}

#[no_mangle]
pub extern "C" fn isalnum_l(c: c_int, _loc: locale_t) -> c_int {
    isalnum(c)
}

#[no_mangle]
pub extern "C" fn isalpha_l(c: c_int, _loc: locale_t) -> c_int {
    isalpha(c)
}

#[no_mangle]
pub extern "C" fn isblank_l(c: c_int, _loc: locale_t) -> c_int {
    isblank(c)
}

#[no_mangle]
pub extern "C" fn iscntrl_l(c: c_int, _loc: locale_t) -> c_int {
    iscntrl(c)
}

#[no_mangle]
pub extern "C" fn isdigit_l(c: c_int, _loc: locale_t) -> c_int {
    isdigit(c)
}

#[no_mangle]
pub extern "C" fn isgraph_l(c: c_int, _loc: locale_t) -> c_int {
    isgraph(c)
}

#[no_mangle]
pub extern "C" fn islower_l(c: c_int, _loc: locale_t) -> c_int {
    islower(c)
}

#[no_mangle]
pub extern "C" fn isprint_l(c: c_int, _loc: locale_t) -> c_int {
    isprint(c)
}

#[no_mangle]
pub extern "C" fn ispunct_l(c: c_int, _loc: locale_t) -> c_int {
    ispunct(c)
}

#[no_mangle]
pub extern "C" fn isspace_l(c: c_int, _loc: locale_t) -> c_int {
    isspace(c)
}

#[no_mangle]
pub extern "C" fn isupper_l(c: c_int, _loc: locale_t) -> c_int {
    isupper(c)
}

#[no_mangle]
pub extern "C" fn isxdigit_l(c: c_int, _loc: locale_t) -> c_int {
    isxdigit(c)
}

#[no_mangle]
pub extern "C" fn tolower_l(c: c_int, _loc: locale_t) -> c_int {
    tolower(c)
}

#[no_mangle]
pub extern "C" fn toupper_l(c: c_int, _loc: locale_t) -> c_int {
    toupper(c)
}
