use crate::acpi::impl_acpitable_defaults;
use crate::acpi::ACPITable;

#[repr(C)]
pub struct ACPISDTHeader {
    pub(super) Signature: [u8; 4],
    pub(super) Length: u32,
    Revision: u8,
    Checksum: u8,
    OEMID: [u8; 6],
    OEMTableID: [u8; 8],
    OEMRevision: u32,
    CreatorID: u32,
    CreatorRevision: u32,
}

#[repr(C)]
pub struct RSDT {
    Header: ACPISDTHeader,
    Entries: &'static [u32],
}

#[repr(C)]
pub struct XSDT {
    Header: ACPISDTHeader,
    Entries: &'static [u32],
}

impl_acpitable_defaults!(RSDT, b"RSDT");
impl_acpitable_defaults!(XSDT, b"XSDT");

impl RSDT {
    unsafe fn getTableFromEntry<T: ACPITable>(entry: u32) -> Option<&'static T> {
        let ptr = entry as usize as *const T;
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { &*ptr })
    }

    pub fn findTable<T: ACPITable>(&self) -> Option<&'static T> {
        let count = (self.Header.Length - size_of::<ACPISDTHeader>() as u32) / 4;
        for i in 0..count {
            let entry = self.Entries[i as usize];
            let hdr = unsafe { &*(entry as usize as *const ACPISDTHeader) };
            if &hdr.Signature == T::SIGNATURE {
                return unsafe { Self::getTableFromEntry::<T>(entry) };
            }
        }

        None
    }
}
