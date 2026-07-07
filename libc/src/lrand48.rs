// rand48 PRNG family (musl semantics)
// xi = __seed48[0..3] = 48-bit state as three u16
// lc = __seed48[3..7] = multiplier a (three u16) + addend c (one u16)
// Default: xi=[0,0,0] lc=[0xe66d,0xdeec,0x0005,0x000b] => a=0x5DEECE66D c=0xB

// ponytail: static mut; no_std, no threads, matches musl's global __seed48
static mut __SEED48: [u16; 7] = [0, 0, 0, 0xe66d, 0xdeec, 0x0005, 0x000b];

// ponytail: saved old seed for seed48() return
static mut __SEED48_OLD: [u16; 3] = [0, 0, 0];

#[inline]
unsafe fn __rand48_step(xi: *mut u16, lc: *const u16) -> u64 {
    let x = *xi.add(0) as u64
        | (*xi.add(1) as u64) << 16
        | (*xi.add(2) as u64) << 32;
    let a = *lc.add(0) as u64
        | (*lc.add(1) as u64) << 16
        | (*lc.add(2) as u64) << 32;
    let c = *lc.add(3) as u64;
    let x = a.wrapping_mul(x).wrapping_add(c);
    *xi.add(0) = x as u16;
    *xi.add(1) = (x >> 16) as u16;
    *xi.add(2) = (x >> 32) as u16;
    x & 0xffffffffffff
}

#[no_mangle]
pub unsafe extern "C" fn nrand48(s: *mut u16) -> c_long {
    (__rand48_step(s, core::ptr::addr_of!(__SEED48).cast::<u16>().add(3)) >> 17) as c_long
}

#[no_mangle]
pub unsafe extern "C" fn lrand48() -> c_long {
    nrand48(core::ptr::addr_of_mut!(__SEED48).cast::<u16>())
}

#[no_mangle]
pub unsafe extern "C" fn jrand48(s: *mut u16) -> c_long {
    (__rand48_step(s, core::ptr::addr_of!(__SEED48).cast::<u16>().add(3)) >> 16) as i32 as c_long
}

#[no_mangle]
pub unsafe extern "C" fn mrand48() -> c_long {
    jrand48(core::ptr::addr_of_mut!(__SEED48).cast::<u16>())
}

#[no_mangle]
pub unsafe extern "C" fn srand48(seed: c_long) {
    seed48([0x330e, seed as u16, (seed >> 16) as u16].as_mut_ptr());
}

#[no_mangle]
pub unsafe extern "C" fn seed48(s: *mut u16) -> *mut u16 {
    core::ptr::copy_nonoverlapping(core::ptr::addr_of!(__SEED48).cast::<u16>(), core::ptr::addr_of_mut!(__SEED48_OLD).cast::<u16>(), 3);
    core::ptr::copy_nonoverlapping(s, core::ptr::addr_of_mut!(__SEED48).cast::<u16>(), 3);
    core::ptr::addr_of_mut!(__SEED48_OLD).cast::<u16>()
}

#[no_mangle]
pub unsafe extern "C" fn erand48(s: *mut u16) -> f64 {
    let x = __rand48_step(s, core::ptr::addr_of!(__SEED48).cast::<u16>().add(3));
    // Build double in [1.0, 2.0) by setting exponent bits, then subtract 1.0
    let bits: u64 = 0x3ff0000000000000 | (x << 4);
    f64::from_bits(bits) - 1.0
}

#[no_mangle]
pub unsafe extern "C" fn drand48() -> f64 {
    erand48(core::ptr::addr_of_mut!(__SEED48).cast::<u16>())
}

#[no_mangle]
pub unsafe extern "C" fn lcong48(p: *mut u16) {
    core::ptr::copy_nonoverlapping(p, core::ptr::addr_of_mut!(__SEED48).cast::<u16>(), 7);
}
