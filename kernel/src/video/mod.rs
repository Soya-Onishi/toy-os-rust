extern crate alloc;

use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use alloc::vec;
use alloc::vec::Vec;

pub struct Screen {
  frame_buffer: &'static mut [u8],
  info: FrameBufferInfo,
  cursor: (usize, usize),
  text_size: (usize, usize),
  text_buffer: Vec<Vec<u8>>, 
}

#[derive(Clone, Copy)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8
}

static FONT_BINARY: &[u8; 127 * 16] = core::include_bytes!(core::env!("RAW_FONT_PATH"));

impl Screen {
  pub fn new(buffer: &'static mut FrameBuffer) -> Screen {
    let info = buffer.info();
    let frame_buffer = buffer.buffer_mut();
    
    let text_buf_size = (info.width / 8, info.height / 16);
    let row = vec![0; text_buf_size.0 / 8];
    let buf = vec![row.clone(); info.height / 16];

    Screen{ frame_buffer, info, text_buffer: buf, text_size: text_buf_size, cursor: (0, 0) }
  }

  pub fn clear(&mut self, color: Color) {
    let height = self.info.height;
    let width = self.info.stride;
    let pixel_size = self.info.bytes_per_pixel;
    let mut pixel = [0; 8];

    match self.info.pixel_format {
      PixelFormat::Rgb => {
        pixel[0] = color.r;
        pixel[1] = color.g;
        pixel[2] = color.b;
      } 
      PixelFormat::Bgr => {
        pixel[0] = color.b;
        pixel[1] = color.g;
        pixel[2] = color.r;
      }
      PixelFormat::U8  => {
        let r = color.r as u16;
        let g = color.g as u16;
        let b = color.b as u16;
        let mono = (r + g + b) / 3;
        for p in pixel.iter_mut() {
          *p = mono as u8;
        }
      }
      _ => panic!("not supported PixelFormat. abort."),
    };

    for dst in self.frame_buffer.chunks_mut(pixel_size) {
      dst.copy_from_slice(&pixel[0..pixel_size]);
    }
    for i in 0..height * width {
      let start = i * pixel_size;
      let end = (i + 1) * pixel_size;
      self.frame_buffer[start..end].copy_from_slice(&pixel[0..pixel_size])
    } 
  }

  pub fn write_str(&mut self, s: &[u8]) {
    for &c in s.iter() {
      self.write_char(c);
    }   
  }

  pub fn write_char(&mut self, c: u8) {
    let (x, y) = self.cursor;
    self.text_buffer[y][x] = c; 
    self.draw_char(x, y, c);

    let (x, y) = &mut self.cursor;
    if c == '\n' as u8 {
      *y += 1;
      *x = 0; 
    } else {
      *x = core::cmp::min(*x + 1, self.text_size.0)
    }
  }

  fn draw_char(&mut self, x: usize, y: usize, c: u8) {
    let offset = c as usize * 16;
    let font = &FONT_BINARY[offset..(offset + 16)];
    let screen_pos = (x * 8, y * 16);

    let black = [0; 8];
    let white = [255; 8];
    
    for row in 0..16 {
      let y = screen_pos.1 + row * self.info.stride; 
      for col in 0..8 {
        let x = screen_pos.0 + col;
        let color = if font[row] & (1 << (7 - col)) == 0 {
          black
        } else {
          white
        };

        for idx in 0..self.info.bytes_per_pixel {
          let offset = self.info.bytes_per_pixel * (y + x);
          self.frame_buffer[offset + idx] = color[idx];
        }
      }
    }
  }
}

impl Color {
  pub fn black() -> Color {
    Color{ r: 0, g: 0, b: 0 }
  }

  pub fn white() -> Color {
    Color{ r: 255, g: 255, b: 255 }
  }
}