#[allow(unused)]

mod constants;
mod mci_host_config;
mod mci_host_transfer;
pub mod mci_sdif;
mod err;
pub mod sd;
mod mci_card_base;
mod mci_host_card_detect;

use core::{cell::Cell, ptr::NonNull};

use alloc::{boxed::Box, rc::Rc};

use constants::*;
use err::{MCIHostError, MCIHostStatus};
use mci_host_card_detect::MCIHostCardDetect;
use mci_host_config::MCIHostConfig;
use mci_host_transfer::{MCIHostCmd, MCIHostTransfer};
use mci_sdif::sdif_device::SDIFDev;

type MCIHostCardIntFn = fn();

#[allow(unused)]
pub struct MCIHost {
    pub(crate) dev: Box<SDIFDev>,
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
    pub(crate) fn new(dev: Box<SDIFDev>, config: MCIHostConfig) -> Self {
        MCIHost {
            dev,
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

}
