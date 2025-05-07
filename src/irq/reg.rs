use bitflags::bitflags;

use super::{consts::*, err::GicError};
use crate::regs::{FlagReg, Reg};

pub type GicReg = Reg<GicError>;

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct GicdIRouter: u64 {
        const TEMP_FLAG = 0;
    }
}

impl FlagReg for GicdIRouter {
    const REG: u32 = FGIC_GICD_IROUTER_OFFSET;
}