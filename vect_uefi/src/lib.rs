#![no_std]
#![no_main]

use core::panic::PanicInfo;
use uefi::prelude::*;

extern crate uefi;
extern crate uefi_services;
extern crate vect_bootapi;

#[entry]
fn efi_main() -> Status {
    uefi::helpers::init().unwrap();
    log::info!("Hello from Vect UEFI!");
    boot::stall(50_000_000);
    Status::SUCCESS
}

// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     loop {}
// }
