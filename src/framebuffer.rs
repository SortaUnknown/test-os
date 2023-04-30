use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, RasterizedChar, get_raster, get_raster_width};
use spin::{Mutex, Lazy};
use crate::FRAME_BUFFER;
use crate::VEC;
use core::fmt::Write;

const LINE_SPACING: usize = 2;
const LETTER_SPACING: usize = 0;
const BORDER_PADDING: usize = 1;

const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;
const FONT_WEIGHT: FontWeight = FontWeight::Regular;
const CHAR_RASTER_WIDTH: usize = get_raster_width(FONT_WEIGHT, CHAR_RASTER_HEIGHT);
const BACKUP_CHAR: char = '�';

static WRITER: Lazy<Mutex<FrameBufferWriter>> = Lazy::new(||
{
    let buf_ref = FRAME_BUFFER.get().expect("ass").buffer();
    let buf_mut = unsafe{VEC.as_mut_slice()};
    copy_slice(buf_mut, buf_ref);
    let info = FRAME_BUFFER.get().expect("ass").info();
    Mutex::new(FrameBufferWriter::new(buf_mut, info))
});

fn copy_slice(dst: &mut [u8], src: &[u8]) -> usize
{
    let mut c = 0;
    for (d, s) in dst.iter_mut().zip(src.iter())
    {
        *d = *s;
        c += 1;
    }
    c
}

fn get(c: char) -> Option<RasterizedChar>
{
    get_raster(c, FONT_WEIGHT, CHAR_RASTER_HEIGHT)
}

fn get_char_raster(c: char) -> RasterizedChar
{
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("should get raster of backup char"))
}

pub struct FrameBufferWriter
{
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize
}

impl FrameBufferWriter
{
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self
    {
        let mut logger = FrameBufferWriter{framebuffer, info, x_pos: 0, y_pos: 0};
        logger.clear();
        logger
    }

    pub fn clear(&mut self)
    {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.framebuffer.fill(0);
    }

    fn carriage_return(&mut self)
    {
        self.x_pos = BORDER_PADDING;
    }

    fn newline(&mut self)
    {
        self.y_pos += CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return();
    }

    fn width(&self) -> usize
    {
        self.info.width
    }

    fn height(&self) -> usize
    {
        self.info.height
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8)
    {
        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format
        {
            PixelFormat::Rgb => [intensity, intensity, intensity / 2, 0],
            PixelFormat::Bgr => [intensity / 2, intensity, intensity, 0],
            PixelFormat::U8 => [if intensity > 200 {0xf} else {0}, 0, 0, 0],
            other =>
            {
                self.info.pixel_format = PixelFormat::Rgb;
                panic!("pixel format {:?} not supported in logger", other);
            }
        };
        let bpp = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bpp;
        self.framebuffer[byte_offset..(byte_offset + bpp)].copy_from_slice(&color[..bpp]);
        unsafe{core::ptr::read_volatile(&self.framebuffer[byte_offset]);} //why do we do this
    }

    fn write_rendered_char(&mut self, rendered_char: RasterizedChar)
    {
        for (y, row) in rendered_char.raster().iter().enumerate()
        {
            for (x, byte) in row.iter().enumerate()
            {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn write_char(&mut self, c: char)
    {
        match c
        {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c =>
            {
                if self.x_pos + CHAR_RASTER_WIDTH >= self.width() {self.newline();}
                if CHAR_RASTER_HEIGHT.val() + BORDER_PADDING >= self.height() {self.clear();}
                self.write_rendered_char(get_char_raster(c));
            }
        }
    }
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl core::fmt::Write for FrameBufferWriter
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result
    {
        for c in s.chars()
        {
            self.write_char(c);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! print
{
    ($($arg:tt)*) => ($crate::framebuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println
{
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments)
{
    x86_64::instructions::interrupts::without_interrupts(|| {WRITER.lock().write_fmt(args).unwrap();});
}