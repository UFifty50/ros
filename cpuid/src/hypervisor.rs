use super::base::{execute, bit, copyRegToBuf};
use core::arch::x86_64::CpuidResult;
use super::features::FeatureInfo;


#[derive(Debug, Clone, Copy)]
pub struct HypervisorInfo {
    pub maxLeaf: u32,
    pub signature: [u8; 12],
}

#[derive(Debug, Clone, Copy)]
pub struct KvmFeatures {
    eax: u32,
    edx: u32,
}

impl HypervisorInfo {
    pub fn read() -> Option<Self> {
        if FeatureInfo::read().hypervisor() == false {
            return None;
        }

        let res = execute(0x40000000, 0);
        let mut sig = [0u8; 12];
        copyRegToBuf(&mut sig, 0, res.ebx);
        copyRegToBuf(&mut sig, 4, res.ecx);
        copyRegToBuf(&mut sig, 8, res.edx);

        Some(Self {
            maxLeaf: res.eax,
            signature: sig,
        })
    }

    pub fn identify(&self) -> &str {
        str::from_utf8(&self.signature).unwrap_or("Unknown")
    }

    /// Helper to read a raw hypervisor-specific leaf
    pub fn readRaw(leafOffset: u32) -> CpuidResult {
        execute(0x40000000 + leafOffset, 0)
    }
}

impl KvmFeatures {
    // Should only be called if `HypervisorInfo::identify()` returns "KVMKVMKVM".
    pub fn read() -> Self {
        let res = execute(0x40000001, 0);
        Self {
            eax: res.eax,
            edx: res.edx,
        }
    }

    // eax
    pub fn clocksource(&self) -> bool { bit(self.eax, 0) }
    pub fn nopIODelay(&self) -> bool { bit(self.eax, 1) }
    pub fn mmuOp(&self) -> bool { bit(self.eax, 2) }
    pub fn clocksource2(&self) -> bool { bit(self.eax, 3) }
    pub fn asyncPf(&self) -> bool { bit(self.eax, 4) }
    pub fn stealTime(&self) -> bool { bit(self.eax, 5) }
    pub fn pvEoi(&self) -> bool { bit(self.eax, 6) }
    pub fn pvUnhalt(&self) -> bool { bit(self.eax, 7) }
    pub fn pvTLBFlush(&self) -> bool { bit(self.eax, 9) }
    pub fn asyncPFVMExit(&self) -> bool { bit(self.eax, 10) }
    pub fn pvSendIPI(&self) -> bool { bit(self.eax, 11) }
    pub fn pvPollControl(&self) -> bool { bit(self.eax, 12) }
    pub fn pvSchedYield(&self) -> bool { bit(self.eax, 13) }
    pub fn pvClockSourceStable(&self) -> bool { bit(self.eax, 24) }

    // edx
    pub fn hintsRealtime(&self) -> bool { bit(self.edx, 0) }
}
