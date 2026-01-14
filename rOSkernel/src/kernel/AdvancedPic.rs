use alloc::vec::Vec;
use core::alloc::Allocator;
use core::ops::Add;
use crate::kernel::kernelContext;
use crate::mem::memory;
use acpi::InterruptModel;
use acpi::platform::interrupt::Apic;
use log::{error, info};
use x86_64::registers::model_specific::Msr;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::structures::paging::mapper::MapToError;
use crate::kernel::binIO;
use crate::kernel::interrupts::InterruptIndex;


const LAPIC_ID_REG: usize = 0x20;
const LAPIC_TIMER_REG: usize = 0x320;
const LAPIC_TIMER_INIT_COUNT: usize = 0x380;
const LAPIC_TIMER_CURRENT_COUNT: usize = 0x390;
const LAPIC_TIMER_DIVIDE: usize = 0x3E0;

/// AdvancedPIC provides APIC management similar to the ChainedPics from pic8259.
#[derive(Debug)]
pub struct AdvancedPic {
    lapicBase: VirtAddr,
    ioApicBases: Vec<VirtAddr>,
}

impl AdvancedPic {
    pub fn new() -> Self {
        // get interrupt model and processor info from MADT

        let apic = kernelContext()
            .constants
            .ACPI_INTERRUPT_MODEL
            .get()
            .expect("ACPI Interrupt Model not initialized");

        let _processorInfo = kernelContext()
            .constants
            .ACPI_PROCESSOR_INFO
            .get()
            .expect("ACPI Processor Info not initialized");

        let mut advancedPic = AdvancedPic {
            lapicBase: VirtAddr::zero(),
            ioApicBases: Vec::new(),
        };

        match apic {
            InterruptModel::Apic(apic) => unsafe {
                let lapicAddrPhys =  Msr::new(0x1B).read() & 0xFFFF_F000;
                advancedPic.lapicBase = memory::physToVirt(lapicAddrPhys);
                let lapicID = (advancedPic.lapicRead(LAPIC_ID_REG) >> 24) as u8;

               // Self::mapMMIO(mapper, frameAllocator, lapicAddrPhys, lapicAddrVirt.as_u64()).expect("Failed to map APIC memory");

                let mut apicMsr = Msr::new(0x1B); // IA32_APIC_BASE
                let value = apicMsr.read();

                // set bit 11 (Apic global enable) and disable 8259 PIC
                apicMsr.write(value | 1u64 << 11);
                binIO::out8(0x21, 0xFF);
                binIO::out8(0xA1, 0xFF);
                // set lapic TPR=0
                advancedPic.lapicWrite(0x80, 0);
                info!("Local APIC enabled (IA32_APIC_BASE=0x{:X})", value);

                const LAPIC_SVR: usize = 0xF0;
                advancedPic.lapicWrite(LAPIC_SVR, 0x100 | 0xFF);

                let mut ioApicIdx = 0;
                for ioApic in apic.io_apics.iter() {
                    let ioApicAddrPhys = ioApic.address as u64 & 0xFFFF_F000;
                    advancedPic.ioApicBases.push(memory::physToVirt(ioApicAddrPhys));
                  //  Self::mapMMIO(mapper, frameAllocator, ioApicAddrPhys, ioApicAddrVirt).expect("Failed to map IOAPIC memory");

                    let baseGSI = ioApic.global_system_interrupt_base;
                    let gsiPIT = Self::isaIRQtoGSI(apic, 0);
                    let gsiKeyboard = Self::isaIRQtoGSI(apic, 1);
                    let gsiRTC = Self::isaIRQtoGSI(apic, 8);

                    if gsiPIT >= baseGSI {
                        let idx = gsiPIT - baseGSI;
                        let pitVec =  InterruptIndex::ProgIntTimer as u8;
                        advancedPic.ioApicSetRedirEntry(ioApicIdx, idx, pitVec, lapicID, false);
                    }

                    if gsiKeyboard >= baseGSI {
                        let idx = gsiKeyboard - baseGSI;
                        let kbdVec =  InterruptIndex::Keyboard as u8;
                        advancedPic.ioApicSetRedirEntry(ioApicIdx, idx, kbdVec, lapicID, false);
                    }

                    if gsiRTC >= baseGSI {
                        let idx = gsiRTC - baseGSI;
                        let rtcVec =  InterruptIndex::RealTimeClock as u8;
                        advancedPic.ioApicSetRedirEntry(ioApicIdx, idx, rtcVec, lapicID, false);
                    }

                    ioApicIdx += 1;
                }
            },

            _ => {
                error!("Unsupported interrupt model: {:?}", apic);
            }
        }

        advancedPic
    }

    unsafe fn lapicRead(&self, register: usize) -> u32 { unsafe {
        let addr = (self.lapicBase.as_u64() as usize + register) as *const u32;
        core::ptr::read_volatile(addr)
    }}

    unsafe fn lapicWrite(&self, register: usize, value: u32) { unsafe {
        let addr = (self.lapicBase.as_u64() as usize + register) as *mut u32;
        core::ptr::write_volatile(addr, value);
        let _ = core::ptr::read_volatile(addr);
    }}

    unsafe fn ioApicRead(&self, ioApicIdx: usize, register: u8) -> u32 { unsafe {
        without_interrupts(|| {
            core::ptr::write_volatile(self.ioApicBases[ioApicIdx].as_u64() as *mut u32, register as u32);
            core::ptr::read_volatile((self.ioApicBases[ioApicIdx].as_u64() + 0x10) as *const u32)
        })
    }}

    unsafe fn ioApicWrite(&self, ioApicIdx: usize, register: u8, value: u32) { unsafe {
        without_interrupts(|| {
            core::ptr::write_volatile(self.ioApicBases[ioApicIdx].as_u64() as *mut u32, register as u32);
            core::ptr::write_volatile((self.ioApicBases[ioApicIdx].as_u64() + 0x10) as *mut u32, value);
        });
    }}

    unsafe fn ioApicSetRedirEntry(&self, ioApicIdx: usize, gsiIndex: u32, vector: u8, apicID: u8, mask: bool) {
        let registerLo = 0x10 + (2 * gsiIndex);
        let registerHi = registerLo + 1;

        let high = (apicID as u32) << 24;
        let mut low = vector as u32;
        if mask {
            low |= 1 << 16;
        }

        unsafe {
            self.ioApicWrite(ioApicIdx, registerHi as u8, high);
            self.ioApicWrite(ioApicIdx, registerLo as u8, low);
        }
    }

    unsafe fn calibrateApicTimer(&self) -> u32 { unsafe {
        const PIT_CH2_GATE: u16 = 0x61;
        const PIT_CH2_DATA: u16 = 0x42;
        const PIT_CMD: u16 = 0x43;
        const PIT_FREQ_HZ: u32 = 1_193_182;
        const CALIBRATION_MS: u32 = 1;
        let waitTicks = (PIT_FREQ_HZ * CALIBRATION_MS) / 1000;

        // Configure PIT to one-shot mode
        let initial = binIO::in8(PIT_CH2_GATE);
        binIO::out8(PIT_CH2_GATE, (initial & 0xFC) | 1);

        binIO::out8(PIT_CMD, 0b10110000); // Channel 2, LSB/MSB, one-shot
        binIO::out8(PIT_CH2_DATA, (waitTicks & 0xFF) as u8);
        binIO::out8(PIT_CH2_DATA, ((waitTicks >> 8) & 0xFF) as u8);

        // start timer
        let current = binIO::in8(PIT_CH2_GATE) & 0xFE;
        binIO::out8(PIT_CH2_GATE, current);
        binIO::out8(PIT_CH2_GATE, current | 1);

        self.lapicWrite(LAPIC_TIMER_DIVIDE, 0x3); // divide by 16
        self.lapicWrite(LAPIC_TIMER_INIT_COUNT, 0xFFFFFFFF);

        // wait for PIT to expire
        while (binIO::in8(PIT_CH2_GATE) & 0x20) == 0 {
            core::hint::spin_loop();
        }

        self.lapicWrite(LAPIC_TIMER_REG, 1 << 16); // mask timer
        let elapsed = 0xFFFFFFFF - self.lapicRead(LAPIC_TIMER_CURRENT_COUNT);

        // restore PIT
        binIO::out8(PIT_CH2_GATE, initial);

        let ticksPerMs = elapsed / CALIBRATION_MS;
        log::trace!("APIC Timer calibrated: {} ticks/ms", ticksPerMs);
        ticksPerMs
    }}

    pub fn initAPICTimer(&self) {
        let ticksPerMs = unsafe { self.calibrateApicTimer() };

        unsafe {
            log::trace!("initializing APICTimer divide");
            self.lapicWrite(LAPIC_TIMER_DIVIDE, 0x3);
            log::trace!("initializing APICTimer init count");
            // Set initial count for 10ms
            self.lapicWrite(LAPIC_TIMER_INIT_COUNT, ticksPerMs * 10);
            log::trace!("initializing APICTimer reg");
            // Set timer to Periodic mode (bit 17) and unmask (clear bit 16)
            self.lapicWrite(LAPIC_TIMER_REG, (1 << 17) | (InterruptIndex::LApicTimer as u32));

            log::trace!("APIC timer configured for 10ms interval with {} ticks.", 10 * ticksPerMs);
        }

    }

    fn isaIRQtoGSI<A: Allocator>(apic: &Apic<A>, isaIRQ: u8) -> u32 {
        if let Some(intSrcOvrd) = apic.interrupt_source_overrides.iter().find(
            |intSrcOvrd| intSrcOvrd.isa_source == isaIRQ
        ) {
            intSrcOvrd.global_system_interrupt
        } else {
            // TODO: probably a better way of doing this
            apic.io_apics[0].global_system_interrupt_base + isaIRQ as u32
        }
    }

    pub fn notifyEOI(&self) {
        unsafe { self.lapicWrite(0xB0, 0) };
    }

    fn mapMMIO(
        mapper: &mut impl Mapper<Size4KiB>,
        frameAllocator: &mut impl FrameAllocator<Size4KiB>,
        physAddr: u64,
        virtAddr: u64
    ) -> Result<(), MapToError<Size4KiB>> {
        let page = Page::containing_address(VirtAddr::new(virtAddr & !0xFFF));
        let frame = PhysFrame::containing_address(PhysAddr::new(physAddr & !0xFFF));
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE
            | PageTableFlags::NO_EXECUTE | PageTableFlags::NO_CACHE;

        unsafe { mapper.map_to(page, frame, flags, frameAllocator)?.flush() };
        Ok(())
    }

    #[allow(dead_code)]
    pub fn sendIPI(&self, apic_id: u8, vector: u8) {
        // The Interrupt Command Register (ICR) is split into two 32-bit registers.
        // Typically, one writes to the high and low parts separately.
        // Here, we write to the ICR high (offset 0x310) to set the destination APIC ID,
        // then to the ICR low (offset 0x300) to trigger the IPI.
        unsafe {
            // ICR high: destination field (bits 24..31)
            let icr_high = self.lapicBase.add(0x310).as_mut_ptr();
            core::ptr::write_volatile(icr_high, (apic_id as u32) << 24);
            // ICR low: delivery mode, level, trigger mode and vector.
            let icr_low = self.lapicBase.add(0x300).as_mut_ptr();
            // For a fixed delivery mode, assert level, edge-triggered.
            let icr_value = (vector as u32) | (0 << 8) | (1 << 14) | (0 << 15);
            core::ptr::write_volatile(icr_low, icr_value);
            info!(
                "IPI sent to APIC ID {} with vector 0x{:X} (ICR=0x{:X})",
                apic_id, vector, icr_value
            );
        }
    }
}
