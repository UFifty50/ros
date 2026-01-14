use super::features::FeatureInfo;
use super::base::{execute, bit};


#[derive(Debug, Clone, Copy)]
pub struct XSaveInfo {
    pub currentMaxSaveArea: u32,
    pub absMaxSaveArea: u32,
    pub supportedXCR0ComponentsMap: u64,
    extendedFeatureFlags: u32,
    pub currentMaxSaveAreaXSS: u32,
    pub supportedXSSComponentsMap: u64,
    pub stateComponents: [Option<XSaveComponent>; 63],
}

#[derive(Debug, Clone, Copy)]
pub struct XSaveComponent {
    pub componentID: u32,
    pub size: u32,
    pub saveAreaOffset: u32,
    pub supervisorComponent: bool,
    pub aligned: bool,
}

impl XSaveInfo {
    pub fn read() -> Option<Self> {
        if !FeatureInfo::read().xsave() {
            return None;
        }

        let res = execute(0xD, 0);
        let leaf1 = execute(0xD, 1);
        let mut stateComponents = [None; 63];
        
        for componentID in 2..65 {
            let leafn = execute(0xD, componentID);
            if leafn.eax == 0 {
                continue;
            }
            stateComponents[(componentID - 2) as usize] = Some(XSaveComponent {
                componentID,
                size: leafn.eax,
                saveAreaOffset: leafn.ebx,
                supervisorComponent: bit(leafn.ecx, 0),
                aligned: bit(leafn.ecx, 1),
            });
        }
        
        Some(Self {
            currentMaxSaveArea: res.ebx,
            absMaxSaveArea: res.ecx,
            supportedXCR0ComponentsMap: ((res.edx as u64) << 32) | (res.eax as u64),
            extendedFeatureFlags: leaf1.eax,
            currentMaxSaveAreaXSS: leaf1.ebx,
            supportedXSSComponentsMap: ((leaf1.edx as u64) << 32) | (leaf1.ecx as u64),
            stateComponents,
        })
    }

    pub fn xsaveopt(&self) -> bool { bit(self.extendedFeatureFlags, 0) }
    pub fn xsavec(&self) -> bool { bit(self.extendedFeatureFlags, 1) }
    pub fn xgetbv_ecx1(&self) -> bool { bit(self.extendedFeatureFlags, 2) }
    pub fn xss(&self) -> bool { bit(self.extendedFeatureFlags, 3) }
    pub fn xfd(&self) -> bool { bit(self.extendedFeatureFlags, 4) }
}
