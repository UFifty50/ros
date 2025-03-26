use core::arch::asm;

pub unsafe fn out8(port: u16, value: u8) {
    unsafe { asm!("out dx, al", in("dx") port, in("al") value); };
}

pub unsafe fn in8(port: u16) -> u8 {
    let mut value: u8;
    unsafe { asm!("in al, dx", in("dx") port, out("al") value); };
    value
}