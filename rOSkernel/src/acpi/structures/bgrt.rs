use crate::acpi::impl_acpitable_defaults;
use crate::acpi::ACPITable;
use crate::acpi::SystemDescriptorTable::ACPISDTHeader;

#[repr(C)]
pub(crate) struct BGRT {
    Header: ACPISDTHeader,
    VersionID: u16,
    Status: u8,
    ImageType: u8,
    ImageAddress: u64,
    ImageOffsetX: u32,
    ImageOffsetY: u32,
}

impl_acpitable_defaults!(BGRT, b"BGRT");
