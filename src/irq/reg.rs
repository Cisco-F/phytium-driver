use core::ptr::NonNull;

use bitflags::bitflags;

use super::{consts::*, err::{GicError, GicStatus}};
use crate::regs::{FlagReg, Reg};

pub fn gic_read_32(base: u32, offset: u32) -> u32 {
    unsafe {
        let ptr = NonNull::new_unchecked(base as *mut u8).add(offset as _);
        ptr.cast().read_volatile()
    }
}

pub fn gic_write_32(base: u32, offset: u32, data: u32) {
    unsafe {
        let ptr = NonNull::new_unchecked(base as *mut u8).add(offset as _);
        ptr.cast().write_volatile(data);
    }
}

pub fn gic_clear_bit(base: u32, offset: u32, bits: u32) {
    let mut val = gic_read_32(base, offset);
    val &= !bits;
    gic_write_32(base, offset, val);
}

pub fn gic_retry_for<F: Fn(u32) -> bool>(
    f: F,
    try_count: u32,
    base: u32,
    offset: u32
) -> GicStatus {
    for _ in 0..try_count {
        if f(gic_read_32(base, offset)) {
            return Ok(())
        }
    }
    Err(GicError::Timeout)
}

pub type GicReg = Reg<GicError>;

bitflags! {
    pub struct GicrCtrl: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}

impl FlagReg for GicrCtrl {
    const REG: u32 = FGIC_GICR_CTLR_OFFSET;
}

bitflags! {
    pub struct TyperL: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}

impl FlagReg for TyperL {
    const REG: u32 = FGIC_GICR_TYPER_L_OFFSET;
}

bitflags! {
    pub struct TyperH: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}

impl FlagReg for TyperH {
    const REG: u32 = FGIC_GICR_TYPER_H_OFFSET;
}

bitflags! {
    pub struct GicdIRouter: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}

impl FlagReg for GicdIRouter {
    const REG: u32 = FGIC_GICD_IROUTER_OFFSET;
}