use core::arch::global_asm;

global_asm!(include_str!("switchThread.asm"), options(raw));

unsafe extern "C" {
    pub fn timerInterruptEntry();
}