pub mod SystemDescriptorPointer;
pub mod SystemDescriptorTable;
pub mod structures;

macro_rules! impl_acpitable_defaults {
    ($ty:ident, $sig:literal) => {
        impl ACPITable for $ty {
            const SIGNATURE: &'static [u8; 4] = $sig;

            fn header(&self) -> &ACPISDTHeader {
                &self.Header
            }

            fn validate(&self) -> bool {
                // validate signature
                if &self.header().Signature != Self::SIGNATURE {
                    return false;
                }

                // validate checksum
                let length = self.header().Length as usize;
                let ptr = self as *const _ as *const u8;
                let bytes = unsafe { core::slice::from_raw_parts(ptr, length) };

                bytes.iter().fold(0u8, |sum, &b| sum.wrapping_add(b)) == 0
            }
        }
    };
}

use crate::acpi::SystemDescriptorTable::ACPISDTHeader;
pub(crate) use impl_acpitable_defaults;

pub trait ACPITable: Sized + 'static {
    const SIGNATURE: &'static [u8; 4];

    fn header(&self) -> &ACPISDTHeader;
    fn validate(&self) -> bool;
}
