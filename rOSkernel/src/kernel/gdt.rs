use lazy_static::lazy_static;
use x86_64::registers::segmentation::DS;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::gdt::{DescriptorFlags, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

const STACK_SIZE: usize = 4096 * 5;
pub const DOUBLE_FAULT_IST_INDEX: usize = 0;
pub const PAGE_FAULT_IST_INDEX: usize = 2;
pub const GENERAL_FAULT_IST_INDEX: usize = 3;

pub static DOUBLE_FAULT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
pub static PAGE_FAULT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
pub static GENERAL_FAULT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = {
            let stackStart = VirtAddr::from_ptr(&raw const DOUBLE_FAULT_STACK);
            let stackEnd = stackStart + STACK_SIZE as u64;
            stackEnd
        };
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX] = {
            let stackStart = VirtAddr::from_ptr(&raw const PAGE_FAULT_STACK);
            let stackEnd = stackStart + STACK_SIZE as u64;
            stackEnd
        };
        tss.interrupt_stack_table[GENERAL_FAULT_IST_INDEX] = {
            let stackStart = VirtAddr::from_ptr(&raw const GENERAL_FAULT_STACK);
            let stackEnd = stackStart + STACK_SIZE as u64;
            stackEnd
        };
        tss
    };
}

pub fn kernel_code_segment() -> Descriptor {
    let flags = DescriptorFlags::USER_SEGMENT
        | DescriptorFlags::PRESENT
        | DescriptorFlags::EXECUTABLE
        | DescriptorFlags::LONG_MODE;
    Descriptor::UserSegment(flags.bits())
}

pub fn kernel_data_segment() -> Descriptor {
    let flags =
        DescriptorFlags::USER_SEGMENT | DescriptorFlags::PRESENT | DescriptorFlags::LONG_MODE;
    Descriptor::UserSegment(flags.bits())
}

pub fn user_code_segment() -> Descriptor {
    let flags = DescriptorFlags::USER_SEGMENT
        | DescriptorFlags::PRESENT
        | DescriptorFlags::EXECUTABLE
        | DescriptorFlags::LONG_MODE
        | DescriptorFlags::DPL_RING_3;
    Descriptor::UserSegment(flags.bits())
}

pub fn user_data_segment() -> Descriptor {
    let flags = DescriptorFlags::USER_SEGMENT
        | DescriptorFlags::PRESENT
        | DescriptorFlags::LONG_MODE
        | DescriptorFlags::WRITABLE
        | DescriptorFlags::DPL_RING_3;
    Descriptor::UserSegment(flags.bits())
}

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernelCodeSelector = gdt.append(kernel_code_segment());
        let kernelDataSelector = gdt.append(kernel_data_segment());
        let userCodeSelector = gdt.append(user_code_segment());
        let userDataSelector = gdt.append(user_data_segment());
        let tssSelector = gdt.append(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                kernelCodeSelector,
                kernelDataSelector,
                userCodeSelector,
                userDataSelector,
                tssSelector,
            },
        )
    };
}

pub struct Selectors {
    pub kernelCodeSelector: SegmentSelector,
    pub kernelDataSelector: SegmentSelector,
    pub userCodeSelector: SegmentSelector,
    pub userDataSelector: SegmentSelector,
    pub tssSelector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.kernelCodeSelector);
        DS::set_reg(GDT.1.kernelDataSelector);
        load_tss(GDT.1.tssSelector);
    }
}
