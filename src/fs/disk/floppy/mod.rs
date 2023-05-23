use crate::println;
use crate::kernel::{RTC, binIO::{in8, out8}};

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

enum FloppyRegisters {}

impl FloppyRegisters {
    pub const SRA:  u16  = 0x3F0;   // Status Register A; R/O
    pub const SRB:  u16  = 0x3F1;   // Status Register B; R/O
    pub const DOR:  u16  = 0x3F2;   // Digital Output Register
    pub const TDR:  u16  = 0x3F3;   // Tape Drive Register
    pub const MSR:  u16  = 0x3F4;   // Master Status Register; R/O
    pub const DSR:  u16  = 0x3F4;   // Data Rate Select Register; W/O
    pub const FIFO: u16  = 0x3F5;   // data FIFO
    pub const DIR:  u16  = 0x3F7;   // Digital Input Register; R/O
    pub const CCR:  u16  = 0x3F7;   // Configuration Control Register; W/O
}


// Floppy Disk Commands
//There are more, but we only need these for now
enum FloppyCommands {
    SPECIFY = 3,       // Specify
    WRITE = 5,         // Write Data
    READ = 6,          // Read Data
    RECALIBRATE = 7,   // Recalibrate
    INTERRUPT = 8,     // Sense Interrupt
    SEEK = 15,         // Seek
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

#[derive(Clone, Copy, Debug)]
struct FloppyDrive {
    driveType: DriveType,
    BASE_ADDR: u16,
}

#[derive(Debug)]
struct FloppyController {
    master: FloppyDrive,
    slave: FloppyDrive,
    selectedDrive: Option<u8>,
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
        }
    }

    fn selectDrive(&mut self, driveID: u8) {
        self.selectedDrive = Some(driveID);
    }

    fn getSelected(&self) -> FloppyDrive {
        if self.selectedDrive.unwrap() == 1 {
            return self.master;
        } else if self.selectedDrive.unwrap() == 2 {
            return self.slave;
        }

        panic!("Invalid drive ID {}, (1:master, 2:slave)", self.selectedDrive.unwrap());
    }

    unsafe fn writeCmd(&self, cmd: char) {
        if let None = self.selectedDrive {
            panic!("No drive selected!");
        }
        let addr = self.getSelected().BASE_ADDR;

        // sleep loop here
        if (0x80 & in8(addr + FloppyRegisters::MSR as u16) as u16) > 0 {
            return out8(addr + FloppyRegisters::FIFO as u16, cmd as u8);
        }

        panic!("Floppy write command error: timeout");
    }

    unsafe fn readData(&self) -> char {
        if let None = self.selectedDrive {
            panic!("No drive selected!");
        }

        let addr = self.getSelected().BASE_ADDR;

        // sleep loop here
        if (0x80 & in8(addr + FloppyRegisters::MSR as u16) as u16) > 0 {
            return in8(addr + FloppyRegisters::FIFO as u16) as char
        }

        panic!("Floppy read error: timeout");
    }
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

    println!("Floppy drives: {:#?}", floppyController);
    println!("retVal: {:#x}", driveTypes);
}
