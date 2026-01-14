use super::base::{execute, bit};


#[derive(Debug, Default, Clone, Copy)]
pub struct AmxTileInfo {
    pub tilePalettes: [AmxTilePalette; 2],
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AmxTilePalette {
    pub totalTileBytes: u16,
    pub bytesPerTile: u16,
    pub bytesPerRow: u16,
    pub maxNames: u16,
    pub maxRows: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct AmxTmulInfo {
    pub maxRowsCols: u8,
    pub maxBytesPerCol: u8,
    eax: u32,
}

//TODO: add AMX TMUL feature info

impl AmxTileInfo {
    pub fn read() -> Option<Self> {
        if execute(0, 0).eax < 0x1D { return None; }

        let numPalettes = execute(0x1D, 0).eax as usize;
        if numPalettes < 1 { return None; }

        let mut tileinfo = AmxTileInfo::default();

        for i in 1..=numPalettes {
        let res = execute(0x1D, i as u32);
            let palette = AmxTilePalette {
                totalTileBytes: res.eax as u16,
                bytesPerTile: (res.eax >> 16) as u16,
                bytesPerRow: res.ebx as u16,
                maxNames: (res.ebx >> 16) as u16,
                maxRows: res.ecx as u16,
            };
            tileinfo.tilePalettes[i - 1] = palette;
        }

        Some(tileinfo)
    }
}

impl AmxTmulInfo {
    pub fn read() -> Option<Self> {
        if execute(0, 0).eax < 0x1E { return None; }

        let mainLeaf = execute(0x1E, 0);
        let featureInfo = execute(0x1E, 1);
        Some(Self {
            maxRowsCols: (mainLeaf.ebx & 0xFF) as u8,
            maxBytesPerCol: ((mainLeaf.ebx >> 8) & 0xFFFF) as u8,
            eax: featureInfo.eax,
        })
    }

    pub fn amx_int8(&self) -> bool { bit(self.eax, 0) }
    pub fn amx_bf16(&self) -> bool { bit(self.eax, 1) }
    pub fn amx_complex(&self) -> bool { bit(self.eax, 2) }
    pub fn amx_fp16(&self) -> bool { bit(self.eax, 3) }
    pub fn amx_fp8(&self) -> bool { bit(self.eax, 4) }
    pub fn amx_tf32(&self) -> bool { bit(self.eax, 6) }
    pub fn amx_avx512(&self) -> bool { bit(self.eax, 7) }
    pub fn amx_movrs(&self) -> bool { bit(self.eax, 8) }
}
