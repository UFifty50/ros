use core::sync::atomic::{AtomicU32, Ordering};

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::registers::control::Cr2;
use ps2::Controller;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;

use crate::{print, println};
use crate::kernel::{gdt, RTC, binIO};


lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpointHandler);
        idt.invalid_opcode.set_handler_fn(invalidOpcodeHandler);
        idt.page_fault.set_handler_fn(pageFaultHandler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timerInterruptHandler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboardInterruptHandler);
        idt[InterruptIndex::Floppy.as_usize()].set_handler_fn(floppyInterruptHandler);
        idt[InterruptIndex::RealTimeClock.as_usize()].set_handler_fn(realTimeClockInterruptHandler);
        unsafe {
            idt.double_fault.set_handler_fn(doubleFaultHandler)
               .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        
        idt
    };

    pub static ref CONTROLLER: Mutex<Controller> = unsafe { Mutex::new(
        Controller::with_timeout(50000)
    )};
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> = Mutex::new(
    unsafe {
        ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
    }
);

pub(crate) static TICK_COUNTER: AtomicU32 = AtomicU32::new(0);


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Floppy = PIC_1_OFFSET + 6,
    RealTimeClock = PIC_2_OFFSET
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
       usize::from(self as u8)
    }
}

pub fn initIDT() {
    IDT.load();
}

extern "x86-interrupt" fn breakpointHandler(stackFrame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stackFrame);
}

extern "x86-interrupt" fn invalidOpcodeHandler(stackFrame: InterruptStackFrame) {
    println!("EXCEPTION: INVALID OPCODE\n{:#?}", stackFrame);
}

extern "x86-interrupt" fn pageFaultHandler(stackFrame: InterruptStackFrame, errCode: PageFaultErrorCode) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", errCode);
    println!("{:#?}", stackFrame);
    loop { x86_64::instructions::hlt(); };
}

extern "x86-interrupt" fn timerInterruptHandler(_stackFrame: InterruptStackFrame) {
    print!(".");
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
    
    TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
}

extern "x86-interrupt" fn keyboardInterruptHandler(_stackFrame: InterruptStackFrame) {
    let mut controller = unsafe {
        CONTROLLER.force_unlock();
        CONTROLLER.lock()
    };

    let scanread = controller.read_data();
    if let Ok(mut scancode) = scanread {
        scancode = scanread.unwrap();
        crate::task::keyboard::addScancode(scancode);
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn floppyInterruptHandler(_stackFrame: InterruptStackFrame) {
    print!("F");
    // TODO: call fs::floppy::<methods to read/write floppy>
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Floppy.as_u8());
    }
}

extern "x86-interrupt" fn realTimeClockInterruptHandler(_stackFrame: InterruptStackFrame) {
    unsafe {
        binIO::out8(0x70, 0x0C);
        binIO::in8(0x71);
        RTC::readRTC();

        PICS.lock().notify_end_of_interrupt(InterruptIndex::RealTimeClock.as_u8());
    }
}

extern "x86-interrupt" fn doubleFaultHandler(stackFrame: InterruptStackFrame, _errCode: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stackFrame);
}
