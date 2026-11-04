// Parsing of DNS messages for the ns_* API from <arpa/nameser.h>.

pub const NS_MAXDNAME_V: usize = 1025;
const NS_INT16SZ_V: usize = 2;
const NS_INT32SZ_V: usize = 4;
const NS_S_MAX_V: usize = 4;

#[repr(C)]
pub struct NsMsg {
    pub _msg: *const u8,
    pub _eom: *const u8,
    pub _id: u16,
    pub _flags: u16,
    pub _counts: [u16; NS_S_MAX_V],
    pub _sections: [*const u8; NS_S_MAX_V],
    pub _sect: c_int,
    pub _rrnum: c_int,
    pub _msg_ptr: *const u8,
}

#[repr(C)]
pub struct NsRr {
    pub name: [c_char; NS_MAXDNAME_V],
    pub type_: u16,
    pub rr_class: u16,
    pub ttl: u32,
    pub rdlength: u16,
    __pad: u16,
    pub rdata: *const u8,
}

#[no_mangle]
pub static _ns_flagdata: [[c_int; 2]; 16] = [
    [0x8000, 15],
    [0x7800, 11],
    [0x0400, 10],
    [0x0200, 9],
    [0x0100, 8],
    [0x0080, 7],
    [0x0040, 6],
    [0x0020, 5],
    [0x0010, 4],
    [0x000f, 0],
    [0x0000, 0],
    [0x0000, 0],
    [0x0000, 0],
    [0x0000, 0],
    [0x0000, 0],
    [0x0000, 0],
];

unsafe fn ns_skiprr_inner(ptr: *const u8, eom: *const u8, section: c_int, mut count: c_int) -> c_int {
    let mut p = ptr;
    while count > 0 {
        count -= 1;
        let r = dn_skipname(p, eom);
        if r < 0 { return -1; }
        if r as usize + 2 * NS_INT16SZ_V > eom as usize - p as usize { return -1; }
        p = p.add(r as usize + 2 * NS_INT16SZ_V);
        if section != 0 {
            if NS_INT32SZ_V + NS_INT16SZ_V > eom as usize - p as usize { return -1; }
            p = p.add(NS_INT32SZ_V);
            let rdlen = ns_get16(p) as usize;
            p = p.add(NS_INT16SZ_V);
            if rdlen > eom as usize - p as usize { return -1; }
            p = p.add(rdlen);
        }
    }
    (p as usize - ptr as usize) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn ns_skiprr(ptr: *const u8, eom: *const u8, section: c_int, count: c_int) -> c_int {
    let r = ns_skiprr_inner(ptr, eom, section, count);
    if r < 0 { ERRNO = 90; } // EMSGSIZE
    r
}

#[no_mangle]
pub unsafe extern "C" fn ns_initparse(msg: *const u8, msglen: c_int, handle: *mut NsMsg) -> c_int {
    if msg.is_null() || handle.is_null() || msglen < 0 {
        ERRNO = 90;
        return -1;
    }
    (*handle)._msg = msg;
    (*handle)._eom = msg.add(msglen as usize);
    if (msglen as usize) < (2 + NS_S_MAX_V) * NS_INT16SZ_V {
        ERRNO = 90;
        return -1;
    }
    let mut p = msg;
    (*handle)._id = ns_get16(p) as u16;
    p = p.add(2);
    (*handle)._flags = ns_get16(p) as u16;
    p = p.add(2);
    for i in 0..NS_S_MAX_V {
        (*handle)._counts[i] = ns_get16(p) as u16;
        p = p.add(2);
    }
    for i in 0..NS_S_MAX_V {
        if (*handle)._counts[i] != 0 {
            (*handle)._sections[i] = p;
            let r = ns_skiprr_inner(p, (*handle)._eom, i as c_int, (*handle)._counts[i] as c_int);
            if r < 0 {
                ERRNO = 90;
                return -1;
            }
            p = p.add(r as usize);
        } else {
            (*handle)._sections[i] = core::ptr::null();
        }
    }
    if p != (*handle)._eom {
        ERRNO = 90;
        return -1;
    }
    (*handle)._sect = NS_S_MAX_V as c_int;
    (*handle)._rrnum = -1;
    (*handle)._msg_ptr = core::ptr::null();
    0
}

#[no_mangle]
pub unsafe extern "C" fn ns_name_uncompress(
    msg: *const u8,
    eom: *const u8,
    src: *const u8,
    dst: *mut c_char,
    dstsiz: usize,
) -> c_int {
    let r = crate::dn_expand(msg, eom, src, dst, dstsiz as c_int);
    if r < 0 { ERRNO = 90; }
    r
}

#[no_mangle]
pub unsafe extern "C" fn ns_parserr(handle: *mut NsMsg, section: c_int, rrnum_in: c_int, rr: *mut NsRr) -> c_int {
    if handle.is_null() || rr.is_null() || section < 0 || section >= NS_S_MAX_V as c_int {
        ERRNO = 19; // ENODEV
        return -1;
    }
    let sec = section as usize;
    if section != (*handle)._sect {
        (*handle)._sect = section;
        (*handle)._rrnum = 0;
        (*handle)._msg_ptr = (*handle)._sections[sec];
    }
    let mut rrnum = rrnum_in;
    if rrnum == -1 { rrnum = (*handle)._rrnum; }
    if rrnum < 0 || rrnum >= (*handle)._counts[sec] as c_int {
        ERRNO = 19;
        return -1;
    }
    if rrnum < (*handle)._rrnum {
        (*handle)._rrnum = 0;
        (*handle)._msg_ptr = (*handle)._sections[sec];
    }
    if rrnum > (*handle)._rrnum {
        let r = ns_skiprr_inner(
            (*handle)._msg_ptr,
            (*handle)._eom,
            section,
            rrnum - (*handle)._rrnum,
        );
        if r < 0 {
            ERRNO = 90;
            return -1;
        }
        (*handle)._msg_ptr = (*handle)._msg_ptr.add(r as usize);
        (*handle)._rrnum = rrnum;
    }
    let r = ns_name_uncompress(
        (*handle)._msg,
        (*handle)._eom,
        (*handle)._msg_ptr,
        (*rr).name.as_mut_ptr(),
        NS_MAXDNAME_V,
    );
    if r < 0 { return -1; }
    (*handle)._msg_ptr = (*handle)._msg_ptr.add(r as usize);
    if 2 * NS_INT16SZ_V > (*handle)._eom as usize - (*handle)._msg_ptr as usize {
        ERRNO = 90;
        return -1;
    }
    (*rr).type_ = ns_get16((*handle)._msg_ptr) as u16;
    (*handle)._msg_ptr = (*handle)._msg_ptr.add(2);
    (*rr).rr_class = ns_get16((*handle)._msg_ptr) as u16;
    (*handle)._msg_ptr = (*handle)._msg_ptr.add(2);
    if section != 0 {
        if NS_INT32SZ_V + NS_INT16SZ_V > (*handle)._eom as usize - (*handle)._msg_ptr as usize {
            ERRNO = 90;
            return -1;
        }
        (*rr).ttl = ns_get32((*handle)._msg_ptr) as u32;
        (*handle)._msg_ptr = (*handle)._msg_ptr.add(4);
        (*rr).rdlength = ns_get16((*handle)._msg_ptr) as u16;
        (*handle)._msg_ptr = (*handle)._msg_ptr.add(2);
        if (*rr).rdlength as usize > (*handle)._eom as usize - (*handle)._msg_ptr as usize {
            ERRNO = 90;
            return -1;
        }
        (*rr).rdata = (*handle)._msg_ptr;
        (*handle)._msg_ptr = (*handle)._msg_ptr.add((*rr).rdlength as usize);
    } else {
        (*rr).ttl = 0;
        (*rr).rdlength = 0;
        (*rr).rdata = core::ptr::null();
    }
    (*handle)._rrnum += 1;
    if (*handle)._rrnum > (*handle)._counts[sec] as c_int {
        (*handle)._sect = section + 1;
        if (*handle)._sect == NS_S_MAX_V as c_int {
            (*handle)._rrnum = -1;
            (*handle)._msg_ptr = core::ptr::null();
        } else {
            (*handle)._rrnum = 0;
        }
    }
    0
}
