#![allow(dead_code)]

#[inline(always)]
pub unsafe fn dsb() {
    core::arch::asm!("dsb sy");
    core::arch::asm!("isb sy");
}

#[inline(always)]
pub unsafe fn isb() {
    core::arch::asm!("isb", options(nostack, preserves_flags));
}