use core::{arch::asm, cell::RefCell};

const VIRTUAL_MASK: usize = 0x8000_0000;

pub fn virtual_to_physical(addr: usize) -> usize {
    addr & !VIRTUAL_MASK
}

pub fn physical_to_virtual(addr: usize) -> usize {
    addr | VIRTUAL_MASK
}

pub fn without_interrupts(fun: impl FnOnce()) {
    let had_interrupts = disable_interrupts();
    fun();
    unsafe { set_interrupts(had_interrupts) };
}

#[derive(Copy, Clone)]
pub struct Msr(u32);

impl core::fmt::Debug for Msr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct(core::any::type_name::<Self>())
            .field("ee", &((self.0 & Self::EE) != 0))
            .field("pr", &((self.0 & Self::PR) != 0))
            .field("fp", &((self.0 & Self::FP) != 0))
            .field("ir", &((self.0 & Self::IR) != 0))
            .field("dr", &((self.0 & Self::DR) != 0))
            .finish()
    }
}

impl Msr {
    const EE: u32 = 1 << (31 - 16);
    const PR: u32 = 1 << (31 - 17);
    const FP: u32 = 1 << (31 - 18);
    const IR: u32 = 1 << (31 - 26);
    const DR: u32 = 1 << (31 - 27);

    #[inline]
    pub fn disable_interrupts(self) -> Self {
        Msr(self.0 & !Self::EE)
    }

    #[inline]
    pub fn enable_interrupts(self) -> Self {
        Msr(self.0 | Self::EE)
    }

    #[inline]
    pub fn interrupts(self) -> bool {
        (self.0 & Self::EE) != 0
    }

    #[inline]
    pub fn set_interrupts(self, enabled: bool) -> Self {
        if enabled {
            self.enable_interrupts()
        } else {
            self.disable_interrupts()
        }
    }

    #[inline]
    pub fn raw(self) -> u32 {
        self.0
    }
}

#[inline]
pub fn srr0() -> u32 {
    let mut srr0: u32;
    unsafe { asm!("mfsrr0 {srr0}", srr0 = out(reg) srr0) };
    srr0
}

#[inline]
pub fn srr1() -> u32 {
    let mut srr1: u32;
    unsafe { asm!("mfsrr1 {srr1}", srr1 = out(reg) srr1) };
    srr1
}

#[inline]
pub fn msr() -> Msr {
    let mut msr: u32;
    unsafe { asm!("mfmsr {msr}", msr = out(reg) msr) };
    Msr(msr)
}

#[inline]
pub unsafe fn set_msr(msr: Msr) {
    unsafe { asm!("mtmsr {msr}", msr = in(reg) msr.0) };
}

#[inline]
pub fn disable_interrupts() -> bool {
    let msr = msr();
    unsafe { set_msr(msr.disable_interrupts()) };
    msr.interrupts()
}

#[inline]
pub unsafe fn enable_interrupts() {
    unsafe { set_msr(msr().enable_interrupts()) };
}

#[inline]
pub unsafe fn set_interrupts(enabled: bool) {
    unsafe { set_msr(msr().set_interrupts(enabled)) }
}

#[derive(Copy, Clone)]
pub struct Decrementer(pub u32);

#[inline]
pub fn decrementer() -> Decrementer {
    let mut dec: u32;
    unsafe { asm!("mfdec {dec}", dec = out(reg) dec) };
    Decrementer(dec)
}

#[inline]
pub fn set_decrementer(value: u32) {
    unsafe { asm!("mtdec {value}", value = in(reg) value) };
}

const CACHE_SIZE: usize = 32;
pub fn flush_dcache(addr: *const (), size: usize) {
    for offset in (0..size).step_by(CACHE_SIZE) {
        unsafe { asm!("dcbf {}, {}", in(reg) addr, in(reg) offset) };
    }
}

pub fn invalidate_icache(addr: *const (), size: usize) {
    for offset in (0..size).step_by(CACHE_SIZE) {
        unsafe { asm!("icbi {}, {}", in(reg) addr, in(reg) offset) };
    }
}

pub fn store_dcache(addr: *const (), size: usize) {
    for offset in (0..size).step_by(CACHE_SIZE) {
        unsafe { asm!("dcbst {}, {}", in(reg) addr, in(reg) offset) };
    }
}

#[repr(transparent)]
pub struct InterruptLock<T>(T);

unsafe impl<T> Sync for InterruptLock<T> {}

impl<T> InterruptLock<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let msr = msr();
        assert!(!msr.interrupts());

        f(&self.0)
    }

    pub fn lock(&self) -> InterruptLockGuard<'_, T> {
        let msr = msr();
        assert!(!msr.interrupts());

        InterruptLockGuard { lock: self }
    }
}

impl<T> InterruptLock<RefCell<T>> {
    pub fn with_cell_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        self.with(|cell| {
            let mut borrow = cell.borrow_mut();
            f(&mut *borrow)
        })
    }
}

pub struct InterruptLockGuard<'a, T> {
    lock: &'a InterruptLock<T>,
}

impl<T> core::ops::Deref for InterruptLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock.0
    }
}
