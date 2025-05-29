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

    /// Interrupt handler for SDIF instance
    pub fn fsdif_interrupt_handler(&mut self) {
        let reg = self.config().reg();

        let events = reg.read_reg::<MCIRawInts>();
        let dmac_events = reg.read_reg::<MCIDMACStatus>();
        let event_mask = reg.read_reg::<MCIIntMask>();
        let dmac_evt_mask = reg.read_reg::<MCIDMACIntEn>();

        // no interrupt status
        if !events.contains(MCIRawInts::ALL_BITS) && !dmac_events.contains(MCIDMACStatus::ALL_BITS) {
            warn!("irq exit with no action");
            return;
        }

        reg.write_reg::<IrqTempRegister>(IrqTempRegister::from_bits_truncate(0));

        // no need to handle interrput
        if (events.bits() == 0) && !dmac_events.contains(MCIDMACStatus::from_bits_truncate(0x1FFF)) {
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
            self.sdio_interrupt();
        }

        // handle card detect event
        if events.bits() & event_mask.bits() & MCIRawInts::CD_BIT.bits() != 0 &&
            !self.config().non_removable() 
        {
            warn!("SD status changed here! status:[{}]", reg.read_reg::<MCICardDetect>().bits());
            self.card_detected();
        }

        // handle error state
        if dmac_events.contains(MCIDMACStatus::DMAC_ERR_INTS_MASK) || 
            events.contains(MCIRawInts::CMD_ERR_INTS_MASK)
        {  
            error!(
                "Cmd index:{}, arg: 0x{:x}",
                self.cur_cmd_index(),
                self.cur_cmd_arg(),
            );
            error!(
                "ERR: events: 0x{:x}, mask: 0x{:x}, dmac_evts: 0x{:x}, dmac_mask: {:x}",
                events.bits(), event_mask.bits(), dmac_events.bits(), dmac_evt_mask.bits()
            );
            self.error_occur(events.bits(), dmac_events.bits());
        }

        // handle cmd && data done
        if events.contains(MCIRawInts::DTO_BIT) &&
            events.contains(MCIRawInts::CMD_BIT)
        {
            warn!("cmd and data over!");
            self.cmd_done();
            self.data_done(events.bits(), dmac_events.bits());
        } 
        else if events.contains(MCIRawInts::CMD_BIT) ||
            (events.contains(MCIRawInts::HTO_BIT) && self.cur_cmd_index() == MCI::SWITCH_VOLTAGE as isize) // handle cmd done
        {
            warn!("cmd over!");
            self.cmd_done();
        }
        else if events.contains(MCIRawInts::DTO_BIT) { // handle data done
            warn!("data over!");
            self.data_done(events.bits(), dmac_events.bits());
        }
    }

    fn card_detected(&self) {
        let dev = match &self.evt_arg {
            Some(addr) => unsafe { NonNull::new_unchecked(addr.as_ptr() as *mut SDIFDev).as_ref() },
            None => panic!(),
        };
        dev.event_set(SDMMC_OSA_EVENT_CARD_INSERTED);
    }

    fn cmd_done(&self) {
        let dev = match &self.evt_arg {
            Some(addr) => unsafe { NonNull::new_unchecked(addr.as_ptr() as *mut SDIFDev).as_ref() },
            None => panic!(),
        };
        dev.event_set(SDMMC_OSA_EVENT_TRANSFER_CMD_SUCCESS);
    }

    fn data_done(&self, status: u32, dmac_status: u32) {
        let dev = match &self.evt_arg {
            Some(addr) => unsafe { NonNull::new_unchecked(addr.as_ptr() as *mut SDIFDev).as_ref() },
            None => panic!(),
        };

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

        if !dev.whether_transfer_data() {
            dev.event_set(SDMMC_OSA_EVENT_TRANSFER_DATA_SUCCESS);
        } else if check_status | check_dmac != 0 {
            if check_status & MCIRawInts::DTO_BIT.bits() != 0 {
                dev.event_set(SDMMC_OSA_EVENT_TRANSFER_DATA_SUCCESS);
            } else {
                error!("transfer data error: 0x{:x}, dmac status: 0x{:x}", check_status, check_dmac);
            }
        }
    }

    fn error_occur(&self, status: u32, dmac_status: u32) {
        let dev = match &self.evt_arg {
            Some(addr) => unsafe { NonNull::new_unchecked(addr.as_ptr() as *mut SDIFDev).as_ref() },
            None => panic!(),
        };

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

        self.register_dump();

        if status & MCIRawInts::RE_BIT.bits() != 0 ||
            status & MCIRawInts::RTO_BIT.bits() != 0 
        {
            dev.event_set(SDMMC_OSA_EVENT_TRANSFER_CMD_FAIL);
        }

        if dmac_status & MCIDMACIntEn::DU.bits() != 0 || 
            status & MCIRawInts::DCRC_BIT.bits() != 0 || 
            status & MCIRawInts::RCRC_BIT.bits() != 0
        {
            dev.event_set(SDMMC_OSA_EVENT_TRANSFER_DATA_FAIL);
        }
    }

    fn sdio_interrupt(&self) {

    }
}