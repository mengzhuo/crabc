// crypt_impl.rs - crypt password hashing (MD5, bcrypt, SHA-256, SHA-512)
// Ported from musl libc src/crypt/

unsafe fn cstr_bytes(s: *const c_char) -> &'static [u8] {
    if s.is_null() { return &[]; }
    let len = strlen(s as *const u8);
    core::slice::from_raw_parts(s as *const u8, len)
}

const CRYPT_B64: [u8; 64] = *b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn crypt_to64(out: &mut [u8], mut u: u32, n: usize) -> usize {
    for i in 0..n {
        out[i] = CRYPT_B64[(u % 64) as usize];
        u /= 64;
    }
    n
}

// ============================================================
// MD5
// ============================================================

struct Md5 { len: u64, h: [u32; 4], buf: [u8; 64] }

const MD5_TAB: [u32; 64] = [
    0xd76aa478,0xe8c7b756,0x242070db,0xc1bdceee,0xf57c0faf,0x4787c62a,0xa8304613,0xfd469501,
    0x698098d8,0x8b44f7af,0xffff5bb1,0x895cd7be,0x6b901122,0xfd987193,0xa679438e,0x49b40821,
    0xf61e2562,0xc040b340,0x265e5a51,0xe9b6c7aa,0xd62f105d,0x02441453,0xd8a1e681,0xe7d3fbc8,
    0x21e1cde6,0xc33707d6,0xf4d50d87,0x455a14ed,0xa9e3e905,0xfcefa3f8,0x676f02d9,0x8d2a4c8a,
    0xfffa3942,0x8771f681,0x6d9d6122,0xfde5380c,0xa4beea44,0x4bdecfa9,0xf6bb4b60,0xbebfbc70,
    0x289b7ec6,0xeaa127fa,0xd4ef3085,0x04881d05,0xd9d4d039,0xe6db99e5,0x1fa27cf8,0xc4ac5665,
    0xf4292244,0x432aff97,0xab9423a7,0xfc93a039,0x655b59c3,0x8f0ccc92,0xffeff47d,0x85845dd1,
    0x6fa87e4f,0xfe2ce6e0,0xa3014314,0x4e0811a1,0xf7537e82,0xbd3af235,0x2ad7d2bb,0xeb86d391,
];

impl Md5 {
    fn init(&mut self) {
        self.len = 0;
        self.h = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];
    }
    fn processblock(&mut self, buf: &[u8; 64]) {
        let mut w = [0u32; 16];
        for i in 0..16 {
            w[i] = buf[4*i] as u32 | (buf[4*i+1] as u32)<<8 | (buf[4*i+2] as u32)<<16 | (buf[4*i+3] as u32)<<24;
        }
        let (mut a,mut b,mut c,mut d) = (self.h[0],self.h[1],self.h[2],self.h[3]);
        let mut i = 0usize;
        while i < 16 {
            a = a.wrapping_add((b&c|!b&d).wrapping_add(w[i]).wrapping_add(MD5_TAB[i])).rotate_left(7).wrapping_add(b); i+=1;
            d = d.wrapping_add((a&b|!a&c).wrapping_add(w[i]).wrapping_add(MD5_TAB[i])).rotate_left(12).wrapping_add(a); i+=1;
            c = c.wrapping_add((d&a|!d&b).wrapping_add(w[i]).wrapping_add(MD5_TAB[i])).rotate_left(17).wrapping_add(d); i+=1;
            b = b.wrapping_add((c&d|!c&a).wrapping_add(w[i]).wrapping_add(MD5_TAB[i])).rotate_left(22).wrapping_add(c); i+=1;
        }
        while i < 32 {
            a = a.wrapping_add((b&d|c&!d).wrapping_add(w[(5*i+1)%16]).wrapping_add(MD5_TAB[i])).rotate_left(5).wrapping_add(b); i+=1;
            d = d.wrapping_add((a&c|b&!c).wrapping_add(w[(5*i+1)%16]).wrapping_add(MD5_TAB[i])).rotate_left(9).wrapping_add(a); i+=1;
            c = c.wrapping_add((d&b|a&!b).wrapping_add(w[(5*i+1)%16]).wrapping_add(MD5_TAB[i])).rotate_left(14).wrapping_add(d); i+=1;
            b = b.wrapping_add((c&a|d&!a).wrapping_add(w[(5*i+1)%16]).wrapping_add(MD5_TAB[i])).rotate_left(20).wrapping_add(c); i+=1;
        }
        while i < 48 {
            a = a.wrapping_add((b^c^d).wrapping_add(w[(3*i+5)%16]).wrapping_add(MD5_TAB[i])).rotate_left(4).wrapping_add(b); i+=1;
            d = d.wrapping_add((a^b^c).wrapping_add(w[(3*i+5)%16]).wrapping_add(MD5_TAB[i])).rotate_left(11).wrapping_add(a); i+=1;
            c = c.wrapping_add((d^a^b).wrapping_add(w[(3*i+5)%16]).wrapping_add(MD5_TAB[i])).rotate_left(16).wrapping_add(d); i+=1;
            b = b.wrapping_add((c^d^a).wrapping_add(w[(3*i+5)%16]).wrapping_add(MD5_TAB[i])).rotate_left(23).wrapping_add(c); i+=1;
        }
        while i < 64 {
            a = a.wrapping_add((c^(b|!d)).wrapping_add(w[7*i%16]).wrapping_add(MD5_TAB[i])).rotate_left(6).wrapping_add(b); i+=1;
            d = d.wrapping_add((b^(a|!c)).wrapping_add(w[7*i%16]).wrapping_add(MD5_TAB[i])).rotate_left(10).wrapping_add(a); i+=1;
            c = c.wrapping_add((a^(d|!b)).wrapping_add(w[7*i%16]).wrapping_add(MD5_TAB[i])).rotate_left(15).wrapping_add(d); i+=1;
            b = b.wrapping_add((d^(c|!a)).wrapping_add(w[7*i%16]).wrapping_add(MD5_TAB[i])).rotate_left(21).wrapping_add(c); i+=1;
        }
        self.h[0] = self.h[0].wrapping_add(a);
        self.h[1] = self.h[1].wrapping_add(b);
        self.h[2] = self.h[2].wrapping_add(c);
        self.h[3] = self.h[3].wrapping_add(d);
    }
    fn update(&mut self, data: &[u8]) {
        let mut p = 0usize;
        let mut r = (self.len % 64) as usize;
        self.len += data.len() as u64;
        if r > 0 {
            let avail = 64 - r;
            if data.len() < avail { self.buf[r..r+data.len()].copy_from_slice(data); return; }
            self.buf[r..64].copy_from_slice(&data[..avail]);
            p = avail;
            let b = self.buf; self.processblock(&b);
        }
        while p + 64 <= data.len() {
            let mut blk = [0u8; 64]; blk.copy_from_slice(&data[p..p+64]); self.processblock(&blk); p += 64;
        }
        if p < data.len() { self.buf[..data.len()-p].copy_from_slice(&data[p..]); }
    }
    fn finish(&mut self, md: &mut [u8; 16]) {
        let r = (self.len % 64) as usize;
        self.buf[r] = 0x80;
        if r + 1 > 56 { for i in r+1..64 { self.buf[i]=0; } let b=self.buf; self.processblock(&b); for i in 0..56 { self.buf[i]=0; } }
        else { for i in r+1..56 { self.buf[i]=0; } }
        let bits = self.len.wrapping_mul(8);
        for i in 0..8 { self.buf[56+i] = (bits >> (i*8)) as u8; }
        let b = self.buf; self.processblock(&b);
        for i in 0..4 { md[4*i]=self.h[i] as u8; md[4*i+1]=(self.h[i]>>8) as u8; md[4*i+2]=(self.h[i]>>16) as u8; md[4*i+3]=(self.h[i]>>24) as u8; }
    }
}

fn md5crypt(key: &[u8], setting: &[u8], output: &mut [u8]) -> Option<usize> {
    if setting.len() < 3 || &setting[..3] != b"$1$" { return None; }
    let klen = { let mut n=0usize; while n<key.len() && n<=30000 && key[n]!=0 { n+=1; } if n>30000 { return None; } n };
    let slen = { let mut i=0usize; while i<8 && 3+i<setting.len() && setting[3+i]!=b'$' && setting[3+i]!=0 { i+=1; } i };
    let salt = &setting[3..3+slen];
    let mut ctx = Md5{len:0,h:[0;4],buf:[0;64]};
    let mut md = [0u8; 16];
    ctx.init(); ctx.update(&key[..klen]); ctx.update(salt); ctx.update(&key[..klen]); ctx.finish(&mut md);
    ctx.init(); ctx.update(&key[..klen]); ctx.update(&setting[..3+slen]);
    { let mut i=klen; while i>16 { ctx.update(&md); i-=16; } ctx.update(&md[..i]); }
    { let saved=md[0]; md[0]=0; let mut i=klen; while i>0 { if i&1!=0 { ctx.update(&[0u8]); } else { ctx.update(&[key[0]]); } i>>=1; } md[0]=saved; }
    ctx.finish(&mut md);
    for i in 0..1000u32 {
        ctx.init();
        if i%2!=0 { ctx.update(&key[..klen]); } else { ctx.update(&md); }
        if i%3!=0 { ctx.update(salt); }
        if i%7!=0 { ctx.update(&key[..klen]); }
        if i%2!=0 { ctx.update(&md); } else { ctx.update(&key[..klen]); }
        ctx.finish(&mut md);
    }
    output[..3+slen].copy_from_slice(&setting[..3+slen]);
    let mut p = 3+slen; output[p]=b'$'; p+=1;
    static PERM: [[u8;3];5] = [[0,6,12],[1,7,13],[2,8,14],[3,9,15],[4,10,5]];
    for j in 0..5 { let v=(md[PERM[j][0] as usize] as u32)<<16|(md[PERM[j][1] as usize] as u32)<<8|md[PERM[j][2] as usize] as u32; p+=crypt_to64(&mut output[p..],v,4); }
    p+=crypt_to64(&mut output[p..],md[11] as u32,2);
    output[p]=0;
    Some(p)
}

const MD5_TK: &[u8] = b"Xy01@#\x01\x02\x80\x7f\xff\r\n\x81\t !";
const MD5_TS: &[u8] = b"$1$abcd0123$";
const MD5_TH: &[u8] = b"$1$abcd0123$9Qcg8DyviekV3tDGMZynJ1";

#[no_mangle]
pub unsafe extern "C" fn __crypt_md5(key: *const c_char, setting: *const c_char, output: *mut c_char) -> *mut c_char {
    let ks = cstr_bytes(key); let ss = cstr_bytes(setting);
    let out = core::slice::from_raw_parts_mut(output as *mut u8, 256);
    let mut tb = [0u8; 64]; let q = md5crypt(MD5_TK, MD5_TS, &mut tb);
    let p = md5crypt(ks, ss, out);
    if p.is_none() || q.is_none() || tb[..MD5_TH.len()] != *MD5_TH { *output=b'*' as c_char; *output.add(1)=0; return output; }
    out[p.unwrap()]=0; output
}

// ============================================================
// SHA-256
// ============================================================

struct Sha256 { len: u64, h: [u32; 8], buf: [u8; 64] }

const SHA256_K: [u32; 64] = [
    0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
    0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
    0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
    0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
    0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
    0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
    0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
    0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2,
];

impl Sha256 {
    fn init(&mut self) {
        self.len = 0;
        self.h = [0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19];
    }
    fn processblock(&mut self, buf: &[u8; 64]) {
        let mut w = [0u32; 64];
        for i in 0..16 { w[i]=(buf[4*i] as u32)<<24|(buf[4*i+1] as u32)<<16|(buf[4*i+2] as u32)<<8|buf[4*i+3] as u32; }
        for i in 16..64 { let s0=w[i-15].rotate_right(7)^w[i-15].rotate_right(18)^(w[i-15]>>3); let s1=w[i-2].rotate_right(17)^w[i-2].rotate_right(19)^(w[i-2]>>10); w[i]=w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1); }
        let (mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut h)=(self.h[0],self.h[1],self.h[2],self.h[3],self.h[4],self.h[5],self.h[6],self.h[7]);
        for i in 0..64 {
            let s1=e.rotate_right(6)^e.rotate_right(11)^e.rotate_right(25); let ch=(e&f)^(!e&g); let t1=h.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA256_K[i]).wrapping_add(w[i]);
            let s0=a.rotate_right(2)^a.rotate_right(13)^a.rotate_right(22); let maj=(a&b)^(a&c)^(b&c); let t2=s0.wrapping_add(maj);
            h=g;g=f;f=e;e=d.wrapping_add(t1);d=c;c=b;b=a;a=t1.wrapping_add(t2);
        }
        self.h[0]=self.h[0].wrapping_add(a);self.h[1]=self.h[1].wrapping_add(b);self.h[2]=self.h[2].wrapping_add(c);self.h[3]=self.h[3].wrapping_add(d);
        self.h[4]=self.h[4].wrapping_add(e);self.h[5]=self.h[5].wrapping_add(f);self.h[6]=self.h[6].wrapping_add(g);self.h[7]=self.h[7].wrapping_add(h);
    }
    fn update(&mut self, data: &[u8]) {
        let mut p=0usize; let mut r=(self.len%64) as usize; self.len+=data.len() as u64;
        if r>0 { let avail=64-r; if data.len()<avail { self.buf[r..r+data.len()].copy_from_slice(data); return; } self.buf[r..64].copy_from_slice(&data[..avail]); p=avail; let b=self.buf; self.processblock(&b); }
        while p+64<=data.len() { let mut blk=[0u8;64]; blk.copy_from_slice(&data[p..p+64]); self.processblock(&blk); p+=64; }
        if p<data.len() { self.buf[..data.len()-p].copy_from_slice(&data[p..]); }
    }
    fn finish(&mut self, md: &mut [u8; 32]) {
        let r=(self.len%64) as usize; self.buf[r]=0x80;
        if r+1>56 { for i in r+1..64 { self.buf[i]=0; } let b=self.buf; self.processblock(&b); for i in 0..56 { self.buf[i]=0; } }
        else { for i in r+1..56 { self.buf[i]=0; } }
        let bits=self.len.wrapping_mul(8);
        for i in 0..8 { self.buf[56+i]=(bits>>(56-i*8)) as u8; }
        let b=self.buf; self.processblock(&b);
        for i in 0..8 { md[4*i]=(self.h[i]>>24) as u8; md[4*i+1]=(self.h[i]>>16) as u8; md[4*i+2]=(self.h[i]>>8) as u8; md[4*i+3]=self.h[i] as u8; }
    }
}

fn sha256_hashmd(s: &mut Sha256, n: usize, md: &[u8]) {
    let mut i=n; while i>32 { s.update(&md[..32]); i-=32; } s.update(&md[..i]);
}

fn sha256crypt(key: &[u8], setting: &[u8], output: &mut [u8]) -> Option<usize> {
    if setting.len()<3 || &setting[..3]!=b"$5$" { return None; }
    let klen = { let mut n=0usize; while n<key.len() && n<=256 && key[n]!=0 { n+=1; } if n>256 { return None; } n };
    let mut salt_start=3usize;
    let mut r: u32 = 5000;
    let mut rounds_buf = [0u8; 24]; let mut rounds_len = 0usize;
    if setting.len()>salt_start+7 && &setting[salt_start..salt_start+7]==b"rounds=" {
        salt_start+=7;
        if salt_start>=setting.len() || isdigit(setting[salt_start] as c_int)==0 { return None; }
        let mut ep:*mut c_char=core::ptr::null_mut();
        let u=unsafe{strtoul(setting[salt_start..].as_ptr() as *const c_char,&mut ep,10)};
        let consumed=(ep as usize)-(setting[salt_start..].as_ptr() as usize);
        if consumed==0 || setting.get(salt_start+consumed)!=Some(&b'$') { return None; }
        salt_start+=consumed+1;
        if u<1000 { r=1000; } else if u>9999999 { return None; } else { r=u as u32; }
        rounds_buf[..7].copy_from_slice(b"rounds="); rounds_len=7;
        let mut tmp=r; let mut db=[0u8;10]; let mut dl=0usize;
        if tmp==0 { db[0]=b'0'; dl=1; } else { while tmp>0 { db[dl]=b'0'+(tmp%10) as u8; tmp/=10; dl+=1; } db[..dl].reverse(); }
        rounds_buf[rounds_len..rounds_len+dl].copy_from_slice(&db[..dl]); rounds_len+=dl;
        rounds_buf[rounds_len]=b'$'; rounds_len+=1;
    }
    let salt=&setting[salt_start..];
    let slen={ let mut i=0usize; while i<16 && i<salt.len() && salt[i]!=b'$' && salt[i]!=0 { if salt[i]==b'\n'||salt[i]==b':' { return None; } i+=1; } i };
    let sd=&salt[..slen];
    let mut ctx=Sha256{len:0,h:[0;8],buf:[0;64]};
    let mut md=[0u8;32]; let mut kmd=[0u8;32]; let mut smd=[0u8;32];
    ctx.init(); ctx.update(&key[..klen]); ctx.update(sd); ctx.update(&key[..klen]); ctx.finish(&mut md);
    ctx.init(); ctx.update(&key[..klen]); ctx.update(sd); sha256_hashmd(&mut ctx,klen,&md); { let mut i=klen; while i>0 { if i&1!=0{ctx.update(&md);}else{ctx.update(&key[..klen]);} i>>=1; } } ctx.finish(&mut md);
    ctx.init(); { for _ in 0..klen { ctx.update(&key[..klen]); } } ctx.finish(&mut kmd);
    ctx.init(); { for _ in 0..16+md[0] as usize { ctx.update(sd); } } ctx.finish(&mut smd);
    for i in 0..r { ctx.init(); if i%2!=0{sha256_hashmd(&mut ctx,klen,&kmd);}else{ctx.update(&md);} if i%3!=0{ctx.update(&smd[..slen]);} if i%7!=0{sha256_hashmd(&mut ctx,klen,&kmd);} if i%2!=0{ctx.update(&md);}else{sha256_hashmd(&mut ctx,klen,&kmd);} ctx.finish(&mut md); }
    let mut p=0usize;
    output[p..p+3].copy_from_slice(b"$5$"); p+=3;
    if rounds_len>0 { output[p..p+rounds_len].copy_from_slice(&rounds_buf[..rounds_len]); p+=rounds_len; }
    output[p..p+slen].copy_from_slice(sd); p+=slen; output[p]=b'$'; p+=1;
    static PERM:[[u8;3];10]=[[0,10,20],[21,1,11],[12,22,2],[3,13,23],[24,4,14],[15,25,5],[6,16,26],[27,7,17],[18,28,8],[9,19,29]];
    for j in 0..10 { let v=(md[PERM[j][0]as usize]as u32)<<16|(md[PERM[j][1]as usize]as u32)<<8|md[PERM[j][2]as usize]as u32; p+=crypt_to64(&mut output[p..],v,4); }
    p+=crypt_to64(&mut output[p..],((md[31]as u32)<<8)|md[30]as u32,3); output[p]=0; Some(p)
}

const S256_TK:&[u8]=b"Xy01@#\x01\x02\x80\x7f\xff\r\n\x81\t !";
const S256_TS:&[u8]=b"$5$rounds=1234$abc0123456789$";
const S256_TH:&[u8]=b"$5$rounds=1234$abc0123456789$3VfDjPt05VHFn47C/ojFZ6KRPYrOjj1lLbH.dkF3bZ6";

#[no_mangle]
pub unsafe extern "C" fn __crypt_sha256(key:*const c_char,setting:*const c_char,output:*mut c_char)->*mut c_char {
    let ks=cstr_bytes(key); let ss=cstr_bytes(setting);
    let out=core::slice::from_raw_parts_mut(output as *mut u8,256);
    let mut tb=[0u8;128]; let q=sha256crypt(S256_TK,S256_TS,&mut tb);
    let p=sha256crypt(ks,ss,out);
    if p.is_none()||q.is_none()||tb[..S256_TH.len()]!=*S256_TH { *output=b'*' as c_char; *output.add(1)=0; return output; }
    out[p.unwrap()]=0; output
}

// ============================================================
// SHA-512
// ============================================================

struct Sha512 { len: u64, h: [u64; 8], buf: [u8; 128] }

const SHA512_K:[u64;80]=[
    0x428a2f98d728ae22,0x7137449123ef65cd,0xb5c0fbcfec4d3b2f,0xe9b5dba58189dbbc,
    0x3956c25bf348b538,0x59f111f1b605d019,0x923f82a4af194f9b,0xab1c5ed5da6d8118,
    0xd807aa98a3030242,0x12835b0145706fbe,0x243185be4ee4b28c,0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f,0x80deb1fe3b1696b1,0x9bdc06a725c71235,0xc19bf174cf692694,
    0xe49b69c19ef14ad2,0xefbe4786384f25e3,0x0fc19dc68b8cd5b5,0x240ca1cc77ac9c65,
    0x2de92c6f592b0275,0x4a7484aa6ea6e483,0x5cb0a9dcbd41fbd4,0x76f988da831153b5,
    0x983e5152ee66dfab,0xa831c66d2db43210,0xb00327c898fb213f,0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2,0xd5a79147930aa725,0x06ca6351e003826f,0x142929670a0e6e70,
    0x27b70a8546d22ffc,0x2e1b21385c26c926,0x4d2c6dfc5ac42aed,0x53380d139d95b3df,
    0x650a73548baf63de,0x766a0abb3c77b2a8,0x81c2c92e47edaee6,0x92722c851482353b,
    0xa2bfe8a14cf10364,0xa81a664bbc423001,0xc24b8b70d0f89791,0xc76c51a30654be30,
    0xd192e819d6ef5218,0xd69906245565a910,0xf40e35855771202a,0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8,0x1e376c085141ab53,0x2748774cdf8eeb99,0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63,0x4ed8aa4ae3418acb,0x5b9cca4f7763e373,0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc,0x78a5636f43172f60,0x84c87814a1f0ab72,0x8cc702081a6439ec,
    0x90befffa23631e28,0xa4506cebde82bde9,0xbef9a3f7b2c67915,0xc67178f2e372532b,
    0xca273eceea26619c,0xd186b8c721c0c207,0xeada7dd6cde0eb1e,0xf57d4f7fee6ed178,
    0x06f067aa72176fba,0x0a637dc5a2c898a6,0x113f9804bef90dae,0x1b710b35131c471b,
    0x28db77f523047d84,0x32caab7b40c72493,0x3c9ebe0a15c9bebc,0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6,0x597f299cfc657e2a,0x5fcb6fab3ad6faec,0x6c44198c4a475817,
];

impl Sha512 {
    fn init(&mut self) {
        self.len=0;
        self.h=[0x6a09e667f3bcc908,0xbb67ae8584caa73b,0x3c6ef372fe94f82b,0xa54ff53a5f1d36f1,
                0x510e527fade682d1,0x9b05688c2b3e6c1f,0x1f83d9abfb41bd6b,0x5be0cd19137e2179];
    }
    fn processblock(&mut self, buf: &[u8; 128]) {
        let mut w=[0u64;80];
        for i in 0..16 { w[i]=(buf[8*i]as u64)<<56|(buf[8*i+1]as u64)<<48|(buf[8*i+2]as u64)<<40|(buf[8*i+3]as u64)<<32|(buf[8*i+4]as u64)<<24|(buf[8*i+5]as u64)<<16|(buf[8*i+6]as u64)<<8|buf[8*i+7]as u64; }
        for i in 16..80 { let s0=w[i-15].rotate_right(1)^w[i-15].rotate_right(8)^(w[i-15]>>7); let s1=w[i-2].rotate_right(19)^w[i-2].rotate_right(61)^(w[i-2]>>6); w[i]=w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1); }
        let (mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut h)=(self.h[0],self.h[1],self.h[2],self.h[3],self.h[4],self.h[5],self.h[6],self.h[7]);
        for i in 0..80 {
            let s1=e.rotate_right(14)^e.rotate_right(18)^e.rotate_right(41); let ch=(e&f)^(!e&g); let t1=h.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA512_K[i]).wrapping_add(w[i]);
            let s0=a.rotate_right(28)^a.rotate_right(34)^a.rotate_right(39); let maj=(a&b)^(a&c)^(b&c); let t2=s0.wrapping_add(maj);
            h=g;g=f;f=e;e=d.wrapping_add(t1);d=c;c=b;b=a;a=t1.wrapping_add(t2);
        }
        self.h[0]=self.h[0].wrapping_add(a);self.h[1]=self.h[1].wrapping_add(b);self.h[2]=self.h[2].wrapping_add(c);self.h[3]=self.h[3].wrapping_add(d);
        self.h[4]=self.h[4].wrapping_add(e);self.h[5]=self.h[5].wrapping_add(f);self.h[6]=self.h[6].wrapping_add(g);self.h[7]=self.h[7].wrapping_add(h);
    }
    fn update(&mut self, data: &[u8]) {
        let mut p=0usize; let mut r=(self.len%128) as usize; self.len+=data.len() as u64;
        if r>0 { let avail=128-r; if data.len()<avail { self.buf[r..r+data.len()].copy_from_slice(data); return; } self.buf[r..128].copy_from_slice(&data[..avail]); p=avail; let b=self.buf; self.processblock(&b); }
        while p+128<=data.len() { let mut blk=[0u8;128]; blk.copy_from_slice(&data[p..p+128]); self.processblock(&blk); p+=128; }
        if p<data.len() { self.buf[..data.len()-p].copy_from_slice(&data[p..]); }
    }
    fn finish(&mut self, md: &mut [u8; 64]) {
        let r=(self.len%128) as usize; self.buf[r]=0x80;
        if r+1>112 { for i in r+1..128 { self.buf[i]=0; } let b=self.buf; self.processblock(&b); for i in 0..120 { self.buf[i]=0; } }
        else { for i in r+1..120 { self.buf[i]=0; } }
        let bits=self.len.wrapping_mul(8);
        for i in 0..8 { self.buf[120+i]=(bits>>(56-i*8)) as u8; }
        let b=self.buf; self.processblock(&b);
        for i in 0..8 { md[8*i]=(self.h[i]>>56)as u8; md[8*i+1]=(self.h[i]>>48)as u8; md[8*i+2]=(self.h[i]>>40)as u8; md[8*i+3]=(self.h[i]>>32)as u8; md[8*i+4]=(self.h[i]>>24)as u8; md[8*i+5]=(self.h[i]>>16)as u8; md[8*i+6]=(self.h[i]>>8)as u8; md[8*i+7]=self.h[i]as u8; }
    }
}

fn sha512_hashmd(s: &mut Sha512, n: usize, md: &[u8]) {
    let mut i=n; while i>64 { s.update(&md[..64]); i-=64; } s.update(&md[..i]);
}

fn sha512crypt(key: &[u8], setting: &[u8], output: &mut [u8]) -> Option<usize> {
    if setting.len()<3 || &setting[..3]!=b"$6$" { return None; }
    let klen = { let mut n=0usize; while n<key.len() && n<=256 && key[n]!=0 { n+=1; } if n>256 { return None; } n };
    let mut salt_start=3usize;
    let mut r: u32 = 5000;
    let mut rounds_buf = [0u8; 24]; let mut rounds_len = 0usize;
    if setting.len()>salt_start+7 && &setting[salt_start..salt_start+7]==b"rounds=" {
        salt_start+=7;
        if salt_start>=setting.len() || isdigit(setting[salt_start] as c_int)==0 { return None; }
        let mut ep:*mut c_char=core::ptr::null_mut();
        let u=unsafe{strtoul(setting[salt_start..].as_ptr() as *const c_char,&mut ep,10)};
        let consumed=(ep as usize)-(setting[salt_start..].as_ptr() as usize);
        if consumed==0 || setting.get(salt_start+consumed)!=Some(&b'$') { return None; }
        salt_start+=consumed+1;
        if u<1000 { r=1000; } else if u>9999999 { return None; } else { r=u as u32; }
        rounds_buf[..7].copy_from_slice(b"rounds="); rounds_len=7;
        let mut tmp=r; let mut db=[0u8;10]; let mut dl=0usize;
        if tmp==0 { db[0]=b'0'; dl=1; } else { while tmp>0 { db[dl]=b'0'+(tmp%10) as u8; tmp/=10; dl+=1; } db[..dl].reverse(); }
        rounds_buf[rounds_len..rounds_len+dl].copy_from_slice(&db[..dl]); rounds_len+=dl;
        rounds_buf[rounds_len]=b'$'; rounds_len+=1;
    }
    let salt=&setting[salt_start..];
    let slen={ let mut i=0usize; while i<16 && i<salt.len() && salt[i]!=b'$' && salt[i]!=0 { if salt[i]==b'\n'||salt[i]==b':' { return None; } i+=1; } i };
    let sd=&salt[..slen];
    let mut ctx=Sha512{len:0,h:[0;8],buf:[0;128]};
    let mut md=[0u8;64]; let mut kmd=[0u8;64]; let mut smd=[0u8;64];
    ctx.init(); ctx.update(&key[..klen]); ctx.update(sd); ctx.update(&key[..klen]); ctx.finish(&mut md);
    ctx.init(); ctx.update(&key[..klen]); ctx.update(sd); sha512_hashmd(&mut ctx,klen,&md); { let mut i=klen; while i>0 { if i&1!=0{ctx.update(&md);}else{ctx.update(&key[..klen]);} i>>=1; } } ctx.finish(&mut md);
    ctx.init(); { for _ in 0..klen { ctx.update(&key[..klen]); } } ctx.finish(&mut kmd);
    ctx.init(); { for _ in 0..16+md[0] as usize { ctx.update(sd); } } ctx.finish(&mut smd);
    for i in 0..r { ctx.init(); if i%2!=0{sha512_hashmd(&mut ctx,klen,&kmd);}else{ctx.update(&md);} if i%3!=0{ctx.update(&smd[..slen]);} if i%7!=0{sha512_hashmd(&mut ctx,klen,&kmd);} if i%2!=0{ctx.update(&md);}else{sha512_hashmd(&mut ctx,klen,&kmd);} ctx.finish(&mut md); }
    let mut p=0usize;
    output[p..p+3].copy_from_slice(b"$6$"); p+=3;
    if rounds_len>0 { output[p..p+rounds_len].copy_from_slice(&rounds_buf[..rounds_len]); p+=rounds_len; }
    output[p..p+slen].copy_from_slice(sd); p+=slen; output[p]=b'$'; p+=1;
    static PERM:[[u8;3];21]=[[0,21,42],[22,43,1],[44,2,23],[3,24,45],[25,46,4],[47,5,26],[6,27,48],[28,49,7],[50,8,29],[9,30,51],[31,52,10],[53,11,32],[12,33,54],[34,55,13],[56,14,35],[15,36,57],[37,58,16],[59,17,38],[18,39,60],[40,61,19],[62,20,41]];
    for j in 0..21 { let v=(md[PERM[j][0]as usize]as u32)<<16|(md[PERM[j][1]as usize]as u32)<<8|md[PERM[j][2]as usize]as u32; p+=crypt_to64(&mut output[p..],v,4); }
    p+=crypt_to64(&mut output[p..],md[63] as u32,2); output[p]=0; Some(p)
}

const S512_TK:&[u8]=b"Xy01@#\x01\x02\x80\x7f\xff\r\n\x81\t !";
const S512_TS:&[u8]=b"$6$rounds=1234$abc0123456789$";
const S512_TH:&[u8]=b"$6$rounds=1234$abc0123456789$BCpt8zLrc/RcyuXmCDOE1ALqMXB2MH6n1g891HhFj8.w7LxGv.FTkqq6Vxc/km3Y0jE0j24jY5PIv/oOu6reg1";

#[no_mangle]
pub unsafe extern "C" fn __crypt_sha512(key:*const c_char,setting:*const c_char,output:*mut c_char)->*mut c_char {
    let ks=cstr_bytes(key); let ss=cstr_bytes(setting);
    let out=core::slice::from_raw_parts_mut(output as *mut u8,256);
    let mut tb=[0u8;128]; let q=sha512crypt(S512_TK,S512_TS,&mut tb);
    let p=sha512crypt(ks,ss,out);
    if p.is_none()||q.is_none()||tb[..S512_TH.len()]!=*S512_TH { *output=b'*' as c_char; *output.add(1)=0; return output; }
    out[p.unwrap()]=0; output
}

// ============================================================
// Blowfish (bcrypt)
// ============================================================

const BF_N: usize = 16;

const BF_INIT_P: [u32; 18] = [
    0x243f6a88,0x85a308d3,0x13198a2e,0x03707344,0xa4093822,0x299f31d0,0x082efa98,0xec4e6c89,
    0x452821e6,0x38d01377,0xbe5466cf,0x34e90c6c,0xc0ac29b7,0xc97c50dd,0x3f84d5b5,0xb5470917,
    0x9216d5d9,0x8979fb1b,
];

const BF_ITOA64: &[u8; 64] = b"./ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

const BF_ATOI64: [u8; 0x60] = [
    64,64,64,64,64,64,64,64,64,64,64,64,64,64,0,1,
    54,55,56,57,58,59,60,61,62,63,64,64,64,64,64,64,
    64,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,
    17,18,19,20,21,22,23,24,25,26,27,64,64,64,64,64,
    64,28,29,30,31,32,33,34,35,36,37,38,39,40,41,42,
    43,44,45,46,47,48,49,50,51,52,53,64,64,64,64,64,
];

const BF_FLAGS_BY_SUBTYPE: [u8; 26] = [2,4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,4,0];

const BF_MAGIC_W: [u32; 6] = [0x4F727068, 0x65616E42, 0x65686F6C, 0x64657253, 0x63727944, 0x6F756274];

// ponytail: Blowfish S-boxes stored as 4 flat arrays; total ~4KB static data
const BF_S0: [u32; 256] = [
    0xd1310ba6,0x98dfb5ac,0x2ffd72db,0xd01adfb7,0xb8e1afed,0x6a267e96,0xba7c9045,0xf12c7f99,
    0x24a19947,0xb3916cf7,0x0801f2e2,0x858efc16,0x636920d8,0x71574e69,0xa458fea3,0xf4933d7e,
    0x0d95748f,0x728eb658,0x718bcd58,0x82154aee,0x7b54a41d,0xc25a59b5,0x9c30d539,0x2af26013,
    0xc5d1b023,0x286085f0,0xca417918,0xb8db38ef,0x8e79dcb0,0x603a180e,0x6c9e0e8b,0xb01e8a3e,
    0xd71577c1,0xbd314b27,0x78af2fda,0x55605c60,0xe65525f3,0xaa55ab94,0x57489862,0x63e81440,
    0x55ca396a,0x2aab10b6,0xb4cc5c34,0x1141e8ce,0xa15486af,0x7c72e993,0xb3ee1411,0x636fbc2a,
    0x2ba9c55d,0x741831f6,0xce5c3e16,0x9b87931e,0xafd6ba33,0x6c24cf5c,0x7a325381,0x28958677,
    0x3b8f4898,0x6b4bb9af,0xc4bfe81b,0x66282193,0x61d809cc,0xfb21a991,0x487cac60,0x5dec8032,
    0xef845d5d,0xe98575b1,0xdc262302,0xeb651b88,0x23893e81,0xd396acc5,0x0f6d6ff3,0x83f44239,
    0x2e0b4482,0xa4842004,0x69c8f04a,0x9e1f9b5e,0x21c66842,0xf6e96c9a,0x670c9c61,0xabd388f0,
    0x6a51a0d2,0xd8542f68,0x960fa728,0xab5133a3,0x6eef0b6c,0x137a3be4,0xba3bf050,0x7efb2a98,
    0xa1f1651d,0x39af0176,0x66ca593e,0x82430e88,0x8cee8619,0x456f9fb4,0x7d84a5c3,0x3b8b5ebe,
    0xe06f75d8,0x85c12073,0x401a449f,0x56c16aa6,0x4ed3aa62,0x363f7706,0x1bfedf72,0x429b023d,
    0x37d0d724,0xd00a1248,0xdb0fead3,0x49f1c09b,0x075372c9,0x80991b7b,0x25d479d8,0xf6e8def7,
    0xe3fe501a,0xb6794c3b,0x976ce0bd,0x04c006ba,0xc1a94fb6,0x409f60c4,0x5e5c9ec2,0x196a2463,
    0x68fb6faf,0x3e6c53b5,0x1339b2eb,0x3b52ec6f,0x6dfc511f,0x9b30952c,0xcc814544,0xaf5ebd09,
    0xbee3d004,0xde334afd,0x660f2807,0x192e4bb3,0xc0cba857,0x45c8740f,0xd20b5f39,0xb9d3fbdb,
    0x5579c0bd,0x1a60320a,0xd6a100c6,0x402c7279,0x679f25fe,0xfb1fa3cc,0x8ea5e9f8,0xdb3222f8,
    0x3c7516df,0xfd616b15,0x2f501ec8,0xad0552ab,0x323db5fa,0xfd238760,0x53317b48,0x3e00df82,
    0x9e5c57bb,0xca6f8ca0,0x1a87562e,0xdf1769db,0xd542a8f6,0x287effc3,0xac6732c6,0x8c4f5573,
    0x695b27b0,0xbbca58c8,0xe1ffa35d,0xb8f011a0,0x10fa3d98,0xfd2183b8,0x4afcb56c,0x2dd1d35b,
    0x9a53e479,0xb6f84565,0xd28e49bc,0x4bfb9790,0xe1ddf2da,0xa4cb7e33,0x62fb1341,0xcee4c6e8,
    0xef20cada,0x36774c01,0xd07e9efe,0x2bf11fb4,0x95dbda4d,0xae909198,0xeaad8e71,0x6b93d5a0,
    0xd08ed1d0,0xafc725e0,0x8e3c5b2f,0x8e7594b7,0x8ff6e2fb,0xf2122b64,0x8888b812,0x900df01c,
    0x4fad5ea0,0x688fc31c,0xd1cff191,0xb3a8c1ad,0x2f2f2218,0xbe0e1777,0xea752dfe,0x8b021fa1,
    0xe5a0cc0f,0xb56f74e8,0x18acf3d6,0xce89e299,0xb4a84fe0,0xfd13e0b7,0x7cc43b81,0xd2ada8d9,
    0x165fa266,0x80957705,0x93cc7314,0x211a1477,0xe6ad2065,0x77b5fa86,0xc75442f5,0xfb9d35cf,
    0xebcdaf0c,0x7b3e89a0,0xd6411bd3,0xae1e7e49,0x00250e2d,0x2071b35e,0x226800bb,0x57b8e0af,
    0x2464369b,0xf009b91e,0x5563911d,0x59dfa6aa,0x78c14389,0xd95a537f,0x207d5ba2,0x02e5b9c5,
    0x83260376,0x6295cfa9,0x11c81968,0x4e734a41,0xb3472dca,0x7b14a94a,0x1b510052,0x9a532915,
    0xd60f573f,0xbc9bc6e4,0x2b60a476,0x81e67400,0x08ba6fb5,0x571be91f,0xf296ec6b,0x2a0dd915,
    0xb6636521,0xe7b9f9b6,0xff34052e,0xc5855664,0x53b02d5d,0xa99f8fa1,0x08ba4799,0x6e85076a,
];
const BF_S1: [u32; 256] = [
    0x4b7a70e9,0xb5b32944,0xdb75092e,0xc4192623,0xad6ea6b0,0x49a7df7d,0x9cee60b8,0x8fedb266,
    0xecaa8c71,0x699a17ff,0x5664526c,0xc2b19ee1,0x193602a5,0x75094c29,0xa0591340,0xe4183a3e,
    0x3f54989a,0x5b429d65,0x6b8fe4d6,0x99f73fd6,0xa1d29c07,0xefe830f5,0x4d2d38e6,0xf0255dc1,
    0x4cdd2086,0x8470eb26,0x6382e9c6,0x021ecc5e,0x09686b3f,0x3ebaefc9,0x3c971814,0x6b6a70a1,
    0x687f3584,0x52a0e286,0xb79c5305,0xaa500737,0x3e07841c,0x7fdeae5c,0x8e7d44ec,0x5716f2b8,
    0xb03ada37,0xf0500c0d,0xf01c1f04,0x0200b3ff,0xae0cf51a,0x3cb574b2,0x25837a58,0xdc0921bd,
    0xd19113f9,0x7ca92ff6,0x94324773,0x22f54701,0x3ae5e581,0x37c2dadc,0xc8b57634,0x9af3dda7,
    0xa9446146,0x0fd0030e,0xecc8c73e,0xa4751e41,0xe238cd99,0x3bea0e2f,0x3280bba1,0x183eb331,
    0x4e548b38,0x4f6db908,0x6f420d03,0xf60a04bf,0x2cb81290,0x24977c79,0x5679b072,0xbcaf89af,
    0xde9a771f,0xd9930810,0xb38bae12,0xdccf3f2e,0x5512721f,0x2e6b7124,0x501adde6,0x9f84cd87,
    0x7a584718,0x7408da17,0xbc9f9abc,0xe94b7d8c,0xec7aec3a,0xdb851dfa,0x63094366,0xc464c3d2,
    0xef1c1847,0x3215d908,0xdd433b37,0x24c2ba16,0x12a14d43,0x2a65c451,0x50940002,0x133ae4dd,
    0x71dff89e,0x10314e55,0x81ac77d6,0x5f11199b,0x043556f1,0xd7a3c76b,0x3c11183b,0x5924a509,
    0xf28fe6ed,0x97f1fbfa,0x9ebabf2c,0x1e153c6e,0x86e34570,0xeae96fb1,0x860e5e0a,0x5a3e2ab3,
    0x771fe71c,0x4e3d06fa,0x2965dcb9,0x99e71d0f,0x803e89d6,0x5266c825,0x2e4cc978,0x9c10b36a,
    0xc6150eba,0x94e2ea78,0xa5fc3c53,0x1e0a2df4,0xf2f74ea7,0x361d2b3d,0x1939260f,0x19c27960,
    0x5223a708,0xf71312b6,0xebadfe6e,0xeac31f66,0xe3bc4595,0xa67bc883,0xb17f37d1,0x018cff28,
    0xc332ddef,0xbe6c5aa5,0x65582185,0x68ab9802,0xeecea50f,0xdb2f953b,0x2aef7dad,0x5b6e2f84,
    0x1521b628,0x29076170,0xecdd4775,0x619f1510,0x13cca830,0xeb61bd96,0x0334fe1e,0xaa0363cf,
    0xb5735c90,0x4c70a239,0xd59e9e0b,0xcbaade14,0xeecc86bc,0x60622ca7,0x9cab5cab,0xb2f3846e,
    0x648b1eaf,0x19bdf0ca,0xa02369b9,0x655abb50,0x40685a32,0x3c2ab4b3,0x319ee9d5,0xc021b8f7,
    0x9b540b19,0x875fa099,0x95f7997e,0x623d7da8,0xf837889a,0x97e32d77,0x11ed935f,0x16681281,
    0x0e358829,0xc7e61fd6,0x96dedfa1,0x7858ba99,0x57f584a5,0x1b227263,0x9b83c3ff,0x1ac24696,
    0xcdb30aeb,0x532e3054,0x8fd948e4,0x6dbc3128,0x58ebf2ef,0x34c6ffea,0xfe28ed61,0xee7c3c73,
    0x5d4a14d9,0xe864b7e3,0x42105d14,0x203e13e0,0x45eee2b6,0xa3aaabea,0xdb6c4f15,0xfacb4fd0,
    0xc742f442,0xef6abbb5,0x654f3b1d,0x41cd2105,0xd81e799e,0x86854dc7,0xe44b476a,0x3d816250,
    0xcf62a1f2,0x5b8d2646,0xfc8883a0,0xc1c7b6a3,0x7f1524c3,0x69cb7492,0x47848a0b,0x5692b285,
    0x095bbf00,0xad19489d,0x1462b174,0x23820e00,0x58428d2a,0x0c55f5ea,0x1dadf43e,0x233f7061,
    0x3372f092,0x8d937e41,0xd65fecf1,0x6c223bdb,0x7cde3759,0xcbee7460,0x4085f2a7,0xce77326e,
    0xa6078084,0x19f8509e,0xe8efd855,0x61d99735,0xa969a7aa,0xc50c06c2,0x5a04abfc,0x800bcadc,
    0x9e447a2e,0xc3453484,0xfdd56705,0x0e1e9ec9,0xdb73dbd3,0x105588cd,0x675fda79,0xe3674340,
    0xc5c43465,0x713e38d8,0x3d28f89e,0xf16dff20,0x153e21e7,0x8fb03d4a,0xe6e39f2b,0xdb83adf7,
];
const BF_S2: [u32; 256] = [
    0xe93d5a68,0x948140f7,0xf64c261c,0x94692934,0x411520f7,0x7602d4f7,0xbcf46b2e,0xd4a20068,
    0xd4082471,0x3320f46a,0x43b7d4b7,0x500061af,0x1e39f62e,0x97244546,0x14214f74,0xbf8b8840,
    0x4d95fc1d,0x96b591af,0x70f4ddd3,0x66a02f45,0xbfbc09ec,0x03bd9785,0x7fac6dd0,0x31cb8504,
    0x96eb27b3,0x55fd3941,0xda2547e6,0xabca0a9a,0x28507825,0x530429f4,0x0a2c86da,0xe9b66dfb,
    0x68dc1462,0xd7486900,0x680ec0a4,0x27a18dee,0x4f3ffea2,0xe887ad8c,0xb58ce006,0x7af4d6b6,
    0xaace1e7c,0xd3375fec,0xce78a399,0x406b2a42,0x20fe9e35,0xd9f385b9,0xee39d7ab,0x3b124e8b,
    0x1dc9faf7,0x4b6d1856,0x26a36631,0xeae397b2,0x3a6efa74,0xdd5b4332,0x6841e7f7,0xca7820fb,
    0xfb0af54e,0xd8feb397,0x454056ac,0xba489527,0x55533a3a,0x20838d87,0xfe6ba9b7,0xd096954b,
    0x55a867bc,0xa1159a58,0xcca92963,0x99e1db33,0xa62a4a56,0x3f3125f9,0x5ef47e1c,0x9029317c,
    0xfdf8e802,0x04272f70,0x80bb155c,0x05282ce3,0x95c11548,0xe4c66d22,0x48c1133f,0xc70f86dc,
    0x07f9c9ee,0x41041f0f,0x404779a4,0x5d886e17,0x325f51eb,0xd59bc0d1,0xf2bcc18f,0x41113564,
    0x257b7834,0x602a9c60,0xdff8e8a3,0x1f636c1b,0x0e12b4c2,0x02e1329e,0xaf664fd1,0xcad18115,
    0x6b2395e0,0x333e92e1,0x3b240b62,0xeebeb922,0x85b2a20e,0xe6ba0d99,0xde720c8c,0x2da2f728,
    0xd0127845,0x95b794fd,0x647d0862,0xe7ccf5f0,0x5449a36f,0x877d48fa,0xc39dfd27,0xf33e8d1e,
    0x0a476341,0x992eff74,0x3a6f6eab,0xf4f8fd37,0xa812dc60,0xa1ebddf8,0x991be14c,0xdb6e6b0d,
    0xc67b5510,0x6d672c37,0x2765d43b,0xdcd0e804,0xf1290dc7,0xcc00ffa3,0xb5390f92,0x690fed0b,
    0x667b9ffb,0xcedb7d9c,0xa091cf0b,0xd9155ea3,0xbb132f88,0x515bad24,0x7b9479bf,0x763bd6eb,
    0x37392eb3,0xcc115979,0x8026e297,0xf42e312d,0x6842ada7,0xc66a2b3b,0x12754ccc,0x782ef11c,
    0x6a124237,0xb79251e7,0x06a1bbe6,0x4bfb6350,0x1a6b1018,0x11caedfa,0x3d25bdd8,0xe2e1c3c9,
    0x44421659,0x0a121386,0xd90cec6e,0xd5abea2a,0x64af674e,0xda86a85f,0xbebfe988,0x64e4c3fe,
    0x9dbc8057,0xf0f7c086,0x60787bf8,0x6003604d,0xd1fd8346,0xf6381fb0,0x7745ae04,0xd736fccc,
    0x83426b33,0xf01eab71,0xb0804187,0x3c005e5f,0x77a057be,0xbde8ae24,0x55464299,0xbf582e61,
    0x4e58f48f,0xf2ddfda2,0xf474ef38,0x8789bdc2,0x5366f9c3,0xc8b38e74,0xb475f255,0x46fcd9b9,
    0x7aeb2661,0x8b1ddf84,0x846a0e79,0x915f95e2,0x466e598e,0x20b45770,0x8cd55591,0xc902de4c,
    0xb90bace1,0xbb8205d0,0x11a86248,0x7574a99e,0xb77f19b6,0xe0a9dc09,0x662d09a1,0xc4324633,
    0xe85a1f02,0x09f0be8c,0x4a99a025,0x1d6efe10,0x1ab93d1d,0x0ba5a4df,0xa186f20f,0x2868f169,
    0xdcb7da83,0x573906fe,0xa1e2ce9b,0x4fcd7f52,0x50115e01,0xa70683fa,0xa002b5c4,0x0de6d027,
    0x9af88c27,0x773f8641,0xc3604c06,0x61a806b5,0xf0177a28,0xc0f586e0,0x006058aa,0x30dc7d62,
    0x11e69ed7,0x2338ea63,0x53c2dd94,0xc2c21634,0xbbcbee56,0x90bcb6de,0xebfc7da1,0xce591d76,
    0x6f05e409,0x4b7c0188,0x39720a3d,0x7c927c24,0x86e3725f,0x724d9db9,0x1ac15bb4,0xd39eb8fc,
    0xed545578,0x08fca5b5,0xd83d7cd3,0x4dad0fc4,0x1e50ef5e,0xb161e6f8,0xa28514d9,0x6c51133c,
    0x6fd5c7e7,0x56e14ec4,0x362abfce,0xddc6c837,0xd79a3234,0x92638212,0x670efa8e,0x406000e0,
];
const BF_S3: [u32; 256] = [
    0x3a39ce37,0xd3faf5cf,0xabc27737,0x5ac52d1b,0x5cb0679e,0x4fa33742,0xd3822740,0x99bc9bbe,
    0xd5118e9d,0xbf0f7315,0xd62d1c7e,0xc700c47b,0xb78c1b6b,0x21a19045,0xb26eb1be,0x6a366eb4,
    0x5748ab2f,0xbc946e79,0xc6a376d2,0x6549c2c8,0x530ff8ee,0x468dde7d,0xd5730a1d,0x4cd04dc6,
    0x2939bbdb,0xa9ba4650,0xac9526e8,0xbe5ee304,0xa1fad5f0,0x6a2d519a,0x63ef8ce2,0x9a86ee22,
    0xc089c2b8,0x43242ef6,0xa51e03aa,0x9cf2d0a4,0x83c061ba,0x9be96a4d,0x8fe51550,0xba645bd6,
    0x2826a2f9,0xa73a3ae1,0x4ba99586,0xef5562e9,0xc72fefd3,0xf752f7da,0x3f046f69,0x77fa0a59,
    0x80e4a915,0x87b08601,0x9b09e6ad,0x3b3ee593,0xe990fd5a,0x9e34d797,0x2cf0b7d9,0x022b8b51,
    0x96d5ac3a,0x017da67d,0xd1cf3ed6,0x7c7d2d28,0x1f9f25cf,0xadf2b89b,0x5ad6b472,0x5a88f54c,
    0xe029ac71,0xe019a5e6,0x47b0acfd,0xed93fa9b,0xe8d3c48d,0x283b57cc,0xf8d56629,0x79132e28,
    0x785f0191,0xed756055,0xf7960e44,0xe3d35e8c,0x15056dd4,0x88f46dba,0x03a16125,0x0564f0bd,
    0xc3eb9e15,0x3c9057a2,0x97271aec,0xa93a072a,0x1b3f6d9b,0x1e6321f5,0xf59c66fb,0x26dcf319,
    0x7533d928,0xb155fdf5,0x03563482,0x8aba3cbb,0x28517711,0xc20ad9f8,0xabcc5167,0xccad925f,
    0x4de81751,0x3830dc8e,0x379d5862,0x9320f991,0xea7a90c2,0xfb3e7bce,0x5121ce64,0x774fbe32,
    0xa8b6e37e,0xc3293d46,0x48de5369,0x6413e680,0xa2ae0810,0xdd6db224,0x69852dfd,0x09072166,
    0xb39a460a,0x6445c0dd,0x586cdecf,0x1c20c8ae,0x5bbef7dd,0x1b588d40,0xccd2017f,0x6bb4e3bb,
    0xdda26a7e,0x3a59ff45,0x3e350a44,0xbcb4cdd5,0x72eacea8,0xfa6484bb,0x8d6612ae,0xbf3c6f47,
    0xd29be463,0x542f5d9e,0xaec2771b,0xf64e6370,0x740e0d8d,0xe75b1357,0xf8721671,0xaf537d5d,
    0x4040cb08,0x4eb4e2cc,0x34d2466a,0x0115af84,0xe1b00428,0x95983a1d,0x06b89fb4,0xce6ea048,
    0x6f3f3b82,0x3520ab82,0x011a1d4b,0x277227f8,0x611560b1,0xe7933fdc,0xbb3a792b,0x344525bd,
    0xa08839e1,0x51ce794b,0x2f32c9b7,0xa01fbac9,0xe01cc87e,0xbcc7d1f6,0xcf0111c3,0xa1e8aac7,
    0x1a908749,0xd44fbd9a,0xd0dadecb,0xd50ada38,0x0339c32a,0xc6913667,0x8df9317c,0xe0b12b4f,
    0xf79e59b7,0x43f5bb3a,0xf2d519ff,0x27d9459c,0xbf97222c,0x15e6fc2a,0x0f91fc71,0x9b941525,
    0xfae59361,0xceb69ceb,0xc2a86459,0x12baa8d1,0xb6c1075e,0xe3056a0c,0x10d25065,0xcb03a442,
    0xe0ec6e0e,0x1698db3b,0x4c98a0be,0x3278e964,0x9f1f9532,0xe0d392df,0xd3a0342b,0x8971f21e,
    0x1b0a7441,0x4ba3348c,0xc5be7120,0xc37632d8,0xdf359f8d,0x9b992f2e,0xe60b6f47,0x0fe3f11d,
    0xe54cda54,0x1edad891,0xce6279cf,0xcd3e7e6f,0x1618b166,0xfd2c1d05,0x848fd2c5,0xf6fb2299,
    0xf523f357,0xa6327623,0x93a83531,0x56cccd02,0xacf08162,0x5a75ebb5,0x6e163697,0x88d273cc,
    0xde966292,0x81b949d0,0x4c50901b,0x71c65614,0xe6c6c7bd,0x327a140a,0x45e1d006,0xc3f27b9a,
    0xc9aa53fd,0x62a80f00,0xbb25bfe2,0x35bdd2f6,0x71126905,0xb2040222,0xb6cbcf7c,0xcd769c2b,
    0x53113ec0,0x1640e3d3,0x38abbd60,0x2547adf0,0xba38209c,0xf746ce76,0x77afa1c5,0x20756060,
    0x85cbfe4e,0x8ae88dd8,0x7aaaf9b0,0x4cf9aa7e,0x1948c25c,0x02fb8a8c,0x01c36ae4,0xd6ebe1f9,
    0x90d4f869,0xa65cdea0,0x3f09252d,0xc208e69f,0xb74e6132,0xce77e25b,0x578fdfe3,0x3ac372e6,
];

fn bf_s(idx: usize) -> u32 {
    match idx >> 8 {
        0 => BF_S0[idx & 0xFF],
        1 => BF_S1[idx & 0xFF],
        2 => BF_S2[idx & 0xFF],
        _ => BF_S3[idx & 0xFF],
    }
}

// BF_ROUND macro inlined: encrypts L through one Feistel round using P[i+1]
// Returns new R value
fn bf_round(l: u32, r: u32, p: &[u32; 18], pidx: usize) -> u32 {
    let mut tmp = bf_s(0x300 + (l & 0xFF) as usize);
    tmp = tmp.wrapping_add(bf_s(0x200 + ((l >> 8) & 0xFF) as usize));
    tmp ^= bf_s(0x100 + ((l >> 16) & 0xFF) as usize);
    tmp = tmp.wrapping_add(bf_s((l >> 24) as usize));
    r ^ p[pidx] ^ tmp
}

fn bf_crypt_inner(key: &[u8], setting: &[u8], output: &mut [u8], min: u32) -> Option<usize> {
    if setting.len() < 7 { return None; }
    if setting[0] != b'$' || setting[1] != b'2' { return None; }
    let subtype = setting[2];
    if subtype < b'a' || subtype > b'z' { return None; }
    let st_idx = (subtype - b'a') as usize;
    if st_idx >= 26 || BF_FLAGS_BY_SUBTYPE[st_idx] == 0 { return None; }
    if setting[3] != b'$' { return None; }
    if setting[4] < b'0' || setting[4] > b'1' { return None; }
    if setting[5] < b'0' || setting[5] > b'9' { return None; }
    if setting[6] != b'$' { return None; }

    let count = 1u32 << (((setting[4] - b'0') as u32) * 10 + ((setting[5] - b'0') as u32));
    if count < min { return None; }

    if setting.len() < 7 + 22 { return None; }
    let mut salt_bytes = [0u8; 16];
    if bf_decode(&mut salt_bytes, &setting[7..7+22]) { return None; }
    let salt_words = [
        (salt_bytes[0] as u32) << 24 | (salt_bytes[1] as u32) << 16 | (salt_bytes[2] as u32) << 8 | salt_bytes[3] as u32,
        (salt_bytes[4] as u32) << 24 | (salt_bytes[5] as u32) << 16 | (salt_bytes[6] as u32) << 8 | salt_bytes[7] as u32,
        (salt_bytes[8] as u32) << 24 | (salt_bytes[9] as u32) << 16 | (salt_bytes[10] as u32) << 8 | salt_bytes[11] as u32,
        (salt_bytes[12] as u32) << 24 | (salt_bytes[13] as u32) << 16 | (salt_bytes[14] as u32) << 8 | salt_bytes[15] as u32,
    ];

    let flags = BF_FLAGS_BY_SUBTYPE[st_idx];
    let mut expanded = [0u32; 18];
    let mut initial = [0u32; 18];
    bf_set_key(key, &mut expanded, &mut initial, flags);

    let mut p_box = initial;
    // Copy init S-boxes
    let mut s0 = BF_S0;
    let mut s1 = BF_S1;
    let mut s2 = BF_S2;
    let mut s3 = BF_S3;

    // Phase 1: Eksblowfish setup - encrypt salt through P and S boxes
    let (mut l, mut r) = (0u32, 0u32);
    for i in (0..BF_N + 2).step_by(2) {
        l ^= salt_words[0]; r ^= salt_words[1];
        for _ in 0..16 {
            l ^= p_box[0];
            let mut tmp = s3[(l & 0xFF) as usize];
            tmp = tmp.wrapping_add(s2[((l >> 8) & 0xFF) as usize]);
            tmp ^= s1[((l >> 16) & 0xFF) as usize];
            tmp = tmp.wrapping_add(s0[(l >> 24) as usize]);
            r ^= p_box[1]; r ^= tmp;
            let tmp_l = r; r = l; l = tmp_l;
            // Repeat for second half-round
            l ^= p_box[2];
            let mut tmp2 = s3[(l & 0xFF) as usize];
            tmp2 = tmp2.wrapping_add(s2[((l >> 8) & 0xFF) as usize]);
            tmp2 ^= s1[((l >> 16) & 0xFF) as usize];
            tmp2 = tmp2.wrapping_add(s0[(l >> 24) as usize]);
            r ^= p_box[3]; r ^= tmp2;
            let tmp_l2 = r; r = l; l = tmp_l2;
        }
        let old_r = r ^ p_box[BF_N + 1];
        r = l;
        l = old_r;
        p_box[i] = l;

        l ^= salt_words[2]; r ^= salt_words[3];
        for _ in 0..16 {
            l ^= p_box[0];
            let mut tmp = s3[(l & 0xFF) as usize];
            tmp = tmp.wrapping_add(s2[((l >> 8) & 0xFF) as usize]);
            tmp ^= s1[((l >> 16) & 0xFF) as usize];
            tmp = tmp.wrapping_add(s0[(l >> 24) as usize]);
            r ^= p_box[1]; r ^= tmp;
            let tmp_l = r; r = l; l = tmp_l;
            l ^= p_box[2];
            let mut tmp2 = s3[(l & 0xFF) as usize];
            tmp2 = tmp2.wrapping_add(s2[((l >> 8) & 0xFF) as usize]);
            tmp2 ^= s1[((l >> 16) & 0xFF) as usize];
            tmp2 = tmp2.wrapping_add(s0[(l >> 24) as usize]);
            r ^= p_box[3]; r ^= tmp2;
            let tmp_l2 = r; r = l; l = tmp_l2;
        }
        let old_r2 = r ^ p_box[BF_N + 1];
        r = l;
        l = old_r2;
        if i + 1 < BF_N + 2 { p_box[i + 1] = r; }
    }

    // ponytail: Phase 2 is the expensive Eksblowfish expansion.
    // This is the exact musl algorithm using BF_encrypt through entire state.
    // We flatten P+S into a single array to match musl's BF_ctx union.
    let mut ps = [0u32; 18 + 4 * 256];
    ps[..18].copy_from_slice(&p_box);
    ps[18..274].copy_from_slice(&s0);
    ps[274..530].copy_from_slice(&s1);
    ps[530..786].copy_from_slice(&s2);
    ps[786..1042].copy_from_slice(&s3);

    for _ in 0..count {
        // XOR P-box with expanded key
        for i in (0..BF_N + 2).step_by(2) {
            ps[i] ^= expanded[i];
            ps[i + 1] ^= expanded[i + 1];
        }
        // Encrypt (0,0) through entire PS state
        bf_encrypt_full(&mut ps, 0, 0);
        // XOR P-box with salt
        for i in (0..BF_N).step_by(4) {
            ps[i] ^= salt_words[0];
            ps[i + 1] ^= salt_words[1];
            ps[i + 2] ^= salt_words[2];
            ps[i + 3] ^= salt_words[3];
        }
        ps[16] ^= salt_words[0];
        ps[17] ^= salt_words[1];
        // Encrypt (0,0) through entire PS state again
        bf_encrypt_full(&mut ps, 0, 0);
    }

    // Final 64 Blowfish encryptions of magic string
    let mut output_words = [0u32; 6];
    for i in (0..6).step_by(2) {
        let mut l = BF_MAGIC_W[i];
        let mut r = BF_MAGIC_W[i + 1];
        for _ in 0..64 {
            for j in (0..16).step_by(2) {
                l ^= ps[j];
                let mut tmp = ps[786 + (l & 0xFF) as usize];
                tmp = tmp.wrapping_add(ps[530 + ((l >> 8) & 0xFF) as usize]);
                tmp ^= ps[274 + ((l >> 16) & 0xFF) as usize];
                tmp = tmp.wrapping_add(ps[18 + (l >> 24) as usize]);
                r ^= ps[j + 1]; r ^= tmp;
                let tl = r; r = l; l = tl;
            }
            let old_r = r ^ ps[BF_N + 1];
            r = l;
            l = old_r;
        }
        output_words[i] = l;
        output_words[i + 1] = r;
    }

    // Format output
    output[..7 + 22 - 1].copy_from_slice(&setting[..7 + 22 - 1]);
    output[7 + 22 - 1] = BF_ITOA64[(BF_ATOI64[setting[7 + 22 - 1] as usize - 0x20] & 0x30) as usize];
    // Byte-swap output words for encoding (little-endian to big-endian)
    for w in output_words.iter_mut() { *w = w.swap_bytes(); }
    let enc_src: &[u8] = unsafe { core::slice::from_raw_parts(output_words.as_ptr() as *const u8, 24) };
    bf_encode(&mut output[7 + 22..], &enc_src[..23]);
    output[7 + 22 + 31] = 0;
    Some(7 + 22 + 31)
}

// Encrypt (l,r) through all 1042 entries of PS array (BF_encrypt with start=&PS[0], end=&PS[PS_SIZE])
fn bf_encrypt_full(ps: &mut [u32; 1042], mut l: u32, mut r: u32) {
    let mut ptr = 0usize;
    while ptr < 1042 {
        l ^= ps[0]; // P[0]
        for i in (0..16).step_by(2) {
            let mut tmp = ps[786 + (l & 0xFF) as usize];
            tmp = tmp.wrapping_add(ps[530 + ((l >> 8) & 0xFF) as usize]);
            tmp ^= ps[274 + ((l >> 16) & 0xFF) as usize];
            tmp = tmp.wrapping_add(ps[18 + (l >> 24) as usize]);
            r ^= ps[i + 1]; r ^= tmp;
            let tl = r; r = l; l = tl;

            let mut tmp2 = ps[786 + (l & 0xFF) as usize];
            tmp2 = tmp2.wrapping_add(ps[530 + ((l >> 8) & 0xFF) as usize]);
            tmp2 ^= ps[274 + ((l >> 16) & 0xFF) as usize];
            tmp2 = tmp2.wrapping_add(ps[18 + (l >> 24) as usize]);
            r ^= ps[i + 2]; r ^= tmp2;
            let tl2 = r; r = l; l = tl2;
        }
        let old_r = r ^ ps[BF_N + 1];
        r = l;
        l = old_r;
        ps[ptr] = l;
        ps[ptr + 1] = r;
        ptr += 2;
    }
}

fn bf_set_key(key: &[u8], expanded: &mut [u32; 18], initial: &mut [u32; 18], flags: u8) {
    let bug = flags & 1;
    let safety: u32 = ((flags & 2) as u32) << 15;
    let mut sign: u32 = 0;
    let mut diff: u32 = 0;
    let mut ptr = 0usize;
    let klen = key.len();
    if klen == 0 { return; }

    for i in 0..BF_N + 2 {
        let mut tmp = [0u32; 2];
        for j in 0..4 {
            tmp[0] <<= 8;
            tmp[0] |= key[ptr] as u32;
            tmp[1] <<= 8;
            tmp[1] |= key[ptr] as i8 as i32 as u32;
            if j != 0 { sign |= tmp[1] & 0x80; }
            ptr += 1;
            if key[ptr - 1] == 0 { ptr = 0; }
            if ptr >= klen { ptr = 0; }
        }
        diff |= tmp[0] ^ tmp[1];
        expanded[i] = tmp[bug as usize];
        initial[i] = BF_INIT_P[i] ^ tmp[bug as usize];
    }

    diff |= diff >> 16;
    diff &= 0xffff;
    diff = diff.wrapping_add(0xffff);
    sign <<= 9;
    sign &= !diff & safety;
    initial[0] ^= sign;
}

fn bf_decode(dst: &mut [u8], src: &[u8]) -> bool {
    let mut di = 0usize;
    let mut si = 0usize;
    while di < dst.len() {
        if si + 3 >= src.len() { return true; }
        let c1 = src[si] as usize;
        let c2 = src[si + 1] as usize;
        if c1 < 0x20 || c1 >= 0x80 || c2 < 0x20 || c2 >= 0x80 { return true; }
        let v1 = BF_ATOI64[c1 - 0x20];
        let v2 = BF_ATOI64[c2 - 0x20];
        if v1 > 63 || v2 > 63 { return true; }
        dst[di] = (v1 << 2) | ((v2 & 0x30) >> 4);
        di += 1;
        if di >= dst.len() { break; }
        let c3 = src[si + 2] as usize;
        if c3 < 0x20 || c3 >= 0x80 { return true; }
        let v3 = BF_ATOI64[c3 - 0x20];
        if v3 > 63 { return true; }
        dst[di] = ((v2 & 0x0F) << 4) | ((v3 & 0x3C) >> 2);
        di += 1;
        if di >= dst.len() { break; }
        let c4 = src[si + 3] as usize;
        if c4 < 0x20 || c4 >= 0x80 { return true; }
        let v4 = BF_ATOI64[c4 - 0x20];
        if v4 > 63 { return true; }
        dst[di] = ((v3 & 0x03) << 6) | v4;
        di += 1;
        si += 4;
    }
    false
}

fn bf_encode(dst: &mut [u8], src: &[u8]) {
    let mut di = 0usize;
    let mut si = 0usize;
    while si < src.len() {
        let c1 = src[si] as usize;
        dst[di] = BF_ITOA64[c1 >> 2]; di += 1;
        let c1_left = (c1 & 0x03) << 4;
        si += 1;
        if si >= src.len() { dst[di] = BF_ITOA64[c1_left]; break; }
        let c2 = src[si] as usize;
        dst[di] = BF_ITOA64[c1_left | (c2 >> 4)]; di += 1;
        let c2_left = (c2 & 0x0f) << 2;
        si += 1;
        if si >= src.len() { dst[di] = BF_ITOA64[c2_left]; break; }
        let c3 = src[si] as usize;
        dst[di] = BF_ITOA64[c2_left | (c3 >> 6)]; di += 1;
        dst[di] = BF_ITOA64[c3 & 0x3f]; di += 1;
        si += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn __crypt_blowfish(key: *const c_char, setting: *const c_char, output: *mut c_char) -> *mut c_char {
    let ks = cstr_bytes(key);
    let ss = cstr_bytes(setting);
    let out = core::slice::from_raw_parts_mut(output as *mut u8, 256);

    let retval = bf_crypt_inner(ks, ss, out, 16);
    if retval.is_some() {
        out[retval.unwrap()] = 0;
        return output;
    }
    *output = b'*' as c_char;
    *output.add(1) = 0;
    output
}

// ============================================================
// crypt_r / crypt
// ============================================================

#[no_mangle]
pub unsafe extern "C" fn __crypt_r(key: *const c_char, setting: *const c_char, data: *mut crate::c_void) -> *mut c_char {
    let output = data as *mut c_char;
    let s = setting as *const u8;
    if *s == b'$' && *s.add(1) != 0 && *s.add(2) != 0 {
        if *s.add(1) == b'1' && *s.add(2) == b'$' {
            return __crypt_md5(key, setting, output);
        }
        if *s.add(1) == b'2' && *s.add(3) == b'$' {
            return __crypt_blowfish(key, setting, output);
        }
        if *s.add(1) == b'5' && *s.add(2) == b'$' {
            return __crypt_sha256(key, setting, output);
        }
        if *s.add(1) == b'6' && *s.add(2) == b'$' {
            return __crypt_sha512(key, setting, output);
        }
    }
    // ponytail: DES not implemented; return "*" for unsupported salts
    *output = b'*' as c_char;
    *output.add(1) = 0;
    output
}

static mut CRYPT_BUF: [u8; 128] = [0; 128];

#[no_mangle]
pub unsafe extern "C" fn crypt(key: *const c_char, setting: *const c_char) -> *mut c_char {
    __crypt_r(key, setting, CRYPT_BUF.as_mut_ptr() as *mut crate::c_void)
}

#[no_mangle]
pub unsafe extern "C" fn crypt_r(key: *const c_char, setting: *const c_char, data: *mut crate::c_void) -> *mut c_char {
    __crypt_r(key, setting, data)
}
