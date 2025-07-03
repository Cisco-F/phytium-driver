use core::ptr::NonNull;

use log::debug;
use log::error;
use log::warn;

use crate::mci_sdif::sdif_device::SDIFDev;
use crate::osa::consts::SDMMC_OSA_EVENT_CARD_INSERTED;
use crate::osa::consts::SDMMC_OSA_EVENT_TRANSFER_CMD_FAIL;
use crate::osa::consts::SDMMC_OSA_EVENT_TRANSFER_CMD_SUCCESS;
use crate::osa::consts::SDMMC_OSA_EVENT_TRANSFER_DATA_FAIL;
use crate::osa::consts::SDMMC_OSA_EVENT_TRANSFER_DATA_SUCCESS;
use crate::osa::osa_event_set;
use crate::sd::REG_BASE;

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

/// Interrupt handler for SDIF instance
pub fn fsdif_interrupt_handler() {
    warn!("captured irq!");
    let reg = unsafe { MCIReg::new(REG_BASE) };

    let events = reg.read_reg::<MCIRawInts>();
    let dmac_events = reg.read_reg::<MCIDMACStatus>();
    let event_mask = reg.read_reg::<MCIIntMask>();
    let dmac_evt_mask = reg.read_reg::<MCIDMACIntEn>();

    warn!("raw ints reg: 0x{:x}, dmac status: 0x{:x}", events.bits(), dmac_events.bits());

    // no interrupt status
    if (events.bits() & MCIRawInts::ALL_BITS.bits() == 0) && (dmac_events.bits() & MCIDMACStatus::ALL_BITS.bits() == 0) {
        warn!("irq exit with no action");
        reg.write_reg::<MCIRawInts>(events);
        reg.write_reg::<MCIDMACStatus>(dmac_events);
        return;
    }

    reg.write_reg::<IrqTempRegister>(IrqTempRegister::from_bits_truncate(0));

    // no need to handle interrput
    if (events.bits() == 0) && (dmac_events.bits() & 0x1FFF == 0) {
        reg.write_reg::<MCIRawInts>(events);
        reg.write_reg::<MCIDMACStatus>(dmac_events);
        return;
    }

    debug!("events:0x{:x},mask:0x{:x},dmac_events:{:x},dmac_mask:0x{:x}", events, event_mask, dmac_events, dmac_evt_mask);

    // Clear interrupt status
    let events_copy = reg.read_reg::<MCIRawInts>();
    let dmac_events_copy = reg.read_reg::<MCIDMACStatus>();
    reg.write_reg::<MCIRawInts>(events_copy);
    reg.write_reg::<MCIDMACStatus>(dmac_events_copy);

    // handle sdio irq
    if (events.bits() & event_mask.bits()) & MCIRawInts::SDIO_BIT.bits() != 0 {
        warn!("SDIO interrupt here!");
        sdio_interrupt();
    }

    // handle card detect event
    // todo 尚未实现卡检测相关事件
    // if events.bits() & event_mask.bits() & MCIRawInts::CD_BIT.bits() != 0 &&
    //     !self.config().non_removable() 
    // {
    //     warn!("SD status changed here! status:[{}]", reg.read_reg::<MCICardDetect>().bits());
    //     card_detected();
    // }

    // handle error state
    if dmac_events.contains(MCIDMACStatus::DMAC_ERR_INTS_MASK) || 
        events.contains(MCIRawInts::CMD_ERR_INTS_MASK)
    {  
        error!(
            "ERR: events: 0x{:x}, mask: 0x{:x}, dmac_evts: 0x{:x}, dmac_mask: {:x}",
            events.bits(), event_mask.bits(), dmac_events.bits(), dmac_evt_mask.bits()
        );
        error_occur(&reg, events.bits(), dmac_events.bits());
    }

    // handle cmd && data done
    if events.contains(MCIRawInts::DTO_BIT) &&
        events.contains(MCIRawInts::CMD_BIT)
    {
        warn!("cmd and data over!");
        cmd_done();
        data_done(events.bits(), dmac_events.bits());
    } 
    else if events.contains(MCIRawInts::CMD_BIT)
        // todo 这里无法得到MCI实例 暂时无法处理这种情况
        // (events.contains(MCIRawInts::HTO_BIT) && self.cur_cmd_index() == MCI::SWITCH_VOLTAGE as isize) 
    {
        warn!("cmd over!");
        cmd_done();
    }
    else if events.contains(MCIRawInts::CMD_BIT) { // handle cmd done
        warn!("cmd over!");
        cmd_done();
    }
    else if events.contains(MCIRawInts::DTO_BIT) { // handle data done
        warn!("data over!");
        data_done(events.bits(), dmac_events.bits());
    }
}

fn card_detected() {
    osa_event_set(SDMMC_OSA_EVENT_CARD_INSERTED);
}

pub fn cmd_done() {
    osa_event_set(SDMMC_OSA_EVENT_TRANSFER_CMD_SUCCESS);
}

pub fn data_done(status: u32, dmac_status: u32) {
    let check_status = status & (
        MCIRawInts::DTO_BIT      // Data transfer over
            | MCIRawInts::RCRC_BIT    // Response CRC error
            | MCIRawInts::DCRC_BIT    // Data CRC error
            | MCIRawInts::RE_BIT      // Response error
            | MCIRawInts::DRTO_BIT    // Data read timeout
            | MCIRawInts::EBE_BIT     // End-bit error
            | MCIRawInts::SBE_BCI_BIT // Start-bit error
            | MCIRawInts::RTO_BIT     // Response timeout
    ).bits();
    let check_dmac = dmac_status & (MCIDMACIntEn::AIS | MCIDMACIntEn::DU).bits();

    // todo 这里无法得到MCI实例 暂时无法处理这种情况
    // if !dev.whether_transfer_data() {
    //     osa_event_set(SDMMC_OSA_EVENT_TRANSFER_DATA_SUCCESS);
    // } else if check_status | check_dmac != 0 {
    //     if check_status & MCIRawInts::DTO_BIT.bits() != 0 {
    //         osa_event_set(SDMMC_OSA_EVENT_TRANSFER_DATA_SUCCESS);
    //     } else {
    //         error!("transfer data error: 0x{:x}, dmac status: 0x{:x}", check_status, check_dmac);
    //     }
    // }
    if check_status | check_dmac != 0 {
        if check_status & MCIRawInts::DTO_BIT.bits() != 0 {
            osa_event_set(SDMMC_OSA_EVENT_TRANSFER_DATA_SUCCESS);
        } else {
            error!("transfer data error: 0x{:x}, dmac status: 0x{:x}", check_status, check_dmac);
        }
    }
}

fn error_occur(reg: &MCIReg, status: u32, dmac_status: u32) {
    warn!("Error occur!!!");
    warn!("status: 0x{:x}, dmac status: 0x{:x}", status, dmac_status);

    if status & MCIRawInts::RE_BIT.bits() != 0 {
        warn!("response error! 0x{:x}", MCIRawInts::RE_BIT.bits());
    }
    
    if status & MCIRawInts::RTO_BIT.bits() != 0 {
        warn!("response timeout! 0x{:x}", MCIRawInts::RTO_BIT.bits());
    }
    
    if status & MCIRawInts::DCRC_BIT.bits() != 0 {
        warn!("data crc error! 0x{:x}", MCIRawInts::DCRC_BIT.bits());
    }

    if status & MCIRawInts::RCRC_BIT.bits() != 0 {
        warn!("data crc error! 0x{:x}", MCIRawInts::RCRC_BIT.bits());
    }

    if dmac_status & MCIDMACIntEn::DU.bits() != 0 {
        warn!("Descriptor un-readable! 0x{:x}", MCIDMACIntEn::DU.bits());
    }

    register_dump(reg);

    if status & MCIRawInts::RE_BIT.bits() != 0 ||
        status & MCIRawInts::RTO_BIT.bits() != 0 
    {
        osa_event_set(SDMMC_OSA_EVENT_TRANSFER_CMD_FAIL);
    }

    if dmac_status & MCIDMACIntEn::DU.bits() != 0 || 
        status & MCIRawInts::DCRC_BIT.bits() != 0 || 
        status & MCIRawInts::RCRC_BIT.bits() != 0
    {
        osa_event_set(SDMMC_OSA_EVENT_TRANSFER_DATA_FAIL);
    }
}

fn sdio_interrupt() {

}

pub fn register_dump(reg: &MCIReg) {
    warn!("cntrl: 0x{:x}", reg.read_reg::<MCICtrl>());
    warn!("pwren: 0x{:x}", reg.read_reg::<MCIPwrEn>());
    warn!("clkdiv: 0x{:x}", reg.read_reg::<MCIClkDiv>());
    warn!("clkena: 0x{:x}", reg.read_reg::<MCIClkEn>());
    warn!("tmout: 0x{:x}", reg.read_reg::<MCITimeout>());
    warn!("ctype: 0x{:x}", reg.read_reg::<MCICType>());
    warn!("blksz: 0x{:x}", reg.read_reg::<MCIBlkSiz>());
    warn!("blkcnt: 0x{:x}", reg.read_reg::<MCIBytCnt>());
    warn!("intmask: 0x{:x}", reg.read_reg::<MCIIntMask>());
    warn!("cmdarg: 0x{:x}", reg.read_reg::<MCICmdArg>());
    warn!("cmd: 0x{:x}", reg.read_reg::<MCICmd>());
    warn!("resp0: 0x{:x}", reg.read_reg::<MCIResp0>());
    warn!("reps1: 0x{:x}", reg.read_reg::<MCIResp1>());
    warn!("resp2: 0x{:x}", reg.read_reg::<MCIResp2>());
    warn!("resp3: 0x{:x}", reg.read_reg::<MCIResp3>());
    warn!("maskints: 0x{:x}", reg.read_reg::<MCIMaskedInts>());
    warn!("rawints: 0x{:x}", reg.read_reg::<MCIRawInts>());
    warn!("status: 0x{:x}", reg.read_reg::<MCIStatus>());
    warn!("fifoth: 0x{:x}", reg.read_reg::<MCIFifoTh>());
    warn!("carddet: 0x{:x}", reg.read_reg::<MCICardDetect>());
    warn!("wrtprt: 0x{:x}", reg.read_reg::<MCICardWrtp>());
    warn!("cksts: 0x{:x}", reg.read_reg::<MCIClkSts>());
    warn!("trans_cardcnt: 0x{:x}", reg.read_reg::<MCITranCardCnt>());
    warn!("trans_fifocnt: 0x{:x}", reg.read_reg::<MCITranFifoCnt>());
    warn!("debnce: 0x{:x}", reg.read_reg::<MCIDebnce>());
    warn!("uid: 0x{:x}", reg.read_reg::<MCIUid>());
    warn!("vid: 0x{:x}", reg.read_reg::<MCIVid>());
    warn!("hwconf: 0x{:x}", reg.read_reg::<MCIHwconf>());
    warn!("uhsreg: 0x{:x}", reg.read_reg::<MCIUhsReg>());
    warn!("cardreset: 0x{:x}", reg.read_reg::<MCICardReset>());
    warn!("busmode: 0x{:x}", reg.read_reg::<MCIBusMode>());
    warn!("descaddrl: 0x{:x}", reg.read_reg::<MCIDescListAddrL>());
    warn!("descaddrh: 0x{:x}", reg.read_reg::<MCIDescListAddrH>());
    warn!("dmacstatus: 0x{:x}", reg.read_reg::<MCIDMACStatus>());
    warn!("dmacinten: 0x{:x}", reg.read_reg::<MCIDMACIntEn>());
    warn!("curdescaddrl: 0x{:x}", reg.read_reg::<MCICurDescAddrL>());
    warn!("curdescaddrh: 0x{:x}", reg.read_reg::<MCIDescAddrH>());
    warn!("curbufaddrl: 0x{:x}", reg.read_reg::<MCICurBufAddrL>());
    warn!("curbufaddrh: 0x{:x}", reg.read_reg::<MCIBufAddrH>());
    warn!("card_thrctl: 0x{:x}", reg.read_reg::<MCICardThrctl>());
    warn!("clock_src: 0x{:x}", reg.read_reg::<MCIClkSrc>());
    warn!("emmcddr: 0x{:x}", reg.read_reg::<MCIEmmcDdrReg>());
    warn!("enableshift: 0x{:x}", reg.read_reg::<MCIEnableShift>());
}
