use crate::util::OnceInit::OnceInit;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

#[derive(Debug)]
pub struct BootInfoFrameAllocator {
    memoryRegions: &'static MemoryRegions,
    next: usize,
}
unsafe impl Send for BootInfoFrameAllocator {}
unsafe impl Sync for BootInfoFrameAllocator {}

pub static PHYSICAL_MEMORY_OFFSET: OnceInit<u64> = OnceInit::new();

pub(crate) fn physToVirt(physAddr: u64) -> VirtAddr {
    let offset = PHYSICAL_MEMORY_OFFSET.get_copy().unwrap();
    let virtAddr = physAddr + offset;
    VirtAddr::new(virtAddr)
}

pub unsafe fn init(physicalMemoryOffset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level4Table = activeLevel4Table(physicalMemoryOffset);
        OffsetPageTable::new(level4Table, physicalMemoryOffset)
    }
}

unsafe fn activeLevel4Table(physicalMemoryOffset: VirtAddr) -> &'static mut PageTable {
    unsafe {
        let (level4TableFrame, _) = Cr3::read();
        let physAddr: PhysAddr = level4TableFrame.start_address();
        let virtAddr: VirtAddr = physicalMemoryOffset + physAddr.as_u64();
        &mut *virtAddr.as_mut_ptr()
    }
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memoryRegions: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memoryRegions,
            next: 0,
        }
    }

    fn usableFrames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memoryRegions.iter();
        let usableRegions = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        let addrRanges = usableRegions.map(|r| r.start..r.end);
        let frameAddresses = addrRanges.flat_map(|r| r.step_by(4096));
        frameAddresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usableFrames().nth(self.next);
        self.next += 1;
        frame
    }
}
