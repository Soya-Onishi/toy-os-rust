#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

mod video;
mod alloc;

use core::panic::PanicInfo;
use crate::video::Screen;

bootloader_api::entry_point!(kernel_main);

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {} 
}

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let frame_buffer = boot_info.framebuffer.as_mut().unwrap();
    let mut screen = Screen::new(frame_buffer);
    screen.clear();

    let text = b"abcdefghijklmnopqrstuvwxyz";
    for _ in 0..10 {
        for i in 0..26 {
            let s = &text[..=i];
            screen.write_str(s); 
            screen.write_char('\n' as u8);
        }
    
        for i in 0..26 {
            let s = &text[..=(25 - i)];
            screen.write_str(s);
            screen.write_char('\n' as u8);
        }
    }    

    loop {}
}
