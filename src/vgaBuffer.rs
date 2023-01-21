#![allow(dead_code)]

use lazy_static::lazy_static;
use volatile::Volatile;
use spin::Mutex;
use core::fmt;
use core::ops::{Deref, DerefMut};


lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        columnPosition: 0,
        colourCode: ColourCode::new(Colour::Yellow, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Colour {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
struct ColourCode(u8);

impl ColourCode {
    fn new(foreground: Colour, background: Colour) -> ColourCode {
        ColourCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    asciiChar: u8,
    colourCode: ColourCode,
}

impl Deref for ScreenChar {
    type Target = ScreenChar;

    fn deref(&self) -> &Self::Target {
        &self
    }
}

impl DerefMut for ScreenChar {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    columnPosition: usize,
    colourCode: ColourCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn writeByte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newLine(),
            byte => {
                if self.columnPosition >= BUFFER_WIDTH {
                    self.newLine();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.columnPosition;

                let colourCode = self.colourCode;
                self.buffer.chars[row][col].write(ScreenChar {
                    asciiChar: byte,
                    colourCode,
                });
                self.columnPosition += 1;
            }
        }
    }

    pub fn writeString(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.writeByte(byte),
                _ => self.writeByte(0xfe),
            }
        }
    }

    fn newLine(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(char);
            }
        }
        self.clearRow(BUFFER_HEIGHT - 1);
        self.columnPosition = 0;
    }

    fn clearRow(&mut self, row: usize) {
        let blank = ScreenChar {
            asciiChar: b' ',
            colourCode: self.colourCode,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn clearColumn(&mut self, column: usize) {
        let blank = ScreenChar {
            asciiChar: b' ',
            colourCode: self.colourCode,
        };
        for row in 0..BUFFER_HEIGHT {
            self.buffer.chars[row][column].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.writeString(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vgaBuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}


#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}


#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screenChar = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screenChar.asciiChar), c);
        }
    });
}
