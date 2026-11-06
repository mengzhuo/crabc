// Network database backends: /etc/services, /etc/protocols, /etc/networks,
// /etc/ethers and /etc/hosts, plus getaddrinfo/getnameinfo.

#[repr(C)]
pub struct ServEnt {
    pub s_name: *mut c_char,
    pub s_aliases: *mut *mut c_char,
    pub s_port: c_int,
    pub s_proto: *mut c_char,
}

#[repr(C)]
pub struct ProtoEnt {
    pub p_name: *mut c_char,
    pub p_aliases: *mut *mut c_char,
    pub p_proto: c_int,
}

#[repr(C)]
pub struct NetEnt {
    pub n_name: *mut c_char,
    pub n_aliases: *mut *mut c_char,
    pub n_addrtype: c_int,
    pub n_net: u32,
}

const NDB_ENOENT: c_int = 2;
const NDB_ERANGE: c_int = 34;
const NDB_ENOTTY: c_int = 25;

const EAI_BADFLAGS_V: c_int = -1;
const EAI_NONAME_V: c_int = -2;
const EAI_AGAIN_V: c_int = -3;
const EAI_FAIL_V: c_int = -4;
const EAI_FAMILY_V: c_int = -6;
const EAI_SOCKTYPE_V: c_int = -7;
const EAI_SERVICE_V: c_int = -8;
const EAI_MEMORY_V: c_int = -10;
const EAI_SYSTEM_V: c_int = -11;
const EAI_OVERFLOW_V: c_int = -12;

const AI_PASSIVE_V: c_int = 0x01;
const AI_CANONNAME_V: c_int = 0x02;
const AI_NUMERICHOST_V: c_int = 0x04;
const AI_V4MAPPED_V: c_int = 0x08;
const AI_ALL_V: c_int = 0x10;
const AI_ADDRCONFIG_V: c_int = 0x20;
const AI_NUMERICSERV_V: c_int = 0x400;

const NI_NUMERICHOST_V: c_int = 0x01;
const NI_NUMERICSERV_V: c_int = 0x02;
const NI_NOFQDN_V: c_int = 0x04;
const NI_NAMEREQD_V: c_int = 0x08;
const NI_DGRAM_V: c_int = 0x10;
const NI_NUMERICSCOPE_V: c_int = 0x100;

unsafe fn nd_isspace(c: c_char) -> bool {
    c == b' ' as c_char || c == b'\t' as c_char || c == b'\r' as c_char || c == b'\n' as c_char
}

unsafe fn nd_copy_line(src: *const c_char, dst: *mut c_char, cap: usize) -> bool {
    let mut i = 0usize;
    while i + 1 < cap {
        let c = *src.add(i);
        *dst.add(i) = c;
        if c == 0 { return true; }
        i += 1;
    }
    *dst.add(cap - 1) = 0;
    true
}

/// Split a mutable line into whitespace-separated tokens, honouring '#'.
unsafe fn nd_split(buf: *mut c_char, toks: *mut *mut c_char, maxtok: usize) -> usize {
    let mut n = 0usize;
    let mut p = buf;
    loop {
        while *p != 0 && nd_isspace(*p) { p = p.add(1); }
        if *p == 0 || *p == b'#' as c_char { break; }
        if n < maxtok {
            *toks.add(n) = p;
            n += 1;
        }
        while *p != 0 && !nd_isspace(*p) && *p != b'#' as c_char { p = p.add(1); }
        if *p == 0 { break; }
        if *p == b'#' as c_char { *p = 0; break; }
        *p = 0;
        p = p.add(1);
    }
    n
}

unsafe fn nd_tokens(line: *const c_char, buf: *mut c_char, cap: usize, toks: &mut [*mut c_char; 24]) -> usize {
    if !nd_copy_line(line, buf, cap) { return 0; }
    nd_split(buf, toks.as_mut_ptr(), toks.len())
}

// ------------------------------------------------------------------
// /etc/services
// ------------------------------------------------------------------

static mut SERV_FILE: *mut FILE = core::ptr::null_mut();
static mut SERV_ENT: ServEnt = ServEnt {
    s_name: core::ptr::null_mut(),
    s_aliases: core::ptr::null_mut(),
    s_port: 0,
    s_proto: core::ptr::null_mut(),
};
static mut SERV_BUF: [c_char; 1024] = [0; 1024];
static mut SERV_ALIASES: [*mut c_char; 16] = [core::ptr::null_mut(); 16];

unsafe fn serv_parse(line: *const c_char) -> bool {
    let mut toks: [*mut c_char; 24] = [core::ptr::null_mut(); 24];
    let n = nd_tokens(line, SERV_BUF.as_mut_ptr(), SERV_BUF.len(), &mut toks);
    if n < 2 { return false; }
    let mut p = toks[1];
    let port = crate::strtoul(p, &mut p, 10);
    if p == toks[1] || port > 65535 { return false; }
    let mut proto: *mut c_char = b"tcp\0".as_ptr() as *mut c_char;
    if *p == b'/' as c_char {
        p = p.add(1);
        if *p == 0 { return false; }
        proto = p;
    }
    SERV_ENT.s_name = toks[0];
    SERV_ENT.s_port = htons(port as u16) as c_int;
    SERV_ENT.s_proto = proto;
    let mut ai = 0usize;
    for i in 2..n {
        if ai + 1 >= SERV_ALIASES.len() { break; }
        SERV_ALIASES[ai] = toks[i];
        ai += 1;
    }
    SERV_ALIASES[ai] = core::ptr::null_mut();
    SERV_ENT.s_aliases = SERV_ALIASES.as_mut_ptr();
    true
}

unsafe fn serv_next(f: *mut FILE) -> bool {
    let mut line = [0 as c_char; 1024];
    while !crate::fgets(line.as_mut_ptr(), line.len() as c_int, f).is_null() {
        if serv_parse(line.as_ptr()) { return true; }
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn setservent(stayopen: c_int) {
    endservent();
    SERV_FILE = crate::fopen(b"/etc/services\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    let _ = stayopen;
}

#[no_mangle]
pub unsafe extern "C" fn endservent() {
    if !SERV_FILE.is_null() {
        crate::fclose(SERV_FILE);
        SERV_FILE = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn getservent() -> *mut ServEnt {
    if SERV_FILE.is_null() {
        SERV_FILE = crate::fopen(b"/etc/services\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
        if SERV_FILE.is_null() { return core::ptr::null_mut(); }
    }
    if serv_next(SERV_FILE) { &raw mut SERV_ENT } else { core::ptr::null_mut() }
}

unsafe fn serv_name_matches(name: *const c_char) -> bool {
    if crate::strcmp(SERV_ENT.s_name as *const u8, name as *const u8) == 0 { return true; }
    let mut i = 0usize;
    while !(*SERV_ENT.s_aliases.add(i)).is_null() {
        if crate::strcmp(*SERV_ENT.s_aliases.add(i) as *const u8, name as *const u8) == 0 {
            return true;
        }
        i += 1;
    }
    false
}

unsafe fn serv_proto_matches(proto: *const c_char) -> bool {
    proto.is_null() || crate::strcmp(SERV_ENT.s_proto as *const u8, proto as *const u8) == 0
}

unsafe fn serv_search(name: *const c_char, proto: *const c_char, port: c_int) -> bool {
    let f = crate::fopen(b"/etc/services\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    if f.is_null() { return false; }
    let mut found = false;
    while serv_next(f) {
        if !serv_proto_matches(proto) { continue; }
        let hit = if name.is_null() {
            SERV_ENT.s_port == port
        } else {
            serv_name_matches(name)
        };
        if hit { found = true; break; }
    }
    crate::fclose(f);
    found
}

#[no_mangle]
pub unsafe extern "C" fn getservbyname(name: *const c_char, proto: *const c_char) -> *mut ServEnt {
    if name.is_null() { return core::ptr::null_mut(); }
    if serv_search(name, proto, 0) { &raw mut SERV_ENT } else { core::ptr::null_mut() }
}

#[no_mangle]
pub unsafe extern "C" fn getservbyport(port: c_int, proto: *const c_char) -> *mut ServEnt {
    if serv_search(core::ptr::null(), proto, port) { &raw mut SERV_ENT } else { core::ptr::null_mut() }
}

unsafe fn serv_copy_out(se: *mut ServEnt, buf: *mut c_char, buflen: usize, res: *mut *mut ServEnt) -> c_int {
    if res.is_null() || se.is_null() { return EINVAL; }
    *res = core::ptr::null_mut();
    let ptrsz = core::mem::size_of::<*mut c_char>();
    let mut need = crate::strlen((*se).s_name) + 1 + crate::strlen((*se).s_proto) + 1;
    let mut nalias = 0usize;
    while !(*(*se).s_aliases.add(nalias)).is_null() {
        need += crate::strlen(*(*se).s_aliases.add(nalias)) + 1;
        nalias += 1;
    }
    let align = (ptrsz - (buf as usize & (ptrsz - 1))) & (ptrsz - 1);
    need += align + (nalias + 1) * ptrsz;
    if need > buflen { return NDB_ERANGE; }

    let mut p = buf.add(align);
    let aliases = p as *mut *mut c_char;
    p = p.add((nalias + 1) * ptrsz);

    let n = crate::strlen((*se).s_name);
    core::ptr::copy_nonoverlapping((*se).s_name, p, n + 1);
    (*se).s_name = p;
    p = p.add(n + 1);

    let n = crate::strlen((*se).s_proto);
    core::ptr::copy_nonoverlapping((*se).s_proto, p, n + 1);
    (*se).s_proto = p;
    p = p.add(n + 1);

    for j in 0..nalias {
        let src = *(*se).s_aliases.add(j);
        let n = crate::strlen(src);
        core::ptr::copy_nonoverlapping(src, p, n + 1);
        *aliases.add(j) = p;
        p = p.add(n + 1);
    }
    *aliases.add(nalias) = core::ptr::null_mut();
    (*se).s_aliases = aliases;
    *res = se;
    0
}

#[no_mangle]
pub unsafe extern "C" fn getservbyname_r(
    name: *const c_char,
    proto: *const c_char,
    _se: *mut ServEnt,
    buf: *mut c_char,
    buflen: usize,
    res: *mut *mut ServEnt,
) -> c_int {
    if res.is_null() { return EINVAL; }
    *res = core::ptr::null_mut();
    let p = getservbyname(name, proto);
    if p.is_null() { return NDB_ENOENT; }
    let mut tmp = ServEnt { s_name: (*p).s_name, s_aliases: (*p).s_aliases, s_port: (*p).s_port, s_proto: (*p).s_proto };
    serv_copy_out(&mut tmp, buf, buflen, res)
}

#[no_mangle]
pub unsafe extern "C" fn getservbyport_r(
    port: c_int,
    proto: *const c_char,
    _se: *mut ServEnt,
    buf: *mut c_char,
    buflen: usize,
    res: *mut *mut ServEnt,
) -> c_int {
    if res.is_null() { return EINVAL; }
    *res = core::ptr::null_mut();
    let p = getservbyport(port, proto);
    if p.is_null() { return NDB_ENOENT; }
    let mut tmp = ServEnt { s_name: (*p).s_name, s_aliases: (*p).s_aliases, s_port: (*p).s_port, s_proto: (*p).s_proto };
    serv_copy_out(&mut tmp, buf, buflen, res)
}

// ------------------------------------------------------------------
// /etc/protocols (with a builtin fallback table)
// ------------------------------------------------------------------

static PROTO_TABLE: [(&[u8], c_int); 36] = [
    (b"ip\0", 0), (b"icmp\0", 1), (b"igmp\0", 2), (b"ggp\0", 3),
    (b"ipencap\0", 4), (b"st\0", 5), (b"tcp\0", 6), (b"egp\0", 8),
    (b"pup\0", 12), (b"udp\0", 17), (b"hmp\0", 20), (b"xns-idp\0", 22),
    (b"rdp\0", 27), (b"iso-tp4\0", 29), (b"xtp\0", 36), (b"ddp\0", 37),
    (b"idpr-cmtp\0", 38), (b"ipv6\0", 41), (b"ipv6-route\0", 43),
    (b"ipv6-frag\0", 44), (b"idrp\0", 45), (b"rsvp\0", 46), (b"gre\0", 47),
    (b"esp\0", 50), (b"ah\0", 51), (b"skip\0", 57), (b"ipv6-icmp\0", 58),
    (b"ipv6-nonxt\0", 59), (b"ipv6-opts\0", 60), (b"rspf\0", 73),
    (b"vmtp\0", 81), (b"ospf\0", 89), (b"ipip\0", 94), (b"encap\0", 98),
    (b"pim\0", 103), (b"raw\0", 255),
];

static mut PROTO_FILE: *mut FILE = core::ptr::null_mut();
static mut PROTO_ENT: ProtoEnt = ProtoEnt {
    p_name: core::ptr::null_mut(),
    p_aliases: core::ptr::null_mut(),
    p_proto: 0,
};
static mut PROTO_BUF: [c_char; 1024] = [0; 1024];
static mut PROTO_ALIASES: [*mut c_char; 16] = [core::ptr::null_mut(); 16];
static mut PROTO_IDX: usize = 0;

unsafe fn proto_parse(line: *const c_char) -> bool {
    let mut toks: [*mut c_char; 24] = [core::ptr::null_mut(); 24];
    let n = nd_tokens(line, PROTO_BUF.as_mut_ptr(), PROTO_BUF.len(), &mut toks);
    if n < 2 { return false; }
    let num = crate::strtol(toks[1], core::ptr::null_mut(), 10);
    if num < 0 || num > 255 { return false; }
    PROTO_ENT.p_name = toks[0];
    PROTO_ENT.p_proto = num as c_int;
    let mut ai = 0usize;
    for i in 2..n {
        if ai + 1 >= PROTO_ALIASES.len() { break; }
        PROTO_ALIASES[ai] = toks[i];
        ai += 1;
    }
    PROTO_ALIASES[ai] = core::ptr::null_mut();
    PROTO_ENT.p_aliases = PROTO_ALIASES.as_mut_ptr();
    true
}

unsafe fn proto_from_table(i: usize) -> bool {
    if i >= PROTO_TABLE.len() { return false; }
    PROTO_ENT.p_name = PROTO_TABLE[i].0.as_ptr() as *mut c_char;
    PROTO_ENT.p_proto = PROTO_TABLE[i].1;
    PROTO_ALIASES[0] = core::ptr::null_mut();
    PROTO_ENT.p_aliases = PROTO_ALIASES.as_mut_ptr();
    true
}

unsafe fn proto_file() -> *mut FILE {
    if PROTO_FILE.is_null() {
        PROTO_FILE = crate::fopen(b"/etc/protocols\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    }
    PROTO_FILE
}

#[no_mangle]
pub unsafe extern "C" fn setprotoent(stayopen: c_int) {
    endprotoent();
    PROTO_IDX = 0;
    PROTO_FILE = crate::fopen(b"/etc/protocols\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    let _ = stayopen;
}

#[no_mangle]
pub unsafe extern "C" fn endprotoent() {
    if !PROTO_FILE.is_null() {
        crate::fclose(PROTO_FILE);
        PROTO_FILE = core::ptr::null_mut();
    }
    PROTO_IDX = 0;
}

#[no_mangle]
pub unsafe extern "C" fn getprotoent() -> *mut ProtoEnt {
    if PROTO_FILE.is_null() {
        let f = crate::fopen(b"/etc/protocols\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
        if f.is_null() {
            let ok = proto_from_table(PROTO_IDX);
            PROTO_IDX += 1;
            return if ok { &raw mut PROTO_ENT } else { core::ptr::null_mut() };
        }
        PROTO_FILE = f;
    }
    let mut line = [0 as c_char; 1024];
    while !crate::fgets(line.as_mut_ptr(), line.len() as c_int, PROTO_FILE).is_null() {
        if proto_parse(line.as_ptr()) { return &raw mut PROTO_ENT; }
    }
    core::ptr::null_mut()
}

unsafe fn proto_name_matches(name: *const c_char) -> bool {
    if crate::strcmp(PROTO_ENT.p_name as *const u8, name as *const u8) == 0 { return true; }
    let mut i = 0usize;
    while !(*PROTO_ENT.p_aliases.add(i)).is_null() {
        if crate::strcmp(*PROTO_ENT.p_aliases.add(i) as *const u8, name as *const u8) == 0 { return true; }
        i += 1;
    }
    false
}

unsafe fn proto_scan(name: *const c_char, num: c_int, by_name: bool) -> bool {
    let f = crate::fopen(b"/etc/protocols\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    if !f.is_null() {
        let mut line = [0 as c_char; 1024];
        while !crate::fgets(line.as_mut_ptr(), line.len() as c_int, f).is_null() {
            if !proto_parse(line.as_ptr()) { continue; }
            let hit = if by_name { proto_name_matches(name) } else { PROTO_ENT.p_proto == num };
            if hit { crate::fclose(f); return true; }
        }
        crate::fclose(f);
    }
    for i in 0..PROTO_TABLE.len() {
        proto_from_table(i);
        let hit = if by_name { proto_name_matches(name) } else { PROTO_ENT.p_proto == num };
        if hit { return true; }
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn getprotobyname(name: *const c_char) -> *mut ProtoEnt {
    if name.is_null() { return core::ptr::null_mut(); }
    if proto_scan(name, 0, true) { &raw mut PROTO_ENT } else { core::ptr::null_mut() }
}

#[no_mangle]
pub unsafe extern "C" fn getprotobynumber(num: c_int) -> *mut ProtoEnt {
    if proto_scan(core::ptr::null(), num, false) { &raw mut PROTO_ENT } else { core::ptr::null_mut() }
}

// ------------------------------------------------------------------
// /etc/networks
// ------------------------------------------------------------------

static mut NET_FILE: *mut FILE = core::ptr::null_mut();
static mut NET_ENT: NetEnt = NetEnt {
    n_name: core::ptr::null_mut(),
    n_aliases: core::ptr::null_mut(),
    n_addrtype: AF_INET,
    n_net: 0,
};
static mut NET_BUF: [c_char; 1024] = [0; 1024];
static mut NET_ALIASES: [*mut c_char; 16] = [core::ptr::null_mut(); 16];

unsafe fn net_parse(line: *const c_char) -> bool {
    let mut toks: [*mut c_char; 24] = [core::ptr::null_mut(); 24];
    let n = nd_tokens(line, NET_BUF.as_mut_ptr(), NET_BUF.len(), &mut toks);
    if n < 2 { return false; }
    let net = inet_network(toks[1]);
    if net == u32::MAX { return false; }
    NET_ENT.n_name = toks[0];
    NET_ENT.n_addrtype = AF_INET;
    NET_ENT.n_net = net;
    let mut ai = 0usize;
    for i in 2..n {
        if ai + 1 >= NET_ALIASES.len() { break; }
        NET_ALIASES[ai] = toks[i];
        ai += 1;
    }
    NET_ALIASES[ai] = core::ptr::null_mut();
    NET_ENT.n_aliases = NET_ALIASES.as_mut_ptr();
    true
}

unsafe fn net_next(f: *mut FILE) -> bool {
    let mut line = [0 as c_char; 1024];
    while !crate::fgets(line.as_mut_ptr(), line.len() as c_int, f).is_null() {
        if net_parse(line.as_ptr()) { return true; }
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn setnetent(stayopen: c_int) {
    endnetent();
    NET_FILE = crate::fopen(b"/etc/networks\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    let _ = stayopen;
}

#[no_mangle]
pub unsafe extern "C" fn endnetent() {
    if !NET_FILE.is_null() {
        crate::fclose(NET_FILE);
        NET_FILE = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn getnetent() -> *mut NetEnt {
    if NET_FILE.is_null() {
        NET_FILE = crate::fopen(b"/etc/networks\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
        if NET_FILE.is_null() { return core::ptr::null_mut(); }
    }
    if net_next(NET_FILE) { &raw mut NET_ENT } else { core::ptr::null_mut() }
}

unsafe fn net_name_matches(name: *const c_char) -> bool {
    if crate::strcmp(NET_ENT.n_name as *const u8, name as *const u8) == 0 { return true; }
    let mut i = 0usize;
    while !(*NET_ENT.n_aliases.add(i)).is_null() {
        if crate::strcmp(*NET_ENT.n_aliases.add(i) as *const u8, name as *const u8) == 0 { return true; }
        i += 1;
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn getnetbyname(name: *const c_char) -> *mut NetEnt {
    if name.is_null() { return core::ptr::null_mut(); }
    let f = crate::fopen(b"/etc/networks\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    if !f.is_null() {
        while net_next(f) {
            if net_name_matches(name) { crate::fclose(f); return &raw mut NET_ENT; }
        }
        crate::fclose(f);
    }
    // "loopback" / "default" are always available even without the file.
    if crate::strcmp(name as *const u8, b"loopback\0".as_ptr()) == 0 {
        net_parse(b"loopback 127.0.0.0\0".as_ptr() as *const c_char);
        return &raw mut NET_ENT;
    }
    if crate::strcmp(name as *const u8, b"default\0".as_ptr()) == 0 {
        net_parse(b"default 0.0.0.0\0".as_ptr() as *const c_char);
        return &raw mut NET_ENT;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn getnetbyaddr(net: u32, type_: c_int) -> *mut NetEnt {
    if type_ != AF_INET { return core::ptr::null_mut(); }
    let f = crate::fopen(b"/etc/networks\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    if !f.is_null() {
        while net_next(f) {
            let shift = if NET_ENT.n_net < 0x80000000 { 24 }
                else if NET_ENT.n_net < 0xc0000000 { 16 } else { 8 };
            if (NET_ENT.n_net >> shift) == (net >> shift) { crate::fclose(f); return &raw mut NET_ENT; }
        }
        crate::fclose(f);
    }
    core::ptr::null_mut()
}

// ------------------------------------------------------------------
// /etc/ethers
// ------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ether_line(line: *const c_char, ea: *mut u8, hostname: *mut c_char) -> c_int {
    if line.is_null() || ea.is_null() || hostname.is_null() { return -1; }
    let mut buf = [0 as c_char; 512];
    let mut toks: [*mut c_char; 24] = [core::ptr::null_mut(); 24];
    let n = nd_tokens(line, buf.as_mut_ptr(), buf.len(), &mut toks);
    if n < 2 { return -1; }
    if ether_aton_r(toks[0], ea).is_null() { return -1; }
    let mut i = 0usize;
    while *toks[1].add(i) != 0 {
        *hostname.add(i) = *toks[1].add(i);
        i += 1;
    }
    *hostname.add(i) = 0;
    0
}

unsafe fn ether_scan(want_name: *const c_char, ea: *mut u8, hostname: *mut c_char) -> c_int {
    let f = crate::fopen(b"/etc/ethers\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    if f.is_null() { return -1; }
    let mut line = [0 as c_char; 512];
    let mut found = -1;
    while !crate::fgets(line.as_mut_ptr(), line.len() as c_int, f).is_null() {
        let mut mac = [0u8; 6];
        let mut host = [0 as c_char; 256];
        if ether_line(line.as_ptr(), mac.as_mut_ptr(), host.as_mut_ptr()) != 0 { continue; }
        if !want_name.is_null() {
            if crate::strcmp(host.as_ptr() as *const u8, want_name as *const u8) != 0 { continue; }
            core::ptr::copy_nonoverlapping(mac.as_ptr(), ea, 6);
            found = 0;
            break;
        } else {
            let mut same = true;
            for i in 0..6 {
                if mac[i] != *ea.add(i) { same = false; break; }
            }
            if !same { continue; }
            let mut i = 0usize;
            while host[i] != 0 {
                *hostname.add(i) = host[i];
                i += 1;
            }
            *hostname.add(i) = 0;
            found = 0;
            break;
        }
    }
    crate::fclose(f);
    found
}

#[no_mangle]
pub unsafe extern "C" fn ether_hostton(hostname: *const c_char, ea: *mut u8) -> c_int {
    if hostname.is_null() || ea.is_null() { return -1; }
    ether_scan(hostname, ea, core::ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn ether_ntohost(hostname: *mut c_char, ea: *const u8) -> c_int {
    if hostname.is_null() || ea.is_null() { return -1; }
    ether_scan(core::ptr::null(), ea as *mut u8, hostname)
}

// ------------------------------------------------------------------
// /etc/hosts iteration and reverse lookups
// ------------------------------------------------------------------

static mut HOSTS_FILE: *mut FILE = core::ptr::null_mut();
static mut HOSTS_BUF: [c_char; 1024] = [0; 1024];
static mut HOSTS_ADDR: [u8; 4] = [0; 4];
static mut HOSTS_ADDR_PTRS: [*mut c_char; 2] = [core::ptr::null_mut(); 2];
static mut HOSTS_ALIASES: [*mut c_char; 16] = [core::ptr::null_mut(); 16];

unsafe fn hosts_parse(line: *const c_char) -> bool {
    let mut toks: [*mut c_char; 24] = [core::ptr::null_mut(); 24];
    let n = nd_tokens(line, HOSTS_BUF.as_mut_ptr(), HOSTS_BUF.len(), &mut toks);
    if n < 2 { return false; }
    if inet_pton(AF_INET, toks[0], HOSTS_ADDR.as_mut_ptr() as *mut c_void) != 1 { return false; }
    HOST_ENT.h_name = toks[1];
    HOST_ENT.h_addrtype = AF_INET;
    HOST_ENT.h_length = 4;
    HOST_ENT.h_addr_list = HOSTS_ADDR_PTRS.as_mut_ptr();
    HOSTS_ADDR_PTRS[0] = HOSTS_ADDR.as_mut_ptr() as *mut c_char;
    HOSTS_ADDR_PTRS[1] = core::ptr::null_mut();
    let mut ai = 0usize;
    for i in 2..n {
        if ai + 1 >= HOSTS_ALIASES.len() { break; }
        HOSTS_ALIASES[ai] = toks[i];
        ai += 1;
    }
    HOSTS_ALIASES[ai] = core::ptr::null_mut();
    HOST_ENT.h_aliases = HOSTS_ALIASES.as_mut_ptr();
    true
}

#[no_mangle]
pub unsafe extern "C" fn sethostent(_stayopen: c_int) {
    endhostent();
}

#[no_mangle]
pub unsafe extern "C" fn endhostent() {
    if !HOSTS_FILE.is_null() {
        crate::fclose(HOSTS_FILE);
        HOSTS_FILE = core::ptr::null_mut();
    }
}

#[no_mangle]
pub unsafe extern "C" fn gethostent() -> *mut HostEnt {
    if HOSTS_FILE.is_null() {
        HOSTS_FILE = crate::fopen(b"/etc/hosts\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
        if HOSTS_FILE.is_null() { return core::ptr::null_mut(); }
    }
    let mut line = [0 as c_char; 1024];
    while !crate::fgets(line.as_mut_ptr(), line.len() as c_int, HOSTS_FILE).is_null() {
        if hosts_parse(line.as_ptr()) { return &raw mut HOST_ENT; }
    }
    core::ptr::null_mut()
}

unsafe fn hosts_lookup_addr(addr: *const u8, family: c_int) -> bool {
    let f = crate::fopen(b"/etc/hosts\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    if f.is_null() { return false; }
    let mut line = [0 as c_char; 1024];
    let mut found = false;
    while !crate::fgets(line.as_mut_ptr(), line.len() as c_int, f).is_null() {
        let mut toks: [*mut c_char; 24] = [core::ptr::null_mut(); 24];
        let n = nd_tokens(line.as_ptr(), HOSTS_BUF.as_mut_ptr(), HOSTS_BUF.len(), &mut toks);
        if n < 2 { continue; }
        let mut tmp = [0u8; 16];
        if inet_pton(AF_INET, toks[0], tmp.as_mut_ptr() as *mut c_void) == 1 {
            if family != AF_INET { continue; }
        } else if inet_pton(AF_INET6, toks[0], tmp.as_mut_ptr() as *mut c_void) == 1 {
            if family != AF_INET6 { continue; }
        } else {
            continue;
        }
        let len = if family == AF_INET6 { 16 } else { 4 };
        let mut same = true;
        for i in 0..len {
            if tmp[i] != *addr.add(i) { same = false; break; }
        }
        if !same { continue; }
        let nl = crate::strlen(toks[1]);
        core::ptr::copy_nonoverlapping(toks[1], HOST_NAME.as_mut_ptr(), nl + 1);
        found = true;
        break;
    }
    crate::fclose(f);
    found
}

unsafe fn hosts_lookup_name(name: *const c_char) -> bool {
    let f = crate::fopen(b"/etc/hosts\0".as_ptr() as *const c_char, b"r\0".as_ptr() as *const c_char);
    if f.is_null() { return false; }
    let mut line = [0 as c_char; 1024];
    let mut found = false;
    while !crate::fgets(line.as_mut_ptr(), line.len() as c_int, f).is_null() {
        if !hosts_parse(line.as_ptr()) { continue; }
        let mut hit = crate::strcmp(HOST_ENT.h_name as *const u8, name as *const u8) == 0;
        if !hit {
            let mut i = 0usize;
            while !(*HOST_ENT.h_aliases.add(i)).is_null() {
                if crate::strcmp(*HOST_ENT.h_aliases.add(i) as *const u8, name as *const u8) == 0 {
                    hit = true;
                    break;
                }
                i += 1;
            }
        }
        if hit { found = true; break; }
    }
    crate::fclose(f);
    found
}

// ------------------------------------------------------------------
// getnameinfo
// ------------------------------------------------------------------

unsafe fn nd_itoa(mut v: u32, out: *mut c_char) -> usize {
    let mut digits = [0u8; 10];
    let mut n = 0usize;
    if v == 0 { digits[0] = 0; n = 1; }
    while v > 0 { digits[n] = (v % 10) as u8; v /= 10; n += 1; }
    let mut o = 0usize;
    while n > 0 { n -= 1; *out.add(o) = (b'0' + digits[n]) as c_char; o += 1; }
    *out.add(o) = 0;
    o
}

unsafe fn nd_reverse6(addr: *const u8, out: *mut c_char, cap: usize) -> bool {
    if cap < 74 { return false; }
    let hex = b"0123456789abcdef";
    let mut o = 0usize;
    let mut i = 16isize;
    while i > 0 {
        i -= 1;
        let b = *addr.add(i as usize);
        *out.add(o) = hex[(b & 0xf) as usize] as c_char; o += 1;
        *out.add(o) = b'.' as c_char; o += 1;
        *out.add(o) = hex[(b >> 4) as usize] as c_char; o += 1;
        *out.add(o) = b'.' as c_char; o += 1;
    }
    for &c in b"ip6.arpa\0" { *out.add(o) = c as c_char; o += 1; }
    true
}

#[no_mangle]
pub unsafe extern "C" fn getnameinfo(
    sa: *const c_void,
    salen: u32,
    host: *mut c_char,
    hostlen: u32,
    serv: *mut c_char,
    servlen: u32,
    flags: c_int,
) -> c_int {
    if sa.is_null() { return EAI_FAIL_V; }
    let family = *(sa as *const u16) as c_int;
    let mut addr = [0u8; 16];
    let mut scope: u32 = 0;
    let port: u16;
    match family {
        AF_INET => {
            if salen < 16 { return EAI_FAMILY_V; }
            port = u16::from_be(*((sa as *const u8).add(2) as *const u16));
            core::ptr::copy_nonoverlapping((sa as *const u8).add(4), addr.as_mut_ptr(), 4);
        }
        AF_INET6 => {
            if salen < 28 { return EAI_FAMILY_V; }
            port = u16::from_be(*((sa as *const u8).add(2) as *const u16));
            core::ptr::copy_nonoverlapping((sa as *const u8).add(8), addr.as_mut_ptr(), 16);
            scope = *((sa as *const u8).add(24) as *const u32);
        }
        _ => return EAI_FAMILY_V,
    }

    if !host.is_null() && hostlen > 0 {
        let mut buf = [0 as c_char; 300];
        let mut have = false;
        if flags & NI_NUMERICHOST_V == 0 {
            have = hosts_lookup_addr(addr.as_ptr(), family);
            if have {
                let l = crate::strlen(HOST_NAME.as_ptr());
                let l = core::cmp::min(l, buf.len() - 1);
                core::ptr::copy_nonoverlapping(HOST_NAME.as_ptr(), buf.as_mut_ptr(), l);
                buf[l] = 0;
            }
            if !have {
                let mut ptr = [0 as c_char; 80];
                let ok = if family == AF_INET {
                    let a = addr;
                    // d.c.b.a.in-addr.arpa
                    let mut o = 0usize;
                    let mut i = 4usize;
                    while i > 0 {
                        i -= 1;
                        o += nd_itoa(a[i] as u32, ptr.as_mut_ptr().add(o));
                        *ptr.as_mut_ptr().add(o) = b'.' as c_char;
                        o += 1;
                    }
                    for &c in b"in-addr.arpa\0" { *ptr.as_mut_ptr().add(o) = c as c_char; o += 1; }
                    true
                } else {
                    nd_reverse6(addr.as_ptr(), ptr.as_mut_ptr(), ptr.len())
                };
                if ok {
                    let mut answer = [0u8; 512];
                    let r = res_query(ptr.as_ptr() as *const c_char, 1, 12, answer.as_mut_ptr(), answer.len() as c_int);
                    if r >= 12 {
                        let end = answer.as_ptr().add(r as usize);
                        let qd = dns_read_u16(answer.as_ptr().add(4)) as usize;
                        let an = dns_read_u16(answer.as_ptr().add(6)) as usize;
                        let mut p = answer.as_ptr().add(12);
                        for _ in 0..qd { p = dns_skip_name(p, end); p = p.add(4); }
                        for _ in 0..an {
                            p = dns_skip_name(p, end);
                            if p.add(10) > end { break; }
                            let typ = dns_read_u16(p);
                            let rdlen = dns_read_u16(p.add(8)) as usize;
                            let rdata = p.add(10);
                            if typ == 12 && dns_expand_name(rdata, answer.as_ptr(), buf.as_mut_ptr(), buf.len()) {
                                let mut l = crate::strlen(buf.as_ptr());
                                if l > 0 && buf[l - 1] == b'.' as c_char { l -= 1; }
                                buf[l] = 0;
                                have = l > 0;
                                break;
                            }
                            p = rdata.add(rdlen);
                        }
                    }
                }
            }
        }
        if !have {
            if flags & NI_NAMEREQD_V != 0 { return EAI_NONAME_V; }
            let aptr = if family == AF_INET { addr.as_ptr() as *const c_void } else { addr.as_ptr() as *const c_void };
            if inet_ntop(family, aptr, buf.as_mut_ptr(), buf.len() as u32).is_null() {
                return EAI_FAIL_V;
            }
            if scope != 0 && family == AF_INET6 {
                let mut l = crate::strlen(buf.as_ptr());
                *buf.as_mut_ptr().add(l) = b'%' as c_char;
                l += 1;
                let mut name = [0 as c_char; 32];
                let have_name = flags & NI_NUMERICSCOPE_V == 0
                    && if_indextoname(scope, name.as_mut_ptr()) != core::ptr::null_mut();
                if have_name {
                    let nl = crate::strlen(name.as_ptr());
                    core::ptr::copy_nonoverlapping(name.as_ptr(), buf.as_mut_ptr().add(l), nl + 1);
                } else {
                    nd_itoa(scope, buf.as_mut_ptr().add(l));
                }
            }
        }
        if flags & NI_NOFQDN_V != 0 {
            let mut i = 0usize;
            while buf[i] != 0 {
                if buf[i] == b'.' as c_char { buf[i] = 0; break; }
                i += 1;
            }
        }
        if crate::strlen(buf.as_ptr()) >= hostlen as usize { return EAI_OVERFLOW_V; }
        let l = crate::strlen(buf.as_ptr());
        core::ptr::copy_nonoverlapping(buf.as_ptr(), host, l + 1);
    }

    if !serv.is_null() && servlen > 0 {
        let mut buf = [0 as c_char; 64];
        let mut have = false;
        if flags & NI_NUMERICSERV_V == 0 {
            let proto = if flags & NI_DGRAM_V != 0 { b"udp\0" } else { b"tcp\0" };
            let port_net = (port as u16).to_be() as c_int;
            let se = getservbyport(port_net, proto.as_ptr() as *const c_char);
            if !se.is_null() {
                let l = crate::strlen((*se).s_name);
                if l < buf.len() {
                    core::ptr::copy_nonoverlapping((*se).s_name, buf.as_mut_ptr(), l + 1);
                    have = true;
                }
            }
        }
        if !have {
            nd_itoa(port as u32, buf.as_mut_ptr());
        }
        if crate::strlen(buf.as_ptr()) >= servlen as usize { return EAI_OVERFLOW_V; }
        let l = crate::strlen(buf.as_ptr());
        core::ptr::copy_nonoverlapping(buf.as_ptr(), serv, l + 1);
    }
    0
}

// ------------------------------------------------------------------
// getaddrinfo
// ------------------------------------------------------------------

unsafe fn nd_make_addrinfo(
    family: c_int,
    socktype: c_int,
    protocol: c_int,
    addr: *const u8,
    port: u16,
    flags: c_int,
    canon: *const c_char,
) -> *mut AddrInfo {
    let addrlen: usize = if family == AF_INET6 { 28 } else { 16 };
    let canonlen = if !canon.is_null() { crate::strlen(canon) + 1 } else { 0 };
    let total = core::mem::size_of::<AddrInfo>() + addrlen + canonlen;
    let p = crate::malloc(total) as *mut u8;
    if p.is_null() { return core::ptr::null_mut(); }
    core::ptr::write_bytes(p, 0, total);
    let ai = p as *mut AddrInfo;
    (*ai).ai_flags = flags;
    (*ai).ai_family = family;
    (*ai).ai_socktype = socktype;
    (*ai).ai_protocol = protocol;
    (*ai).ai_addrlen = addrlen as u32;
    let sa = p.add(core::mem::size_of::<AddrInfo>());
    (*ai).ai_addr = sa as *mut c_void;
    if family == AF_INET6 {
        *(sa as *mut u16) = AF_INET6 as u16;
        *(sa.add(2) as *mut u16) = port.to_be();
        core::ptr::copy_nonoverlapping(addr, sa.add(8), 16);
    } else {
        *(sa as *mut u16) = AF_INET as u16;
        *(sa.add(2) as *mut u16) = port.to_be();
        core::ptr::copy_nonoverlapping(addr, sa.add(4), 4);
    }
    if canonlen > 0 {
        let c = sa.add(addrlen);
        core::ptr::copy_nonoverlapping(canon as *const u8, c, canonlen);
        (*ai).ai_canonname = c as *mut c_char;
    }
    ai
}

unsafe fn nd_parse_service(service: *const c_char, socktype: c_int, protocol: c_int, port: *mut u16, proto_out: *mut c_int) -> bool {
    if service.is_null() { *port = 0; return true; }
    let mut end: *mut c_char = core::ptr::null_mut();
    let v = crate::strtoul(service, &mut end, 10);
    let numeric = !end.is_null() && *end == 0;
    if numeric {
        if v > 65535 { return false; }
        *port = v as u16;
        return true;
    }
    let proto_name: *const c_char = if protocol == 6 { b"tcp\0".as_ptr() as *const c_char }
        else if protocol == 17 { b"udp\0".as_ptr() as *const c_char }
        else if socktype == SOCK_STREAM { b"tcp\0".as_ptr() as *const c_char }
        else if socktype == SOCK_DGRAM { b"udp\0".as_ptr() as *const c_char }
        else { core::ptr::null() };
    let se = getservbyname(service, proto_name);
    if se.is_null() { return false; }
    *port = ntohs((*se).s_port as u16);
    *proto_out = if crate::strcmp((*se).s_proto as *const u8, b"udp\0".as_ptr()) == 0 { 17 } else { 6 };
    true
}

#[no_mangle]
pub unsafe extern "C" fn getaddrinfo(
    node: *const c_char,
    service: *const c_char,
    hints: *const AddrInfo,
    res: *mut *mut AddrInfo,
) -> c_int {
    if res.is_null() { return EAI_FAIL_V; }
    *res = core::ptr::null_mut();

    let mut family = 0;
    let mut socktype = 0;
    let mut protocol = 0;
    let mut flags = 0;
    if !hints.is_null() {
        family = (*hints).ai_family;
        socktype = (*hints).ai_socktype;
        protocol = (*hints).ai_protocol;
        flags = (*hints).ai_flags;
    }
    if family != 0 && family != AF_INET && family != AF_INET6 { return EAI_FAMILY_V; }
    if socktype != 0 && socktype != SOCK_STREAM && socktype != SOCK_DGRAM { return EAI_SOCKTYPE_V; }
    if flags & AI_NUMERICSERV_V != 0 && !service.is_null() {
        let mut end: *mut c_char = core::ptr::null_mut();
        let _ = crate::strtoul(service, &mut end, 10);
        if end.is_null() || *end != 0 { return EAI_NONAME_V; }
    }

    let mut port: u16 = 0;
    let mut proto_from_service = 0;
    if !nd_parse_service(service, socktype, protocol, &mut port, &mut proto_from_service) {
        return EAI_SERVICE_V;
    }

    let mut addrs: [[u8; 16]; 8] = [[0; 16]; 8];
    let mut fams: [c_int; 8] = [0; 8];
    let mut count = 0usize;
    let mut canon: [c_char; 256] = [0; 256];

    if node.is_null() {
        if family == 0 || family == AF_INET6 {
            if family == 0 && flags & AI_V4MAPPED_V != 0 && flags & AI_ALL_V != 0 {
                // handled below through IPv4 too
            }
            addrs[count] = [0; 16];
            if flags & AI_PASSIVE_V == 0 {
                addrs[count][15] = 1;
            }
            fams[count] = AF_INET6;
            count += 1;
        }
        if family == 0 || family == AF_INET {
            addrs[count] = [0; 16];
            if flags & AI_PASSIVE_V == 0 {
                addrs[count][0] = 127;
                addrs[count][3] = 1;
            }
            fams[count] = AF_INET;
            count += 1;
        }
    } else {
        let mut numeric = false;
        let mut tmp = [0u8; 16];
        if inet_pton(AF_INET, node, tmp.as_mut_ptr() as *mut c_void) == 1 {
            numeric = true;
            if family == 0 || family == AF_INET {
                addrs[count][..4].copy_from_slice(&tmp[..4]);
                fams[count] = AF_INET;
                count += 1;
            }
        }
        if !numeric && inet_pton(AF_INET6, node, tmp.as_mut_ptr() as *mut c_void) == 1 {
            numeric = true;
            if family == 0 || family == AF_INET6 {
                addrs[count] = tmp;
                fams[count] = AF_INET6;
                count += 1;
            }
        }
        if flags & AI_NUMERICHOST_V != 0 && !numeric {
            return EAI_NONAME_V;
        }
        if !numeric {
            let mut got = false;
            // Prefer /etc/hosts, then DNS.
            if hosts_lookup_name(node) {
                let l = core::cmp::min(crate::strlen(HOST_ENT.h_name), canon.len() - 1);
                core::ptr::copy_nonoverlapping(HOST_ENT.h_name, canon.as_mut_ptr(), l);
                canon[l] = 0;
                if family == 0 || family == AF_INET {
                    core::ptr::copy_nonoverlapping(HOSTS_ADDR.as_ptr(), addrs[count].as_mut_ptr(), 4);
                    fams[count] = AF_INET;
                    count += 1;
                    got = true;
                }
                // gethostbyname2 may return more than the hosts entry; fall
                // through for the requested family only when needed.
                if family == AF_INET6 {
                    got = false;
                }
            }
            if !got {
                let order: [c_int; 2] = if family == AF_INET6 { [AF_INET6, AF_INET] } else { [AF_INET, AF_INET6] };
                for &f in &order {
                    if family != 0 && family != f { continue; }
                    let h = gethostbyname2(node, f);
                    if h.is_null() { continue; }
                    let n = core::cmp::min((*h).h_length as usize, 16);
                    core::ptr::copy_nonoverlapping((*h).h_addr_list.read() as *const u8, addrs[count].as_mut_ptr(), n);
                    fams[count] = f;
                    count += 1;
                    let hl = crate::strlen((*h).h_name);
                    let l = core::cmp::min(hl, canon.len() - 1);
                    core::ptr::copy_nonoverlapping((*h).h_name, canon.as_mut_ptr(), l);
                    canon[l] = 0;
                    got = true;
                    if family != 0 { break; }
                }
            }
            if !got {
                return if h_errno == TRY_AGAIN { EAI_AGAIN_V } else { EAI_NONAME_V };
            }
        }
    }
    if count == 0 { return EAI_NONAME_V; }

    let canonptr: *const c_char = if flags & AI_CANONNAME_V != 0 && canon[0] != 0 {
        canon.as_ptr()
    } else if flags & AI_CANONNAME_V != 0 && !node.is_null() {
        node
    } else {
        core::ptr::null()
    };

    let mut head: *mut AddrInfo = core::ptr::null_mut();
    let mut tail: *mut AddrInfo = core::ptr::null_mut();
    let mut emitted = 0usize;
    for i in 0..count {
        let types: [c_int; 2] = if socktype == 0 { [SOCK_STREAM, SOCK_DGRAM] } else { [socktype, socktype] };
        let ntypes = if socktype == 0 { 2 } else { 1 };
        for t in 0..ntypes {
            let st = types[t];
            if protocol != 0 {
                if (protocol == 6 && st != SOCK_STREAM) || (protocol == 17 && st != SOCK_DGRAM) { continue; }
            }
            if proto_from_service != 0 && protocol == 0 {
                if (proto_from_service == 6 && st != SOCK_STREAM) || (proto_from_service == 17 && st != SOCK_DGRAM) { continue; }
            }
            let proto = if protocol != 0 { protocol } else if st == SOCK_STREAM { 6 } else { 17 };
            let cp = if emitted == 0 { canonptr } else { core::ptr::null() };
            let ai = nd_make_addrinfo(fams[i], st, proto, addrs[i].as_ptr(), port, flags, cp);
            if ai.is_null() {
                freeaddrinfo(head);
                return EAI_MEMORY_V;
            }
            if head.is_null() { head = ai; } else { (*tail).ai_next = ai; }
            tail = ai;
            emitted += 1;
        }
    }
    if emitted == 0 { return EAI_SERVICE_V; }
    *res = head;
    0
}

#[no_mangle]
pub unsafe extern "C" fn freeaddrinfo(mut ai: *mut AddrInfo) {
    while !ai.is_null() {
        let next = (*ai).ai_next;
        crate::free(ai as *mut c_void);
        ai = next;
    }
}

#[no_mangle]
pub unsafe extern "C" fn gai_strerror(code: c_int) -> *const c_char {
    let s: &[u8] = match code {
        EAI_BADFLAGS_V => b"Bad flags\0",
        EAI_NONAME_V => b"Name not known\0",
        EAI_AGAIN_V => b"Try again\0",
        EAI_FAIL_V => b"Non-recoverable failure\0",
        -5 => b"No address data\0",
        EAI_FAMILY_V => b"Unsupported family\0",
        EAI_SOCKTYPE_V => b"Unsupported socket type\0",
        EAI_SERVICE_V => b"Unsupported service\0",
        -9 => b"Address family not supported\0",
        EAI_MEMORY_V => b"Out of memory\0",
        EAI_SYSTEM_V => b"System error\0",
        EAI_OVERFLOW_V => b"Overflow\0",
        _ => b"Unknown error\0",
    };
    s.as_ptr() as *const c_char
}

// ------------------------------------------------------------------
// ttyname
// ------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "C" fn ttyname_r(fd: c_int, name: *mut c_char, size: usize) -> c_int {
    if name.is_null() || size == 0 { return EINVAL; }
    if crate::isatty(fd) != 1 { return NDB_ENOTTY; }
    let mut procname = [0 as c_char; 32];
    let mut o = 0usize;
    for &c in b"/proc/self/fd/" { procname[o] = c as c_char; o += 1; }
    o += nd_itoa(fd as u32, procname.as_mut_ptr().add(o));
    procname[o] = 0;
    let n = crate::readlink(procname.as_ptr(), name, size - 1);
    if n < 0 { return ERRNO!(); }
    if n as usize == size - 1 { return NDB_ERANGE; }
    *name.add(n as usize) = 0;
    0
}

#[no_mangle]
pub unsafe extern "C" fn ttyname(fd: c_int) -> *mut c_char {
    static mut TTY_BUF: [c_char; 64] = [0; 64];
    if ttyname_r(fd, TTY_BUF.as_mut_ptr(), TTY_BUF.len()) != 0 {
        core::ptr::null_mut()
    } else {
        TTY_BUF.as_mut_ptr()
    }
}

// ------------------------------------------------------------------
// Reentrant host lookups
// ------------------------------------------------------------------

unsafe fn host_copy_out(
    h: *mut HostEnt,
    ret: *mut HostEnt,
    buf: *mut c_char,
    buflen: usize,
    result: *mut *mut HostEnt,
) -> c_int {
    if result.is_null() { return EINVAL; }
    *result = core::ptr::null_mut();
    let ptrsz = core::mem::size_of::<*mut c_char>();
    let mut need = crate::strlen((*h).h_name) + 1;
    let mut nalias = 0usize;
    if !(*h).h_aliases.is_null() {
        while !(*(*h).h_aliases.add(nalias)).is_null() {
            need += crate::strlen(*(*h).h_aliases.add(nalias)) + 1;
            nalias += 1;
        }
    }
    let alen = (*h).h_length as usize;
    let align = (ptrsz - (buf as usize & (ptrsz - 1))) & (ptrsz - 1);
    need += align + (nalias + 1) * ptrsz + alen + 2 * ptrsz;
    if need > buflen { return NDB_ERANGE; }

    let mut p = buf.add(align);
    let aliases = p as *mut *mut c_char;
    p = p.add((nalias + 1) * ptrsz);
    let addr_list = p as *mut *mut c_char;
    p = p.add(2 * ptrsz);

    let n = crate::strlen((*h).h_name);
    core::ptr::copy_nonoverlapping((*h).h_name, p, n + 1);
    (*ret).h_name = p;
    p = p.add(n + 1);

    for i in 0..nalias {
        let src = *(*h).h_aliases.add(i);
        let n = crate::strlen(src);
        core::ptr::copy_nonoverlapping(src, p, n + 1);
        *aliases.add(i) = p;
        p = p.add(n + 1);
    }
    *aliases.add(nalias) = core::ptr::null_mut();

    let src = *(*h).h_addr_list;
    core::ptr::copy_nonoverlapping(src, p, alen);
    *addr_list = p;
    *addr_list.add(1) = core::ptr::null_mut();

    (*ret).h_aliases = aliases;
    (*ret).h_addrtype = (*h).h_addrtype;
    (*ret).h_length = (*h).h_length;
    (*ret).h_addr_list = addr_list;
    *result = ret;
    0
}

#[no_mangle]
pub unsafe extern "C" fn gethostbyname_r(
    name: *const c_char,
    ret: *mut HostEnt,
    buf: *mut c_char,
    buflen: usize,
    result: *mut *mut HostEnt,
    errnop: *mut c_int,
) -> c_int {
    gethostbyname2_r(name, AF_INET, ret, buf, buflen, result, errnop)
}

#[no_mangle]
pub unsafe extern "C" fn gethostbyname2_r(
    name: *const c_char,
    af: c_int,
    ret: *mut HostEnt,
    buf: *mut c_char,
    buflen: usize,
    result: *mut *mut HostEnt,
    errnop: *mut c_int,
) -> c_int {
    if result.is_null() { return EINVAL; }
    *result = core::ptr::null_mut();
    let h = gethostbyname2(name, af);
    if h.is_null() {
        if !errnop.is_null() { *errnop = if h_errno != 0 { h_errno } else { HOST_NOT_FOUND }; }
        return 0;
    }
    let r = host_copy_out(h, ret, buf, buflen, result);
    if r == NDB_ERANGE {
        if !errnop.is_null() { *errnop = NO_RECOVERY; }
        *result = core::ptr::null_mut();
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn gethostbyaddr_r(
    addr: *const c_void,
    len: u32,
    type_: c_int,
    ret: *mut HostEnt,
    buf: *mut c_char,
    buflen: usize,
    result: *mut *mut HostEnt,
    errnop: *mut c_int,
) -> c_int {
    if result.is_null() { return EINVAL; }
    *result = core::ptr::null_mut();
    let h = gethostbyaddr(addr, len, type_);
    if h.is_null() {
        if !errnop.is_null() { *errnop = if h_errno != 0 { h_errno } else { HOST_NOT_FOUND }; }
        return 0;
    }
    let r = host_copy_out(h, ret, buf, buflen, result);
    if r == NDB_ERANGE {
        if !errnop.is_null() { *errnop = NO_RECOVERY; }
        *result = core::ptr::null_mut();
    }
    r
}


#[no_mangle]
pub unsafe extern "C" fn initgroups(user: *const c_char, group: u32) -> c_int {
    if user.is_null() { ERRNO!() = EINVAL; return -1; }
    let mut buf = [0u32; 256];
    let mut n: c_int = buf.len() as c_int;
    if getgrouplist(user, group, buf.as_mut_ptr(), &mut n) < 0 {
        if n <= 0 { return -1; }
        let p = crate::malloc(n as usize * core::mem::size_of::<u32>()) as *mut u32;
        if p.is_null() { return -1; }
        let mut n2 = n;
        let r = getgrouplist(user, group, p, &mut n2);
        let rr = if r < 0 { -1 } else { crate::setgroups(n2 as usize, p) };
        crate::free(p as *mut c_void);
        return rr;
    }
    crate::setgroups(n as usize, buf.as_ptr())
}
