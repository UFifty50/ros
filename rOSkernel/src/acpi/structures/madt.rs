#![allow(non_camel_case_types)]

use crate::acpi::impl_acpitable_defaults;
use crate::acpi::ACPITable;
use crate::acpi::SystemDescriptorTable::ACPISDTHeader;
use alloc::vec::Vec;

#[repr(C)]
pub(crate) struct MADTHeader {
    Header: ACPISDTHeader,
    LocalAPICAddress: u32,
    Flags: u32,
    Reserved: [u8; 0x2C - 0x28],
}

impl_acpitable_defaults!(MADTHeader, b"APIC");

#[repr(C)]
pub(crate) struct MADT {
    Header: MADTHeader,
    Records: Vec<MADTEntry>,
}

#[derive(Debug)]
#[repr(u8)]
pub enum MADTEntry {
    ProcLocalAPIC(MADTRecord_ProcLocalAPIC),
    IOAPIC(MADTRecord_IOAPIC) = 1,
    IOAPICIntSrcOvrrd(MADTRecord_IOAPIC_IntSrcOvrrd) = 2,
    IOAPICNMISrc(MADTRecord_IOAPIC_NMISrc) = 3,
    LocalAPICNMI(MADTRecord_LocalAPIC_NMI) = 4,
    LocalAPICAddrOvrrd(MADTRecord_LocalAPIC_AddrOvrrd) = 5,
    ProcLocalX2APIC(MADTRecord_x2APIC_ProcLocal) = 9,
    Unknown {
        header: MADTRecordHeader,
        data: Vec<u8>,
    },
}

#[derive(Debug)]
#[repr(C)]
pub struct MADTRecordHeader {
    Type: u8,
    Length: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct MADTRecord_ProcLocalAPIC {
    Header: MADTRecordHeader,
    ACPIProcessorID: u8,
    APICID: u8,
    APICFlags: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct MADTRecord_IOAPIC {
    Header: MADTRecordHeader,
    IOAPICID: u8,
    Reserved: u8,
    IOAPICAddress: u32,
    GlobalSystemInterruptBase: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct MADTRecord_IOAPIC_IntSrcOvrrd {
    Header: MADTRecordHeader,
    BusSource: u8,
    Source: u8,
    GlobalSystemInterrupt: u32,
    Flags: u16,
}

#[derive(Debug)]
#[repr(C)]
pub struct MADTRecord_IOAPIC_NMISrc {
    Header: MADTRecordHeader,
    NMISource: u8,
    Reserved: u8,
    Flags: u16,
    GlobalSystemInterrupt: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct MADTRecord_LocalAPIC_NMI {
    Header: MADTRecordHeader,
    ACPIProcessorID: u8,
    Flags: u16,
    LINTnum: u8,
}

#[derive(Debug)]
#[repr(C)]
pub struct MADTRecord_LocalAPIC_AddrOvrrd {
    Header: MADTRecordHeader,
    Reserved: u16,
    LocalAPICAddress: u64,
}

#[derive(Debug)]
#[repr(C)]
pub struct MADTRecord_x2APIC_ProcLocal {
    Header: MADTRecordHeader,
    Reserved: u16,
    LocalX2APICID: u32,
    Flags: u32,
    ACPIID: u32,
}
