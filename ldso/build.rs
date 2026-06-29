fn main() {
    println!("cargo:rustc-cdylib-link-arg=-nostartfiles");
    println!("cargo:rustc-cdylib-link-arg=-nostdlib");
    println!("cargo:rustc-cdylib-link-arg=-e");
    println!("cargo:rustc-cdylib-link-arg=_start");
    println!("cargo:rustc-cdylib-link-arg=-Wl,-Bsymbolic");
}
