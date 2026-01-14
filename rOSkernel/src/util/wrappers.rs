use core::arch::x86_64::__cpuid_count;
use core::fmt::Debug;
use core::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FpuSaveMechanism {
    None = 0,
    FXSave = 1,
    XSave = 2,
}

pub static FPU_MECHANISM: AtomicU8 = AtomicU8::new(FpuSaveMechanism::None as u8);

pub fn get_fpu_mechanism() -> FpuSaveMechanism {
    match FPU_MECHANISM.load(Ordering::Relaxed) {
        1 => FpuSaveMechanism::FXSave,
        2 => FpuSaveMechanism::XSave,
        _ => FpuSaveMechanism::None,
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct XFeatures(pub u64);
impl XFeatures {
    pub fn new(features: u64) -> XFeatures {
        XFeatures(features)
    }
    pub fn current() -> Self {
        let xcr0_hi: u32;
        let xcr0_lo: u32;
        unsafe {
            core::arch::asm!(
            "xgetbv",
            in("ecx") 0,
            out("edx") xcr0_hi,
            out("eax") xcr0_lo,
            options(nomem, nostack, preserves_flags));
        }
        XFeatures { 0: (xcr0_hi as u64) << 32 | xcr0_lo as u64 }
    }

    pub fn x87(&self) -> bool        { self.bit(0) }
    pub fn sse(&self) -> bool        { self.bit(1) }
    pub fn avx(&self) -> bool        { self.bit(2) }
    pub fn bndregs(&self) -> bool    { self.bit(3) }
    pub fn bndcsr(&self) -> bool     { self.bit(4) }
    pub fn opmask(&self) -> bool     { self.bit(5) }
    pub fn zmm_hi256(&self) -> bool  { self.bit(6) }
    pub fn hi16_zmm(&self) -> bool   { self.bit(7) }
    pub fn pkru(&self) -> bool       { self.bit(9) }
    pub fn xtilecfg(&self) -> bool   { self.bit(17) }
    pub fn xtiledata(&self) -> bool  { self.bit(18) }

    pub fn mpx(&self) -> bool     { self.bndregs()  && self.bndcsr() }
    pub fn avx512(&self) -> bool  { self.opmask()   && self.zmm_hi256() && self.hi16_zmm() }
    pub fn amx(&self) -> bool     { self.xtilecfg() && self.xtiledata() }

    pub fn bit(&self, bit: u8) -> bool {
        self.0 & (1u64 << bit) != 0
    }

    pub fn to_u64(&self) -> u64 {
        self.0
    }
}

impl Debug for XFeatures {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("XFeatures")
            .field("x87", &self.x87())
            .field("sse", &self.sse())
            .field("avx", &self.avx())
            .field("bndregs", &self.bndregs())
            .field("bndcsr", &self.bndcsr())
            .field("opmask", &self.opmask())
            .field("zmm_hi256", &self.zmm_hi256())
            .field("hi16_zmm", &self.hi16_zmm())
            .field("pkru", &self.pkru())
            .field("_padding1", &"00000000")
            .field("xtilecfg", &self.xtilecfg())
            .field("xtiledata", &self.xtiledata())
            .field("_padding2", &"0000000000000000000000000000000000000000000000")
            .finish()
    }
}

#[inline]
pub unsafe fn CPUID(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    let result = unsafe { __cpuid_count(leaf, subleaf) };
    (result.eax, result.ebx, result.ecx, result.edx)
}

#[inline]
pub unsafe fn readCR0() -> u64 {
    let mut cr0: u64;
    unsafe {
        core::arch::asm!(
        "mov {}, cr0",
        out(reg) cr0,
        options(nomem, nostack, preserves_flags)
        )
    };
    cr0
}

#[inline]
pub unsafe fn writeCR0(cr0: u64) {
    unsafe {
        core::arch::asm!(
        "mov cr0, {}",
        in(reg) cr0,
        options(nomem, nostack, preserves_flags)
        )
    };
}

#[inline]
pub unsafe fn readCR4() -> u64 {
    let mut cr4: u64;
    unsafe {
        core::arch::asm!(
        "mov {}, cr4",
        out(reg) cr4,
        options(nomem, nostack, preserves_flags)
        )
    };
    cr4
}

#[inline]
pub unsafe fn writeCR4(cr4: u64) {
    unsafe {
        core::arch::asm!(
        "mov cr4, {}",
        in(reg) cr4,
        options(nomem, nostack, preserves_flags)
        )
    };
}

#[inline]
pub unsafe fn xsetbv0(xcr0: u64) {
    let lo = xcr0 as u32;
    let hi = (xcr0 >> 32) as u32;
    unsafe {
        core::arch::asm!(
        "xsetbv",
        in("ecx") 0u32,
        in("eax") lo,
        in("edx") hi,
        options(nomem, preserves_flags)
        )
    }
}

#[inline]
pub unsafe fn xgetbv0() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        core::arch::asm!(
        "xgetbv",
        in("ecx") 0u32,
        out("eax") lo,
        out("edx") hi,
        options(nomem, nostack, preserves_flags)
        )
    }
    (hi as u64) << 32 | (lo as u64)
}
