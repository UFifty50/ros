use super::base::{execute, bit, bits};
use alloc::vec::Vec;


#[derive(Debug, PartialEq, Eq)]
pub enum TlbType {
    Null,
    Data,
    Instruction,
    Unified,
    LoadOnly,
    StoreOnly,
    Unknown(u8),
}

#[derive(Debug)]
pub struct TlbDescriptor {
    pub tlbType: TlbType,
    pub pageSizeSupportMask: u8,
    pub partitioning: Option<u8>,
    pub ways: u16,
    pub sets: u32,
    pub level: u8,
    pub fullyAssociative: bool,
    pub maxSharedLogicalIDs: u32,
}

pub struct TlbDescriptors {
    pub maxSubleaf: u32,
    pub descriptors: Vec<TlbDescriptor>,
}

impl TlbDescriptors {
    pub fn read() -> Option<Self> {
        if execute(0, 0).eax < 0x18 { return None; }
        
        let res = execute(0x18, 0);
        let mut descriptors = Vec::with_capacity(res.eax as usize + 1);

        for index in 0..=res.eax {
            let res = execute(0x18, index);
            if bits(res.edx, 0, 4) == 0 {
                continue;
            }

            let partitionBits = bits(res.ebx, 8, 10) as u8;
            let partitioning = if partitionBits == 0 { None } else { Some(partitionBits) };

            let descriptor = TlbDescriptor {
                tlbType: TlbType::from(bits(res.edx, 0, 4) as u8),
                pageSizeSupportMask: bits(res.ebx, 0, 3) as u8,
                partitioning,
                ways: bits(res.ebx, 16, 31) as u16,
                sets: res.ecx,
                level: bits(res.edx, 5, 7) as u8,
                fullyAssociative: bit(res.edx, 8),
                maxSharedLogicalIDs: bits(res.edx, 14, 25) + 1,
            };
            descriptors.push(descriptor);
        }

        Some(Self {
            maxSubleaf: res.eax,
            descriptors,
        })
    }
}

impl TlbDescriptor {
    pub fn page4Kb(&self) -> bool { bit(self.pageSizeSupportMask as u32, 0) }
    pub fn page2Mb(&self) -> bool { bit(self.pageSizeSupportMask as u32, 1) }
    pub fn page4Mb(&self) -> bool { bit(self.pageSizeSupportMask as u32, 2) }
    pub fn page1Gb(&self) -> bool { bit(self.pageSizeSupportMask as u32, 3) }
}

impl From<u8> for TlbType {
    fn from(value: u8) -> Self {
        match value {
            0 => TlbType::Null,
            1 => TlbType::Data,
            2 => TlbType::Instruction,
            3 => TlbType::Unified,
            4 => TlbType::LoadOnly,
            5 => TlbType::StoreOnly,
            other => TlbType::Unknown(other),
        }
    }
}
