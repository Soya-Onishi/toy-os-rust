#![no_std]
#![no_main]

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
    screen.clear(video::Color::black());  

    loop {}
}
