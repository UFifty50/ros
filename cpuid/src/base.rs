use core::arch::x86_64::{__cpuid_count, CpuidResult};


pub fn execute(leaf: u32, subLeaf: u32) -> CpuidResult {
    unsafe { __cpuid_count(leaf, subLeaf) }
}

pub fn copyRegToBuf(buf: &mut [u8], offset: usize, reg: u32) {
    let bytes = reg.to_le_bytes();
    for i in 0..4 {
        if offset + i < buf.len() {
            buf[offset + i] = bytes[i];
        }
    }
}

#[inline(always)]
pub fn bit(value: u32, position: u32) -> bool {
    (value & (1 << position)) != 0
}

// big-endian, inclusive
#[inline(always)]
pub fn bits(value: u32, start: u32, end: u32) -> u32 {
    let length = end - start + 1;
    let mask =  if length == 32 { u32::MAX } else { (1 << length) - 1 };
    (value >> start) & mask
}
