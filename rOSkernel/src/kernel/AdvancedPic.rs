//use crate::kernel::acpi::ACPIHandler;
use crate::kernel::{kernelContext, KernelContext};
use crate::mem::memory;
use acpi::platform::interrupt::Apic;
use acpi::{AcpiTables, InterruptModel};
use core::alloc::Allocator;
use core::ptr::{read_volatile, write_volatile};
use linked_list_allocator::LockedHeap;
use log::{error, info};
use x86_64::registers::model_specific::Msr;
use x86_64::VirtAddr;

/// AdvancedPIC provides APIC management similar to the ChainedPics from pic8259.
#[derive(Debug)]
pub struct AdvancedPic;

impl AdvancedPic {
    /// Initializes the local APIC and sets the spurious interrupt vector.
    ///
    /// # Safety
    ///
    /// This function performs raw pointer writes to APIC registers.
    pub fn init() {
        // TODO: disable PIC8259
        // TODO: IOAPIC
        // get interrupt model and processor info from MADT

        let apic = kernelContext()
            .constants
            .ACPI_INTERRUPT_MODEL
            .get()
            .expect("ACPI Interrupt Model not initialized");

        let processorInfo = kernelContext()
            .constants
            .ACPI_PROCESSOR_INFO
            .get()
            .expect("ACPI Processor Info not initialized");

        unsafe {
            let mut apicMsr = Msr::new(0x1B); // IA32_APIC_BASE
            let mut value = apicMsr.read();
            // enable bit 8
            apicMsr.write(value | 1 << 8);
            info!("Local APIC enabled (IA32_APIC_BASE=0x{:X})", value);
        }

        match apic {
            InterruptModel::Apic(apic) => unsafe {
                // local apic address
                let apicBaseAddr: VirtAddr = memory::physToVirt(apic.local_apic_address);
                let svrAddr = apicBaseAddr.as_u64() as usize;

                const LAPIC_SVR: usize = 0xF0;
                Self::write(svrAddr, LAPIC_SVR, 0x100 | 0xFF);

                // for ioApic in apic.io_apics.iter() {
                //     let ioApicAddr: VirtAddr = memory::physToVirt(ioApic.address as u64);
                //     let ioApicBase = ioApicAddr.as_u64();
                // }
            },

            _ => {
                error!("Unsupported interrupt model: {:?}", apic);
            }
        }
    }

    unsafe fn read(base: usize, register: usize) -> u32 {
        let addr = (base + register) as *const u32;
        read_volatile(addr)
    }

    unsafe fn write(base: usize, register: usize, value: u32) {
        let addr = (base + register) as *mut u32;
        write_volatile(addr, value);
    }

    /// Returns the (virtual) base address of the local APIC.
    ///
    /// The base address is read from the IA32_APIC_BASE MSR (bits 12 to 35).
    fn apicBase() -> *mut u8 {
        unsafe {
            let apicMsr = Msr::new(0x1B); // IA32_APIC_BASE
            let value = apicMsr.read();
            // The base address is stored in bits [12, 35]
            let base = value & 0xFFFFF000;
            base as *mut u8
        }
    }

    /// Notify the APIC that the interrupt has been serviced.
    ///
    /// This writes to the End-Of-Interrupt (EOI) register located at offset 0xB0.
    pub fn notify_end_of_interrupt(&self) {
        unsafe {
            let eoi_reg = Self::apicBase().add(0xB0) as *mut u32;
            write_volatile(eoi_reg, 0);
            info!("APIC End-of-Interrupt (EOI) sent");
        }
    }

    /// (Optional) Send an Inter-Processor Interrupt (IPI) using the APIC.
    ///
    /// This example method shows how you might begin implementing IPI functionality.
    #[allow(dead_code)]
    pub fn send_ipi(&self, apic_id: u8, vector: u8) {
        // The Interrupt Command Register (ICR) is split into two 32-bit registers.
        // Typically, one writes to the high and low parts separately.
        // Here, we write to the ICR high (offset 0x310) to set the destination APIC ID,
        // then to the ICR low (offset 0x300) to trigger the IPI.
        unsafe {
            let apic_base = Self::apicBase();
            // ICR high: destination field (bits 24..31)
            let icr_high = apic_base.add(0x310) as *mut u32;
            write_volatile(icr_high, (apic_id as u32) << 24);
            // ICR low: delivery mode, level, trigger mode and vector.
            let icr_low = apic_base.add(0x300) as *mut u32;
            // For a fixed delivery mode, assert level, edge-triggered.
            let icr_value = (vector as u32) | (0 << 8) | (1 << 14) | (0 << 15);
            write_volatile(icr_low, icr_value);
            info!(
                "IPI sent to APIC ID {} with vector 0x{:X} (ICR=0x{:X})",
                apic_id, vector, icr_value
            );
        }
    }
}
