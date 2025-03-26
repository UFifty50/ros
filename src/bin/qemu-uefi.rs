#![allow(non_snake_case)]

use std::{
    env,
    process::{self, Command},
};
use ovmf_prebuilt::{Arch, FileType, Source, Prebuilt};


fn main() {
    let ovmfEFI = Prebuilt::fetch(Source::LATEST, "target/ovmf").expect("failed to update prebuilt");

    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive");
    qemu.arg(format!("format=raw,file={}", env!("UEFI_IMAGE")));
    qemu.arg("-drive");
    qemu.arg(format!("if=pflash,format=raw,readonly=on,file={}", ovmfEFI.get_file(Arch::X64, FileType::Code).to_str().unwrap()));
    qemu.arg("-drive");
    qemu.arg(format!("if=pflash,format=raw,file={}", ovmfEFI.get_file(Arch::X64, FileType::Vars).to_str().unwrap()));
    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}
