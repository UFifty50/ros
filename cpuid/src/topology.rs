use super::base::execute;
use alloc::vec::Vec;


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TopologyType {
    Invalid,
    LogicalProcessor,
    Core,
    Module,
    Tile,
    Die,
    DieGroup,
    Unknown(u8),
}

#[derive(Debug)]
pub struct TopologyDescriptor {
    pub levelType: TopologyType,
    pub x2APICShift: u8,
    pub logicalProcessors: u16,
    pub x2APICID: u32,
}

pub struct TopologyDescriptors {
    leafInUse: u32,
    descriptors: Vec<TopologyDescriptor>,
}

impl TopologyDescriptors {
    pub fn read() -> Option<Self> {
        let maxLeaf = execute(0, 0).eax;
        if maxLeaf < 0x0B { return None; }

        let leafInUse = if maxLeaf >= 0x1F { 0x1F } else { 0x0B };
        let mut descriptors = Vec::new();

        let mut subLeaf = 0;
        loop {
            let res = execute(leafInUse, subLeaf);
        
            let levelType = TopologyType::from((res.ecx >> 8) as u8);
            if levelType == TopologyType::Invalid { break; }

            subLeaf += 1;

            descriptors.push(TopologyDescriptor {
                levelType,
                x2APICShift: (res.eax & 0x0F) as u8,
                logicalProcessors: (res.ebx & 0xFFFF) as u16,
                x2APICID: res.edx,
            })
        }

        Some(Self { leafInUse, descriptors })
    }
}

impl From<u8> for TopologyType {
    fn from(value: u8) -> Self {
        match value {
            0 => TopologyType::Invalid,
            1 => TopologyType::LogicalProcessor,
            2 => TopologyType::Core,
            3 => TopologyType::Module,
            4 => TopologyType::Tile,
            5 => TopologyType::Die,
            6 => TopologyType::DieGroup,
            other => TopologyType::Unknown(other),
        }
    }
}
