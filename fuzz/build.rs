fn main() {
    println!("cargo::rustc-check-cfg=cfg(fuzzing)");
    println!("cargo:rustc-cfg=fuzzing");
}
