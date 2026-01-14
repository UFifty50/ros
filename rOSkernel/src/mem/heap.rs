use core::alloc::{AllocError, Allocator, GlobalAlloc, Layout};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};
use crate::mem::HEAP;

const MAX_HEAP_REGIONS: u64 = 8;
pub const SMALL_CLASSES: u64 = 8;
pub const SMALL_MIN: u64 = 8;
pub const SMALL_MAX: u64 = SMALL_MIN << (SMALL_CLASSES - 1);
const ALLOC_MAGIC: u32 = 0x1BADF00Du32;


#[inline]
fn sizeToClass(size: u64) -> u64 {
    if size <= SMALL_MIN { return 0; }
    let mut class = 0;
    let mut s = SMALL_MIN;
    while s < size && class + 1 < SMALL_CLASSES {
        s <<= 1;
        class += 1;
    }
    class
}

#[inline]
fn classSize(class: u64) -> u64 {
    SMALL_MIN << class
}

#[inline]
const fn alignUp(addr: u64, align: u64) -> u64 {
    (addr + (align - 1)) & !(align - 1)
}

#[repr(C, packed)]
struct AllocHeader {
    allocSize: u64,
    magic: u32,
    regionIdx: u16,
    _reserved1: [u8; 2], // padding to manually align to 8-byte boundary
}

impl AllocHeader {
    pub const fn size() -> u64 {
        alignUp(size_of::<AllocHeader>() as u64, align_of::<AllocHeader>() as u64)
    }
}

#[derive(Debug)]
pub struct Region {
    base: u64,
    end: u64,
    bump: AtomicU64,
    smallFree: [AtomicPtr<u8>; SMALL_CLASSES as usize],
    largeFree: AtomicPtr<u8>,
    regionIdx: u64
}
unsafe impl Send for Region {}
unsafe impl Sync for Region {}
impl Region {
    pub const fn uninit() -> Self {
        const NULLPTR: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
        Region {
            base: 0,
            end: 0,
            bump: AtomicU64::new(0),
            smallFree: [NULLPTR; SMALL_CLASSES as usize],
            largeFree: AtomicPtr::new(core::ptr::null_mut()),
            regionIdx: 0,
        }
    }

    pub fn init(&mut self, base: u64, size: u64, regionIdx: u64) {
        self.base = base;
        self.end = base + size;
        self.bump.store(base, Ordering::Release);
        self.regionIdx = regionIdx;
        self.largeFree.store(core::ptr::null_mut(), Ordering::Relaxed);

        for free in &self.smallFree {
            free.store(core::ptr::null_mut(), Ordering::Relaxed);
        }
    }

    pub fn alloc(&self, size: u64, align: u64) -> Option<NonNull<u8>> {
        if size <= SMALL_MAX {
            let sizeClass = sizeToClass(size);
            if let Some(headerPtr) = self.popFree(sizeClass) {
                let headerAddr = headerPtr.as_ptr() as u64;
                let payload = (headerAddr + AllocHeader::size()) as *mut u8;
                return NonNull::new(payload);
            }

            let allocSize = classSize(sizeClass);
            let total = AllocHeader::size().saturating_add(allocSize);
            let payloadAlign = allocSize.max(align);
            return self.bumpAllocWithHeader(total, payloadAlign, size);
        }

        // Try to find a suitable block in the large free list
        if let Some(ptr) = self.popLargeFree(size, align) {
            return Some(ptr);
        }

        let total = AllocHeader::size().saturating_add(size);
        self.bumpAllocWithHeader(total, align.max(align_of::<AllocHeader>() as u64), size)
    }

    fn bumpAllocWithHeader(&self, total: u64, align: u64, size: u64) -> Option<NonNull<u8>> {
        loop {
            let curr = self.bump.load(Ordering::Acquire);
            let payloadCandidate = alignUp(curr + AllocHeader::size(), align);
            let headerBase = payloadCandidate - AllocHeader::size();
            let next = headerBase.checked_add(total)?;
            if next > self.end { return None; }

            if self.bump.compare_exchange(curr, next, Ordering::AcqRel, Ordering::Acquire).is_ok() {
                let headerPtr = headerBase as *mut AllocHeader;
                unsafe {
                    core::ptr::write(headerPtr, AllocHeader {
                        allocSize: size,
                        regionIdx: self.regionIdx as u16,
                        magic: ALLOC_MAGIC,
                        _reserved1: [0; 2],
                    });
                }
                let payloadPtr = payloadCandidate as *mut u8;
                return NonNull::new(payloadPtr);
            }

            core::hint::spin_loop();
        }
    }

    pub fn pushFree(&self, sizeClass: u64, header: NonNull<u8>) {
        let head: &AtomicPtr<u8> = &self.smallFree[sizeClass as usize];

        loop {
           let old = head.load(Ordering::Acquire);
            unsafe {
                let slot = header.as_ptr() as *mut*mut u8;
                core::ptr::write(slot, old);
            }
            if head.compare_exchange(old, header.as_ptr(), Ordering::AcqRel, Ordering::Acquire).is_ok() {
                return;
            }

            core::hint::spin_loop();
        }
    }

    pub fn popFree(&self, sizeClass: u64) -> Option<NonNull<u8>> {
        let head: &AtomicPtr<u8> = &self.smallFree[sizeClass as usize];

        loop {
            let curr = head.load(Ordering::Acquire);
            if curr.is_null() { return None }
            let next = unsafe { *(curr as *mut*mut u8) };

            if head.compare_exchange(curr, next, Ordering::AcqRel, Ordering::Acquire).is_ok() {
                return NonNull::new(curr);
            }

            core::hint::spin_loop();
        }
    }

    /// Push a large allocation onto the large free list.
    /// The free list node is stored at the header address:
    /// [next_ptr: *mut u8][size: u64][...payload...]
    pub fn pushLargeFree(&self, header: NonNull<u8>, size: u64) {
        let head = &self.largeFree;
        let node_ptr = header.as_ptr();

        loop {
            let old = head.load(Ordering::Acquire);
            unsafe {
                // Store next pointer at offset 0
                let next_slot = node_ptr as *mut *mut u8;
                core::ptr::write(next_slot, old);
                // Store size at offset 8 (after next pointer)
                let size_slot = (node_ptr as *mut u8).add(8) as *mut u64;
                core::ptr::write(size_slot, size);
            }
            if head.compare_exchange(old, node_ptr, Ordering::AcqRel, Ordering::Acquire).is_ok() {
                return;
            }
            core::hint::spin_loop();
        }
    }

    /// Try to pop a suitable large allocation from the free list.
    /// Uses first-fit: returns the first block that is large enough.
    pub fn popLargeFree(&self, size: u64, align: u64) -> Option<NonNull<u8>> {
        let head = &self.largeFree;
        let _total_needed = AllocHeader::size() + size;

        loop {
            let curr = head.load(Ordering::Acquire);
            if curr.is_null() {
                return None;
            }

            let (next, block_size) = unsafe {
                let next = *(curr as *const *mut u8);
                let block_size = *((curr as *const u8).add(8) as *const u64);
                (next, block_size)
            };

            // Check if this block is large enough
            let payload_addr = curr as u64 + AllocHeader::size();
            let aligned_payload = alignUp(payload_addr, align);
            let actual_header = aligned_payload - AllocHeader::size();
            let space_needed = (aligned_payload - curr as u64) + size;

            if block_size >= space_needed {
                // Try to remove this block from the list
                if head.compare_exchange(curr, next, Ordering::AcqRel, Ordering::Acquire).is_ok() {
                    // Rewrite the header at the aligned position
                    let header_ptr = actual_header as *mut AllocHeader;
                    unsafe {
                        core::ptr::write(header_ptr, AllocHeader {
                            allocSize: size,
                            regionIdx: self.regionIdx as u16,
                            magic: ALLOC_MAGIC,
                            _reserved1: [0; 2],
                        });
                    }
                    return NonNull::new(aligned_payload as *mut u8);
                }
                core::hint::spin_loop();
                continue;
            }

            // Block too small, but we can't easily skip in a lock-free list
            // For now, return None and fall back to bump allocation
            // A more sophisticated implementation would use a skiplist or tree
            return None;
        }
    }
}

#[derive(Debug)]
pub struct HeapInner {
    regions: [Region; MAX_HEAP_REGIONS as usize],
    regionCount: AtomicU64,
    rrNext: AtomicU64,
}

impl HeapInner {
    pub const fn new() -> Self {
        HeapInner {
            regions: [const { Region::uninit() }; MAX_HEAP_REGIONS as usize],
            regionCount: AtomicU64::new(0),
            rrNext: AtomicU64::new(0),
        }
    }

    #[inline(always)]
    pub unsafe fn as_ref(&self) -> &Self {
        unsafe { &*(self as *const Self) }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct Heap {
    inner: *const HeapInner,
}

impl Heap {
    pub const fn new() -> Self {
        Heap {
            inner: &HeapInner::new(),
        }
    }

    pub const fn from_ptr(ptr: *const HeapInner) -> Self {
        Heap { inner: ptr }
    }

    #[inline(always)]
    fn inner(&self) -> &HeapInner {
        unsafe { &*self.inner }
    }

    #[inline(always)]
    fn inner_mut(&self) -> &mut HeapInner {
        unsafe { &mut *(self.inner as *mut HeapInner) }
    }

    pub fn addRegion(&self, base: u64, size: u64) -> Result<u64, ()> {
        let idx = self.inner().regionCount.fetch_add(1, Ordering::AcqRel);
        if idx >= MAX_HEAP_REGIONS {
            self.inner().regionCount.fetch_sub(1, Ordering::AcqRel);
            return Err(());
        }

        let region: &mut Region = unsafe { self.inner_mut().regions.get_unchecked_mut(idx as usize) };
        region.init(base, size, idx);
        Ok(idx)
    }

    pub fn allocSize(&self, size: u64, align: u64) -> Option<NonNull<u8>> {
        let count = self.inner().regionCount.load(Ordering::Acquire);
        if count == 0 { return None; }

        let start = self.inner().rrNext.fetch_add(1, Ordering::Relaxed) % count;
        for i in 0..count {
            let idx = (start + i) % count;
            let region: &Region = &self.inner().regions[idx as usize];
            if let Some(ptr) = region.alloc(size, align) {
                return Some(ptr);
            }
        }

        None
    }

    pub fn deallocPayload(&self, payload: NonNull<u8>) {
        let headerAddr = payload.as_ptr() as u64 - AllocHeader::size();
        let headerPtr = headerAddr as *const AllocHeader;
        // payload must be valid
        let header = unsafe { core::ptr::read(headerPtr) };
        if header.magic != ALLOC_MAGIC {
            // TODO: maybe panic? do something
            return;
        }

        let count = self.inner().regionCount.load(Ordering::Acquire);
        if header.regionIdx as u64 >= count {
            return;
        }

        let region = &self.inner().regions[header.regionIdx as usize];
        if header.allocSize <= SMALL_MAX {
            let headerNonNull = NonNull::new(headerAddr as *mut u8).unwrap();
            region.pushFree(sizeToClass(header.allocSize), headerNonNull);
        }
        else {
            // Add to large free list for potential reuse
            let totalSize = AllocHeader::size() + header.allocSize;
            let headerNonNull = NonNull::new(headerAddr as *mut u8).unwrap();
            region.pushLargeFree(headerNonNull, totalSize);
        }
    }
}

unsafe impl Send for Heap {}
unsafe impl Sync for Heap {}

impl Clone for Heap {
    fn clone(&self) -> Self {
        Heap { inner: self.inner }
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match HEAP.allocSize(layout.size() as u64, layout.align() as u64) {
            Some(nonNull) => nonNull.as_ptr(),
            None => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() { return; }

        let nonNull = NonNull::new(ptr).unwrap();
        HEAP.deallocPayload(nonNull);
    }
}

unsafe impl Allocator for Heap {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let ptr = HEAP.alloc(layout);
            if ptr.is_null() {
                Err(AllocError)
            } else {
                Ok(NonNull::slice_from_raw_parts(NonNull::new_unchecked(ptr), layout.size()))
            }
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { HEAP.dealloc(ptr.as_ptr(), layout) };
    }
}
