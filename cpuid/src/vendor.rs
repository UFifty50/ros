use super::base::{execute, copyRegToBuf};
use core::str;


#[derive(Debug, Clone)]
pub struct VendorInfo {
    pub maxLeaf: u32,
    pub vendorString: [u8; 12],
}

impl VendorInfo {
    pub fn read() -> Self {
        let res = execute(0, 0);
        let mut s = [0u8; 12];
        copyRegToBuf(&mut s, 0, res.ebx);
        copyRegToBuf(&mut s, 4, res.edx);
        copyRegToBuf(&mut s, 8, res.ecx);
        Self { maxLeaf: res.eax, vendorString: s }
    }

    pub fn as_str(&self) -> &str {
        str::from_utf8(&self.vendorString).unwrap_or("Unknown")
    }
}
