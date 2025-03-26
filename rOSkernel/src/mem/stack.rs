use x86_64::{
    structures::paging::{mapper, FrameAllocator, Mapper, Page, Size4KiB},
    VirtAddr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct StackBounds {
    pub start: VirtAddr,
    pub end: VirtAddr,
}

impl StackBounds {
    pub fn contains(&self, addr: VirtAddr) -> bool {
        self.start <= addr && addr < self.end
    }
}

fn reserveStackMem(pages: u64) -> Page {
    use core::sync::atomic::{AtomicU64, Ordering};
    static STACK_ALLOC_NEXT: AtomicU64 = AtomicU64::new(0x5555_5555_0000);
    let startAddr = VirtAddr::new(
        STACK_ALLOC_NEXT.fetch_add(pages * Page::<Size4KiB>::SIZE, Ordering::Relaxed),
    );

    Page::from_start_address(startAddr).expect("`STACK_ALLOC_NEXT` is not page aligned")
}

pub fn allocStack(
    pages: u64,
    mapper: &mut impl Mapper<Size4KiB>,
    frameAllocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<StackBounds, mapper::MapToError<Size4KiB>> {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let guardPage = reserveStackMem(pages + 1);
    let stackStart = guardPage + 1;
    let stackEnd = stackStart + pages;

    for page in Page::range(stackStart, stackEnd) {
        let frame = frameAllocator
            .allocate_frame()
            .ok_or(mapper::MapToError::FrameAllocationFailed)?;
        let flags = Flags::PRESENT | Flags::WRITABLE | Flags::USER_ACCESSIBLE;
        unsafe {
            mapper.map_to(page, frame, flags, frameAllocator)?.flush();
        }
    }

    Ok(StackBounds {
        start: stackStart.start_address(),
        end: stackEnd.start_address(),
    })
}
