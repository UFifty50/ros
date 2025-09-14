use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::gdt::{DescriptorFlags, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

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

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let kernelCodeSelector = gdt.append(Descriptor::kernel_code_segment());
        let kernelDataSelector = gdt.append(Descriptor::kernel_data_segment());
        let userSegmentSelector =
            gdt.append(Descriptor::UserSegment(DescriptorFlags::USER_CODE32.bits()));
        let userDataSelector = gdt.append(Descriptor::user_data_segment());
        let userCodeSelector = gdt.append(Descriptor::user_code_segment());
        let tssSelector = gdt.append(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                kernelCodeSelector,
                kernelDataSelector,
                userSegmentSelector,
                userDataSelector,
                userCodeSelector,
                tssSelector,
            },
        )
    };
}

pub struct Selectors {
    pub kernelCodeSelector: SegmentSelector,
    pub kernelDataSelector: SegmentSelector,
    pub userSegmentSelector: SegmentSelector,
    pub userDataSelector: SegmentSelector,
    pub userCodeSelector: SegmentSelector,
    pub tssSelector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::registers::segmentation::{CS, DS, ES, FS, GS, SS, Segment};

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.kernelCodeSelector);
        DS::set_reg(GDT.1.kernelDataSelector);
        ES::set_reg(GDT.1.kernelDataSelector);
        FS::set_reg(GDT.1.kernelDataSelector);
        GS::set_reg(GDT.1.kernelDataSelector);
        SS::set_reg(GDT.1.kernelDataSelector);
        load_tss(GDT.1.tssSelector);
    }
}
