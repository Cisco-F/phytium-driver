use super::constants::*;
use crate::regs::FlagReg;
use bitflags::bitflags;
use core::ops;

pub trait XReg0: FlagReg {
    fn func_set(x: u32) -> Self {
        Self::from_bits_truncate(set_reg32_bits!(x, 2, 0))
    }

    fn drive_set(x: u32) -> Self {
        Self::from_bits_truncate(set_reg32_bits!(x, 7, 4))
    }

    fn pull_set(x: u32) -> Self {
        Self::from_bits_truncate(set_reg32_bits!(x, 9, 8))
    }

    fn func_get(self) -> u32 {
        get_reg32_bits!(self.bits(), 2, 0)
    }

    fn drive_get(self) -> u32 {
        get_reg32_bits!(self.bits(), 7, 4)
    }

    fn pull_get(self) -> u32 {
        get_reg32_bits!(self.bits(), 9, 8)
    }
}

pub trait XReg1: FlagReg {
    fn out_delay_delicate_set(x: u32) -> Self {
        Self::from_bits_truncate(set_reg32_bits!(x, 11, 9))
    }

    fn out_delay_rough_set(x: u32) -> Self {
        Self::from_bits_truncate(set_reg32_bits!(x, 14, 12))
    }

    fn in_delay_delicate_set(x: u32) -> Self {
        Self::from_bits_truncate(set_reg32_bits!(x, 3, 1))
    }

    fn in_delay_rough_set(x: u32) -> Self {
        Self::from_bits_truncate(set_reg32_bits!(x, 6, 4))
    }

    fn out_delay_delicate_get(self) -> u32 {
        get_reg32_bits!(self.bits(), 11, 9)
    }

    fn out_delay_rough_get(self) -> u32 {
        get_reg32_bits!(self.bits(), 14, 12)
    }

    fn in_delay_delicate_get(self) -> u32 {
        get_reg32_bits!(self.bits(), 3, 1)
    }

    fn in_delay_rough_get(self) -> u32 {
        get_reg32_bits!(self.bits(), 6, 4)
    }

    fn out_delay_en() -> Self {
        Self::from_bits_truncate(1 << 8)
    }

    fn in_delay_en() -> Self {
        Self::from_bits_truncate(1 << 0)
    }
}

#[macro_export]
macro_rules! X_REG0 {
    ($reg_name:ident, $reg_addr:expr) => {
        bitflags! {
            #[derive(Clone, Copy)]
            pub struct $reg_name: u32 {
                const PULL_MASK = genmask!(9, 8);
                const DRIVE_MASK = genmask!(7, 4);
                const FUNC_MASK = genmask!(2, 0);
                const FUNC_BIT0 = 1 << 0;
                const FUNC_BIT1 = 1 << 1;
                const FUNC_BIT2 = 1 << 2;
                const DRIVE_BIT0 = 1 << 4;
                const DRIVE_BIT1 = 1 << 5;
                const DRIVE_BIT2 = 1 << 6;
                const DRIVE_BIT3 = 1 << 7;
                const PULL_BIT0 = 1 << 8;
                const PULL_BIT1 = 1 << 9;
            }
        }

        impl FlagReg for $reg_name {
            const REG: u32 = $reg_addr;
        }

        impl XReg0 for $reg_name {}
    };
}

#[macro_export]
macro_rules! X_REG1 {
    ($reg_name:ident, $reg_addr:expr) => {
        bitflags! {
            #[derive(Clone, Copy)]
            pub struct $reg_name: u32 {
                const OUT_DELAY_EN = 1 << 8;
                const OUT_DELAY_DELICATE_MASK = genmask!(11,9);
                const OUT_DELAY_DELICATE_BIT0 = 1 << 9;
                const OUT_DELAY_DELICATE_BIT1 = 1 << 10;
                const OUT_DELAY_DELICATE_BIT2 = 1 << 11;
                const OUT_DELAY_ROUGH_MASK = genmask!(14,12);
                const OUT_DELAY_ROUGH_BIT0 = 1 << 12;
                const OUT_DELAY_ROUGH_BIT1 = 1 << 13;
                const OUT_DELAY_ROUGH_BIT2 = 1 << 14;
                const IN_DELAY_EN = 1 << 0;
                const IN_DELAY_DELICATE_MASK = genmask!(3,1);
                const IN_DELAY_DELICATE_BIT0 = 1 << 1;
                const IN_DELAY_DELICATE_BIT1 = 1 << 2;
                const IN_DELAY_DELICATE_BIT2 = 1 << 3;
                const IN_DELAY_ROUGH_MASK = genmask!(6,4);
                const IN_DELAY_ROUGH_BIT0 = 1 << 4;
                const IN_DELAY_ROUGH_BIT1 = 1 << 5;
                const IN_DELAY_ROUGH_BIT2 = 1 << 6;
            }
        }

        impl FlagReg for $reg_name {
            const REG: u32 = $reg_addr;
        }

        impl XReg1 for $reg_name {}
    };
}

X_REG0!(An59Reg0, FIOPAD_AN59_REG0_OFFSET);

X_REG1!(Aj49Reg1, FIOPAD_AJ49_REG1_OFFSET);
X_REG1!(J53Reg1, FIOPAD_J53_REG1_OFFSET);
