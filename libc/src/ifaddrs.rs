// getifaddrs() via rtnetlink, plus if_nametoindex/if_indextoname/
// if_nameindex backed by /sys/class/net.

#[repr(C)]
pub struct IfAddrs {
    pub ifa_next: *mut IfAddrs,
    pub ifa_name: *mut c_char,
    pub ifa_flags: c_uint,
    pub ifa_addr: *mut c_void,
    pub ifa_netmask: *mut c_void,
    pub ifa_ifu: *mut c_void,
    pub ifa_data: *mut c_void,
}

#[repr(C, align(8))]
#[derive(Copy, Clone)]
struct SockAny {
    data: [u8; 64],
}

#[repr(C)]
struct IfaStorage {
    ifa: IfAddrs,
    hash_next: *mut IfaStorage,
    addr: SockAny,
    netmask: SockAny,
    ifu: SockAny,
    index: c_uint,
    name: [c_char; 17],
}

const IFADDRS_HASH_SIZE: usize = 64;

struct IfaCtx {
    first: *mut IfAddrs,
    last: *mut IfAddrs,
    hash: [*mut IfaStorage; IFADDRS_HASH_SIZE],
}

#[repr(C)]
struct NlMsgHdr {
    len: u32,
    type_: u16,
    flags: u16,
    seq: u32,
    pid: u32,
}

#[repr(C)]
struct NlRta {
    len: u16,
    type_: u16,
}

#[repr(C)]
struct IfInfoMsg {
    family: u8,
    pad: u8,
    type_: u16,
    index: i32,
    flags: u32,
    change: u32,
}

#[repr(C)]
struct IfAddrMsg {
    family: u8,
    prefixlen: u8,
    flags: u8,
    scope: u8,
    index: u32,
}

#[repr(C)]
struct SockAddrNl {
    family: u16,
    pad: u16,
    pid: u32,
    groups: u32,
}

const NLMSG_ERROR: u16 = 2;
const NLMSG_DONE: u16 = 3;
const RTM_NEWLINK: u16 = 16;
const RTM_NEWADDR: u16 = 20;
const RTM_GETLINK: u16 = 18;
const RTM_GETADDR: u16 = 22;
const IFLA_ADDRESS: u16 = 1;
const IFLA_BROADCAST: u16 = 2;
const IFLA_IFNAME: u16 = 3;
const IFA_ADDRESS: u16 = 1;
const IFA_LOCAL: u16 = 2;
const IFA_LABEL: u16 = 3;
const IFA_BROADCAST: u16 = 4;

unsafe fn copy_addr(
    r: *mut *mut c_void,
    af: c_int,
    sa: *mut SockAny,
    addr: *const u8,
    addrlen: usize,
    ifindex: c_int,
) {
    let base = sa as *mut u8;
    let dst: *mut u8;
    let len: usize;
    if af == AF_INET {
        dst = base.add(4);
        len = 4;
    } else if af == AF_INET6 {
        dst = base.add(8);
        len = 16;
        let linklocal = addrlen >= 2 && *addr == 0xfe && ((*addr.add(1) & 0xc0) == 0x80);
        let mclink = addrlen >= 2 && *addr == 0xff && ((*addr.add(1) & 0x0f) == 2);
        if linklocal || mclink {
            *(base.add(24) as *mut u32) = ifindex as u32;
        }
    } else {
        return;
    }
    if addrlen < len { return; }
    *(base as *mut u16) = af as u16;
    core::ptr::copy_nonoverlapping(addr, dst, len);
    *r = sa as *mut c_void;
}

unsafe fn gen_netmask(r: *mut *mut c_void, af: c_int, sa: *mut SockAny, prefixlen: c_int) {
    let mut addr = [0u8; 16];
    let mut pl = prefixlen;
    if pl > 128 { pl = 128; }
    let i = (pl / 8) as usize;
    for k in 0..i { addr[k] = 0xff; }
    if i < 16 {
        addr[i] = (0xffu16 << (8 - (pl % 8))) as u8;
    }
    copy_addr(r, af, sa, addr.as_ptr(), 16, 0);
}

unsafe fn copy_lladdr(
    r: *mut *mut c_void,
    sa: *mut SockAny,
    addr: *const u8,
    addrlen: usize,
    ifindex: c_int,
    hatype: u16,
) {
    if addrlen > 24 { return; }
    let base = sa as *mut u8;
    *(base as *mut u16) = 17; // AF_PACKET
    *(base.add(2) as *mut u16) = 0;
    *(base.add(4) as *mut i32) = ifindex;
    *(base.add(8) as *mut u16) = hatype;
    *base.add(10) = 0;
    *base.add(11) = addrlen as u8;
    core::ptr::copy_nonoverlapping(addr, base.add(12), addrlen);
    *r = sa as *mut c_void;
}

unsafe fn parse_rtattrs<F: FnMut(u16, *const u8, usize)>(
    hdr: *const NlMsgHdr,
    payload_off: usize,
    mut cb: F,
) {
    let start = hdr as *const u8;
    let end = start.add((*hdr).len as usize);
    let mut p = start.add(payload_off);
    while p.add(4) <= end {
        let rta = p as *const NlRta;
        let alen = (*rta).len as usize;
        if alen < 4 || p.add(alen) > end { break; }
        let atype = (*rta).type_ & 0x3fff;
        cb(atype, p.add(4), alen - 4);
        p = p.add((alen + 3) & !3usize);
    }
}

unsafe fn netlink_msg_to_ifaddr(ctx: *mut IfaCtx, hdr: *const NlMsgHdr) -> c_int {
    let htype = (*hdr).type_;
    let mut ifs: *mut IfaStorage;
    let mut ifs0: *mut IfaStorage = core::ptr::null_mut();
    if htype == RTM_NEWLINK {
        let hb = hdr as *const u8;
        let payload = hb.add(16) as *const IfInfoMsg;
        ifs = crate::calloc(1, core::mem::size_of::<IfaStorage>()) as *mut IfaStorage;
        if ifs.is_null() { return -1; }
        (*ifs).index = (*payload).index as c_uint;
        (*ifs).ifa.ifa_flags = (*payload).flags;
        let index = (*payload).index;
        let hatype = (*payload).type_;
        parse_rtattrs(hdr, 16 + core::mem::size_of::<IfInfoMsg>(), |atype, data, len| {
            match atype {
                IFLA_IFNAME => {
                    if len < (*ifs).name.len() {
                        core::ptr::copy_nonoverlapping(data, (*ifs).name.as_mut_ptr() as *mut u8, len);
                        (*ifs).name[len] = 0;
                        (*ifs).ifa.ifa_name = (*ifs).name.as_mut_ptr();
                    }
                }
                IFLA_ADDRESS => {
                    copy_lladdr(&mut (*ifs).ifa.ifa_addr, &mut (*ifs).addr, data, len, index, hatype);
                }
                IFLA_BROADCAST => {
                    copy_lladdr(&mut (*ifs).ifa.ifa_ifu, &mut (*ifs).ifu, data, len, index, hatype);
                }
                _ => {}
            }
        });
        if !(*ifs).ifa.ifa_name.is_null() {
            let bucket = ((*ifs).index as usize) % IFADDRS_HASH_SIZE;
            (*ifs).hash_next = (*ctx).hash[bucket];
            (*ctx).hash[bucket] = ifs;
        }
    } else {
        let payload = (hdr as *const u8).add(16) as *const IfAddrMsg;
        let index = (*payload).index;
        let bucket = (index as usize) % IFADDRS_HASH_SIZE;
        let mut it = (*ctx).hash[bucket];
        while !it.is_null() {
            if (*it).index == index { ifs0 = it; break; }
            it = (*it).hash_next;
        }
        if ifs0.is_null() { return 0; }
        ifs = crate::calloc(1, core::mem::size_of::<IfaStorage>()) as *mut IfaStorage;
        if ifs.is_null() { return -1; }
        (*ifs).index = index as c_uint;
        (*ifs).ifa.ifa_name = (*ifs0).ifa.ifa_name;
        (*ifs).ifa.ifa_flags = (*ifs0).ifa.ifa_flags;
        let family = (*payload).family as c_int;
        let prefixlen = (*payload).prefixlen as c_int;
        parse_rtattrs(hdr, 16 + core::mem::size_of::<IfAddrMsg>(), |atype, data, len| {
            match atype {
                IFA_ADDRESS => {
                    if !(*ifs).ifa.ifa_addr.is_null() {
                        copy_addr(&mut (*ifs).ifa.ifa_ifu, family, &mut (*ifs).ifu, data, len, index as c_int);
                    } else {
                        copy_addr(&mut (*ifs).ifa.ifa_addr, family, &mut (*ifs).addr, data, len, index as c_int);
                    }
                }
                IFA_BROADCAST => {
                    copy_addr(&mut (*ifs).ifa.ifa_ifu, family, &mut (*ifs).ifu, data, len, index as c_int);
                }
                IFA_LOCAL => {
                    if !(*ifs).ifa.ifa_addr.is_null() {
                        (*ifs).ifu = (*ifs).addr;
                        (*ifs).ifa.ifa_ifu = &mut (*ifs).ifu as *mut SockAny as *mut c_void;
                        (*ifs).addr = SockAny { data: [0; 64] };
                    }
                    copy_addr(&mut (*ifs).ifa.ifa_addr, family, &mut (*ifs).addr, data, len, index as c_int);
                }
                IFA_LABEL => {
                    if len < (*ifs).name.len() {
                        core::ptr::copy_nonoverlapping(data, (*ifs).name.as_mut_ptr() as *mut u8, len);
                        (*ifs).name[len] = 0;
                        (*ifs).ifa.ifa_name = (*ifs).name.as_mut_ptr();
                    }
                }
                _ => {}
            }
        });
        if !(*ifs).ifa.ifa_addr.is_null() {
            gen_netmask(&mut (*ifs).ifa.ifa_netmask, family, &mut (*ifs).netmask, prefixlen);
        }
    }

    if !(*ifs).ifa.ifa_name.is_null() {
        if (*ctx).first.is_null() {
            (*ctx).first = &mut (*ifs).ifa;
        } else {
            (*(*ctx).last).ifa_next = &mut (*ifs).ifa;
        }
        (*ctx).last = &mut (*ifs).ifa;
    } else {
        crate::free(ifs as *mut c_void);
    }
    0
}

unsafe fn netlink_enumerate(
    fd: c_int,
    seq: u32,
    msgtype: u16,
    cb: unsafe fn(*mut IfaCtx, *const NlMsgHdr) -> c_int,
    ctx: *mut IfaCtx,
) -> c_int {
    let mut req = [0u8; 20];
    let h = req.as_mut_ptr() as *mut NlMsgHdr;
    (*h).len = 20;
    (*h).type_ = msgtype;
    (*h).flags = 0x0301; // NLM_F_DUMP | NLM_F_REQUEST
    (*h).seq = seq;
    (*h).pid = 0;
    req[16] = 0; // rtgen_family = AF_UNSPEC
    let sa = SockAddrNl { family: 16, pad: 0, pid: 0, groups: 0 };
    let sent = crate::sendto(
        fd,
        req.as_ptr() as *const c_void,
        req.len(),
        0,
        &sa as *const SockAddrNl as *const crate::sockaddr,
        core::mem::size_of::<SockAddrNl>() as u32,
    );
    if sent < 0 { return -1; }
    let mut buf = [0u8; 8192];
    loop {
        let n = crate::recv(fd, buf.as_mut_ptr() as *mut c_void, buf.len(), 0);
        if n <= 0 { return -1; }
        let mut off = 0usize;
        while off + 16 <= n as usize {
            let hdr = buf.as_ptr().add(off) as *const NlMsgHdr;
            let len = (*hdr).len as usize;
            if len < 16 || off + len > n as usize { break; }
            if (*hdr).type_ == NLMSG_DONE { return 0; }
            if (*hdr).type_ == NLMSG_ERROR { return -1; }
            let r = cb(ctx, hdr);
            if r != 0 { return r; }
            off += (len + 3) & !3usize;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn getifaddrs(ifap: *mut *mut IfAddrs) -> c_int {
    if ifap.is_null() { ERRNO = EINVAL; return -1; }
    *ifap = core::ptr::null_mut();
    let mut ctx: IfaCtx = core::mem::zeroed();
    let fd = crate::socket(16, 3 | 0x80000, 0); // AF_NETLINK, SOCK_RAW|SOCK_CLOEXEC
    if fd < 0 { return -1; }
    let mut r = netlink_enumerate(fd, 1, RTM_GETLINK, netlink_msg_to_ifaddr, &mut ctx);
    if r == 0 {
        r = netlink_enumerate(fd, 2, RTM_GETADDR, netlink_msg_to_ifaddr, &mut ctx);
    }
    crate::close(fd);
    if r == 0 {
        *ifap = ctx.first;
    } else {
        freeifaddrs(ctx.first);
    }
    r
}

#[no_mangle]
pub unsafe extern "C" fn freeifaddrs(mut ifa: *mut IfAddrs) {
    while !ifa.is_null() {
        let next = (*ifa).ifa_next;
        crate::free(ifa as *mut c_void);
        ifa = next;
    }
}

// ------------------------------------------------------------------
// Interface names via /sys/class/net
// ------------------------------------------------------------------

unsafe fn sys_read_ifindex(name: *const c_char) -> u32 {
    let mut path = [0 as c_char; 128];
    let mut o = 0usize;
    for &c in b"/sys/class/net/" { path[o] = c as c_char; o += 1; }
    let n = crate::strlen(name);
    if o + n + 12 >= path.len() { return 0; }
    core::ptr::copy_nonoverlapping(name, path.as_mut_ptr().add(o), n);
    o += n;
    for &c in b"/ifindex\0" { path[o] = c as c_char; o += 1; }
    let fd = crate::open(path.as_ptr(), 0, 0);
    if fd < 0 { return 0; }
    let mut buf = [0u8; 32];
    let n = crate::read(fd, buf.as_mut_ptr() as *mut c_void, buf.len() - 1);
    crate::close(fd);
    if n <= 0 { return 0; }
    buf[n as usize] = 0;
    crate::strtoul(buf.as_ptr() as *const c_char, core::ptr::null_mut(), 10) as u32
}

#[no_mangle]
pub unsafe extern "C" fn if_nametoindex(ifname: *const c_char) -> c_uint {
    if ifname.is_null() { return 0; }
    sys_read_ifindex(ifname)
}

#[repr(align(8))]
struct DirentBuf([u8; 4096]);

unsafe fn list_net_names<F: FnMut(*const c_char, usize)>(mut cb: F) {
    let fd = crate::open(b"/sys/class/net\0".as_ptr() as *const c_char, 0o200000, 0);
    if fd < 0 { return; }
    let mut buf = DirentBuf([0u8; 4096]);
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
            let len = crate::strlen(name);
            if len > 0 && *name != b'.' as c_char {
                cb(name, len);
            }
            off += reclen;
        }
    }
    crate::close(fd);
}

#[no_mangle]
pub unsafe extern "C" fn if_indextoname(ifindex: c_uint, ifname: *mut c_char) -> *mut c_char {
    if ifindex == 0 || ifname.is_null() { ERRNO = 19; return core::ptr::null_mut(); }
    let mut found = false;
    list_net_names(|name, len| {
        if found { return; }
        if sys_read_ifindex(name) == ifindex {
            core::ptr::copy_nonoverlapping(name, ifname, len + 1);
            found = true;
        }
    });
    if found { ifname } else { ERRNO = 19; core::ptr::null_mut() }
}

#[repr(C)]
pub struct IfNameIndex {
    pub if_index: c_uint,
    pub if_name: *mut c_char,
}

struct NameList {
    arr: *mut IfNameIndex,
    count: usize,
    cap: usize,
}

#[no_mangle]
pub unsafe extern "C" fn if_nameindex() -> *mut IfNameIndex {
    let mut list = NameList {
        arr: core::ptr::null_mut(),
        count: 0,
        cap: 0,
    };
    list.arr = crate::calloc(8, core::mem::size_of::<IfNameIndex>()) as *mut IfNameIndex;
    if list.arr.is_null() { return core::ptr::null_mut(); }
    list.cap = 8;
    list_net_names(|name, len| {
        let idx = sys_read_ifindex(name);
        if idx == 0 { return; }
        if list.count + 1 >= list.cap {
            let ncap = list.cap * 2;
            let na = crate::realloc(list.arr as *mut c_void, ncap * core::mem::size_of::<IfNameIndex>()) as *mut IfNameIndex;
            if na.is_null() { return; }
            list.arr = na;
            list.cap = ncap;
        }
        let copy = crate::malloc(len + 1) as *mut c_char;
        if copy.is_null() { return; }
        core::ptr::copy_nonoverlapping(name, copy, len + 1);
        (*list.arr.add(list.count)).if_index = idx;
        (*list.arr.add(list.count)).if_name = copy;
        list.count += 1;
    });
    if list.count == 0 {
        crate::free(list.arr as *mut c_void);
        return core::ptr::null_mut();
    }
    (*list.arr.add(list.count)).if_index = 0;
    (*list.arr.add(list.count)).if_name = core::ptr::null_mut();
    list.arr
}

#[no_mangle]
pub unsafe extern "C" fn if_freenameindex(mut ptr: *mut IfNameIndex) {
    if ptr.is_null() { return; }
    let base = ptr;
    while (*ptr).if_index != 0 && !(*ptr).if_name.is_null() {
        crate::free((*ptr).if_name as *mut c_void);
        ptr = ptr.add(1);
    }
    crate::free(base as *mut c_void);
}
