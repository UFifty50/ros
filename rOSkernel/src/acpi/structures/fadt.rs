use crate::acpi::impl_acpitable_defaults;
use crate::acpi::structures::GenericAddressStructure;
use crate::acpi::ACPITable;
use crate::acpi::SystemDescriptorTable::ACPISDTHeader;

#[repr(C)]
pub(crate) struct FADT {
    Header: ACPISDTHeader,
    FirmwareCtrl: u32,
    Dsdt: u32,

    // field used in ACPI 1.0; no longer in use, for compatibility only
    Reserved: u32,

    PreferredPowerManagementProfile: u8,
    SCI_Interrupt: u16,
    SMI_CommandPort: u32,
    AcpiEnable: u8,
    AcpiDisable: u8,
    S4BIOS_REQ: u8,
    PSTATE_Control: u8,
    PM1aEventBlock: u32,
    PM1bEventBlock: u32,
    PM1aControlBlock: u32,
    PM1bControlBlock: u32,
    PM2ControlBlock: u32,
    PMTimerBlock: u32,
    GPE0Block: u32,
    GPE1Block: u32,
    PM1EventLength: u8,
    PM1ControlLength: u8,
    PM2ControlLength: u8,
    PMTimerLength: u8,
    GPE0Length: u8,
    GPE1Length: u8,
    GPE1Base: u8,
    CStateControl: u8,
    WorstC2Latency: u16,
    WorstC3Latency: u16,
    FlushSize: u16,
    FlushStride: u16,
    DutyOffset: u8,
    DutyWidth: u8,
    DayAlarm: u8,
    MonthAlarm: u8,
    Century: u8,

    // reserved in ACPI 1.0; used since ACPI 2.0+
    BootArchitectureFlags: u16,

    Reserved2: u8,
    Flags: u32,

    // 12 byte structure; see below for details
    ResetReg: GenericAddressStructure,

    ResetValue: u8,
    Reserved3: [u8; 3],

    // 64bit pointers - Available on ACPI 2.0+
    X_FirmwareControl: u64,
    X_Dsdt: u64,

    X_PM1aEventBlock: GenericAddressStructure,
    X_PM1bEventBlock: GenericAddressStructure,
    X_PM1aControlBlock: GenericAddressStructure,
    X_PM1bControlBlock: GenericAddressStructure,
    X_PM2ControlBlock: GenericAddressStructure,
    X_PMTimerBlock: GenericAddressStructure,
    X_GPE0Block: GenericAddressStructure,
    X_GPE1Block: GenericAddressStructure,
}

impl_acpitable_defaults!(FADT, b"FACP");
