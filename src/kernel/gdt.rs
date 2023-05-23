use x86_64::VirtAddr;
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use lazy_static::lazy_static;


pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096*5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stackStart = VirtAddr::from_ptr(unsafe { &STACK });
            let stackEnd = stackStart + STACK_SIZE;
            stackEnd
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let codeSelector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tssSelector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { codeSelector, tssSelector })
    };
}

struct Selectors {
    codeSelector: SegmentSelector,
    tssSelector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.codeSelector);
        load_tss(GDT.1.tssSelector);
    }
}
