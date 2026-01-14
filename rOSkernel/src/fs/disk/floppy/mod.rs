use alloc::borrow::ToOwned;
use alloc::boxed::Box;

use crate::kernel::RTC::waitTicks;
use crate::kernel::{
    RTC,
    binIO::{in8, out8},
};

// The MSR byte:
//  7   6   5    4    3    2    1    0
// MRQ DIO NDMA BUSY ACTD ACTC ACTB ACTA
//
// MRQ  - is FIFO ready (1:yes, 0:no)
// DIO  - is controller expecting read/write (1:write, 0:read)
//
// NDMA - is controller in DMA mode (1:noDMA, 0:DMA)
// BUSY - is controller executing a command (1=busy)
//
// ACTA - which drives position/calibrated (1:yes, 0:no)
// ACTB - "
// ACTC - "
// ACTD - "

// The DOR byte: [write-only]
//  7    6    5    4    3   2    1   0
// MOTD MOTC MOTB MOTA DMA NRST DR1 DR0
//
// DR1 and DR0 together select "current drive" = a/00, b/01, c/10, d/11
// MOTA, MOTB, MOTC, MOTD control motors for the four drives (1=on)
//
// DMA line enables (1 = enable) interrupts and DMA
// NRST is "not reset" so controller is enabled when it's 1

// The CCR byte:
//  7 - 2     1      0
// reserved  RAT1  RAT0
// RAT1 and RAT0 together select the data rate:
// 00 = 500kbits/s
// 01 = 300kbits/s
// 10 = 250kbits/s
// 11 = 1Mbits/s

enum FloppyRegisters {}

impl FloppyRegisters {
    pub const SRA: u16 = 0; // Status Register A; R/O
    pub const SRB: u16 = 1; // Status Register B; R/O
    pub const DOR: u16 = 2; // Digital Output Register
    pub const TDR: u16 = 3; // Tape Drive Register
    pub const MSR: u16 = 4; // Master Status Register; R/O
    pub const DSR: u16 = 4; // Data Rate Select Register; W/O
    pub const FIFO: u16 = 5; // data FIFO
    pub const DIR: u16 = 7; // Digital Input Register; R/O
    pub const CCR: u16 = 7; // Configuration Control Register; W/O
}

// Floppy Disk Commands
//There are more, but we only need these for now
enum FloppyCommands {}

impl FloppyCommands {
    pub const SPECIFY: u8 = 3; // Specify
    pub const WRITE: u8 = 5; // Write Data
    pub const READ: u8 = 6; // Read Data
    pub const RECALIBRATE: u8 = 7; // Recalibrate
    pub const INTERRUPT: u8 = 8; // Sense Interrupt
    pub const SEEK: u8 = 15; // Seek
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum FloppyMotor {
    ON = 5,
    OFF,
    WAITING,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum DriveType {
    None,
    Fdd360kb,
    Fdd1_2mb,
    Fdd720kb,
    Fdd1_44mb,
    Fdd2_88mb,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Drive {
    MASTER,
    SLAVE,
}

#[derive(Clone, Copy, Debug)]
struct FloppyDrive {
    driveType: DriveType,
    BASE_ADDR: u16,
}

#[derive(Debug)]
struct FloppyController {
    master: FloppyDrive,
    slave: FloppyDrive,
    selectedDrive: Option<FloppyDrive>,
    motorState: FloppyMotor,
}

impl DriveType {
    pub fn intToDriveType(int: u8) -> DriveType {
        match int {
            0x01 => DriveType::Fdd360kb,
            0x02 => DriveType::Fdd1_2mb,
            0x03 => DriveType::Fdd720kb,
            0x04 => DriveType::Fdd1_44mb,
            0x05 => DriveType::Fdd2_88mb,
            _ => DriveType::None,
        }
    }
}

impl FloppyDrive {
    fn new(typ: DriveType, baseAddr: u16) -> FloppyDrive {
        FloppyDrive {
            driveType: typ,
            BASE_ADDR: baseAddr,
        }
    }
}

impl FloppyController {
    fn new(masterType: DriveType, slaveType: DriveType) -> FloppyController {
        FloppyController {
            master: FloppyDrive::new(masterType, 0x3F0),
            slave: FloppyDrive::new(slaveType, 0x370),
            selectedDrive: None,
            motorState: FloppyMotor::OFF,
        }
    }

    fn selectDrive(&mut self, drive: Drive) {
        match drive {
            Drive::MASTER => self.selectedDrive = Some(self.master),
            Drive::SLAVE => self.selectedDrive = Some(self.slave),
        }
    }

    fn getSelected(&self) -> Result<FloppyDrive, &str> {
        if let Some(drive) = self.selectedDrive {
            return Ok(drive);
        }

        return Err("No drive selected!");
    }

    async unsafe fn writeCmd(&self, cmd: u8) -> Result<(), &str> {
        unsafe {
            if self.selectedDrive.is_none() {
                return Err("No drive selected!");
            }

            let addr = self.getSelected().unwrap().BASE_ADDR;

            for _ in 0..600 {
                waitTicks(2).await; // 10ms

                if 0x80 & in8(addr + FloppyRegisters::MSR) != 0 {
                    return Ok(out8(addr + FloppyRegisters::FIFO, cmd));
                }
            }

            return Err("Floppy write command error: timeout");
        }
    }

    async unsafe fn readData(&self) -> Result<u8, &str> {
        unsafe {
            if self.selectedDrive.is_none() {
                return Err("No drive selected!");
            }

            let addr = self.getSelected().unwrap().BASE_ADDR;

            for _ in 0..600 {
                waitTicks(2).await; // 10ms

                if 0x80 & in8(addr + FloppyRegisters::MSR) != 0 {
                    return Ok(in8(addr + FloppyRegisters::FIFO));
                }
            }

            return Err("Floppy read command error: timeout");
        }
    }

    unsafe fn checkInt(&self) -> (u8, u8) {
        unsafe {
            let addr = self.getSelected().unwrap().BASE_ADDR;

            out8(addr, FloppyCommands::INTERRUPT);

            let st0 = in8(addr);
            let cyl = in8(addr);

            return (st0, cyl);
        }
    }

    // move to cylinder 0
    async unsafe fn calibrate(&mut self) -> Result<(), Box<str>> {
        unsafe {
            self.motor(FloppyMotor::ON).await;

            for _ in 0..10 {
                if let Err(e) = self.writeCmd(FloppyCommands::RECALIBRATE).await {
                    return Err(alloc::format!("Floppy recalibrate error: {}", e).into_boxed_str());
                }

                if let Err(e) = self.writeCmd(0).await {
                    return Err(alloc::format!("Floppy recalibrate error: {}", e).into_boxed_str());
                }

                // TODO: wait for irq 6

                let (st0, cyl) = self.checkInt();

                if st0 & 0xC0 != 0 {
                    let status = match st0 >> 6 {
                        0x00 => "normal",
                        0x01 => "error",
                        0x02 => "invalid",
                        0x03 => "drive not ready",
                        _ => "unknown",
                    };

                    log::trace!("Floppy recalibrate: status = {}", status);
                }

                if cyl == 0 {
                    self.motor(FloppyMotor::OFF).await;
                    return Ok(());
                }
            }

            self.motor(FloppyMotor::OFF).await;
            return Err(
                alloc::format!("Floppy recalibrate error: 10 retries exhausted").into_boxed_str(),
            );
        }
    }

    async unsafe fn reset(&mut self) -> Result<(), Box<str>> {
        unsafe {
            let addr = self.getSelected().unwrap().BASE_ADDR;

            out8(addr + FloppyRegisters::DOR, 0x00); // disable controller
            out8(addr + FloppyRegisters::DOR, 0x0C); // enable controller

            // TODO: wait for irq 6

            self.checkInt();

            // set transfer speed
            match self.getSelected().unwrap().driveType {
                DriveType::Fdd360kb => {
                    out8(addr + FloppyRegisters::CCR, 0x02);
                }
                DriveType::Fdd720kb => {
                    out8(addr + FloppyRegisters::CCR, 0x01);
                }
                DriveType::Fdd1_2mb | DriveType::Fdd1_44mb => {
                    out8(addr + FloppyRegisters::CCR, 0x00);
                }
                DriveType::Fdd2_88mb => {
                    out8(addr + FloppyRegisters::CCR, 0x03);
                }
                DriveType::None => {
                    return Err("Must select a drive before resetting!"
                        .to_owned()
                        .into_boxed_str());
                }
            }

            out8(addr, FloppyCommands::SPECIFY);
            out8(addr, 0xDF); // StepRate = 3ms, UnloadTime = 240ms
            out8(addr, 0x02); // LoadTime = 16ms, NoDMA = 0

            return self.calibrate().await;
        }
    }

    async unsafe fn motor(&mut self, state: FloppyMotor) {
        unsafe {
            let addr = self.getSelected().unwrap().BASE_ADDR;

            if state == FloppyMotor::ON {
                if self.motorState == FloppyMotor::OFF {
                    out8(addr + FloppyRegisters::DOR, 0x1C);
                    waitTicks(10).await; // 10ms
                }
                self.motorState = FloppyMotor::ON;
            } else {
                out8(addr + FloppyRegisters::DOR, 0x0C);
                self.motorState = FloppyMotor::OFF;
                // if self.motorState == FloppyMotor::WAITING {
                //     println!("Floppy motor: already waiting");
                // }
                // ticks = 300ms
                // self.motorState = FloppyMotor::WAITING;
            }
        }
    }

    async unsafe fn seek(&mut self, cylinder: u8, head: u8) -> Result<(), &str> {
        unsafe {
            let addr = self.getSelected().unwrap().BASE_ADDR;

            self.motor(FloppyMotor::ON).await;

            for _ in 0..10 {
                out8(addr, FloppyCommands::SEEK);
                out8(addr, head << 2);
                out8(addr, cylinder);

                // wait for IRQ 6

                let (st0, cyl) = self.checkInt();
                if st0 & 0xC0 != 0 {
                    let status = match st0 >> 6 {
                        0x00 => "normal",
                        0x01 => "error",
                        0x02 => "invalid",
                        0x03 => "drive not ready",
                        _ => "unknown",
                    };

                    log::trace!("Floppy recalibrate: status = {}", status);
                }

                if cyl == cylinder {
                    self.motor(FloppyMotor::OFF).await;
                    return Ok(());
                }
            }

            self.motor(FloppyMotor::OFF).await;
            return Err("Floppy seek error: 10 retries exhausted");
        }
    }

    // unsafe fn killMotor(&mut self) {
    //     let addr = self.getSelected().unwrap().BASE_ADDR;
    //     out8(addr + FloppyRegisters::DOR, 0x0C);
    //     self.motorState = FloppyMotor::OFF;
    // }
}

pub async fn detectFloppyDrives() {
    let driveTypes: u8;
    unsafe {
        out8(0x70, (1 << 7) | 0x10);
        // wait 10ms
        RTC::waitTicks(1).await;
        driveTypes = in8(0x71);
    };

    // bits 0 to 3 are the slave floppy type, bits 4 to 7 are the master floppy type
    let mut floppyController = FloppyController::new(
        DriveType::intToDriveType(driveTypes >> 4),
        DriveType::intToDriveType(driveTypes & 0x0F),
    );

    log::info!("Floppy drives: {:#?}", floppyController);
    log::info!("retVal: {:#x}", driveTypes);

    floppyController.selectDrive(Drive::MASTER);
    unsafe {
        if let Err(e) = floppyController.reset().await {
            log::error!("Floppy reset error: {}", e);
        }

        if let Err(e) = floppyController.seek(0, 0).await {
            log::error!("Floppy seek error: {}", e);
        }

        let data = floppyController.readData().await;
        log::info!("Floppy data: {:?}", data);
    }
}
