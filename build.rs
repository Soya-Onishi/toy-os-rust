use std::path;
use std::env;

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = env::var_os("OUT_DIR").unwrap();
    // set by cargo's artifact dependency feature, see
    // https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies
    let kernel_path = env::var_os("CARGO_BIN_FILE_KERNEL_kernel")
        .map(|p| path::PathBuf::from(p))
        .expect("kernel binary file is not found");

    // create a BIOS disk image (optional) 
    let out_dir = out_dir.into_string().unwrap();
    let out_dir = path::Path::new(&out_dir);
    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel_path).create_disk_image(&bios_path).unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
    println!("cargo:rustc-env=KERNEL_PATH={}", kernel_path.display());
}