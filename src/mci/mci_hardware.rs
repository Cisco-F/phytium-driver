use super::MCI;

use super::err::*;
use super::consts::*;
use super::regs::*;

use log::*;

/// 直接操作寄存器相关的 API
impl MCI {
    pub(crate) fn status_get(&self) -> MCIStatus {
        let reg = self.config.reg();
        reg.read_reg::<MCIStatus>()
    }

    pub(crate) fn fifoth_set(&self, trans_size:MCIFifoThDMATransSize, rx_wmark:u32, tx_wmark:u32){
        let reg = self.config.reg();
        reg.write_reg(MCIFifoTh::fifoth(trans_size,rx_wmark,tx_wmark));
    }

    pub(crate) fn cardthr_set(&self,cardthr:MCIFifoDepth){
        let reg = self.config.reg();
        reg.write_reg(MCICardThrctl::CARDRD | cardthr.into());
    }

    pub(crate) fn clock_set(&self, enable:bool) {
        let reg = self.config.reg();
        if enable {
            reg.set_reg(MCIClkEn::CCLK_ENABLE);
        } else {
            reg.clear_reg(MCIClkEn::CCLK_ENABLE);
        }
    }

    pub(crate) fn update_exteral_clk(&self,uhs_reg:MCIClkSrc) -> MCIResult {
        let reg = self.config.reg();
        reg.write_reg(MCIClkSrc::empty());
        reg.write_reg(uhs_reg);
        reg.retry_for(|reg:MCIClkSts|{
            reg.contains(MCIClkSts::READY)
        }, Some(RETRIES_TIMEOUT))?;
        Ok(())
    }

    pub(crate) fn init_external_clk(&self) -> MCIResult {
        let reg_val = 
        MCIClkSrc::uhs_reg(0, 0, 0x5) | 
        MCIClkSrc::UHS_EXT_CLK_ENA;
        if 0x502 == reg_val.bits() {
            info!("invalid uhs config"); 
        }
        self.update_exteral_clk(reg_val)?;
        Ok(())
    }

    pub(crate) fn power_set(&self, enable:bool) {
        let reg = self.config.reg();
        if enable {
            reg.set_reg(MCIPwrEn::ENABLE);
        } else {
            reg.clear_reg(MCIPwrEn::ENABLE);
        }
    }

    pub(crate) fn clock_src_set(&self, enable:bool) {
        let reg = self.config.reg();
        if enable {
            reg.set_reg(MCIClkSrc::UHS_EXT_CLK_ENA);
        } else {
            reg.clear_reg(MCIClkSrc::UHS_EXT_CLK_ENA);
        }
    }

    pub(crate) fn voltage_1_8v_set(&self,enable:bool) {
        let reg = self.config.reg();
        if enable {
            reg.set_reg(MCIUhsReg::VOLT_180);
        } else {
            reg.clear_reg(MCIUhsReg::VOLT_180);
        }
    }

    pub(crate) fn bus_width_set(&self, width: u32) {
        let reg = self.config.reg();
        reg.write_reg::<MCICType>(width.into());
    }

    pub(crate) fn ctrl_reset(&self, reset_bits: MCICtrl) -> MCIResult {
        let reg = self.config.reg();

        reg.modify_reg(|reg| {
            reset_bits | reg
        });
        if let Err(e) = reg.retry_for(|reg:MCICtrl| {
            !reg.contains(reset_bits)
        }, Some(RETRIES_TIMEOUT)) {
            error!("Reset failed, bits = 0x{:x}", reset_bits);
            return Err(e);
        }

        /* update clock after reset */
        if let Err(e) = self.private_cmd_send(MCICmd::UPD_CLK, 0) {
            error!("Update clock failed!");
            return Err(e);
        }

        /* for fifo reset, need to check if fifo empty */
        if reset_bits.contains(MCICtrl::FIFO_RESET) {
            if let Err(e) = reg.retry_for(|reg: MCIStatus| {
                reg.contains(MCIStatus::FIFO_EMPTY)
            }, Some(RETRIES_TIMEOUT)) {
                error!("Fifo not empty!");
                return Err(e);
            }
        }
        Ok(())
    }

    pub(crate) fn cardreset_config(&self) {
        let reg = self.config.reg();
        if self.config.non_removable() {
            reg.set_reg(MCICardReset::ENABLE);
        } else {
            reg.clear_reg(MCICardReset::ENABLE);
        }
    }

    pub(crate) fn clear_interrupt_status(&self) {
        let reg = self.config.reg();

        /* 清空中断使能位 */
        reg.write_reg(MCIIntMask::empty());

        /* 清空当前的中断状态 */
        let reg_val = reg.read_reg::<MCIRawInts>();
        reg.write_reg(reg_val);

        /* 清空DMAC 中断使能 */
        reg.write_reg(MCIDMACIntEn::empty());

        /* 清空DMAC 中断状态 */
        let reg_val = reg.read_reg::<MCIDMACStatus>();
        reg.write_reg(reg_val);
    }

    pub(crate) fn descriptor_set(&self, descriptor: usize) {
        let reg = self.config.reg();
        let hi = (descriptor >> 32) as u32;
        reg.write_reg(MCIDescListAddrH::from_bits_truncate(hi));
        reg.write_reg(MCIDescListAddrL::from_bits_truncate(descriptor as u32));
    }

    pub(crate) fn idma_reset(&self) {
        let reg = self.config.reg();
        reg.set_reg(MCIBusMode::SWR); /* 写1软复位idma，复位完成后硬件自动清0 */
    }

    pub(crate) fn raw_status_get(&self) -> MCIRawInts {
        let reg = self.config.reg();
        reg.read_reg::<MCIRawInts>()
    }

    pub(crate) fn raw_status_clear(&self) {
        let reg = self.config.reg();
        /* 读写RawInts 使之清空 */
        reg.write_reg(self.raw_status_get());
    }

    pub(crate) fn dma_status_get(&self) -> MCIDMACStatus {
        let reg = self.config.reg();
        reg.read_reg::<MCIDMACStatus>()
    }

    pub(crate) fn dma_status_clear(&self) {
        let reg = self.config.reg();
        /* 读写DMACStatus 使之清空 */
        reg.write_reg(self.dma_status_get());
    }

    pub(crate) fn check_if_card_exist(&self) -> bool {
        let reg = self.config.reg();
        !reg.read_reg::<MCICardDetect>().contains(MCICardDetect::DETECTED)
    }

    pub(crate) fn check_if_card_busy(&self) -> bool {
        self.status_get().contains(MCIStatus::DATA_BUSY)
    }

    pub(crate) fn trans_bytes_set(&self, bytes: u32) {
        let reg = self.config.reg();
        reg.write_reg(MCIBytCnt::from_bits_truncate(bytes));
    }

    pub(crate) fn blksize_set(&self, blksize: u32) {
        let reg = self.config.reg();
        reg.write_reg(MCIBlkSiz::from_bits_truncate(blksize));
    }
}