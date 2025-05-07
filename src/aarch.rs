use core::arch::asm;

#[inline(always)]
pub unsafe fn dsb() {
    core::arch::asm!("dsb sy");
    core::arch::asm!("isb sy");
}

#[inline(always)]
pub unsafe fn flush(addr: *const u8, size: usize) {
    let mut addr = addr as usize;
    let end = addr + size;
    while addr < end {
        asm!("dc civac, {0}", in(reg) addr, options(nostack, preserves_flags));
        addr += 64;
    }
    dsb();
}

#[inline(always)]
pub unsafe fn invalidate(addr: *const u8, size: usize) {
    let mut addr = addr as usize;
    let end = addr + size;
    while addr < end {
        asm!("dc ivac, {0}", in(reg) addr, options(nostack, preserves_flags));
        addr += core::mem::size_of::<u32>();
    }
    asm!("dsb sy");
}

#[inline(always)]
pub unsafe fn get_affinity() -> u64 {
    let mpidr: u64;
    asm!("mrs {0}, MPIDR_EL1", out(reg) mpidr);
    mpidr
}