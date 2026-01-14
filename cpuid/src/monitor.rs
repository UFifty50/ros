use super::base::{execute, bit};


#[derive(Debug, Clone, Copy)]
pub struct MonitorMwaitInfo {
    pub minLineSize: u16,
    pub maxLineSize: u16,
    ecx: u32,
    edx: u32,
}

impl MonitorMwaitInfo {
    pub fn read() -> Option<Self> {
        if execute(0, 0).eax < 5 { return None; }

        let res = execute(5, 0);
        
        Some(Self {
            minLineSize: res.eax as u16,
            maxLineSize: res.ebx as u16,
            ecx: res.ecx,
            edx: res.edx,
        })
    }

    // ecx
    pub fn emx(&self) -> bool { bit(self.ecx, 0) }
    pub fn ibe(&self) -> bool { bit(self.ecx, 1) }
    pub fn monitorless_mwait(&self) -> bool { bit(self.ecx, 3) }

    // edx
    pub fn supportedC0substates(&self) -> u8 { (self.edx & 0x0F) as u8 }
    pub fn supportedC1substates(&self) -> u8 { ((self.edx >> 4) & 0x0F) as u8 }
    pub fn supportedC2substates(&self) -> u8 { ((self.edx >> 8) & 0x0F) as u8 }
    pub fn supportedC3substates(&self) -> u8 { ((self.edx >> 12) & 0x0F) as u8 }
    pub fn supportedC4substates(&self) -> u8 { ((self.edx >> 16) & 0x0F) as u8 }
    pub fn supportedC5substates(&self) -> u8 { ((self.edx >> 20) & 0x0F) as u8 }
    pub fn supportedC6substates(&self) -> u8 { ((self.edx >> 24) & 0x0F) as u8 }
    pub fn supportedC7substates(&self) -> u8 { ((self.edx >> 28) & 0x0F) as u8 }
}
