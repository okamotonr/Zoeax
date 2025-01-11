use std::env;
fn main() {
    let build_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rerun-if-changed={:}/src/user.ld", build_root);
    println!("cargo::rustc-link-arg=-T{:}/src/user.ld", build_root);
    println!("cargo::rustc-link-arg=-Map={:}/pong.map", build_root);
}
