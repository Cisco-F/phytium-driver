//* 引入的包的模块 */
use core::ptr::NonNull;

//* 同一个包的模块 */
use super::mci_timing::*;
use super::constants::*;
use super::regs::*;

#[derive(Debug, PartialEq, Clone)]
pub struct MCIConfig {
    instance_id: MCIId,           /* Device instance id */
    reg: MCIReg,              /* Device register base address */
    irq_num: u32,               /* Device IRQ number */
    trans_mode: MCITransMode, /* Trans mode, PIO/DMA */
    non_removable: bool,        /* Non-removable media, e.g. eMMC */
}

impl MCIConfig {

    /* Get the device instance default configure  */
    pub fn lookup_config(addr: NonNull<u8>,id: MCIId) -> Self {
        match id {
            MCIId::MCI0 => {
                MCIConfig::new_mci0(addr)
            },
            MCIId::MCI1 => {
                MCIConfig::new_mci1(addr)
            },
        }
    }
    /* Get time-tuning related parameters and method */
    pub fn get_tuning(clock_freq: MCIClkSpeed, non_removable: bool) ->  Option<MCITiming> {
        if clock_freq == MCIClkSpeed::ClkSpeed400KHz {
            return Some(MMC_SD_400K_HZ);
        }
        match (non_removable, clock_freq) {
            (true, MCIClkSpeed::ClkSpeed26Mhz) => Some(MMC_26MHZ),
            (true, MCIClkSpeed::ClkSpeed52Mhz) => Some(MMC_52MHZ),
            (true, MCIClkSpeed::ClkSpeed66Mhz) => Some(MMC_66MHZ),
            (true, MCIClkSpeed::ClkSpeed100Mhz) => Some(MMC_100MHZ),
            (false, MCIClkSpeed::ClkSpeed25Mhz) => Some(SD_25MHZ),
            (false, MCIClkSpeed::ClkSpeed50Mhz) => Some(SD_50MHZ),
            (false, MCIClkSpeed::ClkSpeed100Mhz) => Some(SD_100MHZ),
            _ => None,
        }
    }

    pub fn new(addr:NonNull<u8>) -> Self {
        MCIConfig {
            instance_id: MCIId::MCI0,
            reg: MCIReg::new(addr),
            irq_num: 104,
            trans_mode: MCITransMode::DMA,
            non_removable: false,
        }
    }

    pub fn new_mci0(addr:NonNull<u8>) -> Self {
        MCIConfig {
            instance_id: MCIId::MCI0,
            reg: MCIReg::new(addr),
            irq_num: 104,
            trans_mode: MCITransMode::PIO,
            non_removable: false,
        }
    }

    pub fn new_mci1(addr:NonNull<u8>) -> Self {
        MCIConfig {
            instance_id: MCIId::MCI1,
            reg: MCIReg::new(addr),
            irq_num: 105,
            trans_mode: MCITransMode::DMA,
            non_removable: false,
        }
    }

    pub fn restart(addr: NonNull<u8>, id: MCIId) -> Self {
        match id {
            MCIId::MCI0 => Self::restart_mci0(addr),
            MCIId::MCI1 => Self::restart_mci1(addr),
        }
    }

    pub fn restart_mci0(addr:NonNull<u8>) -> Self {
        MCIConfig {
            instance_id: MCIId::MCI0,
            reg: MCIReg::new(addr),
            irq_num: 104,
            trans_mode: MCITransMode::PIO,
            non_removable: false,
        }
    }

    

    pub fn restart_mci1(addr: NonNull<u8>) -> Self {
        MCIConfig {
            instance_id: MCIId::MCI1,
            reg: MCIReg::new(addr),
            irq_num: 105,
            trans_mode: MCITransMode::DMA,
            non_removable: false,
        }
    }

    pub fn reg(&self) -> &MCIReg {
        &self.reg
    }

    pub fn trans_mode(&self) -> MCITransMode {
        self.trans_mode
    }

    pub fn trans_mode_set(&mut self, mode: MCITransMode) {
        self.trans_mode = mode;
    }

    pub fn non_removable(&self) -> bool {
        self.non_removable
    }

    pub fn instance_id(&self) -> MCIId {
        self.instance_id
    }
}
