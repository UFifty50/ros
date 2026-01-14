use super::base::execute;
use alloc::vec::Vec;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyDescriptorType {
    Null,
    NoL3Cache,
    NoTLBInfo,   // use leaf 0x18
    NoCacheInfo, // use leaf 0x04
    Prefetch {size: u8},

    L1Data {sizeKb: u32, associativity: u8, lineSize: u8},
    L1Instruction {sizeKb: u32, associativity: u8, lineSize: u8},
    TraceCache {kuops: u8, associativity: u8},
    L2Cache {sizeKb: u32, associativity: u8, lineSize: u8},
    L3Cache {sizeKb: u32, associativity: u8, lineSize: u8},

    InstructionTLB {entries: u16, pageSizeKb: u32, associativity: u8 },
    DataTLB {entries: u16, pageSizeKb: u32, associativity: u8 },
    L2SharedTLB {entries: u16, pageSizeKb: u32, associativity: u8 },

    Reserved(u8),
}

pub struct LegacyCacheDescriptors {
    descriptors: Vec<LegacyDescriptorType>,
}

impl LegacyCacheDescriptors {
    pub fn read() -> Self {
        let mut descriptors = Vec::with_capacity(15);

        let result = execute(0x02, 0);

        let mut parseRegister = |reg: u32, isEAX: bool| {
            if reg & 0x80000000 != 0 {
                return;
            }

            let startByte = if isEAX { 1 } else { 0 };

            for i in startByte..4 {
                let byte = (reg >> (i * 8)) as u8;
                descriptors.push(Self::ByteToDescriptor(byte));
            }
        };

        parseRegister(result.eax, true);
        parseRegister(result.ebx, false);
        parseRegister(result.ecx, false);
        parseRegister(result.edx, false);

        Self { descriptors }
    }

    fn ByteToDescriptor(byte: u8) -> LegacyDescriptorType {
        use LegacyDescriptorType::*;
        const FULLY_ASSOCIATIVE: u8 = 0;

        match byte {
            0x00 => Null,
            
            // TLB
            0x01 => InstructionTLB { entries: 32, pageSizeKb: 4, associativity: 4 },
            0x02 => InstructionTLB { entries: 2, pageSizeKb: 4096, associativity: FULLY_ASSOCIATIVE },
            0x03 => DataTLB { entries: 64, pageSizeKb: 4, associativity: 4 },
            0x04 => DataTLB { entries: 8, pageSizeKb: 4096, associativity: 4 },
            0x05 => DataTLB { entries: 32, pageSizeKb: 4096, associativity: 4 },
            0x0B => InstructionTLB { entries: 4, pageSizeKb: 4096, associativity: 4 },
            0x4F => InstructionTLB { entries: 32, pageSizeKb: 4, associativity: FULLY_ASSOCIATIVE },
            0x50 => InstructionTLB { entries: 64, pageSizeKb: 4, associativity: FULLY_ASSOCIATIVE }, 
            0x51 => InstructionTLB { entries: 128, pageSizeKb: 4, associativity: FULLY_ASSOCIATIVE },
            0x52 => InstructionTLB { entries: 256, pageSizeKb: 4, associativity: FULLY_ASSOCIATIVE },
            0x55 => InstructionTLB { entries: 7, pageSizeKb: 2048, associativity: FULLY_ASSOCIATIVE },
            0x5A => DataTLB { entries: 32, pageSizeKb: 2048, associativity: 4 },
            0x5B => DataTLB { entries: 64, pageSizeKb: 4, associativity: FULLY_ASSOCIATIVE },
            0x5C => DataTLB { entries: 128, pageSizeKb: 4, associativity: FULLY_ASSOCIATIVE },
            0x5D => DataTLB { entries: 256, pageSizeKb: 4, associativity: FULLY_ASSOCIATIVE },
            0x63 => DataTLB { entries: 32, pageSizeKb: 2048, associativity: 4 },
            0x64 => DataTLB { entries: 512, pageSizeKb: 4, associativity: 4 },
            0x6A => DataTLB { entries: 64, pageSizeKb: 4, associativity: 8 },
            0x6B => DataTLB { entries: 256, pageSizeKb: 4, associativity: 8 },
            0x6C => DataTLB { entries: 128, pageSizeKb: 2048, associativity: 8 },
            0x6D => DataTLB { entries: 16, pageSizeKb: 1048576, associativity: FULLY_ASSOCIATIVE },
            0x76 => InstructionTLB { entries: 8, pageSizeKb: 2048, associativity: FULLY_ASSOCIATIVE },
            0x90 => DataTLB { entries: 64, pageSizeKb: 4, associativity: FULLY_ASSOCIATIVE },
            0x96 => DataTLB { entries: 32, pageSizeKb: 4, associativity: FULLY_ASSOCIATIVE },
            0x9B => DataTLB { entries: 96, pageSizeKb: 4, associativity: FULLY_ASSOCIATIVE },
            0xA0 => DataTLB { entries: 32, pageSizeKb: 4, associativity: FULLY_ASSOCIATIVE },
            0xB0 => InstructionTLB { entries: 128, pageSizeKb: 4, associativity: 4 },
            0xB1 => InstructionTLB { entries: 8, pageSizeKb: 2048, associativity: 4 },
            0xB2 => InstructionTLB { entries: 64, pageSizeKb: 4, associativity: 4 },
            0xB3 => DataTLB { entries: 128, pageSizeKb: 4, associativity: 4 },
            0xB4 => DataTLB { entries: 256, pageSizeKb: 4, associativity: 4 },
            0xB5 => InstructionTLB { entries: 64, pageSizeKb: 4, associativity: 8 },
            0xB6 => InstructionTLB { entries: 128, pageSizeKb: 4, associativity: 8 },
            0xBA => DataTLB { entries: 64, pageSizeKb: 4, associativity: 4 },
            0xC0 => DataTLB { entries: 8, pageSizeKb: 4, associativity: 4 },
            0xC1 => L2SharedTLB { entries: 1024, pageSizeKb: 4, associativity: 8 },
            0xC2 => DataTLB { entries: 16, pageSizeKb: 4, associativity: 4 },
            0xC3 => L2SharedTLB { entries: 1536, pageSizeKb: 4, associativity: 6 },
            0xC4 => DataTLB { entries: 32, pageSizeKb: 2048, associativity: 4 },
            0xCA => L2SharedTLB { entries: 512, pageSizeKb: 4, associativity: 4 },
            
            // L1
            0x06 => L1Instruction { sizeKb: 8, associativity: 4, lineSize: 32 },
            0x08 => L1Instruction { sizeKb: 16, associativity: 4, lineSize: 32 },
            0x09 => L1Instruction { sizeKb: 32, associativity: 4, lineSize: 64 },
            0x0A => L1Data { sizeKb: 8, associativity: 2, lineSize: 32 },
            0x0C => L1Data { sizeKb: 16, associativity: 4, lineSize: 32 },
            0x0D => L1Data { sizeKb: 16, associativity: 4, lineSize: 64 },
            0x0E => L1Data { sizeKb: 24, associativity: 6, lineSize: 64 },
            0x10 => L1Data { sizeKb: 16, associativity: 4, lineSize: 32 },
            0x15 => L1Instruction { sizeKb: 16, associativity: 4, lineSize: 32 },
            0x2C => L1Data { sizeKb: 32, associativity: 8, lineSize: 64 },
            0x30 => L1Instruction { sizeKb: 32, associativity: 8, lineSize: 64 },
            0x56 => L1Data { sizeKb: 4096, associativity: 4, lineSize: 4 }, 
            0x57 => L1Data { sizeKb: 4, associativity: 4, lineSize: 4 },
            0x59 => L1Data { sizeKb: 4, associativity: FULLY_ASSOCIATIVE, lineSize: 4 }, 
            0x60 => L1Data { sizeKb: 16, associativity: 8, lineSize: 64 },
            0x61 => L1Data { sizeKb: 4, associativity: FULLY_ASSOCIATIVE, lineSize: 64 },
            0x66 => L1Data { sizeKb: 8, associativity: 4, lineSize: 64 },
            0x67 => L1Data { sizeKb: 16, associativity: 4, lineSize: 64 },
            0x68 => L1Data { sizeKb: 32, associativity: 4, lineSize: 64 },
            0x77 => L1Instruction { sizeKb: 16, associativity: 4, lineSize: 64 },
            
            // L2
            0x1A => L2Cache { sizeKb: 96, associativity: 6, lineSize: 64 },
            0x1D => L2Cache { sizeKb: 128, associativity: 2, lineSize: 64 },
            0x21 => L2Cache { sizeKb: 256, associativity: 8, lineSize: 64 },
            0x39 => L2Cache { sizeKb: 128, associativity: 4, lineSize: 64 },
            0x3A => L2Cache { sizeKb: 192, associativity: 6, lineSize: 64 },
            0x3B => L2Cache { sizeKb: 128, associativity: 2, lineSize: 64 },
            0x3C => L2Cache { sizeKb: 256, associativity: 4, lineSize: 64 },
            0x3D => L2Cache { sizeKb: 384, associativity: 6, lineSize: 64 },
            0x3E => L2Cache { sizeKb: 512, associativity: 4, lineSize: 64 },
            0x3F => L2Cache { sizeKb: 256, associativity: 2, lineSize: 64 },
            0x41 => L2Cache { sizeKb: 128, associativity: 4, lineSize: 32 },
            0x42 => L2Cache { sizeKb: 256, associativity: 4, lineSize: 32 },
            0x43 => L2Cache { sizeKb: 512, associativity: 4, lineSize: 32 },
            0x44 => L2Cache { sizeKb: 1024, associativity: 4, lineSize: 32 },
            0x45 => L2Cache { sizeKb: 2048, associativity: 4, lineSize: 32 },
            0x48 => L2Cache { sizeKb: 3072, associativity: 12, lineSize: 64 },
            0x49 => L2Cache { sizeKb: 4096, associativity: 16, lineSize: 64 },
            0x4E => L2Cache { sizeKb: 6144, associativity: 24, lineSize: 64 },
            0x78 => L2Cache { sizeKb: 1024, associativity: 4, lineSize: 64 },
            0x79 => L2Cache { sizeKb: 128, associativity: 8, lineSize: 64 },
            0x7A => L2Cache { sizeKb: 256, associativity: 8, lineSize: 64 },
            0x7B => L2Cache { sizeKb: 512, associativity: 8, lineSize: 64 },
            0x7C => L2Cache { sizeKb: 1024, associativity: 8, lineSize: 64 },
            0x7D => L2Cache { sizeKb: 2048, associativity: 8, lineSize: 64 },
            0x7E => L2Cache { sizeKb: 256, associativity: 8, lineSize: 128 },
            0x7F => L2Cache { sizeKb: 512, associativity: 2, lineSize: 64 },
            0x80 => L2Cache { sizeKb: 512, associativity: 8, lineSize: 64 },
            0x81 => L2Cache { sizeKb: 128, associativity: 8, lineSize: 32 },
            0x82 => L2Cache { sizeKb: 256, associativity: 8, lineSize: 32 },
            0x83 => L2Cache { sizeKb: 512, associativity: 8, lineSize: 32 },
            0x84 => L2Cache { sizeKb: 1024, associativity: 8, lineSize: 32 },
            0x85 => L2Cache { sizeKb: 2048, associativity: 8, lineSize: 32 },
            0x86 => L2Cache { sizeKb: 512, associativity: 4, lineSize: 64 },
            0x87 => L2Cache { sizeKb: 1024, associativity: 8, lineSize: 64 },
            
            // L3
            0x22 => L3Cache { sizeKb: 512, associativity: 4, lineSize: 64 },
            0x23 => L3Cache { sizeKb: 1024, associativity: 8, lineSize: 64 },
            0x24 => L2Cache { sizeKb: 1024, associativity: 16, lineSize: 64 },
            0x25 => L3Cache { sizeKb: 2048, associativity: 8, lineSize: 64 },
            0x29 => L3Cache { sizeKb: 4096, associativity: 8, lineSize: 64 },
            0x46 => L3Cache { sizeKb: 4096, associativity: 4, lineSize: 64 },
            0x47 => L3Cache { sizeKb: 8192, associativity: 8, lineSize: 64 },
            0x4A => L3Cache { sizeKb: 6144, associativity: 12, lineSize: 64 },
            0x4B => L3Cache { sizeKb: 8192, associativity: 16, lineSize: 64 },
            0x4C => L3Cache { sizeKb: 12288, associativity: 12, lineSize: 64 },
            0x4D => L3Cache { sizeKb: 16384, associativity: 16, lineSize: 64 },
            0x88 => L3Cache { sizeKb: 2048, associativity: 4, lineSize: 64 },
            0x89 => L3Cache { sizeKb: 4096, associativity: 4, lineSize: 64 },
            0x8A => L3Cache { sizeKb: 8192, associativity: 4, lineSize: 64 },
            0x8D => L3Cache { sizeKb: 3072, associativity: 12, lineSize: 128 },
            0xD0 => L3Cache { sizeKb: 512, associativity: 4, lineSize: 64 },
            0xD1 => L3Cache { sizeKb: 1024, associativity: 4, lineSize: 64 },
            0xD2 => L3Cache { sizeKb: 2048, associativity: 4, lineSize: 64 },
            0xD6 => L3Cache { sizeKb: 1024, associativity: 8, lineSize: 64 },
            0xD7 => L3Cache { sizeKb: 2048, associativity: 8, lineSize: 64 },
            0xD8 => L3Cache { sizeKb: 4096, associativity: 8, lineSize: 64 },
            0xDC => L3Cache { sizeKb: 1536, associativity: 12, lineSize: 64 },
            0xDD => L3Cache { sizeKb: 3072, associativity: 12, lineSize: 64 },
            0xDE => L3Cache { sizeKb: 6144, associativity: 12, lineSize: 64 },
            0xE2 => L3Cache { sizeKb: 2048, associativity: 16, lineSize: 64 },
            0xE3 => L3Cache { sizeKb: 4096, associativity: 16, lineSize: 64 },
            0xE4 => L3Cache { sizeKb: 8192, associativity: 16, lineSize: 64 },
            0xEA => L3Cache { sizeKb: 12288, associativity: 24, lineSize: 64 },
            0xEB => L3Cache { sizeKb: 18432, associativity: 24, lineSize: 64 },
            0xEC => L3Cache { sizeKb: 24576, associativity: 24, lineSize: 64 },
            
            // Trace
            0x70 => TraceCache { kuops: 12, associativity: 8 },
            0x71 => TraceCache { kuops: 16, associativity: 8 },
            0x72 => TraceCache { kuops: 32, associativity: 8 },
            0x73 => TraceCache { kuops: 64, associativity: 8 },
            
            // Prefetch
            0xF0 => Prefetch { size: 64 },
            0xF1 => Prefetch { size: 128 },
            
            // Special
            0x40 => NoL3Cache,
            0xFE => NoTLBInfo,
            0xFF => NoCacheInfo, 
            
            _ => Reserved(byte),
        }
    }
}

