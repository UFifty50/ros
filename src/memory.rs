use x86_64::structures::paging::{OffsetPageTable, PageTable, PhysFrame, Size4KiB, FrameAllocator};
use x86_64::registers::control::Cr3;
use x86_64::{VirtAddr, PhysAddr};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

pub struct BootInfoFrameAllocator {
    memoryMap: &'static MemoryMap,
    next: usize,
}

pub unsafe fn init(physicalMemoryOffset: VirtAddr) -> OffsetPageTable<'static> {
    let level4Table = activeLevel4Table(physicalMemoryOffset);
    OffsetPageTable::new(level4Table, physicalMemoryOffset)
}

unsafe fn activeLevel4Table(physicalMemoryOffset: VirtAddr) -> &'static mut PageTable {
    let (level4TableFrame, _) = Cr3::read();
    let phys = level4TableFrame.start_address();
    let virt = physicalMemoryOffset + phys.as_u64();
    let pageTablePtr: *mut PageTable = virt.as_mut_ptr();
    &mut *pageTablePtr // unsafe
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memoryMap: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memoryMap,
            next: 0,
        }
    }

    fn usableFrames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memoryMap.iter();
        let usableRegions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addrRanges = usableRegions.map(|r| r.range.start_addr()..r.range.end_addr());
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
