use super::base::{execute, copyRegToBuf};
use core::str;

pub struct ProcessorBrand {
    // 3 leaves * 4 reg per leaf * 4 bytes per reg
    pub buffer: [u8; 48],
}

impl ProcessorBrand {
    pub fn read() -> Option<Self> {
        if execute(0x80000000, 0).eax < 0x80000004 {
            return None;
        }

        let mut buffer = [0u8; 48];
        let leaves = [0x80000002, 0x80000003, 0x80000004];
        for (i, &leaf) in leaves.iter().enumerate() {
            let res = execute(leaf, 0);
            let offset = i * 16;
            copyRegToBuf(&mut buffer, offset, res.eax);
            copyRegToBuf(&mut buffer, offset + 4, res.ebx);
            copyRegToBuf(&mut buffer, offset + 8, res.ecx);
            copyRegToBuf(&mut buffer, offset + 12, res.edx);
        }

        Some(Self { buffer })
    }

    pub fn as_str(&self) -> &str {
        // find first \0, otherwise use full length
        let len = self.buffer.iter().position(|&c| c == 0).unwrap_or(self.buffer.len());
        str::from_utf8(&self.buffer[..len])
                .unwrap_or("Unknown Brand")
                .trim()
    }
}
