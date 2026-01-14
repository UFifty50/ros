use crate::mem::memory::physToVirt;
use acpi::{AcpiHandler, PhysicalMapping};
use core::ptr::NonNull;

#[derive(Clone)]
pub struct ACPIHandler;

impl AcpiHandler for ACPIHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virtualAddr = physToVirt(physical_address as u64);
        unsafe {
            PhysicalMapping::new(
                physical_address,
                NonNull::new(virtualAddr.as_mut_ptr()).unwrap(),
                size,
                size,
                Self,
            )
        }
    }

    fn unmap_physical_region<T>(_physical_mapping: &PhysicalMapping<Self, T>) {}
}
