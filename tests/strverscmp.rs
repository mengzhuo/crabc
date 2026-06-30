use core::ffi::{c_char, c_int};

#[inline]
fn isdigit(c: c_int) -> c_int {
    ((c >= '0' as c_int) && (c <= '9' as c_int)) as c_int
}

include!("../wave1/strverscmp.rs");

fn cmp(a: &str, b: &str) -> c_int {
    let a = std::ffi::CString::new(a).unwrap();
    let b = std::ffi::CString::new(b).unwrap();
    unsafe { strverscmp(a.as_ptr(), b.as_ptr()) }
}

#[test]
fn test_strverscmp_equal() {
    assert_eq!(cmp("", ""), 0);
    assert_eq!(cmp("a", "a"), 0);
    assert_eq!(cmp("123", "123"), 0);
}

#[test]
fn test_strverscmp_simple_ordering() {
    assert!(cmp("a", "b") < 0);
    assert!(cmp("b", "a") > 0);
}

#[test]
fn test_strverscmp_leading_zeros() {
    assert!(cmp("000", "00") < 0);
    assert!(cmp("00", "000") > 0);
    assert!(cmp("00", "01") < 0);
    assert!(cmp("01", "010") < 0);
    assert!(cmp("010", "09") < 0);
    assert!(cmp("09", "0") < 0);
}

#[test]
fn test_strverscmp_nonzero_digits() {
    assert!(cmp("9", "10") < 0);
    assert!(cmp("1.2", "1.10") < 0);
    assert!(cmp("a1b2", "a1b10") < 0);
}

#[test]
fn test_strverscmp_mixed() {
    assert!(cmp("a0", "a") > 0);
    assert!(cmp("0a", "0") > 0);
}

#[test]
fn test_strverscmp_version_strings() {
    assert!(cmp("foobar-1.1.2", "foobar-1.1.3") < 0);
    assert!(cmp("foobar-1.1.2", "foobar-1.01.3") > 0);
}

#[test]
fn test_strverscmp_tilde() {
    assert!(cmp("foo", "foo~") < 0);
    assert!(cmp("foo~", "foo") > 0);
}
