#![allow(non_camel_case_types)]

use crate::acpi::impl_acpitable_defaults;
use crate::acpi::ACPITable;
use crate::acpi::SystemDescriptorTable::ACPISDTHeader;

#[repr(C)]
pub struct SRAT {
    Header: ACPISDTHeader,
    Reserved: [u8; 12],
}

impl_acpitable_defaults!(SRAT, b"SRAT");

#[repr(C)]
pub struct SRAS_APIC_ProcLocal {
    Type: u8,
    Length: u8,
    ProxDom_lo: u8,
    APICID: u8,
    Flags: u32,
    SAPICEID: u8,
    ProxDom_hi: u8,
    CDM: u32,
}

#[repr(C)]
pub struct SRAS_Memory {
    Type: u8,
    Length: u8,
    Domain: u32,
    Reserved1: [u8; 2],
    MemRangeBase_lo: u32,
    MemRangeBase_hi: u32,
    MemRangeLength_lo: u32,
    MemRangeLength_hi: u32,
    Reserved2: [u8; 4],
    Flags: u32,
    Reserved3: [u8; 8],
}

#[repr(C)]
pub struct SRAS_X2APIC_ProcLocal {
    Type: u8,
    Length: u8,
    Reserved1: [u8; 2],
    Domain: u32,
    X2APICID: u32,
    Flags: u32,
    CDM: u32,
    Reserved2: [u8; 4],
}
