use std::env;
use std::path::PathBuf;

fn main() {
    // Embed RPATH to LibTorch during linking
    if let Ok(libtorch) = env::var("LIBTORCH") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", PathBuf::from(libtorch).join("lib").to_str().unwrap());
    } else {
        // Fallback to auto-downloaded path (adjust if needed)
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).parent().unwrap().to_path_buf();
        let libtorch = out_dir.join("libtorch");  // tch's download dir
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", libtorch.join("lib").to_str().unwrap());
    }
}