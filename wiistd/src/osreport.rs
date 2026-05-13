use core::ptr;

use crate::ppc;

const EXI_STATUS_C0: *mut u32 = ptr::with_exposed_provenance_mut(0xcd806800);
const EXI_IMM_C0: *mut u32 = ptr::with_exposed_provenance_mut(0xcd806810);
const EXI_CR_C0: *mut u32 = ptr::with_exposed_provenance_mut(0xcd80680c);

const DEVICE1: u32 = 0b010;
const SPEED_8MHZ: u32 = 0b011;
const COMMAND_WRITE: u32 = 1;
const CR_WRITE: u32 = 0b01;
const CR_TSTART: u32 = 0b1;

#[used]
#[unsafe(link_section = ".text")]
static _IS_WII: u32 = 0x7c13fba6;

pub fn report_bytes(data: &[u8]) {
    fn send_data(data: &[u8]) {
        assert!(data.len() <= 4);

        // Write the data first.
        let mut chunk = [0u8; 4];
        chunk[..data.len()].copy_from_slice(data);
        unsafe { EXI_IMM_C0.write_volatile(u32::from_be_bytes(chunk)) };

        // Now execute the write command.
        let cr = (data.len() as u32 - 1) << 4 | CR_WRITE << 2 | CR_TSTART;
        unsafe { EXI_CR_C0.write_volatile(cr) };
    }

    ppc::without_interrupts(|| {
        // Dolphin's OSReport EXI lives on Channel 0 Device 1, configure the device
        unsafe { EXI_STATUS_C0.write_volatile((DEVICE1 << 7) | (SPEED_8MHZ << 4)) };

        // Not sure exactly what the 0x800400 is for but OSReport wants a command before the string data.
        let command = (COMMAND_WRITE << 31) | (0x800400 << 6);
        send_data(&command.to_ne_bytes());

        // EXI only supports up to 4 bytes at once
        for chunk in data.chunks(4) {
            send_data(chunk);
        }
        send_data(b"\r");
    });
}

pub fn report(data: &str) {
    report_bytes(data.as_bytes());
}
