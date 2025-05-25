use log::debug;
use log::error;
use log::warn;

use crate::mci_sdif::sdif_device::SDIFDev;

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
    // pub fn fsdif_interrupt_handler(&mut self) {
    //     let reg = self.config().reg();

    //     let events = reg.read_reg::<MCIRawInts>();
    //     let dmac_events = reg.read_reg::<MCIDMACStatus>();
    //     let event_mask = reg.read_reg::<MCIIntMask>();
    //     let dmac_evt_mask = reg.read_reg::<MCIDMACIntEn>();

    //     // no interrupt status
    //     if !events.contains(MCIRawInts::ALL_BITS) && !dmac_events.contains(MCIDMACStatus::ALL_BITS) {
    //         warn!("irq exit with no action");
    //         return;
    //     }

    //     reg.write_reg::<IrqTempRegister>(IrqTempRegister::from_bits_truncate(0));

    //     // no need to handle interrput
    //     if (events.bits() == 0) && !dmac_events.contains(MCIDMACStatus::from_bits_truncate(0x1FFF)) {
    //         return;
    //     }

    //     debug!("events:0x{:x},mask:0x{:x},dmac_events:{:x},dmac_mask:0x{:x}", events, event_mask, dmac_events, dmac_evt_mask);

    //     // Clear interrupt status
    //     let events_copy = reg.read_reg::<MCIRawInts>();
    //     let dmac_events_copy = reg.read_reg::<MCIDMACStatus>();
    //     reg.write_reg::<MCIRawInts>(events_copy);
    //     reg.write_reg::<MCIDMACStatus>(dmac_events_copy);

    //     // handle sdio irq
    //     if (events.bits() & event_mask.bits()) & MCIRawInts::SDIO_BIT.bits() != 0 {
    //         warn!("SDIO interrupt here!");
    //         self.call_event_handler(FSdifEvtType::SdioIrq, events.bits(), dmac_events.bits());
    //     }

    //     // handle card detect event
    //     if events.bits() & event_mask.bits() & MCIRawInts::CD_BIT.bits() != 0 &&
    //         !self.config().non_removable() 
    //     {
    //         warn!("SD status changed here! status:[{}]", reg.read_reg::<MCICardDetect>().bits());
    //         self.call_event_handler(FSdifEvtType::CardDetected, events.bits(), dmac_events.bits());
    //     }

    //     // handle error state
    //     if dmac_events.contains(MCIDMACStatus::DMAC_ERR_INTS_MASK) || 
    //         events.contains(MCIRawInts::CMD_ERR_INTS_MASK)
    //     {  
    //         error!(
    //             "Cmd index:{}, arg: 0x{:x}",
    //             self.cur_cmd_index(),
    //             self.cur_cmd_arg(),
    //         );
    //         error!(
    //             "ERR: events: 0x{:x}, mask: 0x{:x}, dmac_evts: 0x{:x}, dmac_mask: {:x}",
    //             events.bits(), event_mask.bits(), dmac_events.bits(), dmac_evt_mask.bits()
    //         );
    //         self.call_event_handler(FSdifEvtType::ErrOccured, events.bits(), dmac_events.bits());
    //     }

    //     // handle cmd && data done
    //     if events.contains(MCIRawInts::DTO_BIT) &&
    //         events.contains(MCIRawInts::CMD_BIT)
    //     {
    //         warn!("cmd and data over!");
    //         self.call_event_handler(FSdifEvtType::CmdDone, events.bits(), dmac_events.bits());
    //         self.call_event_handler(FSdifEvtType::DataDone, events.bits(), dmac_events.bits());
    //     } 
    //     else if events.contains(MCIRawInts::CMD_BIT) ||
    //         (events.contains(MCIRawInts::HTO_BIT) && self.cur_cmd_index() == MCI::SWITCH_VOLTAGE as isize) // handle cmd done
    //     {
    //         warn!("cmd over!");
    //         self.call_event_handler(FSdifEvtType::CmdDone, events.bits(), dmac_events.bits());
    //     }
    //     else if events.contains(MCIRawInts::DTO_BIT) { // handle data done
    //         warn!("data over!");
    //         self.call_event_handler(FSdifEvtType::DataDone, events.bits(), dmac_events.bits());
    //     }
    // }

    // fn card_detected(&mut self) {
    //     let evt_type = FSdifEvtType::CardDetected as usize;
    //     let dev = match &self.evt_args[evt_type] {
    //         Some(addr) => unsafe { NonNull::new_unchecked(addr.as_ptr() as *mut SDIFDev).as_ref() },
    //         None => panic!(),
    //     };
    // }
}