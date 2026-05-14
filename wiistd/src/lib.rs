#![no_std]

pub mod fmt;
pub mod osreport;
pub mod ppc;
pub mod util;

#[cfg(feature = "panic_handler")]
#[panic_handler]
fn panic_handler(panic: &core::panic::PanicInfo) -> ! {
    println!("A panic occurred! {panic}");
    loop {}
}
