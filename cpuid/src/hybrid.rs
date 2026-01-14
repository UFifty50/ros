use super::base::execute;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreType {
    Atom,
    Core,
    Unknown(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct HybridInfo {
    pub coreType: CoreType,
    pub nativeModelID: u32,
}

impl HybridInfo {
    pub fn read() -> Option<Self> {
        if execute(0, 0).eax < 0x1A { return None; }

        let res = execute(0x1A, 0);
        let coreTypeRaw = (res.eax >> 24) as u8;

        let coreType = match coreTypeRaw {
            0x20 => CoreType::Atom,
            0x40 => CoreType::Core,
            n => CoreType::Unknown(n as u8),
        };

        Some(Self {
            coreType,
            nativeModelID: res.eax & 0xFFFFFF,
        })
    }
}
