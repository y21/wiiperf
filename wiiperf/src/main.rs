#![no_std]
#![no_main]

use core::{
    arch::{asm, naked_asm},
    mem::MaybeUninit,
    panic::PanicInfo,
    ptr,
};

use wiistd::{ppc, println};

mod assembler;

const STACK_SIZE: usize = 0x400000;

#[repr(transparent)]
struct SyncPtr<T>(*const T);
unsafe impl<T> Sync for SyncPtr<T> {}

#[used]
#[unsafe(link_section = ".bss")]
static _STACK: [MaybeUninit<u8>; STACK_SIZE] = [MaybeUninit::uninit(); STACK_SIZE];
#[unsafe(no_mangle)]
static _STACK_END: SyncPtr<MaybeUninit<u8>> = unsafe { SyncPtr(_STACK.as_ptr().add(STACK_SIZE)) };

#[panic_handler]
fn panic_handler(panic: &PanicInfo) -> ! {
    println!("panic!!! {panic}");
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

#[repr(C)]
struct StubData {
    srr0: u32,
    srr1: u32,
    entry_msr: u32,
}

#[unsafe(no_mangle)]
static mut STUB_DATA: StubData = StubData {
    srr0: 0,
    srr1: 0,
    entry_msr: 0,
};

fn interrupt_handler() {
    wiistd::println!("does this work from within a handler?");
    ppc::set_decrementer(1_000_000_000);
}

#[unsafe(no_mangle)]
extern "C" fn __handle_interrupt() {
    interrupt_handler(); // Make sure drop code all runs within the handler

    // Jump to exit stub
    unsafe {
        asm!(
            "
            lis 5, STUB_DATA@h
            ori 5, 5, STUB_DATA@l
            # TODO: should we invalidate r5 from the Dcache?

            lwz 3, 0(5) # srr0, used in EXIT_STUB
            lwz 4, 4(5) # srr1, used in EXIT_STUB

            lwz 5, 8(5) # entry_msr, temporary
            mtsrr1 5

            lis 5, EXIT_STUB@h
            ori 5, 5, EXIT_STUB@l
            
            clrlwi 5, 5, 1
            mtsrr0 5

            rfi
            "
        )
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn interrupt_stub() {
    naked_asm!(
        "
        lis 3, STUB_DATA@h
        ori 3, 3, STUB_DATA@l

        # virtual to physical
        clrlwi 3, 3, 1

        # save original SRR0/1 and entry MSR
        mfsrr0 4
        stw 4, 0(3)
        mfsrr1 4
        stw 4, 4(3)
        mfmsr 4
        stw 4, 8(3)

        # load interrupt handler into SRR0
        lis 3, __handle_interrupt@h
        ori 3, 3, __handle_interrupt@l
        mtsrr0 3

        # enable IR/DR to exit real mode, but keep interrupts disabled
        mfmsr 3
        ori 3, 3, 0x30
        mtsrr1 3

        rfi
        "
    )
}

const DEC_EXC_VIRT: *mut u32 = ptr::with_exposed_provenance_mut(0x80000900);

#[unsafe(no_mangle)]
static mut EXIT_STUB: [u32; 4] = [0; 4];

#[unsafe(no_mangle)]
fn main() {
    unsafe {
        EXIT_STUB[0] = 0x7c7a03a6; // mtsrr0 r3
        EXIT_STUB[1] = 0x7c9b03a6; // mtsrr1 r4

        // Save the original instruction on the exception handler.
        EXIT_STUB[2] = DEC_EXC_VIRT.read_volatile();

        // Branch to 0x009004
        let offset = (DEC_EXC_VIRT.add(1) as isize) - (&raw const EXIT_STUB[3] as isize);
        EXIT_STUB[3] = assembler::branch(offset, false, false);
    }
    // TODO: flush & invalidate EXIT_STUB!

    // Register the exception handler.
    let dec_eh_offset = (interrupt_stub as *const () as isize) - DEC_EXC_VIRT.addr() as isize;
    unsafe { DEC_EXC_VIRT.write_volatile(assembler::branch(dec_eh_offset, false, false)) };
    // TODO: flush & invalidate EXIT_STUB!

    // bunch of initialization here...
    ppc::set_decrementer(1_000_000_000);

    ppc::enable_interrupts();
}
