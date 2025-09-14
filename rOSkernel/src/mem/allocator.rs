use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::{structures::paging::PageSize, VirtAddr};

/// A multi-heap allocator that tracks the next free virtual address to map.
#[derive(Debug)]
pub struct MultiHeapAllocator {
    next: u64,
}

impl MultiHeapAllocator {
    /// Create a new heap allocator, starting just after the kernel end symbol.
    pub fn new() -> Self {
        unsafe extern "C" {
            static _end: u8;
        }
        let kernel_end = unsafe { &_end as *const u8 as u64 };
        let align = Size4KiB::SIZE;
        let start = (kernel_end + align - 1) & !(align - 1);
        MultiHeapAllocator { next: start }
    }

    /// Map and reserve a heap region of `size` bytes, returning its start address.
    pub fn init_heap(
        &mut self,
        mapper: &mut impl Mapper<Size4KiB>,
        frame_allocator: &mut impl FrameAllocator<Size4KiB>,
        size: u64,
    ) -> Result<(VirtAddr, u64), MapToError<Size4KiB>> {
        let heap_start = VirtAddr::new(self.next);
        let heap_end = heap_start + (size - 1);
        let page_range = {
            let start_page = Page::containing_address(heap_start);
            let end_page = Page::containing_address(heap_end);
            Page::range_inclusive(start_page, end_page)
        };
        for page in page_range {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            let flags = PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE;
            unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
        }
        // advance next to the next aligned address after this heap
        let align = Size4KiB::SIZE;
        let next = (heap_end.as_u64() + 1 + align - 1) & !(align - 1);
        self.next = next;
        Ok((heap_start, size))
    }
}
