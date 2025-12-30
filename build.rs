#![allow(non_snake_case)]

use std::{env, path::PathBuf};

fn main() {
    // set by cargo for the kernel artifact dependency
    let kernelPath = PathBuf::from(env::var("CARGO_BIN_FILE_ROSKERNEL").unwrap());
    let outDir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let biosPath = outDir.join("bios.img");
    bootloader::BiosBoot::new(&kernelPath)
        .create_disk_image(&biosPath)
        .expect("Failed to create BIOS disk image");

    let uefiPath = outDir.join("uefi.img");
    bootloader::UefiBoot::new(&kernelPath)
        .create_disk_image(&uefiPath)
        .expect("Failed to create UEFI disk image");

    // pass the disk image paths via environment variables
    println!("cargo:rustc-env=UEFI_IMAGE={}", uefiPath.display());
    println!("cargo:rustc-env=BIOS_IMAGE={}", biosPath.display());
}

