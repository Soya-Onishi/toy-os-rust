const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

const SCREEN_PTR: usize = 0xB8000;

pub struct VGATextMode {
  col: usize,
  row: usize,
  buffer: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],  
}

#[derive(Clone, Copy)]
#[repr(C)]
struct ScreenChar {
  ascii_character: u8,
  color_code: u8,
}

impl VGATextMode {
  pub fn new() -> VGATextMode {
    VGATextMode { 
      row: 0,
      col: 0,
      buffer: [[ScreenChar::zero(); BUFFER_WIDTH]; BUFFER_HEIGHT],
    }
  }

  pub fn write_str(&mut self, s: &[u8]) {
    for &c in s {
      self.write_char(c)
    }
  }

  pub fn write_char(&mut self, c: u8) {
    let color = 0x0F;
    let c = ScreenChar { 
      ascii_character: c,
      color_code: color,
    };

    self.buffer[self.row][self.col] = c;
    let ptr = SCREEN_PTR as *mut ScreenChar;
    let idx = (self.row * BUFFER_WIDTH + self.col) as isize;
    unsafe { *ptr.offset(idx) = c } 
  }
}

impl ScreenChar {
  fn zero() -> ScreenChar {
    ScreenChar { ascii_character: 0, color_code: 0 }
  }
}