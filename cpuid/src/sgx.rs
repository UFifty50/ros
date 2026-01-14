use super::base::{execute, bit};
use super::features::ExtendedFeatures;


// TODO: iterate all SGX leaves
#[derive(Debug, Clone, Copy)]
pub struct SgxInfo {
    pub supportedLeavesMask: u32,
    ebx: u32,
    pub maxEnclaveSize_non64: u8,
    pub maxEnclaveSize_64: u8,
}

impl SgxInfo {
    pub fn read() -> Option<Self> {
        if !ExtendedFeatures::read()?.sgx() { return None; }

        let res = execute(0x12, 0);
        Some(Self {
            supportedLeavesMask: res.eax,
            ebx: res.ebx,
            maxEnclaveSize_non64: res.edx as u8,
            maxEnclaveSize_64: (res.edx >> 8) as u8,
        })
    }

    // ebx
    pub fn exinfo(&self) -> bool { bit(self.ebx, 0) }
    pub fn cpinfo(&self) -> bool { bit(self.ebx, 1) }
}
