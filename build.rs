fn main() {
    // add ayudame to library search path
    println!("cargo:rustc-link-search=/home/patrick/hlrs/rust_rewrite/helpers/lib");
    println!("cargo:rustc-link-lib=ayudame")
}