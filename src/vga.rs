use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<VGADriver> = Mutex::new(VGADriver {
        pos: 0,
        color: VGAColor::from_colors(Color::Yellow, Color::Black),
        buf: unsafe { &mut *(0xb8000 as *mut VGABuffer) },
    });
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
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

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct VGAColor {
    color: u8,
}

impl VGAColor {
    pub fn from_colors(foreground: Color, background: Color) -> VGAColor {
        VGAColor {
            color: ((background as u8) << 4) | (foreground as u8),
        }
    }
}

pub struct VGADriver {
    buf: &'static mut VGABuffer,
    color: VGAColor,
    pos: usize,
}

#[repr(transparent)]
struct VGABuffer {
    chars: [[VGAChar; 80]; 25],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct VGAChar {
    char: u8,
    color: VGAColor,
}

impl VGADriver {
    fn write_byte(&mut self, byte: u8) {
        if self.pos == 80 {
            self.new_line();
        }
        match byte {
            b'\n' => self.new_line(),
            c => {
                if self.pos > 80 {
                    self.new_line();
                }
                unsafe {
                    core::ptr::write_volatile(
                        &mut self.buf.chars[24][self.pos],
                        VGAChar {
                            char: c,
                            color: self.color,
                        },
                    );
                }
                self.pos += 1;
            }
        }
    }

    pub fn new_line(&mut self) {
        for i in 1..25 {
            unsafe {
                core::ptr::write_volatile(&mut self.buf.chars[i - 1], self.buf.chars[i]);
            }
        }
        self.clear_line(24);
    }

    fn clear_line(&mut self, line: usize) {
        unsafe {
            core::ptr::write_volatile(
                &mut self.buf.chars[line],
                [VGAChar {
                    char: b' ',
                    color: self.color,
                }; 80],
            );
        }
        self.pos = 0;
    }

    fn write_string(&mut self, s: &str) {
        for &c in s.as_bytes() {
            match c {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(c),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn switch_color(&mut self, c: VGAColor) {
        self.color = c;
    }
}

impl Write for VGADriver {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

//tests
#[cfg(test)]
mod test {
    use super::*;
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
        let s = "Some test string that fits on a single line";
        println!("{}", s);
        for (i, c) in s.chars().enumerate() {
            let screen_char = WRITER.lock().buf.chars[25 - 2][i];
            assert_eq!(char::from(screen_char.char), c);
        }
    }
}
