extern crate pkg_config;

fn main() {
    println!("cargo:rustc-link-search=native=../../");
    println!("cargo:rustc-link-lib=knet");
}
