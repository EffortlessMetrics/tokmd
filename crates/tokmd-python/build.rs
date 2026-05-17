use std::env;
use std::process::Command;

fn main() {
    let python = env::var("PYO3_PYTHON").unwrap_or_else(|_| "python3".to_string());
    let output = Command::new(python)
        .args([
            "-c",
            "import sysconfig; print(sysconfig.get_config_var('LIBDIR') or '')",
        ])
        .output();

    let Ok(output) = output else {
        return;
    };
    if !output.status.success() {
        return;
    }

    let libdir = String::from_utf8_lossy(&output.stdout);
    let libdir = libdir.trim();
    if libdir.is_empty() {
        return;
    }

    if env::var_os("CARGO_FEATURE_EXTENSION_MODULE").is_none() {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{libdir}");
    }
}
