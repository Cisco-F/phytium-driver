use core::{num::NonZeroUsize, ptr::NonNull};

use lazy_static::lazy_static;
use log::{debug, error, warn};
use spin::Mutex;

use crate::{mci::{consts::FSdifEvtType, regs::{IrqTempRegister, MCICardDetect, MCIDMACIntEn, MCIDMACStatus, MCIIntMask, MCIRawInts}, MCI}, FSdifEvtHandler};

use super::consts::FGIC_LPI_START_ID;

pub const MAX_HANDLERS: usize = 1024;

pub type InterruptHandler = fn(u32, u32);

#[derive(Clone, Copy)]
struct IrqDesc {
    handler: Option<InterruptHandler>,
    param: Option<NonZeroUsize>,
}

impl IrqDesc {
    pub fn new() -> Self {
        Self {
            handler: None,
            param: None,
        }
    }
    pub fn handler_set(&mut self, handler: Option<InterruptHandler>) {
        self.handler = handler;
    }
    pub fn param_set(&mut self, param: Option<NonZeroUsize>) {
        self.param = param;
    }
}

// static mut ISR_TABLE: [IrqDesc; MAX_HANDLERS as usize] = [IrqDesc::new(); MAX_HANDLERS as usize];

lazy_static! {
    pub static ref ISR_TABLE: Mutex<IsrTable> =
        Mutex::new(IsrTable::new());
}

pub fn interrupt_install(int_id: isize, handler: Option<InterruptHandler>, param: Option<NonZeroUsize>) {
    if (int_id as usize) < MAX_HANDLERS {
        handler_set(handler, int_id as usize);
        param_set(param, int_id as usize);

    } else if int_id as u32 >= FGIC_LPI_START_ID {

    }
}

pub struct IsrTable {
    inner: [IrqDesc; MAX_HANDLERS],
}

impl IsrTable {
    pub fn new() -> Self {
        Self { 
            inner: [IrqDesc::new(); MAX_HANDLERS],
        }
    }
    fn handler_set(&mut self, handler: Option<InterruptHandler>, idx: usize) {
        self.inner[idx].handler = handler;
    }
    fn param_set(&mut self, param: Option<NonZeroUsize>, idx: usize) {
        self.inner[idx].param = param;
    }
}

pub fn handler_set(handler: Option<InterruptHandler>, idx: usize) {
    ISR_TABLE.lock().handler_set(handler, idx);
}

pub fn param_set(param: Option<NonZeroUsize>, idx: usize) {
    ISR_TABLE.lock().param_set(param, idx);
}

/// Interrupt handler for SDIF instance
// todo 从指针构建还是直接操作指针？
// pub fn fsdif_interrupt_handler(param: Option<NonZeroUsize>) {
//     if param.is_none() {
//         return;
//     }

//     let mci_ptr = param.unwrap().get() as *const MCI;
//     let mci = unsafe { &*mci_ptr };
//     let reg = mci.config().reg();

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
//         mci.call_event_handler(FSdifEvtType::SdioIrq, events.bits(), dmac_events.bits());
//     }

//     // handle card detect event
//     if events.bits() & event_mask.bits() & MCIRawInts::CD_BIT.bits() != 0 &&
//         !mci.config().non_removable() 
//     {
//         warn!("SD status changed here! status:[{}]", reg.read_reg::<MCICardDetect>().bits());
//         mci.call_event_handler(FSdifEvtType::CardDetected, events.bits(), dmac_events.bits());
//     }

//     // handle error state
//     if dmac_events.contains(MCIDMACStatus::DMAC_ERR_INTS_MASK) || 
//         events.contains(MCIRawInts::CMD_ERR_INTS_MASK)
//     {  
//         error!(
//             "Cmd index:{}, arg: 0x{:x}",
//             mci.cur_cmd_index(),
//             mci.cur_cmd_arg(),
//         );
//         error!(
//             "ERR: events: 0x{:x}, mask: 0x{:x}, dmac_evts: 0x{:x}, dmac_mask: {:x}",
//             events.bits(), event_mask.bits(), dmac_events.bits(), dmac_evt_mask.bits()
//         );
//         mci.call_event_handler(FSdifEvtType::ErrOccured, events.bits(), dmac_events.bits());
//     }

//     // handle cmd && data done
//     if events.contains(MCIRawInts::DTO_BIT) &&
//         events.contains(MCIRawInts::CMD_BIT)
//     {
//         warn!("cmd and data over!");
//         mci.call_event_handler(FSdifEvtType::CmdDone, events.bits(), dmac_events.bits());
//         mci.call_event_handler(FSdifEvtType::DataDone, events.bits(), dmac_events.bits());
//     } 
//     else if events.contains(MCIRawInts::CMD_BIT) ||
//         (events.contains(MCIRawInts::HTO_BIT) && mci.cur_cmd_index() == MCI::SWITCH_VOLTAGE as isize) // handle cmd done
//     {
//         warn!("cmd over!");
//         mci.call_event_handler(FSdifEvtType::CmdDone, events.bits(), dmac_events.bits());
//     }
//     else if events.contains(MCIRawInts::DTO_BIT) { // handle data done
//         warn!("data over!");
//         mci.call_event_handler(FSdifEvtType::DataDone, events.bits(), dmac_events.bits());
//     }
// }

impl MCI {
    // pub fn call_event_handler(&self, evt: FSdifEvtType, events: u32, dmac_events: u32) {
    //     if let Some(handler) = *self.evt_handler().get(evt as usize).unwrap() {
    //         match self.evt_args(evt) {
    //             Some(arg) => handler(arg, events, dmac_events),
    //             None => error!(
    //                 "no arg set for current event handler! type: {:?}, events: 0x{:x}, dmac_events: {:x}",
    //                 evt, events, dmac_events
    //             ),
    //         }
    //     }
    // }
    pub fn register_event_handler(&mut self, evt: FSdifEvtType, _handler: Option<FSdifEvtHandler>, param: NonNull<u8>) {
        self.evt_handler_set(evt, _handler, param);
    }
}