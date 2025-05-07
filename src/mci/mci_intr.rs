use super::MCI;
use super::consts::*;
use super::regs::*;

impl MCI {
    /* Get SDIF controller interrupt mask */
    pub fn interrupt_mask_get(&self, tp: MCIIntrType) -> u32 {
        let reg = self.config.reg();
        let mut mask = 0;
        if MCIIntrType::GeneralIntr == tp {
            mask = reg.read_reg::<MCIIntMask>().bits();
        } else if MCIIntrType::DmaIntr == tp {
            mask = reg.read_reg::<MCIDMACIntEn>().bits();
        }
        mask
    }

    /* Enable/Disable SDIF controller interrupt */
    pub fn interrupt_mask_set(&self, tp: MCIIntrType, set_mask: u32, enable: bool) {
        let mut mask = self.interrupt_mask_get(tp);
        if enable {
            mask |= set_mask;
        } else {
            mask &= !set_mask;
        }
        let reg = self.config.reg();
        if MCIIntrType::GeneralIntr == tp {
            reg.write_reg(MCIIntMask::from_bits_truncate(mask));
        } else if MCIIntrType::DmaIntr == tp {
            reg.write_reg(MCIDMACIntEn::from_bits_truncate(mask));
        }
    }
}