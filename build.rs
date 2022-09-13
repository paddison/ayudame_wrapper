fn main() {
    // add ayudame to library search path
    println!("cargo:rustc-link-search=PATH/TO/AYUDAME");
    println!("cargo:rustc-link-lib=ayudame")
}
