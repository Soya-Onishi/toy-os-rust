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

  pub fn clear(&mut self) {
    self.frame_buffer.fill(0); 
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
    for text_y in 1..self.text_size.y {
      let src_text_offset = text_y * self.text_size.x;
      let dst_text_offset = (text_y - 1) * self.text_size.x;
      let src_pixel_offset = text_y       * 16 * self.info.stride * self.info.bytes_per_pixel;
      let dst_pixel_offset = (text_y - 1) * 16 * self.info.stride * self.info.bytes_per_pixel;

      let src_text_tail_x = self.text_buffer[src_text_offset..(src_text_offset + self.text_size.x)]
        .iter()
        .position(|r| *r == 0)
        .unwrap_or(self.text_size.x - 1);
      let dst_text_tail_x = self.text_buffer[dst_text_offset..(dst_text_offset + self.text_size.x)]
        .iter()
        .position(|r| *r == 0)
        .unwrap_or(self.text_size.x - 1);

      let src_screen_tail_x = src_text_tail_x * self.info.bytes_per_pixel * 8;
      let dst_screen_tail_x = dst_text_tail_x * self.info.bytes_per_pixel * 8;

      for idx in 0..16 {
        let dst_y = dst_pixel_offset + idx * self.info.stride * self.info.bytes_per_pixel;
        let src_y = src_pixel_offset + idx * self.info.stride * self.info.bytes_per_pixel; 

        self.frame_buffer[dst_y..(dst_y + dst_screen_tail_x)].fill(0);
        self.frame_buffer.copy_within(src_y..(src_y + src_screen_tail_x), dst_y);
      } 
    }
          
    let last_line = self.text_size.x * (self.text_size.y - 1);    
    let last_text_line_y = (self.text_size.y - 1) * 16 * self.info.stride * self.info.bytes_per_pixel; 
    let last_line_text_len = self.text_buffer[last_line..].iter().position(|r| *r == 0).unwrap_or(self.text_size.x - 1);
    let last_line_text_width = last_line_text_len * 8 * self.info.bytes_per_pixel;
    for offset in 0..16 {
      let start = last_text_line_y + offset * self.info.stride * self.info.bytes_per_pixel;
      let end = start + last_line_text_width;

      self.frame_buffer[start..end].fill(0);
    }

    self.text_buffer.copy_within(self.text_size.x.., 0);
    self.text_buffer[last_line..].fill(0);
  }
}

impl Color {
  pub fn black() -> Color {
    Color{ r: 0, g: 0, b: 0 }
  }
}