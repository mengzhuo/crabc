mod loader_core;

use loader_core::*;
use std::env;
use std::ffi::CString;
use std::fs;
use std::os::unix::io::AsRawFd;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: loader <elf-binary> [args...]");
        process::exit(1);
    }
    if let Err(e) = load_and_run(&args[1], &args[1..]) {
        eprintln!("loader: {e}");
        process::exit(1);
    }
}

fn load_and_run(path: &str, argv: &[String]) -> Result<(), String> {
    let file = fs::File::open(path).map_err(|e| format!("open {path}: {e}"))?;
    let data = fs::read(path).map_err(|e| format!("read {path}: {e}"))?;

    let ehdr = parse_ehdr(&data).map_err(|e| e.to_string())?;
    let phdrs = parse_phdrs(&data, &ehdr).map_err(|e| e.to_string())?;

    let load_bias = map_segments(phdrs, file.as_raw_fd(), ehdr.e_type).map_err(|e| e.to_string())?;

    apply_relocations(&data, phdrs, load_bias).map_err(|e| e.to_string())?;

    let entry = ehdr.e_entry + load_bias;
    transfer(entry, argv, ehdr.e_phoff + load_bias, phdrs.len());
}

fn transfer(entry: u64, argv: &[String], phdr_addr: u64, phnum: usize) -> ! {
    let c_argv: Vec<CString> = argv
        .iter()
        .map(|a| CString::new(a.as_str()).unwrap())
        .collect();

    let stack_size = 64 * 1024;
    let stack_base = unsafe {
        sys_mmap(
            core::ptr::null_mut(),
            stack_size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    };
    if stack_base == MAP_FAILED {
        panic!("mmap stack failed");
    }

    let mut sp = stack_base as usize + stack_size;
    let mut str_ptrs = Vec::new();

    for c in &c_argv {
        let bytes = c.as_bytes_with_nul();
        sp -= bytes.len();
        unsafe { core::ptr::copy_nonoverlapping(bytes.as_ptr(), sp as *mut u8, bytes.len()) };
        str_ptrs.push(sp);
    }

    sp &= !7usize;

    let auxv_entries: [(u64, u64); 5] = [
        (AT_PHDR, phdr_addr),
        (AT_PHNUM, phnum as u64),
        (AT_PAGESZ, PAGE_SIZE as u64),
        (AT_ENTRY, entry),
        (AT_NULL, 0),
    ];

    sp -= auxv_entries.len() * 16;
    let auxv_ptr = sp as *mut (u64, u64);
    for (i, &(tag, val)) in auxv_entries.iter().enumerate() {
        unsafe { *auxv_ptr.add(i) = (tag, val) };
    }

    sp -= 8;
    unsafe { *(sp as *mut u64) = 0 };
    sp -= 8;
    unsafe { *(sp as *mut u64) = 0 };

    for &ptr in str_ptrs.iter().rev() {
        sp -= 8;
        unsafe { *(sp as *mut u64) = ptr as u64 };
    }

    sp -= 8;
    unsafe { *(sp as *mut u64) = argv.len() as u64 };

    sp &= !15usize;

    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!(
            "mov rsp, {sp}",
            "jmp {entry}",
            sp = in(reg) sp,
            entry = in(reg) entry,
            in("rdi") argv.len(),
            in("rsi") sp + 8,
            options(noreturn)
        );
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        core::arch::asm!(
            "mov sp, {sp}",
            "mov x0, {argc}",
            "mov x1, {argv}",
            "br {entry}",
            sp = in(reg) sp,
            argc = in(reg) argv.len(),
            argv = in(reg) sp + 8,
            entry = in(reg) entry,
            options(noreturn)
        );
    }

    #[cfg(target_arch = "riscv64")]
    unsafe {
        core::arch::asm!(
            "mv sp, {sp}",
            "mv a0, {argc}",
            "mv a1, {argv}",
            "jr {entry}",
            sp = in(reg) sp,
            argc = in(reg) argv.len(),
            argv = in(reg) sp + 8,
            entry = in(reg) entry,
            options(noreturn)
        );
    }
}
