#![feature(slice_flatten)]

use std::{fs::File, io::Read};
use std::io::Write;
use std::path::Path;

use font_binary_gen;

const FONT_FILE_PATH: &str = "./vga-rom-16.psf";
const RAW_BIN_PATH: &str = "./font.bin";

fn main() {
  let mut file = File::open(FONT_FILE_PATH).unwrap();
  let size = file.metadata().unwrap().len();   
  let mut buf: Vec<u8> = vec![0; size as usize];
  file.read(&mut buf[..]).unwrap();
  
  let fonts = font_binary_gen::generate_font_array(&buf[..]);
  
  let mut ascii_fonts: [[u8; 16]; 127] = [[0; 16]; 127];
  for (font, idx) in fonts.iter().zip(33..) {
    ascii_fonts[idx] = font.clone(); 
  }

  let mut font_file = File::create(RAW_BIN_PATH).unwrap();
  font_file.write(&ascii_fonts.flatten()).unwrap();

  let abs_path = Path::new(RAW_BIN_PATH).to_path_buf().canonicalize().unwrap();

  println!("cargo:rustc-env=RAW_FONT_PATH={}", abs_path.display());
}