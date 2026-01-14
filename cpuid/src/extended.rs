use super::base::{execute, bit};


#[derive(Debug, Clone, Copy)]
pub struct ExtendedProcessorInfo {
    pub ecx: u32,
    pub edx: u32,
    pub leaf1edx: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct AddressSizeInfo {
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
}

impl ExtendedProcessorInfo {
    pub fn read() -> Option<Self> {
        if execute(0x80000000, 0).eax < 0x80000001 { return None }
        
        let res = execute(0x80000001, 0);
        let leaf1edx = execute(0x1, 0).edx;
        Some(Self {
            ecx: res.ecx,
            edx: res.edx,
            leaf1edx
        })
    }

    // edx
    pub fn fpu(&self) -> bool { bit(self.leaf1edx, 0) }
    pub fn vme(&self) -> bool { bit(self.leaf1edx, 1) }
    pub fn de(&self) -> bool { bit(self.leaf1edx, 2) }
    pub fn pse(&self) -> bool { bit(self.leaf1edx, 3) }
    pub fn tsc(&self) -> bool { bit(self.leaf1edx, 4) }
    pub fn msr(&self) -> bool { bit(self.leaf1edx, 5) }
    pub fn pae(&self) -> bool { bit(self.leaf1edx, 6) }
    pub fn mce(&self) -> bool { bit(self.leaf1edx, 7) }
    pub fn cx8(&self) -> bool { bit(self.leaf1edx, 8) }
    pub fn apic(&self) -> bool { bit(self.leaf1edx, 9) }
    pub fn syscallK6(&self) -> bool { bit(self.edx, 10) }
    pub fn syscall(&self) -> bool { bit(self.edx, 11) }
    pub fn mtrr(&self) -> bool { bit(self.leaf1edx, 12) }
    pub fn pge(&self) -> bool { bit(self.leaf1edx, 13) }
    pub fn mca(&self) -> bool { bit(self.leaf1edx, 14) }
    pub fn cmov(&self) -> bool { bit(self.leaf1edx, 15) }
    pub fn pat(&self) -> bool { bit(self.leaf1edx, 16) }
    pub fn pse36(&self) -> bool { bit(self.leaf1edx, 17) }
    pub fn ecc(&self) -> bool { bit(self.edx, 19) }
    pub fn nx(&self) -> bool { bit(self.edx, 20) }
    pub fn mmxext(&self) -> bool { bit(self.edx, 22) }
    pub fn mmx(&self) -> bool { bit(self.leaf1edx, 23) }
    pub fn fxsr(&self) -> bool { bit(self.leaf1edx, 24) }
    pub fn fxsr_opt(&self) -> bool { bit(self.edx, 25) }
    pub fn pdpe1gb(&self) -> bool { bit(self.edx, 26) }
    pub fn rdtscp(&self) -> bool { bit(self.edx, 27) }
    pub fn lm(&self) -> bool { bit(self.edx, 29) }
    pub fn _3dnowext(&self) -> bool { bit(self.edx, 30) }
    pub fn _3dnow(&self) -> bool { bit(self.edx, 31) }

    // ecx
    pub fn lahf_lm(&self) -> bool { bit(self.ecx, 0) }
    pub fn cmp_legacy(&self) -> bool { bit(self.ecx, 1) }
    pub fn svm(&self) -> bool { bit(self.ecx, 2) }
    pub fn extapic(&self) -> bool { bit(self.ecx, 3) }
    pub fn cr8_legacy(&self) -> bool { bit(self.ecx, 4) }
    pub fn abm(&self) -> bool { bit(self.ecx, 5) }
    pub fn sse4a(&self) -> bool { bit(self.ecx, 6) }
    pub fn misalignsse(&self) -> bool { bit(self.ecx, 7) }
    pub fn _3dnowprefetch(&self) -> bool { bit(self.ecx, 8) }
    pub fn osvw(&self) -> bool { bit(self.ecx, 9) }
    pub fn ibs(&self) -> bool { bit(self.ecx, 10) }
    pub fn xop(&self) -> bool { bit(self.ecx, 11) }
    pub fn skinit(&self) -> bool { bit(self.ecx, 12) }
    pub fn wdt(&self) -> bool { bit(self.ecx, 13) }
    pub fn lwp(&self) -> bool { bit(self.ecx, 15) }
    pub fn fma4(&self) -> bool { bit(self.ecx, 16) }
    pub fn tce(&self) -> bool { bit(self.ecx, 17) }
    pub fn nodeid_msr(&self) -> bool { bit(self.ecx, 19) }
    pub fn tbm(&self) -> bool { bit(self.ecx, 21) }
    pub fn topoext(&self) -> bool { bit(self.ecx, 22) }
    pub fn perfctr_core(&self) -> bool { bit(self.ecx, 23) }
    pub fn perfctr_nb(&self) -> bool { bit(self.ecx, 24) }
    pub fn dbx(&self) -> bool { bit(self.ecx, 26) }
    pub fn perftsc(&self) -> bool { bit(self.ecx, 27) }
    pub fn pcx_l2i(&self) -> bool { bit(self.ecx, 28) }
    pub fn monitorx(&self) -> bool { bit(self.ecx, 29) }
    pub fn addr_mask_ext(&self) -> bool { bit(self.ecx, 30) }
}

impl AddressSizeInfo {
    pub fn read() -> Option<Self> {
        if execute(0x80000000, 0).eax < 0x80000008 { 
            return None;
        }
        let res = execute(0x80000008, 0);
        Some(Self {
            ebx: res.ebx,
            eax: res.eax,
            ecx: res.ecx,
            edx: res.edx,
        })
    }

    
    // eax
    pub fn physAddrBits(&self) -> u8 { self.eax as u8 }
    pub fn linearAddBits(&self) -> u8 { (self.eax >> 8)as u8 }
    pub fn guestPhyAddrBits(&self) -> u8 { (self.eax >> 16) as u8 }

    // ebx
    pub fn clzero(&self) -> bool { bit(self.ebx, 0) }
    pub fn retired_instr(&self) -> bool { bit(self.ebx, 1) }
    pub fn xrstor_fp_err(self) -> bool { bit(self.ebx, 2) }
    pub fn invlpgb(&self) -> bool { bit(self.ebx, 3) }
    pub fn rdpru(&self) -> bool { bit(self.ebx, 4) }
    pub fn mbe(&self) -> bool { bit(self.ebx, 6) }
    pub fn mcommit(&self) -> bool { bit(self.ebx, 8) }
    pub fn wbnoinvd(&self) -> bool { bit(self.ebx, 9) }
    pub fn ibpb(&self) -> bool { bit(self.ebx, 12) }
    pub fn wbinvd_int(&self) -> bool { bit(self.ebx, 13) }
    pub fn ibrs(&self) -> bool { bit(self.ebx, 14) }
    pub fn stibp(&self) -> bool { bit(self.ebx, 15) }
    pub fn lbrsAlwaysOn(&self) -> bool { bit(self.ebx, 16) }
    pub fn stibpAlwaysOn(&self) -> bool { bit(self.ebx, 17) }
    pub fn ibrs_preferred(&self) -> bool { bit(self.ebx, 18) }
    pub fn ibrs_same_mode_protection(&self) -> bool { bit (self.ebx, 19) }
    pub fn no_efer_lmsle(&self) -> bool { bit(self.ebx, 20) }
    pub fn invlpgb_nested(&self) -> bool { bit(self.ebx, 21) }
    pub fn ppin(&self) -> bool { bit(self.ebx, 23) }
    pub fn ssbd(&self) -> bool { bit(self.ebx, 24) }
    pub fn ssbd_legacy(&self) -> bool { bit(self.ebx, 25) }
    pub fn ssbd_no(&self) -> bool { bit(self.ebx, 26) }
    pub fn cppc(&self) -> bool { bit(self.ebx, 27) }
    pub fn psfd(&self) -> bool { bit(self.ebx, 28) }
    pub fn btc_no(&self) -> bool { bit(self.ebx, 29) }
    pub fn ibpb_ret(&self) -> bool { bit(self.ebx, 30) }
    pub fn branch_sampling(&self) -> bool { bit(self.ebx, 31) }

    // ecx
    pub fn physThreadsPerProcessor(&self) -> u8 { self.ecx as u8 }
    pub fn APICIDSize(&self) -> u8 { ((self.ecx >> 8) & 0xF0) as u8 }
    pub fn PTSCSize(&self) -> u8 { (self.ecx >> 16) as u8 }

    // edx
    pub fn maxPageCount(&self) -> u16 { self.edx as u16 }
    pub fn maxECXforRDPRU(&self) -> u16 { (self.edx >> 16) as u16 }
}
