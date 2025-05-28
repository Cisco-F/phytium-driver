#[allow(unused)]

mod constants;
mod mci_host_config;
pub mod mci_host_device;
mod mci_host_transfer;
pub mod mci_sdif;
mod err;
pub mod sd;
mod mci_card_base;
mod mci_host_card_detect;

use core::{cell::Cell, ptr::NonNull};

use alloc::{boxed::Box, rc::Rc, sync::{Arc, Weak}};

use bare_test::{driver::intc::{IrqConfig, Trigger}, irq::{IrqHandleResult, IrqParam}};
use constants::*;
use err::{MCIHostError, MCIHostStatus};
use mci_host_card_detect::MCIHostCardDetect;
use mci_host_config::MCIHostConfig;
pub use mci_host_device::MCIHostDevice;
use mci_host_transfer::{MCIHostCmd, MCIHostTransfer};
pub use mci_host_device::FSdifEvtHandler;

type MCIHostCardIntFn = fn();

#[allow(unused)]
pub struct MCIHost {
    pub(crate) dev: Arc<Box<dyn MCIHostDevice>>,
    pub(crate) config: MCIHostConfig,                  
    pub(crate) curr_voltage: Cell<MCIHostOperationVoltage>,  
    pub(crate) curr_bus_width: u32,                    
    pub(crate) curr_clock_freq: Cell<u32>,                   

    pub(crate) source_clock_hz: u32,                   
    pub(crate) capability: MCIHostCapability,          
    pub(crate) max_block_count: Cell<u32>,                   
    pub(crate) max_block_size: u32,                    
    pub(crate) tuning_type: u8,                        

    pub(crate) cd: Option<Rc<MCIHostCardDetect>>,         // 卡检测
    pub(crate) card_int: MCIHostCardIntFn,

    //todo 这里 uint8_t tuningType sdmmc_osa_event_t hostEvent sdmmc_osa_mutex_t lock 都没有移植
    // pub(crate) hc_evt: OSAEvent,
}

#[allow(unused)]
impl MCIHost {
    pub(crate) fn new(dev: Box<dyn MCIHostDevice>, config: MCIHostConfig) -> Self {
        MCIHost {
            dev: Arc::new(dev),
            config,
            curr_voltage: Cell::new(MCIHostOperationVoltage::None),
            curr_bus_width: 0,
            curr_clock_freq: Cell::new(0),
            source_clock_hz: 0,
            capability: MCIHostCapability::empty(),
            max_block_count: Cell::new(0),
            max_block_size: 0,
            tuning_type: 0,
            cd: None,
            card_int: || {},
        }
    }

    // pub fn weak(&self) -> MCIHostWeak {
    //     MCIHostWeak {
    //         dev: Arc::downgrade(&self.dev),
    //         config: self.config.clone(),
    //         curr_voltage: self.curr_voltage.clone(),
    //         curr_bus_width: self.curr_bus_width,
    //         curr_clock_freq: self.curr_clock_freq.clone(),
    //         source_clock_hz: self.source_clock_hz,
    //         capability: self.capability.clone(),
    //         max_block_count: self.max_block_count.clone(),
    //         max_block_size: self.max_block_size,
    //         tuning_type: self.tuning_type,
    //         cd: self.cd.clone(),
    //         card_int: self.card_int,
    //     }
    // }

    pub(crate) fn card_select(&self,relative_address:u32,is_selected:bool) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();

        command.index_set(MCIHostCommonCmd::SelectCard as u32);
        if is_selected {
            command.argument_set(relative_address << 16);
            command.response_type_set(MCIHostResponseType::R1);
        } else {
            command.argument_set(0);
            command.response_type_set(MCIHostResponseType::None);
        }

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));

        let err = self.dev.transfer_function(&mut content,self);
        
        let command = content.cmd().unwrap();
        let response = command.response();

        if err.is_err() || response[0] & MCIHostCardStatusFlag::ALL_ERROR_FLAG.bits() != 0{
            return Err(MCIHostError::TransferFailed);
        }

        Ok(())
    }

    pub(crate) fn application_command_send(&self, relative_address: u32) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();
    
        command.index_set(MCIHostCommonCmd::ApplicationCommand as u32);
        command.argument_set(relative_address << 16);
        command.response_type_set(MCIHostResponseType::R1);
    
        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
    
        let err = self.dev.transfer_function(&mut content,self);
        
        let command = content.cmd().unwrap();
        let response = command.response();
    
        if err.is_err() || response[0] & MCIHostCardStatusFlag::ALL_ERROR_FLAG.bits() != 0 {
            return Err(MCIHostError::TransferFailed);
        }
    
        if response[0] & MCIHostCardStatusFlag::APPLICATION_COMMAND.bits() == 0 {
            return Err(MCIHostError::CardNotSupport);
        }
    
        Ok(())
    }
 
    pub(crate) fn block_count_set(&self,block_count:u32) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();

        command.index_set(MCIHostCommonCmd::SetBlockCount as u32);
        command.argument_set(block_count);
        command.response_type_set(MCIHostResponseType::R1);

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
        
        let err = self.dev.transfer_function(&mut content,self);

        let command = content.cmd().unwrap();
        let response = command.response();

        if err.is_err() || response[0] & MCIHostCardStatusFlag::ALL_ERROR_FLAG.bits() != 0{
            return Err(MCIHostError::TransferFailed);
        }

        Ok(())
    }

    pub(crate) fn go_idle(&self) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();
    
        command.index_set(MCIHostCommonCmd::GoIdleState as u32);
        
        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
        
        let err = self.dev.transfer_function(&mut content,self);
        
        if err.is_err() {
            return Err(MCIHostError::TransferFailed);
        }
        
        Ok(())
    }

    pub(crate) fn block_size_set(&self, block_size: u32) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();
    
        command.index_set(MCIHostCommonCmd::SetBlockLength as u32);
        command.argument_set(block_size);
        command.response_type_set(MCIHostResponseType::R1);
    
        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
        
        let err = self.dev.transfer_function(&mut content,self);
        
        let command = content.cmd().unwrap();
        let response = command.response();
    
        if err.is_err() || response[0] & MCIHostCardStatusFlag::ALL_ERROR_FLAG.bits() != 0 {
            return Err(MCIHostError::TransferFailed);
        }
    
        Ok(())
    }
    
    pub(crate) fn card_inactive_set(&self) -> MCIHostStatus {
        let mut command = MCIHostCmd::new();
    
        command.index_set(MCIHostCommonCmd::GoInactiveState as u32);
        command.argument_set(0);
        command.response_type_set(MCIHostResponseType::None);
    
        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
        
        let err = self.dev.transfer_function(&mut content, self);
        
        if err.is_err() {
            return Err(MCIHostError::TransferFailed);
        }
    
        Ok(())
    }

    pub(crate) fn init(&mut self, addr: NonNull<u8>) -> MCIHostStatus {
        self.dev.init(addr,self)
    }

    pub fn setup_irq(&mut self) -> Result<(), &'static str> {
        let irq_num = 104;
        let dev_arc = Arc::clone(&self.dev);
        IrqParam {
            intc: 0.into(),
            cfg: IrqConfig {
                irq: (irq_num as usize).into(),
                trigger: Trigger::LevelHigh,
            }
        }
        .register_builder({
            move |_irq| {
                dev_arc.fsdif_interrupt_handler();
                IrqHandleResult::Handled
            }
        })
        .register();

        Ok(())
    }
    pub fn fsdif_interrupt_handler(&self) {
        self.dev.as_ref().fsdif_interrupt_handler();
    }
}

#[allow(unused)]
impl MCIHost {
    // todo 将 dev 的操作套壳
}

// pub struct MCIHostWeak {
//     pub(crate) dev: Weak<Box<dyn MCIHostDevice>>,
//     pub(crate) config: MCIHostConfig,                  
//     pub(crate) curr_voltage: Cell<MCIHostOperationVoltage>,  
//     pub(crate) curr_bus_width: u32,                    
//     pub(crate) curr_clock_freq: Cell<u32>,                   

//     pub(crate) source_clock_hz: u32,                   
//     pub(crate) capability: MCIHostCapability,          
//     pub(crate) max_block_count: Cell<u32>,                   
//     pub(crate) max_block_size: u32,                    
//     pub(crate) tuning_type: u8,                        

//     pub(crate) cd: Option<Rc<MCIHostCardDetect>>,         // 卡检测
//     pub(crate) card_int: MCIHostCardIntFn,
// }

// impl MCIHostWeak {
//     pub fn upgrade(&self) -> Option<MCIHost> {
//         self.dev.upgrade().map(|dev| MCIHost {
//             dev,
//             config: self.config.clone(),
//             curr_voltage: self.curr_voltage.clone(),
//             curr_bus_width: self.curr_bus_width,
//             curr_clock_freq: self.curr_clock_freq.clone(),
//             source_clock_hz: self.source_clock_hz,
//             capability: self.capability.clone(),
//             max_block_count: self.max_block_count.clone(),
//             max_block_size: self.max_block_size,
//             tuning_type: self.tuning_type,
//             cd: self.cd.clone(),
//             card_int: self.card_int,
//         })
//     }
// }

