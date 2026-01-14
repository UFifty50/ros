use super::base::{execute, bit, bits};
use alloc::vec::Vec;


#[derive(Debug, PartialEq, Eq)]
pub enum CacheType {
    Null,
    Data,
    Instruction,
    Unified,
    Reserved(u8),
}

#[derive(Debug)]
pub struct CacheDescriptor {
    size: u32,
    eax: u32,
    ebx: u32,
    ecx: u32,
    edx: u32,
}

pub struct CacheDescriptors {
    descriptors: Vec<CacheDescriptor>,
}

impl CacheDescriptors {
    pub fn read() -> Option<Self> {
        if execute(0, 0).eax < 4 { return None; }
        
        let mut descriptors = Vec::new();

        let mut subLeaf = 0;
        loop {
            let res = execute(4, subLeaf);
            if bits(res.eax, 0, 4) == 0 {
                break;
            }
            descriptors.push(CacheDescriptor::new(res.eax, res.ebx, res.ecx, res.edx));
            subLeaf += 1;
        }

        Some(Self { descriptors })
    }
}

impl CacheDescriptor {
    pub fn new(eax: u32, ebx: u32, ecx: u32, edx: u32) -> Self {
        let sets = if bit(eax, 9) { 0 } else { ecx };
        let size = (bits(ebx, 0, 11) + 1) *
                   (bits(ebx, 12, 21) + 1) *
                   (bits(ebx, 22, 31) + 1) *
                   (sets + 1);

        Self {
            size,
            eax,
            ebx,
            ecx: sets,
            edx
        }
    }

    pub fn cacheType(&self) -> CacheType { CacheType::from(bits(self.eax, 0, 4) as u8) }
    pub fn level(&self) -> u8 { bits(self.eax, 5, 7) as u8 }
    pub fn selfInitializing(&self) -> bool { bit(self.eax, 8) }
    pub fn fullyAssociative(&self) -> bool { bit(self.eax, 9) }
    pub fn maxSharedLogicalIDs(&self) -> u32 { bits(self.eax, 14, 25) + 1 as u32 }
    pub fn maxPhysicalCoreIDs_intel(&self) -> u32 { bits(self.eax, 26, 31) + 1 as u32 }
    
    pub fn coherencyLineSize(&self) -> u16 { bits(self.ebx, 0, 11) as u16 + 1 }
    pub fn physicalLinePartitions(&self) -> u16 { bits(self.ebx, 12, 21) as u16 + 1 }
    pub fn associativityWays(&self) -> u16 { bits(self.ebx, 22, 31) as u16 + 1 }
    
    pub fn wbinvdScope(&self) -> bool { bit(self.edx, 0) }
    pub fn inclusive(&self) -> bool { bit(self.edx, 1) }
    pub fn complexCacheIndexing_intel(&self) -> bool { bit(self.edx, 2) }
}

impl From<u8> for CacheType {
    fn from(value: u8) -> Self {
        match value {
            0 => CacheType::Null,
            1 => CacheType::Data,
            2 => CacheType::Instruction,
            3 => CacheType::Unified,
            n => CacheType::Reserved(n),
        }
    }
}
