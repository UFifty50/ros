use std::{
    env,
    process::{self, Command},
};

fn main() {
    let mut qemu = Command::new("qemu-system-x86_64");
    qemu.arg("-drive");
    qemu.arg(format!("format=raw,file={}", env!("BIOS_IMAGE")));
    qemu.arg("-cpu");
    qemu.arg("max,+xsave,+xsavec,+xsaveopt,+xsaves,+xgetbv1");
    qemu.arg("-m");
    qemu.arg("2048");
    qemu.arg("-enable-kvm");
    qemu.arg("-machine");
    qemu.arg("q35");
    let exit_status = qemu.status().unwrap();
    process::exit(exit_status.code().unwrap_or(-1));
}
