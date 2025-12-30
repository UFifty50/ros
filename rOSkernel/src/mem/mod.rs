pub mod allocator;
pub mod memory;
pub mod stack;
pub mod heap;


use heap::{Heap, HeapInner};

#[global_allocator]
pub static HEAP: Heap = Heap::from_ptr(&HEAP_INNER as *const HeapInner);
static HEAP_INNER: HeapInner = HeapInner::new();
