use x86_64::{
    VirtAddr,
    structures::paging::{FrameAllocator, FrameDeallocator, Mapper, Page, Size4KiB, mapper},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct StackBounds {
    pub start: VirtAddr,
    pub end: VirtAddr,
}

impl Default for StackBounds {
    fn default() -> Self {
        Self {
            start: VirtAddr::zero(),
            end: VirtAddr::zero(),
        }
    }
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
        // TODO: only user accessible if process is user mode
        // TODO: consider other flags
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

/// Deallocate a stack by unmapping its pages and returning the frames to the allocator.
/// The guard page was never mapped, so we only unmap the actual stack pages.
/// Note: The guard page's virtual address space is not reclaimed (it remains reserved).
pub fn deallocStack<M, F>(
    bounds: StackBounds,
    pages: u64,
    mapper: &mut M,
    frameAllocator: &mut F,
) -> Result<(), ()>
where
    M: Mapper<Size4KiB>,
    F: FrameDeallocator<Size4KiB>,
{
    let stackStart = Page::<Size4KiB>::containing_address(bounds.start);
    let stackEnd = stackStart + pages;

    for page in Page::range(stackStart, stackEnd) {
        let (frame, flush) = mapper.unmap(page).map_err(|_| ())?;
        flush.flush();
        // Return the frame to the allocator for reuse
        unsafe { frameAllocator.deallocate_frame(frame); }
    }

    Ok(())
}
