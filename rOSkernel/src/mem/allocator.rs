use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::{structures::paging::PageSize, VirtAddr};
use crate::kernel::kernelContext;

#[derive(Debug)]
pub struct HeapRegionAllocator {
    next: u64,
}

impl HeapRegionAllocator {
    pub fn new() -> Self {
        unsafe extern "C" {
            static _end: u8;
        }
        let kernel_end = unsafe { &_end as *const u8 as u64 };
        let align = Size4KiB::SIZE;
        let start = (kernel_end + align - 1) & !(align - 1);
        HeapRegionAllocator { next: start }
    }

    pub fn init_heap(
        &mut self,
        size: u64,
    ) -> Result<(VirtAddr, u64), MapToError<Size4KiB>> {
        let mut mapperGuard = kernelContext().mapper.get().unwrap().lock();
        let mut frameAllocatorGuard = kernelContext().frameAllocator.get().unwrap().lock();

        let heapStart = VirtAddr::new(self.next);
        let heapEnd = heapStart + (size - 1);
        let pageRange = {
            let startPage = Page::containing_address(heapStart);
            let end_page = Page::containing_address(heapEnd);
            Page::range_inclusive(startPage, end_page)
        };
        for page in pageRange {
            let frame = (*frameAllocatorGuard)
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE;
            unsafe { (*mapperGuard).map_to(page, frame, flags, &mut *frameAllocatorGuard)?.flush() };
        }
        // advance next to the next aligned address after this heap
        let align = Size4KiB::SIZE;
        let next = (heapEnd.as_u64() + 1 + align - 1) & !(align - 1);
        self.next = next;
        Ok((heapStart, size))
    }
}
