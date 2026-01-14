use super::base::{execute, bit};


#[derive(Debug, Clone, Copy)]
pub struct PMonInfo {
    eax: u32,
    ebx: u32,
    pub supportedFixedCountersMask: u32,
    edx: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum PMonEvent {
    CoreCycle,
    InstructionRetired,
    ReferenceCycles,
    LastLevelCacheReference,
    LastLevelCacheMiss,
    BranchInstructionRetired,
    BranchMispredictRetired,
    TopDownSlots,
}

impl PMonInfo {
    pub fn read() -> Option<Self> {
        if execute(0, 0).eax < 0x0A { return None; }

        let res = execute(0x0A, 0);
        if res.eax & 0xFF == 0 { return None; }

        Some(Self {
            eax: res.eax,
            ebx: res.ebx,
            supportedFixedCountersMask: res.ecx,
            edx: res.edx,
        })
    }

    // eax
    pub fn versionID(&self) -> u8 { self.eax as u8 }
    pub fn gpCountersPerProcessor(&self) -> u8 { (self.eax >> 8) as u8 }
    pub fn gpCounterBitWidth(&self) -> u8 { (self.eax >> 16) as u8 }
    pub fn ebxBitVectorLength(&self) -> u8 { (self.eax >> 24) as u8 }

    // ebx
    pub fn eventSupported(&self, event: PMonEvent) -> bool {
        // Event is supported if within the bit vector, and the bit is unset
        self.ebxBitVectorLength() > (event as u8) &&
        bit(self.ebx, event as u32) == false 
    }

    pub fn getEvent(&self, index: u8) -> Option<PMonEvent> {
        if index >= self.ebxBitVectorLength() || bit(self.ebx, index as u32) {
            return None;
        }

        Some(PMonEvent::from(index))
    }

    // edx
    pub fn contiguousFixedCounters(&self) -> u8 { (self.edx & 0x0F) as u8 }
    pub fn fixedCounterBitWidth(&self) -> u8 { (self.edx >> 4) as u8 }
    pub fn anyThreadDepracated(&self) -> bool { bit(self.edx, 15) }
}

impl From<u8> for PMonEvent {
    fn from(value: u8) -> Self {
        match value {
            0 => PMonEvent::CoreCycle,
            1 => PMonEvent::InstructionRetired,
            2 => PMonEvent::ReferenceCycles,
            3 => PMonEvent::LastLevelCacheReference,
            4 => PMonEvent::LastLevelCacheMiss,
            5 => PMonEvent::BranchInstructionRetired,
            6 => PMonEvent::BranchMispredictRetired,
            7 => PMonEvent::TopDownSlots,
            _ => PMonEvent::CoreCycle,
        }
    }
}
