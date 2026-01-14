use super::base::{execute, bit};


#[derive(Debug, Default, Clone, Copy)]
pub struct AmdSevInfo {
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
}

impl AmdSevInfo {
    pub fn read() -> Option<Self> {
        let max_ext = execute(0x80000000, 0).eax;
        if max_ext < 0x8000001F { 
            return None;
        }

        let res = execute(0x8000001F, 0);
        Some(Self {
            eax: res.eax,
            ebx: res.ebx,
            ecx: res.ecx,
            edx: res.edx,
        })
    }

    // eax
    pub fn sme(&self) -> bool { bit(self.eax, 0) }
    pub fn sev(&self) -> bool { bit(self.eax, 1) }
    pub fn pageFlushMSR(&self) -> bool { bit(self.eax, 2) }
    pub fn sevES(&self) -> bool { bit(self.eax, 3) }
    pub fn sevSNP(&self) -> bool { bit(self.eax, 4) }
    pub fn vmpl(&self) -> bool { bit(self.eax, 5) }
    pub fn rmpquery(&self) -> bool { bit(self.eax, 6) }
    pub fn vmplSSS(&self) -> bool { bit(self.eax, 7) }
    pub fn secureTSC(&self) -> bool { bit(self.eax, 8) }
    pub fn tscAuxVirtualization(&self) -> bool { bit(self.eax, 9) }
    pub fn hwEnfCacheCoherency(&self) -> bool { bit(self.eax, 10) }
    pub fn host64Bit(&self) -> bool { bit(self.eax, 11) }
    pub fn restrictedInjection(&self) -> bool { bit(self.eax, 12) }
    pub fn alternateInjection(&self) -> bool { bit(self.eax, 13) }
    pub fn debugVirtualization(&self) -> bool { bit(self.eax, 14) }
    pub fn preventHostIBS(&self) -> bool { bit(self.eax, 15) }
    pub fn vte(&self) -> bool { bit(self.eax, 16) }
    pub fn vmgexitParam(&self) -> bool { bit(self.eax, 17) }
    pub fn virtualTomMSR(&self) -> bool { bit(self.eax, 18) }
    pub fn ibsVirtualization(&self) -> bool { bit(self.eax, 19) }
    pub fn pmcVirtualization(&self) -> bool { bit(self.eax, 20) }
    pub fn rmpread(&self) -> bool { bit(self.eax, 21) }
    pub fn guestInterceptControl(&self) -> bool { bit(self.eax, 22) }
    pub fn segmentedRMP(&self) -> bool { bit(self.eax, 23) }
    pub fn vmsaRegisterProt(&self) -> bool { bit(self.eax, 24) }
    pub fn smtProt(&self) -> bool { bit(self.eax, 25) }
    pub fn secureAVIC(&self) -> bool { bit(self.eax, 26) }
    pub fn allowedSEVFeatures(&self) -> bool { bit(self.eax, 27) }
    pub fn svsmCommPageMSR(&self) -> bool { bit(self.eax, 28) }
    pub fn nestedVirtSnpMSR(&self) -> bool { bit(self.eax, 29) }
    pub fn hvInUseWrAllowed(&self) -> bool { bit(self.eax, 30) }
    pub fn ibpdOnEntry(&self) -> bool { bit(self.eax, 31) }

    // ebx
    pub fn cBitPosition(&self) -> u8 { (self.ebx & 0x3F) as u8 }
    pub fn physAddrReduction(&self) -> u8 { ((self.ebx >> 6) & 0x3F) as u8 }
    pub fn numVMPLs(&self) -> u8 { ((self.ebx >> 12) & 0xF) as u8 }

    // ecx
    pub fn maxASID(&self) -> u32 { self.ecx }

    // edx
    pub fn minASIDNoES(&self) -> u32 { self.edx }
}
