// DES, used by crypt(3) with traditional salts and by encrypt(3)/setkey(3).
// Straightforward table-driven implementation of FIPS 46-3.

const IP: [u8; 64] = [
    58, 50, 42, 34, 26, 18, 10, 2, 60, 52, 44, 36, 28, 20, 12, 4,
    62, 54, 46, 38, 30, 22, 14, 6, 64, 56, 48, 40, 32, 24, 16, 8,
    57, 49, 41, 33, 25, 17, 9, 1, 59, 51, 43, 35, 27, 19, 11, 3,
    61, 53, 45, 37, 29, 21, 13, 5, 63, 55, 47, 39, 31, 23, 15, 7,
];

const FP: [u8; 64] = [
    40, 8, 48, 16, 56, 24, 64, 32, 39, 7, 47, 15, 55, 23, 63, 31,
    38, 6, 46, 14, 54, 22, 62, 30, 37, 5, 45, 13, 53, 21, 61, 29,
    36, 4, 44, 12, 52, 20, 60, 28, 35, 3, 43, 11, 51, 19, 59, 27,
    34, 2, 42, 10, 50, 18, 58, 26, 33, 1, 41, 9, 49, 17, 57, 25,
];

const E: [u8; 48] = [
    32, 1, 2, 3, 4, 5, 4, 5, 6, 7, 8, 9, 8, 9, 10, 11, 12, 13,
    12, 13, 14, 15, 16, 17, 16, 17, 18, 19, 20, 21, 20, 21, 22, 23, 24, 25,
    24, 25, 26, 27, 28, 29, 28, 29, 30, 31, 32, 1,
];

const P: [u8; 32] = [
    16, 7, 20, 21, 29, 12, 28, 17, 1, 15, 23, 26, 5, 18, 31, 10,
    2, 8, 24, 14, 32, 27, 3, 9, 19, 13, 30, 6, 22, 11, 4, 25,
];

const PC1: [u8; 56] = [
    57, 49, 41, 33, 25, 17, 9, 1, 58, 50, 42, 34, 26, 18,
    10, 2, 59, 51, 43, 35, 27, 19, 11, 3, 60, 52, 44, 36,
    63, 55, 47, 39, 31, 23, 15, 7, 62, 54, 46, 38, 30, 22,
    14, 6, 61, 53, 45, 37, 29, 21, 13, 5, 28, 20, 12, 4,
];

const PC2: [u8; 48] = [
    14, 17, 11, 24, 1, 5, 3, 28, 15, 6, 21, 10,
    23, 19, 12, 4, 26, 8, 16, 7, 27, 20, 13, 2,
    41, 52, 31, 37, 47, 55, 30, 40, 51, 45, 33, 48,
    44, 49, 39, 56, 34, 53, 46, 42, 50, 36, 29, 32,
];

const SHIFTS: [u8; 16] = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];

const SBOX: [[u8; 64]; 8] = [
    [
        14, 4, 13, 1, 2, 15, 11, 8, 3, 10, 6, 12, 5, 9, 0, 7,
        0, 15, 7, 4, 14, 2, 13, 1, 10, 6, 12, 11, 9, 5, 3, 8,
        4, 1, 14, 8, 13, 6, 2, 11, 15, 12, 9, 7, 3, 10, 5, 0,
        15, 12, 8, 2, 4, 9, 1, 7, 5, 11, 3, 14, 10, 0, 6, 13,
    ],
    [
        15, 1, 8, 14, 6, 11, 3, 4, 9, 7, 2, 13, 12, 0, 5, 10,
        3, 13, 4, 7, 15, 2, 8, 14, 12, 0, 1, 10, 6, 9, 11, 5,
        0, 14, 7, 11, 10, 4, 13, 1, 5, 8, 12, 6, 9, 3, 2, 15,
        13, 8, 10, 1, 3, 15, 4, 2, 11, 6, 7, 12, 0, 5, 14, 9,
    ],
    [
        10, 0, 9, 14, 6, 3, 15, 5, 1, 13, 12, 7, 11, 4, 2, 8,
        13, 7, 0, 9, 3, 4, 6, 10, 2, 8, 5, 14, 12, 11, 15, 1,
        13, 6, 4, 9, 8, 15, 3, 0, 11, 1, 2, 12, 5, 10, 14, 7,
        1, 10, 13, 0, 6, 9, 8, 7, 4, 15, 14, 3, 11, 5, 2, 12,
    ],
    [
        7, 13, 14, 3, 0, 6, 9, 10, 1, 2, 8, 5, 11, 12, 4, 15,
        13, 8, 11, 5, 6, 15, 0, 3, 4, 7, 2, 12, 1, 10, 14, 9,
        10, 6, 9, 0, 12, 11, 7, 13, 15, 1, 3, 14, 5, 2, 8, 4,
        3, 15, 0, 6, 10, 1, 13, 8, 9, 4, 5, 11, 12, 7, 2, 14,
    ],
    [
        2, 12, 4, 1, 7, 10, 11, 6, 8, 5, 3, 15, 13, 0, 14, 9,
        14, 11, 2, 12, 4, 7, 13, 1, 5, 0, 15, 10, 3, 9, 8, 6,
        4, 2, 1, 11, 10, 13, 7, 8, 15, 9, 12, 5, 6, 3, 0, 14,
        11, 8, 12, 7, 1, 14, 2, 13, 6, 15, 0, 9, 10, 4, 5, 3,
    ],
    [
        12, 1, 10, 15, 9, 2, 6, 8, 0, 13, 3, 4, 14, 7, 5, 11,
        10, 15, 4, 2, 7, 12, 9, 5, 6, 1, 13, 14, 0, 11, 3, 8,
        9, 14, 15, 5, 2, 8, 12, 3, 7, 0, 4, 10, 1, 13, 11, 6,
        4, 3, 2, 12, 9, 5, 15, 10, 11, 14, 1, 7, 6, 0, 8, 13,
    ],
    [
        4, 11, 2, 14, 15, 0, 8, 13, 3, 12, 9, 7, 5, 10, 6, 1,
        13, 0, 11, 7, 4, 9, 1, 10, 14, 3, 5, 12, 2, 15, 8, 6,
        1, 4, 11, 13, 12, 3, 7, 14, 10, 15, 6, 8, 0, 5, 9, 2,
        6, 11, 13, 8, 1, 4, 10, 7, 9, 5, 0, 15, 14, 2, 3, 12,
    ],
    [
        13, 2, 8, 4, 6, 15, 11, 1, 10, 9, 3, 14, 5, 0, 12, 7,
        1, 15, 13, 8, 10, 3, 7, 4, 12, 5, 6, 11, 0, 14, 9, 2,
        7, 11, 4, 1, 9, 12, 14, 2, 0, 6, 10, 13, 15, 3, 5, 8,
        2, 1, 14, 7, 4, 10, 8, 13, 15, 12, 9, 0, 3, 5, 6, 11,
    ],
];

#[derive(Copy, Clone)]
pub struct DesKey {
    subkeys: [u64; 16],
}

fn permute(input: u64, in_bits: u32, table: &[u8]) -> u64 {
    let mut out = 0u64;
    for (i, &p) in table.iter().enumerate() {
        let bit = (input >> (in_bits - p as u32)) & 1;
        out |= bit << (table.len() - 1 - i);
    }
    out
}

fn sbox_substitute(x: u64) -> u32 {
    let mut out = 0u32;
    for i in 0..8 {
        let six = ((x >> (42 - 6 * i)) & 0x3f) as usize;
        let row = ((six & 0x20) >> 4) | (six & 1);
        let col = (six >> 1) & 0x0f;
        out = (out << 4) | SBOX[i][row * 16 + col] as u32;
    }
    out
}

pub fn des_setkey(key: &[u8; 8]) -> DesKey {
    let mut k = 0u64;
    for &b in key.iter() { k = (k << 8) | b as u64; }
    let pc1 = permute(k, 64, &PC1);
    let mut c = ((pc1 >> 28) & 0x0fff_ffff) as u32;
    let mut d = (pc1 & 0x0fff_ffff) as u32;
    let mut subkeys = [0u64; 16];
    for r in 0..16 {
        let s = SHIFTS[r] as u32;
        c = ((c << s) | (c >> (28 - s))) & 0x0fff_ffff;
        d = ((d << s) | (d >> (28 - s))) & 0x0fff_ffff;
        let cd = ((c as u64) << 28) | d as u64;
        subkeys[r] = permute(cd, 56, &PC2);
    }
    DesKey { subkeys }
}

/// Raw DES on one block; `saltbits` implements the traditional crypt(3)
/// bit swapping in the E expansion.
pub fn des_do(l_in: u32, r_in: u32, count: u32, saltbits: u32, key: &DesKey) -> (u32, u32) {
    let block = ((l_in as u64) << 32) | r_in as u64;
    let ip = permute(block, 64, &IP);
    let mut l = (ip >> 32) as u32;
    let mut r = ip as u32;
    for _ in 0..count {
        for round in 0..16 {
            let mut e = permute(r as u64, 32, &E);
            if saltbits != 0 {
                let hi = (e >> 24) as u32;
                let lo = (e & 0x00ff_ffff) as u32;
                let f = (hi ^ lo) & saltbits;
                e = (((hi ^ f) as u64) << 24) | (lo ^ f) as u64;
            }
            let x = e ^ key.subkeys[round];
            let s = sbox_substitute(x) as u64;
            let p = permute(s, 32, &P) as u32;
            let nl = r;
            let nr = l ^ p;
            l = nl;
            r = nr;
        }
        // (L16, R16) -> (R16, L16) before the next encryption or the FP.
        core::mem::swap(&mut l, &mut r);
    }
    let pre = ((l as u64) << 32) | r as u64;
    let fp = permute(pre, 64, &FP);
    ((fp >> 32) as u32, fp as u32)
}

pub fn des_cipher_block(key: &DesKey, input: &[u8; 8], count: u32, saltbits: u32) -> [u8; 8] {
    let l = u32::from_be_bytes([input[0], input[1], input[2], input[3]]);
    let r = u32::from_be_bytes([input[4], input[5], input[6], input[7]]);
    let (lo, ro) = des_do(l, r, count, saltbits, key);
    let mut out = [0u8; 8];
    out[0..4].copy_from_slice(&lo.to_be_bytes());
    out[4..8].copy_from_slice(&ro.to_be_bytes());
    out
}

pub fn setup_salt(salt: u32) -> u32 {
    let mut saltbits = 0u32;
    let mut saltbit = 1u32;
    let mut obit = 0x0080_0000u32;
    for _ in 0..24 {
        if salt & saltbit != 0 { saltbits |= obit; }
        saltbit <<= 1;
        obit >>= 1;
    }
    saltbits
}

fn ascii_to_bin(ch: u8) -> u32 {
    let sch = ch as i8 as i32;
    let mut retval = sch - '.' as i32;
    if sch >= 'A' as i32 {
        retval = sch - ('A' as i32 - 12);
        if sch >= 'a' as i32 {
            retval = sch - ('a' as i32 - 38);
        }
    }
    (retval & 0x3f) as u32
}

const ASCII64: &[u8; 64] = b"./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

fn ascii_is_unsafe(ch: u8) -> bool {
    ch == 0 || ch == b'\n' || ch == b':'
}

fn read_setting_char(s: *const u8, i: usize) -> u8 {
    unsafe { *s.add(i) }
}

/// Traditional DES crypt. Returns the number of bytes written (excluding NUL)
/// or 0 on failure.
pub unsafe fn crypt_des(key: *const u8, setting: *const u8, output: *mut u8) -> usize {
    if key.is_null() || setting.is_null() || output.is_null() { return 0; }
    let mut keybuf = [0u8; 8];
    let mut kp = key;
    for i in 0..8 {
        keybuf[i] = (*kp) << 1;
        if *kp != 0 { kp = kp.add(1); }
    }
    let mut ekey = des_setkey(&keybuf);

    let count: u32;
    let salt: u32;
    let mut outpos: usize;
    if *setting == b'_' {
        let mut c = 0u32;
        for i in 1..5 {
            let ch = read_setting_char(setting, i);
            let v = ascii_to_bin(ch);
            if ASCII64[v as usize] != ch { return 0; }
            c |= v << ((i - 1) * 6);
        }
        if c == 0 { return 0; }
        count = c;
        let mut s = 0u32;
        for i in 5..9 {
            let ch = read_setting_char(setting, i);
            let v = ascii_to_bin(ch);
            if ASCII64[v as usize] != ch { return 0; }
            s |= v << ((i - 5) * 6);
        }
        salt = s;
        while *kp != 0 {
            keybuf = des_cipher_block(&ekey, &keybuf, 1, 0);
            for i in 0..8 {
                if *kp == 0 { break; }
                keybuf[i] ^= (*kp) << 1;
                kp = kp.add(1);
            }
            ekey = des_setkey(&keybuf);
        }
        for i in 0..9 { *output.add(i) = *setting.add(i); }
        *output.add(9) = 0;
        outpos = 9;
    } else {
        count = 25;
        let s0 = *setting;
        let s1 = *setting.add(1);
        if s1 == 0 || ascii_is_unsafe(s0) || ascii_is_unsafe(s1) { return 0; }
        salt = (ascii_to_bin(s1) << 6) | ascii_to_bin(s0);
        *output = s0;
        *output.add(1) = s1;
        outpos = 2;
    }

    let (r0, r1) = des_do(0, 0, count, setup_salt(salt), &ekey);

    let mut l = r0 >> 8;
    *output.add(outpos) = ASCII64[((l >> 18) & 0x3f) as usize]; outpos += 1;
    *output.add(outpos) = ASCII64[((l >> 12) & 0x3f) as usize]; outpos += 1;
    *output.add(outpos) = ASCII64[((l >> 6) & 0x3f) as usize]; outpos += 1;
    *output.add(outpos) = ASCII64[(l & 0x3f) as usize]; outpos += 1;
    l = (r0 << 16) | ((r1 >> 16) & 0xffff);
    *output.add(outpos) = ASCII64[((l >> 18) & 0x3f) as usize]; outpos += 1;
    *output.add(outpos) = ASCII64[((l >> 12) & 0x3f) as usize]; outpos += 1;
    *output.add(outpos) = ASCII64[((l >> 6) & 0x3f) as usize]; outpos += 1;
    *output.add(outpos) = ASCII64[(l & 0x3f) as usize]; outpos += 1;
    l = r1 << 2;
    *output.add(outpos) = ASCII64[((l >> 12) & 0x3f) as usize]; outpos += 1;
    *output.add(outpos) = ASCII64[((l >> 6) & 0x3f) as usize]; outpos += 1;
    *output.add(outpos) = ASCII64[(l & 0x3f) as usize]; outpos += 1;
    *output.add(outpos) = 0;
    outpos
}

static mut ENCRYPT_KEY: DesKey = DesKey { subkeys: [0; 16] };

#[no_mangle]
pub unsafe extern "C" fn setkey(key: *const c_char) {
    if key.is_null() { return; }
    let mut bkey = [0u8; 8];
    let mut kp = key as *const u8;
    for i in 0..8 {
        let mut v = 0u8;
        for j in (0..8).rev() {
            v |= ((*kp as u8) & 1) << j;
            kp = kp.add(1);
        }
        bkey[i] = v;
    }
    ENCRYPT_KEY = des_setkey(&bkey);
}

#[no_mangle]
pub unsafe extern "C" fn encrypt(block: *mut c_char, edflag: c_int) {
    if block.is_null() { return; }
    let mut b = [0u32; 2];
    let mut p = block as *const u8;
    for i in 0..2 {
        let mut v = 0u32;
        for j in (0..32).rev() {
            v |= ((*p & 1) as u32) << j;
            p = p.add(1);
        }
        b[i] = v;
    }
    let mut key = ENCRYPT_KEY;
    if edflag != 0 {
        let mut rev = [0u64; 16];
        for i in 0..16 { rev[i] = ENCRYPT_KEY.subkeys[15 - i]; }
        key = DesKey { subkeys: rev };
    }
    let (lo, ro) = des_do(b[0], b[1], 1, 0, &key);
    b[0] = lo;
    b[1] = ro;
    let mut q = block as *mut u8;
    for i in 0..2 {
        for j in (0..32).rev() {
            *q = ((b[i] >> j) & 1) as u8;
            q = q.add(1);
        }
    }
}
