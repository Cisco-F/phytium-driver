#![allow(unused)] 
mod err;
pub(crate) mod constants;
pub(crate) mod regs;

use core::ptr::NonNull;
pub use constants::*;
use err::*;
use regs::{XReg0, XReg1};
use crate::regs::{FlagReg, Reg, BitsOps};

type IoPadReg = Reg<FioPadError>;

#[derive(Debug)]
pub struct IoPad {
    reg: IoPadReg,
    is_ready: bool,
}

impl IoPad {
    pub fn new(reg_base: NonNull<u8>) -> Self {
        IoPad {
            reg: IoPadReg::new(reg_base),
            is_ready: true,
        }
    }

    pub fn get_base_addr(&self) -> NonNull<u8> {
        self.reg.addr
    }

    pub fn func_get<T: FlagReg + XReg0 + BitsOps>(&self) -> FioPadFunc {
        let reg_val = self.reg.read_reg::<T>();
        let func = T::func_get(reg_val);
        func.into()
    }

    pub fn func_set<T: FlagReg + XReg0 + BitsOps>(&mut self, func: FioPadFunc) {
        self.reg.modify_reg::<T>(|reg| {
            reg | T::func_set(func.into())
        });
    }

    pub fn pull_get<T: FlagReg + XReg0 + BitsOps>(&self) -> FioPadPull {
        let reg_val = self.reg.read_reg::<T>();
        let pull = T::pull_get(reg_val);
        pull.into()
    }

    pub fn pull_set<T: FlagReg + XReg0 + BitsOps>(&mut self, pull: FioPadPull) {
        self.reg.modify_reg::<T>(|reg| {
            reg | T::pull_set(pull.into())
        });
    }

    pub fn drive_get<T: FlagReg + XReg0 + BitsOps>(&self) -> FioPadDrive {
        let reg_val = self.reg.read_reg::<T>();
        let drive = T::drive_get(reg_val);
        drive.into()
    }

    pub fn drive_set<T: FlagReg + XReg0 + BitsOps>(&mut self, drive: FioPadDrive) {
        self.reg.modify_reg::<T>(|reg| {
            reg | T::drive_set(drive.into())
        });
    }

    pub fn config_set<T: FlagReg + XReg0 + BitsOps>(&mut self, func: FioPadFunc, pull: FioPadPull, drive: FioPadDrive) {
        self.reg.modify_reg::<T>(|reg| {
            reg | T::func_set(func.into()) | T::pull_set(pull.into()) | T::drive_set(drive.into())
        });
    }

    pub fn config_get <T: FlagReg + XReg0 + BitsOps + Copy>(&self) -> (FioPadFunc, FioPadPull, FioPadDrive) {
        let reg_val = self.reg.read_reg::<T>();
        let func = T::func_get(reg_val);
        let pull = T::pull_get(reg_val);
        let drive = T::drive_get(reg_val);
        return (FioPadFunc::from(func), FioPadPull::from(pull), FioPadDrive::from(drive));
    }

    pub fn delay_get <T: FlagReg + XReg1 + BitsOps>(&self,dir:FioPadDelayDir,typ:FioPadDelayType) -> FioPadDelay {
        let reg_val = self.reg.read_reg::<T>();
        let mut delay= 0;
        if dir == FioPadDelayDir::OutputDelay {
            if typ == FioPadDelayType::DelayFineTuning {
                delay = T::out_delay_delicate_get(reg_val);
            } else if typ == FioPadDelayType::DelayCoarseTuning {
                delay = T::out_delay_rough_get(reg_val);
            }
        } else if dir == FioPadDelayDir::InputDelay {
            if typ == FioPadDelayType::DelayFineTuning {
                delay = T::in_delay_delicate_get(reg_val);
            } else if typ == FioPadDelayType::DelayCoarseTuning {
                delay = T::in_delay_rough_get(reg_val);
            }
        }
        delay.into()
    }

    pub fn delay_set <T: FlagReg + XReg1 + BitsOps>(&mut self, dir:FioPadDelayDir, typ:FioPadDelayType, delay: FioPadDelay) {
        if dir == FioPadDelayDir::OutputDelay {
            if typ == FioPadDelayType::DelayFineTuning {
                self.reg.modify_reg::<T>(|reg| {
                    reg | T::out_delay_delicate_set(delay.into())
                });
            } else if typ == FioPadDelayType::DelayCoarseTuning {
                self.reg.modify_reg::<T>(|reg| {
                    reg | T::out_delay_rough_set(delay.into())
                });
            }
        } else if dir == FioPadDelayDir::InputDelay {
            if typ == FioPadDelayType::DelayFineTuning {
                self.reg.modify_reg::<T>(|reg| {
                    reg | T::in_delay_delicate_set(delay.into())
                });
            } else if typ == FioPadDelayType::DelayCoarseTuning {
                self.reg.modify_reg::<T>(|reg| {
                    reg | T::in_delay_rough_set(delay.into())
                });
            }
        }
    }

    pub fn delay_enable_set <T: FlagReg + XReg1 + BitsOps>(&mut self, dir:FioPadDelayDir, enable: bool) {
        if dir == FioPadDelayDir::OutputDelay {
            self.reg.modify_reg::<T>(|reg| {
                if enable {
                    reg | T::out_delay_en()
                } else {
                    reg & !T::out_delay_en()
                }
            });
        } else if dir == FioPadDelayDir::InputDelay {
            self.reg.modify_reg::<T>(|reg| {
                if enable {
                    reg | T::in_delay_en()
                } else {
                    reg & !T::in_delay_en()
                }
            });
        }
    }

}


