use super::base::{execute, bit};


#[derive(Debug, Clone, Copy)]
pub struct FeatureInfo {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ExtendedFeatures {
    pub maxSubLeaf: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ExtendedFeatures1 {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ExtendedFeatures2 {
    pub edx: u32,
}

impl FeatureInfo {
    pub fn read() -> Self {
        let res = execute(1, 0);
        Self {
            eax: res.eax,
            ebx: res.ebx,
            ecx: res.ecx,
            edx: res.edx,
        }
    }

    // eax
    pub fn steppingID(&self) -> u8 { (self.eax & 0xF) as u8 }
    pub fn model(&self) -> u8 { ((self.eax >> 4) & 0xF) as u8 }
    pub fn familyID(&self) -> u8 { ((self.eax >> 8) & 0xF) as u8 }
    pub fn processorType(&self) -> u8 { ((self.eax >> 12) & 0x3) as u8 }
    pub fn extendedModelID(&self) -> u8 { ((self.eax >> 16) & 0xF) as u8 }
    pub fn extendedFamilyID(&self) -> u8 { (self.eax >> 20) as u8 }
    
    // ebx
    pub fn brandIndex(&self) -> u8 { self.ebx as u8 }
    pub fn clflushLineSize(&self) -> u8 { (self.ebx >> 8) as u8 }
    pub fn maxLogicalProcessors(&self) -> u8 { (self.ebx >> 16) as u8 }
    pub fn localAPICID(&self) -> u8 { (self.ebx >> 24) as u8 }

    // ecx
    pub fn sse3(&self) -> bool { bit(self.ecx, 0) }
    pub fn pclmulqdq(&self) -> bool { bit(self.ecx, 1) }
    pub fn dtes64(&self) -> bool { bit(self.ecx, 2) }
    pub fn monitor(&self) -> bool { bit(self.ecx, 3) }
    pub fn ds_cpl(&self) -> bool { bit(self.ecx, 4) }
    pub fn vmx(&self) -> bool { bit(self.ecx, 5) }
    pub fn smx(&self) -> bool { bit(self.ecx, 6) }
    pub fn est(&self) -> bool { bit(self.ecx, 7) }
    pub fn tm2(&self) -> bool { bit(self.ecx, 8) }
    pub fn ssse3(&self) -> bool { bit(self.ecx, 9) }
    pub fn cnxt_id(&self) -> bool { bit(self.ecx, 10) }
    pub fn sdbg(&self) -> bool { bit(self.ecx, 11) }
    pub fn fma(&self) -> bool { bit(self.ecx, 12) }
    pub fn cx16(&self) -> bool { bit(self.ecx, 13) }
    pub fn xtpr(&self) -> bool { bit(self.ecx, 14) }
    pub fn pdcm(&self) -> bool { bit(self.ecx, 15) }
    pub fn pcid(&self) -> bool { bit(self.ecx, 17) }
    pub fn dca(&self) -> bool { bit(self.ecx, 18) }
    pub fn sse4_1(&self) -> bool { bit(self.ecx, 19) }
    pub fn sse4_2(&self) -> bool { bit(self.ecx, 20) }
    pub fn x2apic(&self) -> bool { bit(self.ecx, 21) }
    pub fn movbe(&self) -> bool { bit(self.ecx, 22) }
    pub fn popcnt(&self) -> bool { bit(self.ecx, 23) }
    pub fn tsc_deadline(&self) -> bool { bit(self.ecx, 24) }
    pub fn aes(&self) -> bool { bit(self.ecx, 25) }
    pub fn xsave(&self) -> bool { bit(self.ecx, 26) }
    pub fn osxsave(&self) -> bool { bit(self.ecx, 27) }
    pub fn avx(&self) -> bool { bit(self.ecx, 28) }
    pub fn f16c(&self) -> bool { bit(self.ecx, 29) }
    pub fn rdrnd(&self) -> bool { bit(self.ecx, 30) }
    pub fn hypervisor(&self) -> bool { bit(self.ecx, 31) }

    // edx
    pub fn fpu(&self) -> bool { bit(self.edx, 0) }
    pub fn vme(&self) -> bool { bit(self.edx, 1) }
    pub fn de(&self) -> bool { bit(self.edx, 2) }
    pub fn pse(&self) -> bool { bit(self.edx, 3) }
    pub fn tsc(&self) -> bool { bit(self.edx, 4) }
    pub fn msr(&self) -> bool { bit(self.edx, 5) }
    pub fn pae(&self) -> bool { bit(self.edx, 6) }
    pub fn mce(&self) -> bool { bit(self.edx, 7) }
    pub fn cx8(&self) -> bool { bit(self.edx, 8) }
    pub fn apic(&self) -> bool { bit(self.edx, 9) }
    pub fn sep(&self) -> bool { bit(self.edx, 11) }
    pub fn mtrr(&self) -> bool { bit(self.edx, 12) }
    pub fn pge(&self) -> bool { bit(self.edx, 13) }
    pub fn mca(&self) -> bool { bit(self.edx, 14) }
    pub fn cmov(&self) -> bool { bit(self.edx, 15) }
    pub fn pat(&self) -> bool { bit(self.edx, 16) }
    pub fn pse36(&self) -> bool { bit(self.edx, 17) }
    pub fn psn(&self) -> bool { bit(self.edx, 18) }
    pub fn clfsh(&self) -> bool { bit(self.edx, 19) }
    pub fn ds(&self) -> bool { bit(self.edx, 21) }
    pub fn acpi(&self) -> bool { bit(self.edx, 22) }
    pub fn mmx(&self) -> bool { bit(self.edx, 23) }
    pub fn fxsr(&self) -> bool { bit(self.edx, 24) }
    pub fn sse(&self) -> bool { bit(self.edx, 25) }
    pub fn sse2(&self) -> bool { bit(self.edx, 26) }
    pub fn ss(&self) -> bool { bit(self.edx, 27) }
    pub fn htt(&self) -> bool { bit(self.edx, 28) }
    pub fn tm(&self) -> bool { bit(self.edx, 29) }
    pub fn pbe(&self) -> bool { bit(self.edx, 31) }
}

impl ExtendedFeatures {
    pub fn read() -> Option<Self> {
        if execute(0, 0).eax < 7 { return None; }

        let res = execute(7, 0);
        Some(Self {
            maxSubLeaf: res.eax,
            ebx: res.ebx,
            ecx: res.ecx,
            edx: res.edx
        })
    }

    // ebx
    pub fn fsgsbase(&self) -> bool { bit(self.ebx, 0) }
    pub fn tsc_adjust(&self) -> bool { bit(self.ebx, 1) }
    pub fn sgx(&self) -> bool { bit(self.ebx, 2) }
    pub fn bmi1(&self) -> bool { bit(self.ebx, 3) }
    pub fn hle(&self) -> bool { bit(self.ebx, 4) }
    pub fn avx2(&self) -> bool { bit(self.ebx, 5) }
    pub fn fdp_excptn_only(&self) -> bool { bit(self.ebx, 6) }
    pub fn smep(&self) -> bool { bit(self.ebx, 7) }
    pub fn bmi2(&self) -> bool { bit(self.ebx, 8) }
    pub fn erms(&self) -> bool { bit(self.ebx, 9) }
    pub fn invpcid(&self) -> bool { bit(self.ebx, 10) }
    pub fn rtm(&self) -> bool { bit(self.ebx, 11) }
    pub fn rdt_m(&self) -> bool { bit(self.ebx, 12) }
    pub fn fcs_fds_depracation(&self) -> bool { bit(self.ebx, 13) }
    pub fn mpx(&self) -> bool { bit(self.ebx, 14) }
    pub fn rdt_a(&self) -> bool { bit(self.ebx, 15) }
    pub fn avx512f(&self) -> bool { bit(self.ebx, 16) }
    pub fn avx512dq(&self) -> bool { bit(self.ebx, 17) }
    pub fn rdseed(&self) -> bool { bit(self.ebx, 18) }
    pub fn adx(&self) -> bool { bit(self.ebx, 19) }
    pub fn smap(&self) -> bool { bit(self.ebx, 20) }
    pub fn avx512_ifma(&self) -> bool { bit(self.ebx, 21) }
    pub fn clflushopt(&self) -> bool { bit(self.ebx, 23) }
    pub fn clwb(&self) -> bool { bit(self.ebx, 24) }
    pub fn pt(&self) -> bool { bit(self.ebx, 25) }
    pub fn avx512pf(&self) -> bool { bit(self.ebx, 26) }
    pub fn avx512er(&self) -> bool { bit(self.ebx, 27) }
    pub fn avx512cd(&self) -> bool { bit(self.ebx, 28) }
    pub fn sha(&self) -> bool { bit(self.ebx, 29) }
    pub fn avx512bw(&self) -> bool { bit(self.ebx, 30) }
    pub fn avx512vl(&self) -> bool { bit(self.ebx, 31) }
    
    // ecx
    pub fn prefetchwt1(&self) -> bool { bit(self.ecx, 0) }
    pub fn avx512_vbmi(&self) -> bool { bit(self.ecx, 1) }
    pub fn umip(&self) -> bool { bit(self.ecx, 2) }
    pub fn pku(&self) -> bool { bit(self.ecx, 3) }
    pub fn ospke(&self) -> bool { bit(self.ecx, 4) }
    pub fn waitpkg(&self) -> bool { bit(self.ecx, 5) }
    pub fn avx512_vbmi2(&self) -> bool { bit(self.ecx, 6) }
    pub fn cet_ss(&self) -> bool { bit(self.ecx, 7) }
    pub fn gfni(&self) -> bool { bit(self.ecx, 8) }
    pub fn vaes(&self) -> bool { bit(self.ecx, 9) }
    pub fn vpclmulqdq(&self) -> bool { bit(self.ecx, 10) }
    pub fn avx512_vnni(&self) -> bool { bit(self.ecx, 11) }
    pub fn avx512_bitalg(&self) -> bool { bit(self.ecx, 12) }
    pub fn tme_en(&self) -> bool { bit(self.ecx, 13) }
    pub fn avx512_vpopcntdq(&self) -> bool { bit(self.ecx, 14) }
    pub fn la57(&self) -> bool { bit(self.ecx, 16) }
    pub fn mawau(&self) -> u8 { ((self.ecx >> 16) & 0x1f) as u8 }
    pub fn rdpid(&self) -> bool { bit(self.ecx, 22) }
    pub fn kl(&self) -> bool { bit(self.ecx, 23) }
    pub fn bus_lock_detect(&self) -> bool { bit(self.ecx, 24) }
    pub fn cldemote(&self) -> bool { bit(self.ecx, 25) }
    pub fn movdiri(&self) -> bool { bit(self.ecx, 27) }
    pub fn movdir64b(&self) -> bool { bit(self.ecx, 28) }
    pub fn sgx_lc(&self) -> bool { bit(self.ecx, 30) }
    pub fn pks(&self) -> bool { bit(self.ecx, 31) }
    
    // edx
    pub fn sgx_keys(&self) -> bool { bit(self.edx, 0) }
    pub fn avx512_4vnniw(&self) -> bool { bit(self.edx, 2) }
    pub fn avx512_4fmaps(&self) -> bool { bit(self.edx, 3) }
    pub fn fsrm(&self) -> bool { bit(self.edx, 4) }
    pub fn uintr(&self) -> bool { bit(self.edx, 5) }
    pub fn avx512_vp2intersect(&self) -> bool { bit(self.edx, 8) }
    pub fn srbds_ctrl(&self) -> bool { bit(self.edx, 9) }
    pub fn md_clear(&self) -> bool { bit(self.edx, 10) }
    pub fn rtm_always_abort(&self) -> bool { bit(self.edx, 11) }
    pub fn rtm_force_abort(&self) -> bool { bit(self.edx, 13) }
    pub fn serialize(&self) -> bool { bit(self.edx, 14) }
    pub fn hybrid(&self) -> bool { bit(self.edx, 15) }
    pub fn tsxldtrk(&self) -> bool { bit(self.edx, 16) }
    pub fn pconfig(&self) -> bool { bit(self.edx, 18) }
    pub fn lbr(&self) -> bool { bit(self.edx, 19) }
    pub fn cet_ibt(&self) -> bool { bit(self.edx, 20) }
    pub fn amx_bf16(&self) -> bool { bit(self.edx, 22) }
    pub fn avx512_fp16(&self) -> bool { bit(self.edx, 23) }
    pub fn amx_tile(&self) -> bool { bit(self.edx, 24) }
    pub fn amx_int8(&self) -> bool { bit(self.edx, 25) }
    pub fn ibrs(&self) -> bool { bit(self.edx, 26) }
    pub fn stibp(&self) -> bool { bit(self.edx, 27) }
    pub fn l1d_flush(&self) -> bool { bit(self.edx, 28) }
    pub fn arch_capabilities(&self) -> bool { bit(self.edx, 29) }
    pub fn core_capabilities(&self) -> bool { bit(self.edx, 30) }
    pub fn ssbd(&self) -> bool { bit(self.edx, 31) }
}

impl ExtendedFeatures1 {
    pub fn read() -> Option<Self> {
        if ExtendedFeatures::read()?.maxSubLeaf < 1 { return None; }
        
        let res = execute(7, 1);
        Some(Self {
            eax: res.eax,
            ebx: res.ebx,
            ecx: res.ecx,
            edx: res.edx,
        })
    }

    // eax
    pub fn sha512(&self) -> bool { bit(self.eax, 0) }
    pub fn sm3(&self) -> bool { bit(self.eax, 1) }
    pub fn sm4(&self) -> bool { bit(self.eax, 2) }
    pub fn rao_int(&self) -> bool { bit(self.eax, 3) }
    pub fn avx_vnni(&self) -> bool { bit(self.eax, 4) }
    pub fn avx512_bf16(&self) -> bool { bit(self.eax, 5) }
    pub fn lass(&self) -> bool { bit(self.eax, 6) }
    pub fn cmpccxadd(&self) -> bool { bit(self.eax, 7) }
    pub fn archperfmonext(&self) -> bool { bit(self.eax, 8) }
    pub fn fzrm(&self) -> bool { bit(self.eax, 10) }
    pub fn fsrs(&self) -> bool { bit(self.eax, 11) }
    pub fn rsrcs(&self) -> bool { bit(self.eax, 12) }
    pub fn fred(&self) -> bool { bit(self.eax, 17) } 
    pub fn lkgs(&self) -> bool { bit(self.eax, 18) }
    pub fn wrmsrns(&self) -> bool { bit(self.eax, 19) }
    pub fn nmi_src(&self) -> bool { bit(self.eax, 20) }
    pub fn amx_fp16(&self) -> bool { bit(self.eax, 21) }
    pub fn hreset(&self) -> bool { bit(self.eax, 22) }
    pub fn avx_ifma(&self) -> bool { bit(self.eax, 23) }
    pub fn lam(&self) -> bool { bit(self.eax, 26) }
    pub fn msrlist(&self) -> bool { bit(self.eax, 27) }
    pub fn invd_disable_post_bios_done(&self) -> bool { bit(self.eax, 30) }
    pub fn movrs(&self) -> bool { bit(self.eax, 31) }

    // ebx
    pub fn ppin(&self) -> bool { bit(self.ebx, 0) }
    pub fn pbndkb(&self) -> bool { bit(self.ebx, 1) }
    pub fn cpuid_maxval_lim_rmv(&self) -> bool { bit(self.ebx, 3) }

    // ecx
    pub fn asymm_rdt_monitoring(&self) -> bool { bit(self.ecx, 0) }
    pub fn asymm_rdt_allocation(&self) -> bool { bit(self.ecx, 1) }
    pub fn msr_imm(&self) -> bool { bit(self.ecx, 5) }

    // edx
    pub fn avx_vnni_int8(&self) -> bool { bit(self.edx, 4) }
    pub fn avx_ne_convert(&self) -> bool { bit(self.edx, 5) }
    pub fn amx_complex(&self) -> bool { bit(self.edx, 8) }
    pub fn avx_vnni_int16(&self) -> bool { bit(self.edx, 10) }
    pub fn utmr(&self) -> bool { bit(self.edx, 13) }
    pub fn prefetchi(&self) -> bool { bit(self.edx, 14) }
    pub fn user_msr(&self) -> bool { bit(self.edx, 15) }
    pub fn uiret_uif_from_rflags(&self) -> bool { bit(self.edx, 17) }
    pub fn cet_sss(&self) -> bool { bit(self.edx, 18) }
    pub fn avx10(&self) -> bool { bit(self.edx, 19) }
    pub fn apx_f(&self) -> bool { bit(self.edx, 21) }
    pub fn mwait(&self) -> bool { bit(self.edx, 23) }
    pub fn slsm(&self) -> bool { bit(self.edx, 24) }
}

impl ExtendedFeatures2 {
    pub fn read() -> Option<Self> {
        if ExtendedFeatures::read()?.maxSubLeaf < 2 { return None; }
        
        let res = execute(7, 2);
        Some(Self {
            edx: res.edx,
        })
    }

    // edx
    pub fn psfd(&self) -> bool { bit(self.edx, 0) }
    pub fn ipred_ctrl(&self) -> bool { bit(self.edx, 1) }
    pub fn rrsba_ctrl(&self) -> bool { bit(self.edx, 2) }
    pub fn ddpd_u(&self) -> bool { bit(self.edx, 3) }
    pub fn bhi_ctrl(&self) -> bool { bit(self.edx, 4) }
    pub fn mcdt_no(&self) -> bool { bit(self.edx, 5) }
    pub fn ulfd(&self) -> bool { bit(self.edx, 6) }
    pub fn monitor_mitg_no(&self) -> bool { bit(self.edx, 7) }
}
