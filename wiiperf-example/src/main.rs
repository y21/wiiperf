#![no_std]
#![no_main]

use core::{arch::naked_asm, panic::PanicInfo};

use wiistd::{ppc, println};

#[panic_handler]
fn panic_handler(panic: &PanicInfo) -> ! {
    println!("A panic occurred! {panic}");
    loop {}
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
extern "C" fn _start() {
    naked_asm!(
        "
        lis 1, 0x8090

        bl main
        loop_forever:
            b loop_forever
    ",
    );
}

#[unsafe(no_mangle)]
fn main() {
    wiiperf::exception::install();
    unsafe { ppc::enable_interrupts() };
}
