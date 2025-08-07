use crate::iopad::constants::{FioPadDelay, FioPadDelayDir, FioPadDelayType};
use crate::iopad::regs::{Aj49Reg1, J53Reg1, XReg1};
use crate::iopad::IoPad;
use crate::regs::BitsOps;

use super::consts::*;

pub struct MCITiming {
    use_hold: bool,
    clk_div: u32,
    clk_src: u32,
    shift: u32,
    pad_delay: MCIPadDelay, //* 用于调整IO的延时 */
}

impl MCITiming {
    pub fn new() -> Self {
        MCITiming {
            use_hold: false,
            clk_div: 0,
            clk_src: 0,
            shift: 0,
            pad_delay: MCIPadDelay::None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum MCIPadDelay {
    Set,
    Unset,
    None,
}

impl MCITiming {
    pub(crate) fn pad_delay(&self, iopad: &mut IoPad, mci_id: MCIId) {
        match self.pad_delay {
            MCIPadDelay::Set => set_pad_delay(iopad, mci_id),
            MCIPadDelay::Unset => unset_pad_delay(iopad, mci_id),
            MCIPadDelay::None => {}
        }
    }

    pub(crate) fn clk_src(&self) -> u32 {
        self.clk_src
    }

    pub(crate) fn clk_div(&self) -> u32 {
        self.clk_div
    }

    pub(crate) fn use_hold(&self) -> bool {
        self.use_hold
    }

    pub(crate) fn shift(&self) -> u32 {
        self.shift
    }
}

pub const MMC_SD_400K_HZ: MCITiming = MCITiming {
    use_hold: true,
    clk_div: 0x7e7dfa,
    clk_src: 0x000502,
    shift: 0x0,
    pad_delay: MCIPadDelay::Unset,
};

pub const SD_25MHZ: MCITiming = MCITiming {
    use_hold: true,
    clk_div: 0x030204,
    clk_src: 0x000302,
    shift: 0x0,
    pad_delay: MCIPadDelay::Unset,
};

pub const SD_50MHZ: MCITiming = MCITiming {
    use_hold: true,
    clk_div: 0x030204,
    clk_src: 0x000502,
    shift: 0x0,
    pad_delay: MCIPadDelay::Set,
};

pub const SD_100MHZ: MCITiming = MCITiming {
    use_hold: false,
    clk_div: 0x010002,
    clk_src: 0x000202,
    shift: 0x0,
    pad_delay: MCIPadDelay::Set,
};

pub const MMC_26MHZ: MCITiming = MCITiming {
    use_hold: true,
    clk_div: 0x030204,
    clk_src: 0x000302,
    shift: 0x0,
    pad_delay: MCIPadDelay::Set,
};

pub const MMC_52MHZ: MCITiming = MCITiming {
    use_hold: false,
    clk_div: 0x030204,
    clk_src: 0x000202,
    shift: 0x0,
    pad_delay: MCIPadDelay::Set,
};

pub const MMC_66MHZ: MCITiming = MCITiming {
    use_hold: false,
    clk_div: 0x010002,
    clk_src: 0x000202,
    shift: 0x0,
    pad_delay: MCIPadDelay::None,
};

pub const MMC_100MHZ: MCITiming = MCITiming {
    use_hold: false,
    clk_div: 0x010002,
    clk_src: 0x000202,
    shift: 0x0,
    pad_delay: MCIPadDelay::Set,
};

/* 管脚相关定义 */
type Fsdif0SdCclkOutDelay = Aj49Reg1;
type Fsdif1SdCclkOutDelay = J53Reg1;

fn apply_delay_settings<T: XReg1 + BitsOps>(
    iopad: &mut IoPad,
    coarse_delay: FioPadDelay,
    fine_delay: FioPadDelay,
    enable: bool,
) where
    T: 'static,
{
    iopad.delay_set::<T>(
        FioPadDelayDir::OutputDelay,
        FioPadDelayType::DelayCoarseTuning,
        coarse_delay,
    );
    iopad.delay_set::<T>(
        FioPadDelayDir::OutputDelay,
        FioPadDelayType::DelayFineTuning,
        fine_delay,
    );
    iopad.delay_enable_set::<T>(FioPadDelayDir::OutputDelay, enable);
}

pub fn set_pad_delay(iopad: &mut IoPad, mci_id: MCIId) {
    match mci_id {
        MCIId::MCI0 => apply_delay_settings::<Fsdif0SdCclkOutDelay>(
            iopad,
            FioPadDelay::Delay1,
            FioPadDelay::Delay7,
            true,
        ),
        MCIId::MCI1 => apply_delay_settings::<Fsdif1SdCclkOutDelay>(
            iopad,
            FioPadDelay::Delay1,
            FioPadDelay::Delay7,
            true,
        ),
    }
}

pub fn unset_pad_delay(iopad: &mut IoPad, mci_id: MCIId) {
    match mci_id {
        MCIId::MCI0 => apply_delay_settings::<Fsdif0SdCclkOutDelay>(
            iopad,
            FioPadDelay::DelayNone,
            FioPadDelay::DelayNone,
            false,
        ),
        MCIId::MCI1 => apply_delay_settings::<Fsdif1SdCclkOutDelay>(
            iopad,
            FioPadDelay::DelayNone,
            FioPadDelay::DelayNone,
            false,
        ),
    }
}
