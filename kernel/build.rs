use std::env;
fn main() {

    let build_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rerun-if-changed={:}/src/kernel.ld", build_root);
    println!("cargo::rustc-link-arg=-T{:}/src/kernel.ld", build_root);
    println!("cargo::rustc-link-arg=-Map={:}/kernel.map", build_root);
    //println!("cargo:rerun-if-changed={:}/shell", build_root);
    //println!("cargo::rustc-link-arg={:}/shell", build_root);
    //println!("cargo::rustc-link-arg=-mcmodel=medium");
    //println!("cargo::rustc-flags=-mcmodel=medany");
    //println!("cargo::rustc-link-arg=-fuse-ld=mold");
    //println!("cargo::rustc-link-arg=-fuse-ld=ld");
}
