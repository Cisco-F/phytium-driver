use core::cell::{Cell, RefCell};
use core::ptr::NonNull;
use core::time::Duration;

use alloc::vec::Vec;
use log::*;

use crate::mci::regs::MCIIntMask;
use crate::mci::mci_data::MCIData;
use crate::mci::{MCICmdData, MCIConfig, MCI};
use crate::mci_host::mci_host_card_detect::MCIHostCardDetect;
use crate::mci_host::mci_host_config::*;
use crate::mci_host::mci_host_transfer::MCIHostTransfer;
use crate::mci_host::MCIHostCardIntFn;
use crate::{sleep, IoPad};
use crate::tools::swap_half_word_byte_sequence_u32;
use crate::mci_host::mci_host_device::MCIHostDevice;
use super::constants::SDStatus;
use super::MCIHost;
use crate::mci_host::err::*;
use crate::mci_host::constants::*;
use crate::mci::constants::*;
use crate::mci_host::sd::constants::SdCmd;


pub(crate) struct SDIFDevPIO {
    hc: RefCell<MCI>,                            // SDIF 硬件控制器
    hc_cfg: RefCell<MCIConfig>,                  // SDIF 配置
    //rw_desc: *mut FSdifIDmaDesc,          // DMA 描述符指针，用于管理数据传输
    desc_num: Cell<u32>,                        // 描述符数量，表示 DMA 描述符的数量
}

impl SDIFDevPIO {
    pub fn new(addr: NonNull<u8>) -> Self {
        Self {
            hc: MCI::new(MCIConfig::new(addr)).into(),
            hc_cfg: MCIConfig::new(addr).into(),
            desc_num: 0.into(),
        }
    }
    pub fn iopad_set(&self,iopad:IoPad) {
        self.hc.borrow_mut().iopad_set(iopad);
    }
}

impl MCIHostDevice for SDIFDevPIO {

    fn init(&self, addr: NonNull<u8>,host:&MCIHost) -> MCIHostStatus {
        let num_of_desc = host.config.max_trans_size/host.config.def_block_size;
        self.desc_num.set(num_of_desc as u32);
        self.do_init(addr,host)
    }

    fn do_init(&self,addr: NonNull<u8>,host:&MCIHost) -> MCIHostStatus {
        let id = host.config.host_id;

        let mci_config = MCIConfig::lookup_config(addr, id);
        *self.hc.borrow_mut() = MCI::new(MCIConfig::lookup_config(addr, id));

        if let Err(_) = self.hc.borrow_mut().config_init(&mci_config) {
            info!("Sdio ctrl init failed.");
            return Err(MCIHostError::Fail);
        }

        if host.config.enable_irq {
            // todo
        }

        if host.config.enable_dma {
            // todo
        }

        *self.hc_cfg.borrow_mut() = mci_config;
        Ok(())
    }

    fn deinit(&self) {
        // todo FSDIFHOST_RevokeIrq
        let _ = self.hc.borrow_mut().config_deinit();
        info!("Sdio ctrl deinited !!!")
    }
    
    fn reset(&self) -> MCIHostStatus {
        match self.hc.borrow_mut().restart() {
            Ok(_) => Ok(()),
            Err(_) => Err(MCIHostError::Fail),
        }
    }

    fn switch_to_voltage(&self, voltage: MCIHostOperationVoltage,host:&MCIHost) -> MCIHostStatus {
        match voltage {
            MCIHostOperationVoltage::Voltage300V => {
                host.curr_voltage.set(voltage);
                self.hc.borrow_mut().voltage_1_8v_set(false);
                info!("Switch to 3.0V");
            },
            MCIHostOperationVoltage::Voltage330V => {
                host.curr_voltage.set(voltage);
                self.hc.borrow_mut().voltage_1_8v_set(false);
                info!("Switch to 3.0V");
            },
            MCIHostOperationVoltage::Voltage180V => {
                host.curr_voltage.set(voltage);
                self.hc.borrow_mut().voltage_1_8v_set(true);
                info!("Switch to 1.8V");
            },
            _ => {
                info!("Invalid target voltage !!!");
            }
        }
        Ok(())
    }

    fn execute_tuning(&self, _tuning_cmd: u32, _rev_buf: &mut Vec<u32>, _block_size: u32) -> MCIHostStatus {
        Ok(())
    }

    fn enable_ddr_mode(&self, _enable: bool, _nibble_pos: u32) {
        // todo  暂时还没有实现
    }

    fn enable_hs400_mode(&self, _enable: bool) {
        info!("Enable HS400 mode Not Implemented !!!");
    }

    fn enable_strobe_dll(&self, _enable: bool) {
        info!("Enable Strobe DLL Not Implemented !!!");
    }

    fn get_signal_line_status(&self, _signal_line: u32) -> bool {
        !self.hc.borrow().check_if_card_busy()
    }

    fn convert_data_to_little_endian(&self, data: &mut Vec<u32>, word_size: usize, format: MCIHostDataPacketFormat,host:&MCIHost) -> MCIHostStatus {
        if host.config.endian_mode == MCIHostEndianMode::Big && 
             format == MCIHostDataPacketFormat::MSBFirst {
            for i in 0..word_size {
                let val = data[i];
                data[i] = val.swap_bytes();
            }
        } else if host.config.endian_mode == MCIHostEndianMode::HalfWordBig {
            for i in 0..word_size {
                let val = data[i];
                data[i] = swap_half_word_byte_sequence_u32(val);
            }
        } else if host.config.endian_mode == MCIHostEndianMode::Big &&
            format == MCIHostDataPacketFormat::LSBFirst {
            for i in 0..word_size {
                let val = data[i];
                data[i] = val.swap_bytes();
            }
        }   
        Ok(())
     }

    fn card_detect_init(&self, _cd: &MCIHostCardDetect) -> MCIHostStatus {
         Ok(())
    }

    fn card_power_set(&self, _enable: bool) {
        
    }

    fn card_int_enable(&self, enable: bool, host:&MCIHost) -> MCIHostStatus {
        if MCIHostCardType::SDIO == host.config.card_type {
            self.hc.borrow().interrupt_mask_set(MCIIntrType::GeneralIntr, MCIIntMask::SDIO_BIT.bits(), enable);
        }
        Ok(())
    }

    fn card_int_init(&self, _sdio_int: &MCIHostCardIntFn) ->MCIHostStatus {
        Ok(())
    }

    fn card_bus_width_set(&self, data_bus_width: MCIHostBusWdith) {
        match data_bus_width {
            MCIHostBusWdith::Bit1 => {
                self.hc.borrow().bus_width_set(data_bus_width as u32);
                info!("Set bus width to 1 bit");
            },
            MCIHostBusWdith::Bit4 => {
                self.hc.borrow().bus_width_set(data_bus_width as u32);
                info!("Set bus width to 4 bit");
            },
            MCIHostBusWdith::Bit8 => {
                self.hc.borrow().bus_width_set(data_bus_width as u32);
                info!("Set bus width to 8 bit");
            },
        }
    }

    fn card_detect_status_polling(&self, wait_card_status: SDStatus, _timeout: u32, host:&MCIHost) -> MCIHostStatus {
        let binding = host.cd.as_ref().ok_or(MCIHostError::NoData)?;
        let cd = binding.borrow();
        let mut retry_times:usize = 100;

        /* Wait card inserted. */
        loop {
            let is_card_inserted = self.card_detect_status() == SDStatus::Inserted;
            sleep(Duration::from_millis(cd.cd_debounce_ms() as u64));
            if wait_card_status == SDStatus::Inserted && is_card_inserted {
                break;
            }

            if wait_card_status == SDStatus::Removed && !is_card_inserted {
                break;
            }

            if retry_times == 0 {
                info!("Wait card insert timeout !!!");
                return Err(MCIHostError::Timeout);
            }

            retry_times -= 1;
        }
        Ok(())
    }

    fn card_detect_status(&self) -> SDStatus {
        if self.hc.borrow().check_if_card_exist() {
            SDStatus::Inserted
        } else {
            SDStatus::Removed
        }
    }

    fn card_active_send(&self) {
        
    }

    fn card_clock_set(&self, target_clock: u32, host:&MCIHost) -> u32 {
            
        // 如果当前时钟频率已经是目标频率，则直接返回
        if host.curr_clock_freq.get() == target_clock {
            return host.curr_clock_freq.get();
        }
        
        // 尝试设置时钟频率
        if self.hc.borrow_mut().clk_freq_set(target_clock).is_ok() {
            info!("BUS CLOCK: {}", target_clock);
            // 更新实例的时钟频率
            host.curr_clock_freq.set(target_clock);
        } else {
            info!("Failed to update clock");
        }
        
        host.curr_clock_freq.get()
    }

    fn force_clock_on(&self, enable: bool) {
        self.hc.borrow().clock_set(enable);
    }

    fn card_is_busy(&self) -> bool {
        self.hc.borrow().check_if_card_busy()
    }

    fn pre_command(&self,content: &mut MCIHostTransfer, host:&MCIHost) -> MCIHostStatus {
        let cmd = match content.cmd() {
            Some(cmd) => cmd,
            None => return Err(MCIHostError::NoData)
        };

        let data = match content.data() {
            Some(data) => data,
            None => return Err(MCIHostError::NoData)
        };

        if cmd.index() == MCIHostCommonCmd::ReadMultipleBlock as u32 ||
            cmd.index() == MCIHostCommonCmd::WriteMultipleBlock as u32 {
           let block_count = data.block_count();

           if block_count > 1 {
                host.max_block_count.set(block_count);
           }
        }
        Ok(())
    }

    fn covert_command_info(&self, in_trans: &mut MCIHostTransfer) -> MCICmdData {
        let in_cmd = match in_trans.cmd() {
            Some(cmd) => cmd,
            None => panic!("Not Inited intrans"),
        };

        let index = in_cmd.index();
        let arg: u32 = in_cmd.argument();
        let mut flag = MCICmdFlag::empty();

        if index == MCIHostCommonCmd::GoIdleState as u32 {
            flag |= MCICmdFlag::NEED_INIT;
        }

        if index == MCIHostCommonCmd::GoInactiveState as u32 || 
            (index == MCISDIOCommand::RWIODirect as u32 && 
            (arg >> 9 & 0x1FFFF) == MCISDIOCCCRAddr::IOAbort as u32 ){
            flag |= MCICmdFlag::ABORT;
        }

        let response_type = in_cmd.response_type();

        if response_type != MCIHostResponseType::None {
            flag |= MCICmdFlag::EXP_RESP;
            if response_type == MCIHostResponseType::R2 {
                /* need 136 bits long response */
                flag |= MCICmdFlag::EXP_LONG_RESP;
            }

            if response_type != MCIHostResponseType::R3 && 
                response_type != MCIHostResponseType::R4 {
                /* most cmds need CRC */
                flag |= MCICmdFlag::NEED_RESP_CRC;
            }
        }

        if index == SdCmd::VoltageSwitch as u32 {
            /* CMD11 need switch voltage */
            flag |= MCICmdFlag::SWITCH_VOLTAGE;
        }

        let mut out_data = MCIData::new();

        if let Some(in_data) = in_trans.data() {
            flag |= MCICmdFlag::EXP_DATA;
            
            let buf = if let Some(rx_data) = in_data.rx_data() {
                // Handle receive data
                flag |= MCICmdFlag::READ_DATA;
                //TODO 这里的CLONE 会降低驱动速度,需要解决这个性能问题
                rx_data.clone()
            } else if let Some(tx_data) = in_data.tx_data() {
                // Handle transmit data
                flag |= MCICmdFlag::WRITE_DATA;
                tx_data.clone()
            } else {
                // Neither rx_data nor tx_data is available
                panic!("Transaction data initialized but contains neither rx_data nor tx_data");
            };
            
            out_data.buf_set(Some(buf));
        }

        let mut out_trans = MCICmdData::new();

        out_trans.cmdidx_set(index);
        out_trans.cmdarg_set(arg);
        out_trans.set_data(Some(out_data));
        out_trans.flag_set(flag);
        
        out_trans

    }

    fn transfer_function(&self,content: &mut MCIHostTransfer, host:&MCIHost) -> MCIHostStatus {
        
        self.pre_command(content,host)?;

        let mut cmd_data = MCICmdData::new();
        let trans_data = MCIData::new();

        if let Some(_) = content.data() {
            cmd_data.set_data(Some(trans_data));
        }

        let mut cmd_data = self.covert_command_info(content);

        if host.config.enable_dma {
            // todo
        }else {

            if let Err(_) = self.hc.borrow_mut().pio_transfer(&mut cmd_data) {
                return Err(MCIHostError::NoData);
            }

            if let Err(_) = self.hc.borrow_mut().poll_wait_pio_end(&mut cmd_data) {
                return Err(MCIHostError::NoData);
            }
        }

        if let Err(_) = self.hc.borrow_mut().cmd_response_get(&mut cmd_data) {
            info!("Transfer cmd and data failed !!!");
            return Err(MCIHostError::Timeout);
        }

        if let Some(cmd) = content.cmd_mut() {
            if cmd.response_type() != MCIHostResponseType::None {
                cmd.response_mut().copy_from_slice(&cmd_data.get_response()[..4]);
            }
        }

        Ok(())
    }
}