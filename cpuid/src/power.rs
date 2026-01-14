use super::base::{execute, bit};


#[derive(Debug, Clone, Copy)]
pub struct ThermalPowerInfo {
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
}

impl ThermalPowerInfo {
    pub fn read() -> Option<Self> {
        if execute(0, 0).eax < 6 {
            return None;
        }

        let res = execute(0x6, 0);
        Some(Self {
            eax: res.eax,
            ebx: res.ebx,
            ecx: res.ecx,
            edx: res.edx
        })
    }

    // eax
    pub fn dts(&self) -> bool { bit(self.eax, 0) }
    pub fn turboboost(&self) -> bool { bit(self.eax, 1) }
    pub fn arat(&self) -> bool { bit(self.eax, 2) }
    pub fn pln(&self) -> bool { bit(self.eax, 4) }
    pub fn ecmd(&self) -> bool { bit(self.eax, 5) }
    pub fn ptm(&self) -> bool { bit(self.eax, 6) }
    pub fn hwp(&self) -> bool { bit(self.eax, 7) }
    pub fn hwp_notification(&self) -> bool { bit(self.eax, 8) }
    pub fn hwp_activity_window(&self) -> bool { bit(self.eax, 9) }
    pub fn hwp_epp(&self) -> bool { bit(self.eax, 10) }
    pub fn hwp_pkg_level_req(&self) -> bool { bit(self.eax, 11) }
    pub fn hdc(&self) -> bool { bit(self.eax, 13) }
    pub fn turboboost_max(&self) -> bool { bit(self.eax, 14) }
    pub fn int_on_hwp_cap_hiperf(&self) -> bool { bit(self.eax, 15) }
    pub fn hwp_peci_override(&self) -> bool { bit(self.eax, 16) }
    pub fn flexible_hwp(&self) -> bool { bit(self.eax, 17) }
    pub fn fast_access_mode(&self) -> bool { bit(self.eax, 18) }
    pub fn hw_feedback(&self) -> bool { bit(self.eax, 19) }
    pub fn ignore_hwp_request_on_half_idle(&self) -> bool { bit(self.eax, 20) }
    pub fn hwp_control_msr(&self) -> bool { bit(self.eax, 22) }
    pub fn intl_thread_director(&self) -> bool { bit(self.eax, 23) }
    pub fn therm_int_bit25(&self) -> bool { bit(self.eax, 24) }

    // ebx
    pub fn int_thrshlds_in_therm_sens(&self) -> u8 { (self.ebx & 0x0F) as u8 }
    
    // ecx
    pub fn effective_freq_interface(&self) -> bool { bit(self.ecx, 0) }
    pub fn acnt2_capability(&self) -> bool { bit(self.ecx, 1) }
    pub fn perf_energy_bias(&self) -> bool { bit(self.ecx, 3) }
    pub fn intl_thread_director_supported(&self) -> u8 { (self.ecx >> 8) as u8 }

    // edx
    pub fn perf_capability_reporting(&self) -> bool { bit(self.edx, 0) }
    pub fn effic_capability_reporting(&self) -> bool { bit(self.edx, 1) }
    // in units of 4KB -1
    pub fn hrdwr_feedback_intrfce_size(&self) -> u8 { ((self.edx >> 8) & 0x0F) as u8 }
    pub fn processor_idx_in_hdrwr_feedback_intrfce(&self) -> u16 { (self.edx >> 16) as u16 }
}
