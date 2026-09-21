fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-cdylib-link-arg=--osabi=none");
    println!("cargo:rustc-link-arg=--osabi=none");
}
