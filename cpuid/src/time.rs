use super::base::execute;


#[derive(Debug, Clone, Copy)]
pub struct TscInfo {
    // TSC/Crystal ratio parts
    pub denominator: u32,
    pub numerator: u32,
    pub coreCrystalHz: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessorFrequency {
    pub baseMHz: u16,
    pub maxMHz: u16,
    pub busMHz: u16,
}

impl TscInfo {
    pub fn read() -> Option<Self> {
        if execute(0, 0).eax < 0x15 { return None; }

        let leaf15 = execute(0x15, 0);
        if leaf15.ecx == 0 {
            let procFreq = ProcessorFrequency::read()?;
            let coreCrystalHz = (procFreq.baseMHz as u32)
                * 10_000_000
                * (leaf15.eax / leaf15.ebx);
            
            return Some(Self {
                denominator: leaf15.eax,
                numerator: leaf15.ebx,
                coreCrystalHz,
            })
        }

        Some(Self {
            denominator: leaf15.eax,
            numerator: leaf15.ebx,
            coreCrystalHz: leaf15.ecx,
        })
    }

    pub fn tscHz(&self) -> u64 {
        self.coreCrystalHz as u64 * self.numerator as u64 / self.denominator as u64
    }
}

impl ProcessorFrequency {
    pub fn read() -> Option<Self> {
        if execute(0, 0).eax < 0x16 { return None; }

        let res = execute(0x16, 0);
        if res.eax == 0 { return None; }

        Some(Self {
            baseMHz: res.eax as u16,
            maxMHz: res.ebx as u16,
            busMHz: res.ecx as u16,
        })
    }
}
