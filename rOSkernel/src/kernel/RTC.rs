use core::ops::Add;

use core::cmp::Ordering;
use core::sync::atomic::Ordering as AtomicOrdering;
use spin::Mutex;

use crate::kernel::{interrupts::TICK_COUNTER, binIO};


static TIME: Mutex<Time> = Mutex::new(Time::new());
static MONTHS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

//#[derive(Eq, PartialEq, PartialOrd)]
struct DateTime {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: u8,
    century: u8,
}

struct Time {
    prevSecond: u8,
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: u8,
    century: u8,
}

impl DateTime {
    const fn new(second: u8, minute: u8, hour: u8, day: u8, month: u8, year: u8) -> DateTime {
        DateTime {
            second,
            minute,
            hour,
            day,
            month,
            year,
            century: 0,
        }
    }
}

impl Add<u16> for DateTime {
    type Output = DateTime;

    fn add(self, rhs: u16) -> DateTime {
        let hoursToAdd: u8 = (rhs / 3600) as u8;
        let minsToAdd: u8 = ((rhs % 3600) / 60) as u8;
        let secondsToAdd: u8 = (rhs % 60) as u8;

        let mut secs = self.second + secondsToAdd;
        let mut mins = self.minute + minsToAdd;
        let mut hours = self.hour + hoursToAdd;
        let mut days: u8 = self.day;
        let mut months: u8 = self.month;
        let mut years: u8 = self.year;

        if secs >= 60 {
            mins += 1;
            secs = self.second + secondsToAdd - 60;
        }

        if mins >= 60 {
            hours += 1;
            mins = self.minute + minsToAdd - 60;
        }

        if hours >= 24 {
            days += 1;
            hours = self.hour + hoursToAdd - 24;
        }

        if days > MONTHS[months as usize + 1] {
            months += 1;
            days -= MONTHS[months as usize];
        }

        if months > 12 {
            years += 1;
            months -= 12;
        }

        DateTime {
            second: secs,
            minute: mins,
            hour: hours,
            day: days,
            month: months,
            year: years,
            century: 2,
        }
    }
}

impl PartialEq for DateTime {
    fn eq(&self, other: &Self) -> bool {
        (
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second
        ) == (
            other.year,
            other.month,
            other.day,
            other.hour,
            other.minute,
            other.second
        )
    }
}

impl Eq for DateTime {}

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &DateTime) -> Option<Ordering> {
        Some((
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second
        ).cmp(&(
            other.year,
            other.month,
            other.day,
            other.hour,
            other.minute,
            other.second
        )))
    }
}

impl Ord for DateTime {
    fn cmp(&self, other: &DateTime) -> Ordering {
        (
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second
        ).cmp(&(
            other.year,
            other.month,
            other.day,
            other.hour,
            other.minute,
            other.second
        ))
    }
}

impl Time {
    const fn new() -> Time {
        Time {
            prevSecond: 0,
            second: 0,
            minute: 0,
            hour: 0,
            day: 0,
            month: 0,
            year: 0,
            century: 0,
        }
    }

    fn update(&mut self) {
        unsafe {
          // if updateInProgress() { return }
            self.prevSecond = self.second;

            self.second = getRTC(0x00);
            self.minute = getRTC(0x02);
            self.hour = getRTC(0x04);
            self.day = getRTC(0x07);
            self.month = getRTC(0x08);
            self.year = getRTC(0x09);
            
        }
    }

    pub fn now(&self) -> DateTime {
        DateTime::new(
            self.second,
            self.minute,
            self.hour,
            self.day,
            self.month,
            self.year
        )
    }
}

pub async fn waitSeconds(seconds: u32) {
    log::info!("{} ticks", seconds * 20);
    waitTicks(seconds * 20).await;
}

pub async fn waitTicks(ticks: u32){
    let now = TICK_COUNTER.load(AtomicOrdering::Relaxed);
    let target = now + ticks;
    
    loop {
        let now = TICK_COUNTER.load(AtomicOrdering::Relaxed);
        if now >= target {
            break;
        }
    }
}


unsafe fn updateInProgress() -> bool { unsafe {
    // disable NMI; get status register A
    binIO::out8(0x70, (0x1 << 7) | 0x0A);
    return binIO::in8(0x71) & 0x80 == 0;
}}

unsafe fn getRTC(port: u8) -> u8 { unsafe {
    // check format
    binIO::out8(0x70, (0x1 << 7) | 0x0B);
    let format = binIO::in8(0x71);

    
    binIO::out8(0x70, (0x1 << 7) | port);
    let value = binIO::in8(0x71);

    // TODO: check 12/24 hr

    // BCD
    if format & 0x04 == 0 {
        return ((value & 0xF0) >> 1) + ((value & 0xF0) >> 3) + (value & 0xf);
    }

    // binary
    return value;
}}

pub fn initRTC() {
    log::info!("RTC: Initializing...");
    unsafe {
        binIO::out8(0x70, 0x8B);
        let prev = binIO::in8(0x71);
        binIO::out8(0x70, 0x8B);
        binIO::out8(0x71, prev | 0x40);
    }
}

pub unsafe fn readRTC() {
    TIME.lock().update();
    let time = TIME.lock();
    
    if time.second == time.prevSecond { return }

    //    century = binIO::in8(ADDRESS);  preferably read from ACPI
    // println!("{}", getDateTime("%h:%m:%s %D-%M-%Y"));
    log::info!("{}:{}:{} {}-{}-{}", time.hour, time.minute, time.second, time.day, time.month, time.year);
}


// pub fn getDateTime(format: &str) -> String {
//     let mut result = String::new();
//     let mut i = 0;
    
//     while i < format.len() {
//         let mut c = format.chars().nth(i).unwrap();
//         if c == '%' {
//             i += 1;
//             c = format.chars().nth(i).unwrap();
//             match c {
//                 's' => result += unsafe { SECOND.to_string().as_str() },
//                 'm' => result += unsafe { MINUTE.to_string().as_str() },
//                 'h' => result += unsafe { HOUR.to_string().as_str() },
//                 'D' => result += unsafe { DAY.to_string().as_str() },
//                 'M' => result += unsafe { MONTH.to_string().as_str() },
//                 'Y' => result += unsafe { YEAR.to_string().as_str() },
//                 _ => result.push(c),
//             }
//         } else {
//             result.push(c);
//         }
//         i += 1;
//     }
//     result
// }
