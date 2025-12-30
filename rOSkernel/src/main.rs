#![allow(non_snake_case)]
#![allow(named_asm_labels)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use acpi::AcpiTables;
use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::info::{FrameBuffer, FrameBufferInfo, MemoryRegions};
use bootloader_api::BootInfo;
use bootloader_x86_64_common::logger::LockedLogger;
use rOSkernel::multitasking::preemptive::{thread::Process, Parent};
use core::alloc::Layout;
use core::panic::PanicInfo;
use spin::Mutex;
use rOSkernel::kernel::framebuffer::FrameBufferEditor;
use rOSkernel::kernel::AdvancedPic::AdvancedPic;
use rOSkernel::kernel::{gdt, initKernelContext, interrupts, kernelContext, setKernelFrameAllocator, setKernelFrameBuffer, setKernelHeapManager, setKernelLogger, setKernelMapper};
use rOSkernel::mem::allocator::HeapRegionAllocator;
use rOSkernel::mem::{memory, memory::BootInfoFrameAllocator, HEAP};
use rOSkernel::tasks::keyboard;
use x86_64::VirtAddr;
use rOSkernel::kernel::kacpi::ACPIHandler;
use rOSkernel::mem::heap::Heap;
use rOSkernel::util::wrappers::{CPUID, FPU_MECHANISM, FpuSaveMechanism, XFeatures, readCR0, readCR4, writeCR0, writeCR4};
use core::sync::atomic::Ordering;


pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

bootloader_api::entry_point!(kMain, config = &BOOTLOADER_CONFIG);

#[panic_handler]
fn kPanic(info: &PanicInfo) -> ! {
    unsafe {
        bootloader_x86_64_common::logger::LOGGER
            .get()
            .map(|l: &LockedLogger| l.force_unlock())
    };
    log::error!("{}", info);

    x86_64::instructions::interrupts::disable();
    loop {
        x86_64::instructions::nop();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

extern "C" fn threadFunc1() {
    // calculate Fibonacci numbers
    let mut a: u64 = 0;
    let mut b: u64 = 1;
    let mut i: u64 = 0;
    loop {
        let c = a.wrapping_add(b);
        a = b;
        b = c;
        i += 1;
        if i % 10_000_000 == 0 {
            log::info!("Hello from PID {:?}! Fibonacci number: {}", rOSkernel::multitasking::preemptive::current_pid().unwrap(), c);
        }
    }
}

extern "C" fn threadFunc2() {
    // calculate e
    let mut n: u64 = 1;
    let mut factorial: u64 = 1;
    let mut e: f64 = 2.0;
    let mut i: u64 = 0;
    loop {
        n += 1;
        factorial = factorial.wrapping_mul(n);
        e += 1.0 / (factorial as f64);
        i += 1;
        if i % 10_000_000 == 0 {
            log::info!("Hello from PID {:?}! Approximation of e: {}", rOSkernel::multitasking::preemptive::current_pid().unwrap(), e);
        }
    }
}

extern "C" fn threadFunc3() {
    // calculate pi using Leibniz formula
    let mut k: u64 = 0;
    let mut pi: f64 = 0.0;
    let mut i: u64 = 0;
    loop {
        let term = if k % 2 == 0 {
            1.0 / (2 * k + 1) as f64
        } else {
            -1.0 / (2 * k + 1) as f64
        };
        pi += term;
        k += 1;
        i += 1;
        if i % 10_000_000 == 0 {
            log::info!("Hello from PID {:?}! Approximation of pi: {}", rOSkernel::multitasking::preemptive::current_pid().unwrap(), pi * 4.0);
        }
    }
}

extern "C" fn kernelInit() {
    log::trace!("Kernel Init Thread started");
    
    // These processes will automatically inherit kernelInit as their parent
    let p1 = Process::create(Parent::Independent);
    let tid1 = p1.create_thread(threadFunc1, 20);
    let _ = p1.start_thread(tid1);

    let p2 = Process::create(Parent::Inherit);
    let tid2 = p2.create_thread(threadFunc2, 20);
    let _ = p2.add_thread_xfeatures(&tid2, XFeatures((1u64 << 17) | (1u64 << 18))); // enable AMX
    let _ = p2.start_thread(tid2);

    let p3 = Process::create(Parent::Inherit);
    let tid3 = p3.create_thread(threadFunc3, 20);
    let _ = p3.add_thread_xfeatures(&tid3, XFeatures((1u64 << 5) | (1u64 << 6) | (1u64 << 7))); // enable AVX-512
    let _ = p3.start_thread(tid3);
    loop {
        let mut i = 0;
        while i < 100_000_000 {
            i += 1;
        }
        log::info!("Current PID: {:?}", rOSkernel::multitasking::preemptive::current_pid().unwrap());
    }
}

fn kMain(bootInfo: &'static mut BootInfo) -> ! {
    initKernelContext();

    let physMemOffset = bootInfo.physical_memory_offset.into_option().unwrap();
    memory::PHYSICAL_MEMORY_OFFSET.get_or_init(|| physMemOffset);

    // Instead of borrowing bootInfo.framebuffer multiple times, get a raw pointer.
    let fbPtr = bootInfo.framebuffer.as_mut().unwrap() as *mut FrameBuffer;
    let fbInfo = unsafe { (*fbPtr).info() };
    let fbBytes = unsafe { (&mut *fbPtr).buffer_mut() };
    initLogger(fbBytes, fbInfo);

    log::trace!("Bootloader framebuffer info: {:?}", fbInfo);

    setKernelFrameBuffer({
        let fb = unsafe { &mut *fbPtr };
        FrameBufferEditor::new(fb, fbInfo)
    });

    gdt::init();

    unsafe {
        core::arch::asm!("mov ss, {0:x}", in(reg) 0u16, options(nostack, preserves_flags));
    }

    interrupts::initIDT();
    initXFeatures();

    // unsafe { interrupts::PICS.lock().initialize() }; // if not using APIC
    // x86_64::instructions::interrupts::enable();
    // initialise PIT
   // RTC::initRTC();

    initMemory(
        memory::PHYSICAL_MEMORY_OFFSET.get_copy().unwrap(),
        &bootInfo.memory_regions,
    );

    let acpiTables = unsafe {
        AcpiTables::from_rsdp(
            ACPIHandler,
            bootInfo.rsdp_addr.into_option().unwrap() as usize,
        )
        .expect("TODO: panic message")
    };
    let _fadt = acpiTables.find_table::<acpi::fadt::Fadt>().unwrap();
    let _dsdt = acpiTables.dsdt().unwrap();
    let madtPhysMap = acpiTables.find_table::<acpi::madt::Madt>().unwrap();
    let madt = madtPhysMap.get();
    let mut heapRegAlloc = kernelContext().heapRegionAllocator.get().unwrap().lock();
    let acpiAllocator: Heap = {
        let (start, len) = heapRegAlloc
            .init_heap(1024 * 1024)
            .unwrap();
        let heap = Heap::new();
        heap.addRegion(start.as_u64(), len).expect("Too many regions (new heap, shouldn't happen)");
        heap
    };
    let madtInfo = madt.parse_interrupt_model_in(acpiAllocator).unwrap();

    kernelContext()
        .constants
        .ACPI_PROCESSOR_INFO
        .get_or_init(|| madtInfo.1.unwrap());

    kernelContext()
        .constants
        .ACPI_INTERRUPT_MODEL
        .get_or_init(|| madtInfo.0);

    let apic = AdvancedPic::new();
    kernelContext()
        .apic
        .set(apic)
        .unwrap();
    kernelContext()
        .apic
        .get()
        .unwrap()
        .initAPICTimer();

    if let Err(e) = keyboard::keyboardInitialize() {
        panic!("Failed to initialize keyboard: {:?}", e);
    }

    log::trace!("Hello from kernel!");

    // kernel_process is the root process (PID 1), so it has no parent
    let kernel_process = Process::create(Parent::Independent);
    log::info!("Spawning kernel init thread");
    let tid = kernel_process.create_thread(kernelInit, 10);
    let _ = kernel_process.start_thread(tid);
    
    log::info!("Kernel initialization complete, starting scheduler");

    // Enable interrupts to allow the APIC timer to fire and switch to the first thread
    x86_64::instructions::interrupts::enable();
    
    rOSkernel::serial_println!("Interrupts enabled, entering HLT loop");

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn initMemory(
    physicalMemoryOffset: u64,
    memoryRegions: &'static MemoryRegions,
) {
    let virtMemOffset = VirtAddr::new(physicalMemoryOffset);
    let mapper = unsafe { memory::init(virtMemOffset) };
    let frameAllocator = unsafe { BootInfoFrameAllocator::init(memoryRegions) };
    setKernelMapper(Mutex::new(mapper));
    setKernelFrameAllocator(Mutex::new(frameAllocator));

    // initialize heap with desired size using a multi-heap allocator
    setKernelHeapManager(HeapRegionAllocator::new());

    let initialHeapSize: u64 = 2 * 1024 * 1024; // 2 MiB

    let (heapStart, heapSize) = kernelContext()
        .heapRegionAllocator
        .get()
        .unwrap()
        .lock()
        .init_heap(initialHeapSize)
        .expect("heap initialization failed");

    // register mapped region with the gobal HEAP
    HEAP.addRegion(heapStart.as_u64(), heapSize).expect("Failed to add region to heap");
}

pub fn initLogger(buffer: &'static mut [u8], info: FrameBufferInfo) {
    let logger = setKernelLogger(LockedLogger::new(buffer, info, true, true));
    log::set_logger(logger.unwrap()).expect("initLogger failed");
    log::set_max_level(log::LevelFilter::Trace);
    log::info!("Initialized kernel logger");
}


pub fn dump_cpuid_basics() {
    unsafe {
        let (_, ebx0, ecx0, edx0) = CPUID(0, 0);
        let mut v = [0u8; 12];
        v[0..4].copy_from_slice(&ebx0.to_le_bytes());
        v[4..8].copy_from_slice(&edx0.to_le_bytes());
        v[8..12].copy_from_slice(&ecx0.to_le_bytes());
        let vendor = core::str::from_utf8(&v).unwrap_or("<invalid vendor>");
        // allocations not allowed yet, global heap not set up
        // String::from_utf8_lossy(&v).into_owned()

        let (eax1, ebx1, ecx1, edx1) = CPUID(1, 0);
        log::info!("CPUID(0): vendor = {}", vendor);
        log::info!("CPUID(1): eax={:#010x} ebx={:#010x} ecx={:#010x} edx={:#010x}",
                   eax1, ebx1, ecx1, edx1);
        let osxsave = (ecx1 >> 27) & 1;
        let hypervisor = (ecx1 >> 31) & 1;
        log::info!("OSXSAVE bit = {} (ECX bit 27). Hypervisor-present bit = {} (ECX bit 31).",
                   osxsave, hypervisor);

        // Also show CPUID leaf 0xD subleaf 0 (supported XCR0 bits)
        let (eaxD, _, _, edxD) = CPUID(0xD, 0);
        let supported = (edxD as u64) << 32 | (eaxD as u64);
        log::info!("CPUID(0xD,0): EAX={:#010x} EDX={:#010x} -> supported XCR0 mask = {:#018x}",
                   eaxD, edxD, supported);
    }
}


pub fn initXFeatures() {
    dump_cpuid_basics();
    unsafe {
        let (_, _, ecx1, edx1) = CPUID(1, 0);
        
        let has_xsave = (ecx1 & (1 << 26)) != 0;
        let has_fxsave = (edx1 & (1 << 24)) != 0;

        // For now, we prioritize FXSAVE and keep XSAVE disabled as requested
        if has_fxsave {
            log::info!("FXSAVE supported. Enabling OSFXSR...");
            
            // Ensure CR0.EM is clear and CR0.MP is set
            let mut cr0 = readCR0();
            const CR0_EM_BIT: u64 = 1 << 2;
            const CR0_MP_BIT: u64 = 1 << 1;
            if (cr0 & CR0_EM_BIT) != 0 {
                log::warn!("CR0.EM was set, clearing it.");
                cr0 &= !CR0_EM_BIT;
            }
            cr0 |= CR0_MP_BIT;
            writeCR0(cr0);

            let mut cr4 = readCR4();
            const CR4_OSFXSR_BIT: u64 = 1 << 9;
            cr4 |= CR4_OSFXSR_BIT;
            writeCR4(cr4);
            
            // Verify CR4 write
            let cr4_verify = readCR4();
            if (cr4_verify & CR4_OSFXSR_BIT) == 0 {
                log::error!("Failed to set CR4.OSFXSR! CR4 is {:#x}", cr4_verify);
            } else {
                FPU_MECHANISM.store(FpuSaveMechanism::FXSave as u8, Ordering::Relaxed);
                log::info!("Using FXSAVE mechanism");
            }
        } else if has_xsave {
             log::warn!("XSAVE supported but disabled by policy. Falling back to None.");
             // To enable XSAVE in future:
             // 1. Check CPUID_ECX_OSXSAVE_BIT (bit 27) ? No, check bit 26 for support.
             // 2. Set CR4_OSXSAVE_BIT (bit 18)
             // 3. Init XCR0
             // FPU_MECHANISM.store(FpuSaveMechanism::XSave as u8, Ordering::Relaxed);
        } else {
            log::warn!("Neither XSAVE nor FXSAVE supported. FPU state will not be saved.");
        }

        /* Original XSAVE init code - kept for reference but disabled
        // is XSAVE supported
        let (_, _, features1, _features2) = CPUID(1, 0);
        const CPUID_ECX_OSXSAVE_BIT: u32 = 1 << 27;
        if (features1 & CPUID_ECX_OSXSAVE_BIT) == 0 {
            log::warn!("CPUID_ECX_OSXSAVE_BIT is zero, XSAVE not supported");
         //   return;
        }

        // Enable XSAVE
        let mut cr4 = readCR4();
        const CR4_OSXSAVE_BIT: u64 = 1 << 18;
        cr4 |= CR4_OSXSAVE_BIT;
        writeCR4(cr4);

        log::info!("XCR0 before init: {:#?}", XFeatures::current());

        // Supported XCR0 flags
        let (supported_lo, _, _, supported_hi) = CPUID(0xD, 0);
        let supportedFeatures = (supported_hi as u64) << 32 | supported_lo as u64;
        kernelContext()
            .constants
            .SUPPORTED_XFEATURES
            .get_or_init(|| XFeatures(supportedFeatures));

        // build the "normal" mask
        const X87_BIT: u64 = 1 << 0;
        const SSE_BIT: u64 = 1 << 1;
        const AVX_BIT: u64 = 1 << 2;
        let desiredFeatures = X87_BIT | SSE_BIT | (supportedFeatures & AVX_BIT);
        xsetbv0(supportedFeatures & desiredFeatures);

        log::info!("XCR0 after init: {:#?}", XFeatures::current());
        */
    }
}
