// strverscmp() - ported from musl/src/string/strverscmp.c
// Add to libc/src/lib.rs in the "String/memory functions" section.

#[no_mangle]
pub unsafe extern "C" fn strverscmp(a: *const c_char, b: *const c_char) -> c_int {
    let l = a as *const u8;
    let r = b as *const u8;
    let mut i: usize = 0;
    let mut dp: usize = 0;
    let mut z: c_int = 1;

    // Find maximal matching prefix and track its maximal digit
    // suffix and whether those digits are all zeros.
    while *l.add(i) == *r.add(i) {
        let c = *l.add(i) as c_int;
        if c == 0 {
            return 0;
        }
        if isdigit(c) == 0 {
            dp = i + 1;
            z = 1;
        } else if c != b'0' as c_int {
            z = 0;
        }
        i += 1;
    }

    if (*l.add(dp)).wrapping_sub(b'1') < 9 && (*r.add(dp)).wrapping_sub(b'1') < 9 {
        // Non-degenerate digit sequences starting with nonzero digits:
        // longest digit string is greater.
        let mut j = i;
        while isdigit(*l.add(j) as c_int) != 0 {
            if isdigit(*r.add(j) as c_int) == 0 {
                return 1;
            }
            j += 1;
        }
        if isdigit(*r.add(j) as c_int) != 0 {
            return -1;
        }
    } else if z != 0
        && dp < i
        && (isdigit(*l.add(i) as c_int) != 0 || isdigit(*r.add(i) as c_int) != 0)
    {
        // Common prefix of digit sequence is all zeros:
        // digits order less than non-digits.
        return (*l.add(i)).wrapping_sub(b'0') as c_int
            - (*r.add(i)).wrapping_sub(b'0') as c_int;
    }

    *l.add(i) as c_int - *r.add(i) as c_int
}
