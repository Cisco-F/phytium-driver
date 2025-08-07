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
use crate::sd::reg_base;

use super::consts::*;
use super::regs::*;
use super::MCI;

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
    let reg = MCIReg::new(reg_base());

    let events = reg.read_reg::<MCIRawInts>();
    let dmac_events = reg.read_reg::<MCIDMACStatus>();

    let event_mask = reg.read_reg::<MCIIntMask>();
    let _dmac_evt_mask = reg.read_reg::<MCIDMACIntEn>();

    reg.write_reg::<MCIRawInts>(events);
    reg.write_reg::<MCIDMACStatus>(dmac_events);

    // no interrupt status
    if (events.bits() & MCIRawInts::ALL_BITS.bits() == 0)
        && (dmac_events.bits() & MCIDMACStatus::ALL_BITS.bits() == 0)
    {
        return;
    }

    reg.write_reg::<IrqTempRegister>(IrqTempRegister::from_bits_truncate(0));

    // no need to handle interrput
    if (events.bits() == 0) && (dmac_events.bits() & 0x1FFF == 0) {
        return;
    }

    // handle sdio irq
    if (events.bits() & event_mask.bits()) & MCIRawInts::SDIO_BIT.bits() != 0 {
        handle_sdio_interrupt();
        return;
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
    if dmac_events.contains(MCIDMACStatus::DMAC_ERR_INTS_MASK)
        || events.contains(MCIRawInts::CMD_ERR_INTS_MASK)
    {
        handle_error_occur(&reg, events.bits(), dmac_events.bits());
        return;
    }

    // handle cmd && data done
    if events.contains(MCIRawInts::DTO_BIT) && events.contains(MCIRawInts::CMD_BIT) {
        handle_cmd_done();
        handle_data_done(events.bits(), dmac_events.bits());
    } else if events.contains(MCIRawInts::CMD_BIT)
    // todo 这里无法得到MCI实例 暂时无法处理这种情况
    // (events.contains(MCIRawInts::HTO_BIT) && self.cur_cmd_index() == MCI::SWITCH_VOLTAGE as isize)
    {
        handle_cmd_done();
        return;
    // } else if events.contains(MCIRawInts::CMD_BIT) {
    //     // handle cmd done
    //     handle_cmd_done();
    } else if events.contains(MCIRawInts::DTO_BIT) {
        // handle data done
        handle_data_done(events.bits(), dmac_events.bits());
        return;
    }
}

#[allow(dead_code)]
fn handle_card_detected() {
    osa_event_set(SDMMC_OSA_EVENT_CARD_INSERTED);
}

pub fn handle_cmd_done() {
    osa_event_set(SDMMC_OSA_EVENT_TRANSFER_CMD_SUCCESS);
}

pub fn handle_data_done(status: u32, dmac_status: u32) {
    let check_status = status
        & (
            MCIRawInts::DTO_BIT      // Data transfer over
            | MCIRawInts::RCRC_BIT    // Response CRC error
            | MCIRawInts::DCRC_BIT    // Data CRC error
            | MCIRawInts::RE_BIT      // Response error
            | MCIRawInts::DRTO_BIT    // Data read timeout
            | MCIRawInts::EBE_BIT     // End-bit error
            | MCIRawInts::SBE_BCI_BIT // Start-bit error
            | MCIRawInts::RTO_BIT
            // Response timeout
        )
            .bits();
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
        }
    }
}

fn handle_error_occur(_reg: &MCIReg, status: u32, dmac_status: u32) {
    if status & MCIRawInts::RE_BIT.bits() != 0 || status & MCIRawInts::RTO_BIT.bits() != 0 {
        osa_event_set(SDMMC_OSA_EVENT_TRANSFER_CMD_FAIL);
    }

    if dmac_status & MCIDMACIntEn::DU.bits() != 0
        || status & MCIRawInts::DCRC_BIT.bits() != 0
        || status & MCIRawInts::RCRC_BIT.bits() != 0
    {
        osa_event_set(SDMMC_OSA_EVENT_TRANSFER_DATA_FAIL);
    }
}

fn handle_sdio_interrupt() {}
