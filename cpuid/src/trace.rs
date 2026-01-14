use super::base::{execute, bit, bits};


#[derive(Debug, Clone, Copy)]
pub struct ProcessorTraceInfo {
    ebx: u32,
    ecx: u32,
    leaf1eax: u32,
    leaf1ebx: u32,
    leaf1ecx: u32,
}

impl ProcessorTraceInfo {
    pub fn read() -> Option<Self> {
        let max_leaf = execute(0, 0).eax;
        if max_leaf < 0x14 {
             return None;
        }

        let res = execute(0x14, 0);
        let resLeaf1 = execute(0x14, 1);

        Some(Self {
            ebx: res.ebx,
            ecx: res.ecx,
            leaf1eax: resLeaf1.eax,
            leaf1ebx: resLeaf1.ebx,
            leaf1ecx: resLeaf1.ecx,
        })
    }

    // ebx
    pub fn cr3_filter(&self) -> bool { bit(self.ebx, 0) }
    pub fn cyc_acc(&self) -> bool { bit(self.ebx, 1) }
    pub fn ip_filter(&self) -> bool { bit(self.ebx, 2) }
    pub fn mtc(&self) -> bool { bit(self.ebx, 3) }
    pub fn ptwrite(&self) -> bool { bit(self.ebx, 4) }
    pub fn pwr_evt_trace(&self) -> bool { bit(self.ebx, 5) }
    pub fn pmi_preserve(&self) -> bool { bit(self.ebx, 6) }
    pub fn event_trace(&self) -> bool { bit(self.ebx, 7) }
    pub fn tnt_dis(&self) -> bool { bit(self.ebx, 8) }
    pub fn pttt(&self) -> bool { bit(self.ebx, 9) }

    // ecx
    pub fn topaout(&self) -> bool { bit(self.ecx, 0) }
    pub fn mentry(&self) -> bool { bit(self.ecx, 1) }
    pub fn sngl_rng_out(&self) -> bool { bit(self.ecx, 2) }
    pub fn trace_transport_subsystem(&self) -> bool { bit(self.ecx, 3) }
    pub fn lip(&self) -> bool { bit(self.ecx, 31) }

    // leaf 1 eax
    pub fn rangecnt(&self) -> u8 { bits(self.leaf1eax, 0, 2) as u8 }
    pub fn rtit_triggerx_msrs(&self) -> u8 { bits(self.leaf1eax, 8, 10) as u8 }
    pub fn mtc_rate(&self) -> u16 { bits(self.leaf1eax, 16, 31) as u16 }

    // leaf 1 ebx
    pub fn cyc_thresholds(&self) -> u16 { bits(self.leaf1ebx, 0, 15) as u16 }
    pub fn psb_rate(&self) -> u16 { bits(self.leaf1ebx, 16, 31) as u16 }

    // leaf 1 ecx
    pub fn trig_action_attrib(self) -> bool { bit(self.leaf1ecx, 0) }
    pub fn trig_pause_resume(self) -> bool { bit(self.leaf1ecx, 1) }
    pub fn trig_dr(self) -> bool { bit(self.leaf1ecx, 15) }
}
