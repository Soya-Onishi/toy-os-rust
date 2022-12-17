extern crate alloc;

use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use alloc::vec;
use alloc::vec::Vec;

struct Vector{ x: usize, y: usize }

pub struct Screen {
  frame_buffer: &'static mut [u8],
  info: FrameBufferInfo,
  cursor: Vector,
  text_size: Vector,
  text_buffer: Vec<u8>, 
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
   
    let text_size = Vector{ x: info.width / 8, y: info.height / 16 };
    let buf = vec![0; text_size.x * text_size.y];

    let cursor = Vector{ x: 0, y: 0 };
    Screen{ frame_buffer, info, text_buffer: buf, text_size, cursor }
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
    let Vector { x, y } = self.cursor;
    self.text_buffer[x + y * self.text_size.x] = c; 

    self.cursor = if c == '\n' as u8 {
      if y >= self.text_size.y - 1 {
        self.newline();
      }

      Vector{ x: 0, y: core::cmp::min(y + 1, self.text_size.y - 1) }
    } else {
      self.draw_char(x, y, c);

      Vector{ x: core::cmp::min(x + 1, self.text_size.x - 1), y }
    };
  }

  fn draw_char(&mut self, x: usize, y: usize, c: u8) {
    let offset = c as usize * 16;
    let font = &FONT_BINARY[offset..(offset + 16)];
    let screen_pos = (x * 8, y * 16 * self.info.stride);

    let black = [0; 8];
    let white = [255; 8];
    
    for row in 0..16 {
      let screen_y = screen_pos.1 + row * self.info.stride;
      for col in 0..8 {
        let screen_x = screen_pos.0 + col;
        let color = if font[row] & (1 << (7 - col)) == 0 {
          black
        } else {
          white
        };
 
        let offset = self.info.bytes_per_pixel * (screen_y + screen_x);
        for idx in 0..self.info.bytes_per_pixel {
          self.frame_buffer[offset + idx] = color[idx];
        }
      }
    }
  }

  fn newline(&mut self) {
    for y in 16..self.info.height {
      let text_y = y / 16;
      let text_offset = text_y * self.text_size.x;
      let text_line_len = self.text_size.x;
      let src_text_tail_x = self.text_buffer[text_offset..(text_offset + text_line_len)]
        .iter()
        .position(|r| *r == 0)
        .unwrap_or(text_line_len - 1);
      let dst_text_tail_x = self.text_buffer[text_offset..(text_offset + text_line_len)]
        .iter()
        .position(|r| *r == 0)
        .unwrap_or(text_line_len - 1);
      
      // let src_screen_tail_x = src_text_tail_x * self.info.bytes_per_pixel * 8;
      // let dst_screen_tail_x = dst_text_tail_x * self.info.bytes_per_pixel * 8;
      let src_screen_tail_x = self.info.stride * self.info.bytes_per_pixel;
      let dst_screen_tail_x = self.info.stride * self.info.bytes_per_pixel;

      let src_y = y * self.info.stride * self.info.bytes_per_pixel;
      let dst_y = (y - 16) * self.info.stride * self.info.bytes_per_pixel;

      for idx in 0..dst_screen_tail_x {
        self.frame_buffer[dst_y + idx] = 0;
      }

      for x in 0..src_screen_tail_x {
        let src_idx = src_y + x;
        let dst_idx = dst_y + x; 
       
        self.frame_buffer[dst_idx] = self.frame_buffer[src_idx];
      }
    }

    let tail_y = (self.text_size.y - 1) * 16 * self.info.stride * self.info.bytes_per_pixel;
    unsafe { core::ptr::write_bytes(self.frame_buffer.as_mut_ptr().offset(tail_y as isize), 0, self.info.stride * self.info.bytes_per_pixel * 16) };

    let last_line = self.text_size.x * (self.text_size.y - 1);
    self.text_buffer.copy_within(self.text_size.x.., 0);
    self.text_buffer[last_line..].fill(0);
  }
}

impl Color {
  pub fn black() -> Color {
    Color{ r: 0, g: 0, b: 0 }
  }
}