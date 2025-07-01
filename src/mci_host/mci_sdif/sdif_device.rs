#![allow(dead_code)]

use core::cell::{Cell, RefCell};
use core::mem::take;
use core::ptr::NonNull;
use core::time::Duration;

use alloc::vec::Vec;
use log::*;

use crate::aarch::dsb;
use crate::mci::regs::MCIIntMask;
use crate::mci::mci_data::MCIData;
use crate::mci::{MCICmdData, MCIConfig, MCI};
use crate::mci_host::mci_host_card_detect::MCIHostCardDetect;
use crate::mci_host::mci_host_config::*;
use crate::mci_host::mci_host_transfer::MCIHostTransfer;
use crate::mci_host::MCIHostCardIntFn;
use crate::osa::OSAEvent;
use crate::osa::pool_buffer::PoolBuffer;
use crate::sd::consts::SD_BLOCK_SIZE;
use crate::{flush, mmap, sleep, IoPad};
use crate::tools::swap_half_word_byte_sequence_u32;
use super::consts::SDStatus;
use super::MCIHost;
use crate::mci_host::err::*;
use crate::mci_host::constants::*;
use crate::mci::consts::*;
use crate::mci_host::sd::consts::SdCmd;
use crate::mci::mci_dma::FSdifIDmaDesc;

pub(crate) struct SDIFDev {
    /// SDIF 硬件控制器
    hc: RefCell<MCI>,
    /// SDIF 配置
    hc_cfg: RefCell<MCIConfig>,
    /// DMA 描述符指针，用于管理数据传输
    rw_desc: PoolBuffer,
    /// 描述符数量，表示 DMA 描述符的数量
    desc_num: Cell<u32>,
    /// 记录信号量，用于中断处理中的线程同步
    hc_evt: RefCell<OSAEvent>,
}

impl SDIFDev {
    pub fn new(addr: NonNull<u8>, desc_num: usize) -> Self {
        let align = SD_BLOCK_SIZE;
        let size = core::mem::size_of::<FSdifIDmaDesc>() * desc_num;
        let rw_desc = match PoolBuffer::new(size, align) {
            Err(e) => {
                panic!("alloc internal buffer failed! err: {:?}", e);
            }
            Ok(buffer) => buffer,
        };
        debug!(
            "rw_desc buffer at {:x}, pa {:x}", 
            rw_desc.addr().as_ptr() as usize, 
            mmap(rw_desc.addr())
        );

        Self {
            hc: MCI::new(MCIConfig::new(addr)).into(),
            hc_cfg: MCIConfig::new(addr).into(),
            rw_desc,
            desc_num: (desc_num as u32).into(),
            hc_evt: RefCell::new(OSAEvent::default()),
        }
    }
    pub fn iopad_set(&self,iopad:IoPad) {
        self.hc.borrow_mut().iopad_set(iopad);
    }
}

/// Auxilary functions
impl SDIFDev {
    pub fn whether_transfer_data(&self) -> bool {
        self.hc.borrow().cur_cmd().as_ref().unwrap().get_data().is_some()
    }
}

impl SDIFDev {
    pub fn init(&self, addr: NonNull<u8>,host:&MCIHost) -> MCIHostStatus {
        let num_of_desc = host.config.max_trans_size/host.config.def_block_size;
        self.desc_num.set(num_of_desc as u32);
        self.do_init(addr,host)?;
        Ok(())
    }

    fn do_init(&self,addr: NonNull<u8>,host:&MCIHost) -> MCIHostStatus {
        info!("dev do init");
        let mci_config = MCIConfig::lookup_config(addr);
        let iopad = self.hc.borrow_mut().iopad_take().ok_or(MCIHostError::NoData)?;

        *self.hc.borrow_mut() = MCI::new(MCIConfig::lookup_config(addr));
        self.hc.borrow_mut().iopad_set(iopad);
        
        // 强行 restart 一下
        let restart_mci = MCI::new_restart(MCIConfig::restart(addr));
        restart_mci.restart().unwrap_or_else(|e| error!("restart failed: {:?}", e));

        if let Err(_) = self.hc.borrow_mut().config_init(&mci_config) {
            info!("Sdio ctrl init failed.");
            return Err(MCIHostError::Fail);
        }

        if host.config.enable_dma {
            if let Err(_) = self.hc.borrow_mut().set_idma_list(&self.rw_desc, self.desc_num.get()) {
                error!("idma list set failed!");
                return Err(MCIHostError::Fail);
            }
        }

        // self.register_event_arg();
        *self.hc_cfg.borrow_mut() = mci_config;
        info!("do_init ok");
        Ok(())
    }

    fn deinit(&self) {
        let _ = self.hc.borrow_mut().config_deinit();
        info!("Sdio ctrl deinited !!!")
    }

    pub fn reset(&self) -> MCIHostStatus {
        match self.hc.borrow_mut().restart() {
            Ok(_) => Ok(()),
            Err(_) => Err(MCIHostError::Fail),
        }
    }

    pub fn switch_to_voltage(&self, voltage: MCIHostOperationVoltage,host:&MCIHost) -> MCIHostStatus {
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

    pub fn execute_tuning(&self, _tuning_cmd: u32, _rev_buf: &mut Vec<u32>, _block_size: u32) -> MCIHostStatus {
        Ok(())
    }

    fn enable_ddr_mode(&self, enable: bool, _nibble_pos: u32) {
        self.hc.borrow().set_ddr_mode(enable);
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

    pub fn convert_data_to_little_endian(&self, data: &mut Vec<u32>, word_size: usize, format: MCIHostDataPacketFormat,host:&MCIHost) -> MCIHostStatus {
        if host.config.endian_mode == MCIHostEndianMode::Little && 
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

    pub fn card_detect_init(&self, _cd: &MCIHostCardDetect) -> MCIHostStatus {
         Ok(())
    }

    pub fn card_power_set(&self, _enable: bool) {
        
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

    pub fn card_bus_width_set(&self, data_bus_width: MCIHostBusWdith) {
        match data_bus_width {
            MCIHostBusWdith::Bit1 => {
                self.hc.borrow().bus_width_set(data_bus_width as u32);
                warn!("Set bus width to 1 bit");
            },
            MCIHostBusWdith::Bit4 => {
                self.hc.borrow().bus_width_set(data_bus_width as u32);
                warn!("Set bus width to 4 bit");
            },
            MCIHostBusWdith::Bit8 => {
                self.hc.borrow().bus_width_set(data_bus_width as u32);
                warn!("Set bus width to 8 bit");
            },
        }
    }

    pub fn card_detect_status_polling(&self, wait_card_status: SDStatus, _timeout: u32, host:&MCIHost) -> MCIHostStatus {
        let cd = host.cd.as_ref().ok_or(MCIHostError::NoData)?;

        let mut retry_times:usize = 100;

        /* Wait card inserted. */
        loop {
            let is_card_inserted = self.card_detect_status() == SDStatus::Inserted;
            sleep(Duration::from_millis(cd.cd_debounce_ms as u64));
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

    pub fn card_active_send(&self) {
        
    }

    pub fn card_clock_set(&self, target_clock: u32, host:&MCIHost) -> u32 {
            
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

    pub fn card_is_busy(&self) -> bool {
        self.hc.borrow().check_if_card_busy()
    }

    fn pre_command(&self,content: &mut MCIHostTransfer, host:&MCIHost) -> MCIHostStatus {
        let cmd = match content.cmd() {
            Some(cmd) => cmd,
            None => return Err(MCIHostError::NoData)
        };

        let data = match content.data() {
            Some(data) => data,
            None => return Ok(()),
        };

        if cmd.index() == MCIHostCommonCmd::ReadMultipleBlock as u32 ||
            cmd.index() == MCIHostCommonCmd::WriteMultipleBlock as u32 {
           let block_count = data.block_count();

           if block_count > 1 {
                return host.block_count_set(block_count);
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

        let out_data = if let Some(in_data) = in_trans.data_mut() {
            let mut out_data = MCIData::new();

            flag |= MCICmdFlag::EXP_DATA;
            
            let buf = if let Some(rx_data) = in_data.rx_data_mut() {
                // Handle receive data
                flag |= MCICmdFlag::READ_DATA;
                take(rx_data)
            } else if let Some(tx_data) = in_data.tx_data_mut() {
                // Handle transmit data
                flag |= MCICmdFlag::WRITE_DATA;
                take(tx_data)
            } else {
                // Neither rx_data nor tx_data is available
                panic!("Transaction data initialized but contains neither rx_data nor tx_data");
            };
            
            out_data.blksz_set(in_data.block_size() as u32);
            out_data.blkcnt_set(in_data.block_count());
            out_data.datalen_set(in_data.block_size() as u32 * in_data.block_count());

            let bus_addr = mmap(NonNull::new(buf.as_ptr() as *mut u8).unwrap().into());
            out_data.buf_dma_set(bus_addr as usize);
            flush(NonNull::new(buf.as_ptr() as *mut u8).unwrap(), buf.len() * size_of::<u32>());
            debug!("in covert command info, buf va {:p}, pa {:x}", buf.as_ptr(), bus_addr);
            out_data.buf_set(Some(buf));

            Some(out_data)
        } else {
            None
        };

        let mut out_trans = MCICmdData::new();

        out_trans.cmdidx_set(index);
        out_trans.cmdarg_set(arg);
        out_trans.set_data(out_data);
        out_trans.flag_set(flag);

        unsafe { dsb(); }
        
        out_trans

    }

    #[cfg(feature="poll")]
    pub fn transfer_function(&self,content: &mut MCIHostTransfer, host:&MCIHost) -> MCIHostStatus {
        use crate::invalidate;

        self.pre_command(content,host)?;
        let mut cmd_data = self.covert_command_info(content);

        if host.config.enable_dma {
            if let Err(_) = self.hc.borrow_mut().dma_transfer(&mut cmd_data) {
                return Err(MCIHostError::NoData);
            }
            if let Err(_) = self.hc.borrow_mut().poll_wait_dma_end(&mut cmd_data) {
                return Err(MCIHostError::NoData);
            }
        } else {
            if let Err(_) = self.hc.borrow_mut().pio_transfer(&mut cmd_data) {
                return Err(MCIHostError::NoData);
            }
            if let Err(_) = self.hc.borrow_mut().poll_wait_pio_end(&mut cmd_data) {
                return Err(MCIHostError::NoData);
            }
        }

        unsafe { dsb(); }

        if let Some(_) = content.data() {
            let data = cmd_data.get_mut_data().unwrap();
            invalidate(NonNull::new(data.buf().unwrap().as_ptr() as *mut u8).unwrap(), data.buf().unwrap().len() * 4);
            if let Some(rx_data) = data.buf_take() {
                if let Some(in_data) = content.data_mut() {
                    in_data.rx_data_set(Some(rx_data));
                }
            }
        }

        if let Err(_) = self.hc.borrow_mut().cmd_response_get(&mut cmd_data) {
            error!("Transfer cmd and data failed!");
            return Err(MCIHostError::Timeout);
        }

        if let Some(cmd) = content.cmd_mut() {
            if cmd.response_type() != MCIHostResponseType::None {
                cmd.response_mut().copy_from_slice(&cmd_data.get_response()[..4]);
            }
        }

        Ok(())
    }

    #[cfg(feature="irq")]
    pub fn transfer_function(&self, content: &mut MCIHostTransfer, host: &MCIHost) -> MCIHostStatus {
        use crate::{
            mci::regs::MCIRawInts, 
            osa::consts::{SDMMC_OSA_EVENT_FLAG_AND, SDMMC_OSA_EVENT_TRANSFER_CMD_SUCCESS, SDMMC_OSA_EVENT_TRANSFER_DATA_SUCCESS}
        };
        use crate::aarch::invalidate;

        self.pre_command(content, host)?;
        let mut cmd_data = self.covert_command_info(content);

        if host.config.enable_dma {
            if let Err(_) = self.hc.borrow_mut().dma_transfer(&mut cmd_data) {
                return Err(MCIHostError::NoData);
            }
        } else {
            if let Err(_) = self.hc.borrow_mut().pio_transfer(&mut cmd_data) {
                return Err(MCIHostError::NoData);
            }
        }

        info!("cmd send!!!");
        let raw_ints = self.hc.borrow().config().reg().read_reg::<MCIRawInts>();
        info!("raw ints 0x{:x}", raw_ints);

        for _ in 0..5 {
            sleep(Duration::from_millis(100));
        }

        let complete_events = if cmd_data.get_data().is_some() {
            SDMMC_OSA_EVENT_TRANSFER_CMD_SUCCESS | SDMMC_OSA_EVENT_TRANSFER_DATA_SUCCESS
        } else {
            SDMMC_OSA_EVENT_TRANSFER_CMD_SUCCESS
        };
        let mut events: u32 = 0;
        if self.hc_evt.borrow().osa_event_wait(complete_events, COMMAND_TIMEOUT, &mut events, SDMMC_OSA_EVENT_FLAG_AND).is_err() ||
            events != complete_events
        {
            error!("wait command done timeout!");
            self.hc.borrow().register_dump();
            self.hc_evt.borrow_mut().osa_event_clear(events);
            return Err(MCIHostError::Timeout);
        }
        
        self.hc_evt.borrow_mut().osa_event_clear(events);

        // check if any error events
        // let err_events = FSDIF_TRANS_ERR_EVENTS;
        // events = 0;
        // let _ = self.hc_evt.borrow_mut().osa_event_wait(err_events, 0, &mut events, SDMMC_OSA_EVENT_FLAG_OR);
        // if events != 0 {
        //     error!("finish command with error 0x{:x}", events);
        //     self.hc.borrow_mut().register_dump();
        //     self.hc_evt.borrow_mut().osa_event_clear(events);
        //     return Err(MCIHostError::Timeout);
        // }

        if let Some(_) = content.data() {
            let data = cmd_data.get_mut_data().unwrap();
            invalidate(NonNull::new(data.buf().unwrap().as_ptr() as *mut u8).unwrap(), data.buf().unwrap().len() * 4);
            if let Some(rx_data) = data.buf_take() {
                if let Some(in_data) = content.data_mut() {
                    in_data.rx_data_set(Some(rx_data));
                }
            }
        }

        // in PIO mode, read PIO data after recv DTO flag
        if let Err(_) = self.hc.borrow_mut().cmd_response_get(&mut cmd_data) {
            error!("Transfer cmd and data failed!");
            return Err(MCIHostError::Timeout);
        }


        if let Some(cmd) = content.cmd_mut() {
            if cmd.response_type() != MCIHostResponseType::None {
                cmd.response_mut().copy_from_slice(&cmd_data.get_response()[..]);
            }
        }

        if content.cmd().unwrap().response()[0] & content.cmd().unwrap().response_error_flags().bits() != 0 {
            return Err(MCIHostError::Fail);
        }

        Ok(())
    }
}