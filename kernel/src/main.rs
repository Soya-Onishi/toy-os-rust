#![no_std]
#![no_main]

mod video;

use core::panic::PanicInfo;
use crate::video::VGATextMode;

bootloader_api::entry_point!(kernel_main);

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn kernel_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let mut vga = VGATextMode::new();
    vga.write_str("Hello World!".as_bytes());

    loop {}
}
