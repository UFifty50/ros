use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use core::ops::Add;
use crate::util::OnceInit::OnceInit;
use noto_sans_mono_bitmap::{
    FontWeight, RasterHeight, RasterizedChar, get_raster, get_raster_width,
};

const LINE_SPACING: usize = 2;
const LETTER_SPACING: usize = 0;
const BORDER_PADDING: usize = 1;
const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;
const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);
const BACKUP_CHAR: char = '�';
const FONT_WEIGHT: FontWeight = FontWeight::Regular;

pub static FRAMEBUFFER: OnceInit<FrameBufferEditor> = OnceInit::new();

fn getRaster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(c, FONT_WEIGHT, CHAR_RASTER_HEIGHT)
    }

    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}

/*
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    FRAMEBUFFER.get_mut().unwrap().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kernel::framebuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Add for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        Position {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelColour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct FrameBufferEditor {
    framebuffer: &'static mut FrameBuffer,
    info: FrameBufferInfo,
    position: Position
}

impl FrameBufferEditor {
    pub fn new(framebuffer: &'static mut FrameBuffer, info: FrameBufferInfo) -> Self {
        let mut fbWriter = Self {
            framebuffer,
            info,
            position: Position { x: 0, y: 0 }
        };
        fbWriter.clear();
        fbWriter
    }

    pub fn clear(&mut self) {
        self.position = Position { x: BORDER_PADDING, y: BORDER_PADDING };
        self.framebuffer.buffer_mut().iter_mut().for_each(|byte| *byte = 0);
    }

    pub fn width(&self) -> usize {
        self.info.width
    }

    pub fn height(&self) -> usize {
        self.info.height
    }
    
    pub fn writePixel(&mut self, position: Position, color: PixelColour) {
        // calculate offset to first byte of pixel
        let byte_offset = {
            // use stride to calculate pixel offset of target line
            let line_offset = position.y * self.info.stride;
            // add x position to get the absolute pixel offset in buffer
            let pixel_offset = line_offset + position.x;
            // convert to byte offset
            pixel_offset * self.info.bytes_per_pixel
        };

        // set pixel based on color format
        let pixel_buffer = &mut self.framebuffer.buffer_mut()[byte_offset..];
        match self.info.pixel_format {
            PixelFormat::Rgb => {
                pixel_buffer[0] = color.r;
                pixel_buffer[1] = color.g;
                pixel_buffer[2] = color.b;
            }
            PixelFormat::Bgr => {
                pixel_buffer[0] = color.b;
                pixel_buffer[1] = color.g;
                pixel_buffer[2] = color.r;
            }
            PixelFormat::U8 => {
                // use a simple average-based grayscale transform
                let gray = color.r / 3 + color.g / 3 + color.b / 3;
                pixel_buffer[0] = gray;
            }
            other => panic!("unknown pixel format {other:?}"),
        }
    }
    
    /*
    fn newline(&mut self) {
        self.carriageReturn();
        self.lineFeed();
    }

    fn lineFeed(&mut self) {
        self.position.y += CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
    }

    fn carriageReturn(&mut self) {
        self.position.x = BORDER_PADDING;
    }
    
    fn writeChar(&mut self, c: char) {
        match c {
            '\n' => self.lineFeed(),
            '\r' => self.carriageReturn(),
            c => {
                if self.position.x + CHAR_RASTER_WIDTH >= self.width() {
                    self.newline();
                }

                if self.position.y + CHAR_RASTER_HEIGHT.val() + BORDER_PADDING >= self.height() {
                    // Scroll up
                    let scroll_offset = CHAR_RASTER_HEIGHT.val() + LINE_SPACING + BORDER_PADDING;
                    let scroll_size = self.info.stride * (self.height() - scroll_offset);
                    let copy_size =
                        self.info.stride * (self.height() - scroll_offset - BORDER_PADDING);
                    let copy_src = unsafe {
                        self.framebuffer.buffer_mut()
                            .as_ptr()
                            .add(scroll_offset * self.info.stride)
                    };
                    let copy_dst = self.framebuffer.buffer_mut().as_mut_ptr();
                    unsafe {
                        ptr::copy(copy_src, copy_dst, copy_size);
                        ptr::write_bytes(copy_dst.add(copy_size), 0, scroll_size - copy_size);
                    }
                    self.position.y -= scroll_offset;
                }
                self.renderChar(getRaster(c));
            }
        }
    }

    fn renderChar(&mut self, renderedChar: RasterizedChar) {
        for (y, row) in renderedChar.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.writePixel(self.position + Position {x, y}, PixelColour{r: *byte, g: *byte, b: *byte});
            }
        }
        self.position.x += renderedChar.width() + LETTER_SPACING;
    }
*/
}

/*
unsafe impl Send for FrameBufferEditor {}
unsafe impl Sync for FrameBufferEditor {}

impl core::fmt::Write for FrameBufferEditor {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.writeChar(c);
        }
        Ok(())
    }
}
*/
