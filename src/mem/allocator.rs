use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::VirtAddr;
use linked_list_allocator::LockedHeap;


pub const HEAP_START: *mut u8 = 0x4444_4444_0000 as *mut u8;
pub const HEAP_SIZE: usize = 100 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn initHeap(
    mapper: &mut impl Mapper<Size4KiB>,
    frameAllocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let pageRange = {
        let heapStart = VirtAddr::new(HEAP_START as u64);
        let heapEnd = heapStart + (HEAP_SIZE - 1);
        let heapStartPage = Page::containing_address(heapStart);
        let heapEndPage = Page::containing_address(heapEnd);
        Page::range_inclusive(heapStartPage, heapEndPage)
    };
    
    for page in pageRange {
        let frame = frameAllocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frameAllocator)?.flush()
        };
    }
    
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

     Ok(())
}
