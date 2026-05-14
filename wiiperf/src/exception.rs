use core::{
    arch::{asm, naked_asm},
    mem::offset_of,
    ptr,
};

use wiistd::ppc;

use crate::{assembler, profiler};

#[repr(C)]
pub(crate) struct StubData {
    pub gprs: [u32; 32],
    pub srr0: u32,
    pub srr1: u32,
    pub msr: u32,
    pub lr: u32,
    pub ctr: u32,
    pub cr: u32,
    pub xer: u32,
}

#[unsafe(no_mangle)]
pub(crate) static mut _WIIPERF_EH_STUB_DATA: StubData = StubData {
    gprs: [0; 32],
    srr0: 0,
    srr1: 0,
    msr: 0,
    lr: 0,
    ctr: 0,
    cr: 0,
    xer: 0,
};

const _: () = {
    // Make sure to update in asm if this changes.
    assert!(offset_of!(StubData, gprs) == 0);
    assert!(offset_of!(StubData, srr0) == 128);
    assert!(offset_of!(StubData, srr1) == 132);
    assert!(offset_of!(StubData, msr) == 136);
    assert!(offset_of!(StubData, lr) == 140);
    assert!(offset_of!(StubData, ctr) == 144);
    assert!(offset_of!(StubData, cr) == 148);
    assert!(offset_of!(StubData, xer) == 152);
};

static DECR_FREQ: u32 = 100_000;

#[unsafe(no_mangle)]
extern "C" fn __handle_interrupt() {
    let srr0 = unsafe { _WIIPERF_EH_STUB_DATA.srr0 };
    profiler::handle_interrupt(srr0);
    ppc::set_decrementer(DECR_FREQ);

    // Jump to exit stub
    unsafe {
        asm!(
            "
            lis 5, _WIIPERF_EH_STUB_DATA@h
            ori 5, 5, _WIIPERF_EH_STUB_DATA@l
            dcbi 0, 5 # we're in virtual mode here, make sure we pick up what we wrote earlier in real mode

            # Restore original SRR0/1; put those in SPRG0/1 as they are needed in EXIT_STUB
            lwz 3, 128(5)
            mtsprg0 3
            lwz 3, 132(5)
            mtsprg1 3

            # Restore SPRs
            lwz 3, 136(5)
            mtsrr1 3
            lwz 3, 140(5)
            mtlr 3
            lwz 3, 144(5)
            mtctr 3
            lwz 3, 148(5)
            mtcr 3
            lwz 3, 152(5)
            mtxer 3

            # Set up SRR0 to exit stub.
            lis 3, _WIIPERF_EH_EXIT_STUB@h
            ori 3, 3, _WIIPERF_EH_EXIT_STUB@l
            clrlwi 3, 3, 1
            mtsrr0 3

            # Lastly, restore GPRs
            lmw 0, 0(5)

            # NOTE: careful with clobbering GPRs from here on!

            rfi
            "
        )
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
extern "C" fn _wiiperf_eh_entry_stub() {
    naked_asm!(
        "
        # save r3 since we'll use that for stmw to save all other GPRs
        mtsprg0 3
        
        lis 3, _WIIPERF_EH_STUB_DATA@h
        ori 3, 3, _WIIPERF_EH_STUB_DATA@l

        # virtual to physical
        clrlwi 3, 3, 1

        # save all GPRs EXCEPT r3 (clobbered by STUB_DATA ptr)
        stmw 0, 0(3)
        # save r3 manually (previously loaded into SPRG0)
        # we'll still need the STUB_DATA ptr so we keep it in r3
        mfsprg0 4
        stw 4, 12(3)
        
        # save original SRR0/1 and entry MSR
        mfsrr0 4
        stw 4, 128(3)
        mfsrr1 4
        stw 4, 132(3)
        mfmsr 4
        stw 4, 136(3)

        # save SPRs
        mflr 4
        stw 4, 140(3)
        mfctr 4
        stw 4, 144(3)
        mfcr 4
        stw 4, 148(3)
        mfxer 4
        stw 4, 152(3)

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

unsafe fn setup_exit_stub() {
    /// Contains necessary code to continue execution to the previous exception handler after our own exception handler.
    /// This is called by the end of the interrupt handler.
    #[unsafe(no_mangle)]
    static mut _WIIPERF_EH_EXIT_STUB: [u32; 8] = [0; 8];

    unsafe {
        // Series of instructions to restore original SRR0/1. The interrupt exit puts them in SPRG0/1
        // and in order to avoid cloberring GPRs, we do some juggling around GPR<->SPRs.
        _WIIPERF_EH_EXIT_STUB[0] = 0x7c7243a6; // mtsprg 2,r3
        _WIIPERF_EH_EXIT_STUB[1] = 0x7c7042a6; // mfsprg r3,0
        _WIIPERF_EH_EXIT_STUB[2] = 0x7c7a03a6; // mtsrr0 r3
        _WIIPERF_EH_EXIT_STUB[3] = 0x7c7142a6; // mfsprg r3,1
        _WIIPERF_EH_EXIT_STUB[4] = 0x7c7b03a6; // mtsrr1 r3
        _WIIPERF_EH_EXIT_STUB[5] = 0x7c7242a6; // mfsprg r3,2

        // Original instruction from the previous exception handler.
        _WIIPERF_EH_EXIT_STUB[6] = DEC_EXC_VIRT.read_volatile();
        // TODO: ^ check if this is a PC-relative instruction and patch it (or warn)?

        // Set up relative branch to 0x009004 (one-past the overwritten original instruction)
        let offset =
            (DEC_EXC_VIRT.add(1) as isize) - (&raw const _WIIPERF_EH_EXIT_STUB[7] as isize);
        _WIIPERF_EH_EXIT_STUB[7] = assembler::branch(offset, false, false);

        #[expect(static_mut_refs, reason = "short lived shared references")]
        {
            ppc::flush_dcache(
                _WIIPERF_EH_EXIT_STUB.as_ptr().cast(),
                _WIIPERF_EH_EXIT_STUB.len() * 4,
            );
            ppc::invalidate_icache(
                _WIIPERF_EH_EXIT_STUB.as_ptr().cast(),
                _WIIPERF_EH_EXIT_STUB.len() * 4,
            );
        }
    }
}

/// Installs the exception handler.
pub fn install() {
    unsafe { setup_exit_stub() };

    // Register the exception handler.
    let dec_eh_offset =
        (_wiiperf_eh_entry_stub as *const () as isize) - DEC_EXC_VIRT.addr() as isize;
    unsafe { DEC_EXC_VIRT.write_volatile(assembler::branch(dec_eh_offset, false, false)) };
    ppc::flush_dcache(DEC_EXC_VIRT.cast(), 4);

    ppc::set_decrementer(DECR_FREQ);
}
