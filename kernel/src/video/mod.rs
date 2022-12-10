use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};

pub struct Screen {
  frame_buffer: &'static mut [u8],
  info: FrameBufferInfo,
}

#[derive(Clone, Copy)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8
}

impl Screen {
  pub fn new(buffer: &'static mut FrameBuffer) -> Screen {
    let info = buffer.info();
    let frame_buffer = buffer.buffer_mut();

    Screen{ frame_buffer, info }
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
}

impl Color {
  pub fn black() -> Color {
    Color{ r: 0, g: 0, b: 0 }
  }
}