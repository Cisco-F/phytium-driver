#![allow(dead_code)]
mod usr_param;
pub(crate) mod consts;
mod io_voltage;
mod cid;
mod csd;
mod scr;
mod status;

use core::cmp::max;
use core::ptr::NonNull;
use core::str;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::vec;
use io_voltage::SdIoVoltage;
use core::time::Duration;

use crate::mci_host::mci_host_config::MCIHostType;
use crate::mci_host::mci_sdif::sdif_device::SDIFDev;
use crate::mci_host::MCIHost;
use crate::osa::{osa_alloc_aligned, osa_init};
use crate::{sleep, IoPad};
use crate::tools::swap_word_byte_sequence_u32;

use super::err::{MCIHostError, MCIHostStatus};
use super::mci_card_base::MCICardBase;
use super::constants::*;
use super::mci_host_card_detect::MCIHostCardDetect;
use super::mci_host_config::MCIHostConfig;
use super::mci_host_transfer::{MCIHostCmd, MCIHostData, MCIHostTransfer};
use super::mci_sdif::consts::SDStatus;
use cid::SdCid;
use consts::*;
use log::{debug, error, info, warn};
use scr::{ScrFlags, SdScr};
use status::SdStatus;
use csd::{CsdFlags, SdCardCmdClass, SdCsd};
use usr_param::SdUsrParam;

pub static mut REG_BASE: NonNull<u8> = unsafe { NonNull::new_unchecked("uninit".as_ptr() as *mut u8) };

pub struct SdCard{
    base: MCICardBase,
    usr_param: SdUsrParam,
    version: SdSpecificationVersion,
    flags: SdCardFlag,
    block_count: u32,
    current_timing: SdTimingMode,
    driver_strength: SdDriverStrength,
    max_current: SdMaxCurrent,
    operation_voltage: MCIHostOperationVoltage,
    cid: SdCid,
    csd: SdCsd,
    scr: SdScr,
    stat: SdStatus,
}

impl SdCard {
    pub fn new(addr: NonNull<u8>,iopad:IoPad) -> Self {
        osa_init();
        unsafe { REG_BASE = addr; }

        let mci_host_config = MCIHostConfig::new();

        // 组装 base
        let internal_buffer = match osa_alloc_aligned(
            mci_host_config.max_trans_size, 
            mci_host_config.def_block_size
        ) {
            Err(e) => {
                error!("alloc internal buffer failed! err: {:?}", e);
                panic!("Failed to allocate internal buffer");
            }
            Ok(buffer) => buffer,
        };
        let base = MCICardBase::from_buffer(internal_buffer);
        info!("Internal buffer@0x{:p}, length = 0x{}",base.internal_buffer.addr().as_ptr(), base.internal_buffer.size());
        
        // 组装 host
        let desc_num = mci_host_config.max_trans_size / mci_host_config.def_block_size;
        let sdif_device = SDIFDev::new(addr, desc_num);
        sdif_device.iopad_set(iopad);
        let host = MCIHost::new(Box::new(sdif_device), mci_host_config);
        let host_type = host.config.host_type;

        // 初步组装 SdCard
        let mut sd_card = SdCard::from_base(base);
        sd_card.base.host = Some(host);

        if host_type == MCIHostType::SDIF {
            if sd_card.sdif_config().is_err() {
                panic!("Config fail!");
            }
        } else {
            if sd_card.sdmmc_config().is_err() {
                panic!("Config fail!");
            }
        }

        sd_card
    }

    fn sdif_config(&mut self) -> MCIHostStatus {
        let mut card_cd = MCIHostCardDetect::new();
        
        card_cd.typ = MCIHostDetectCardType::ByHostCD;
        card_cd.cd_debounce_ms = 10;

        let card_cd = Rc::new(card_cd);

        let usr_param = &mut self.usr_param;

        usr_param.power_off_delay_ms = 0;
        usr_param.power_on_delay_ms = 0;
        
        let capability = 
            MCIHostCapability::SUSPEND_RESUME | 
            MCIHostCapability::BIT4_DATA_WIDTH | 
            MCIHostCapability::BIT8_DATA_WIDTH | 
            MCIHostCapability::DETECT_CARD_BY_DATA3 |
            MCIHostCapability::DETECT_CARD_BY_CD |
            MCIHostCapability::AUTO_CMD12 |
            MCIHostCapability::DRIVER_TYPE_C |
            MCIHostCapability::SET_CURRENT ;
        let capability = capability.bits() | MCIHostCapabilityExt::BIT8_WIDTH.bits();

        usr_param.capability = capability;

        self.base.no_interal_align = false;

        let host = self.base.host.as_mut().ok_or(MCIHostError::HostNotReady)?;

        if host.config.is_uhs_card {
            let mut io_voltage = SdIoVoltage::new();

            io_voltage.typ_set(SdIoVoltageCtrlType::ByHost);
            io_voltage.set_func(None);

            usr_param.io_voltage = Some(io_voltage);

            let capability = 
                MCIHostCapability::VOLTAGE_3V3 |
                MCIHostCapability::VOLTAGE_1V8 |
                MCIHostCapability::HIGH_SPEED |
                MCIHostCapability::SDR104 |
                MCIHostCapability::SDR50;
            
            host.capability=capability;
        } else {
            usr_param.io_voltage = None;
            
            let mut capability = MCIHostCapability::VOLTAGE_3V3;

            if host.config.card_clock >= SD_CLOCK_50MHZ {
                capability |= MCIHostCapability::HIGH_SPEED;
            } 

            host.capability=capability;
        }

        usr_param.max_freq = host.config.card_clock;

        self.usr_param.cd = Some(card_cd.clone());

        host.max_block_count.set(host.config.max_trans_size as u32 /host.config.def_block_size as u32);
        host.max_block_size = MCI_HOST_MAX_BLOCK_LENGTH;
        host.source_clock_hz = 1200000000;
        host.cd = Some(card_cd.clone());

        Ok(())
    }

    fn sdmmc_config(&self) -> MCIHostStatus {
        // todo
        Ok(())
    }

    fn from_base(base: MCICardBase) -> Self {
        SdCard {
            base,
            usr_param: SdUsrParam::new(),
            version: SdSpecificationVersion::Version1_0,
            flags: SdCardFlag::empty(),
            block_count: 0,
            current_timing: SdTimingMode::SDR12DefaultMode,
            driver_strength: SdDriverStrength::TypeB,
            max_current: SdMaxCurrent::Limit200mA,
            operation_voltage: MCIHostOperationVoltage::Voltage330V,
            cid: SdCid::new(),
            csd: SdCsd::new(),
            scr: SdScr::new(),
            stat: SdStatus::new(),
        }
    }
}

/// SD卡其他操作命令
impl SdCard{
    pub fn init(&mut self,addr:NonNull<u8>) -> MCIHostStatus {
        debug!("sd card initializing");
        
        let status = if !self.base.is_host_ready {
            self.host_init(addr)
        } else {
            /* reset host if it's ready */
            self.host_do_reset()
        };

        if status.is_ok() {
            /* check if card is presented */
            if self.polling_card_insert(SDStatus::Inserted).is_err() {
                info!("Polling card failed !!!");
                return Err(MCIHostError::CardDetectFailed);
            } else {
                /* start card init process */
                info!("Start card identification");
                if let Err(err) = self.card_init() {
                    warn!("SD card init failed !!! {:?}",err);
                    return Err(MCIHostError::CardInitFailed);
                }
            }
        }

        info!("SD init finished, error = {:?}",status);
        return status;

    }

    fn deinit(&self) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        if host.dev.reset().is_err() {
            return Err(MCIHostError::Fail);
        }
        Ok(())
    }

    fn card_init(&mut self) -> MCIHostStatus {
        self.card_power_set(true)?;
        self.card_init_proc()?;
        Ok(())
    }

    fn card_init_proc(&mut self) -> MCIHostStatus {
        info!("card init proc");
        /* reset variables */
        self.flags = SdCardFlag::empty();
        /* set DATA bus width */
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        host.dev.card_bus_width_set(MCIHostBusWdith::Bit1);
        /*set card freq to 400KHZ*/
        self.base.bus_clk_hz = host.dev.card_clock_set(MCI_HOST_CLOCK_400KHZ,host);
    
        /* probe bus voltage */
        if self.bus_voltage_prob().is_err() {
            return Err(MCIHostError::SwitchVoltageFail);
        }

        /* Read the card's CID (card identification register) */
        /* Initialize card if the card is SD card. */
        if self.all_cid_send().is_err() { /* CMD2 */
            return Err(MCIHostError::AllSendCidFailed);
        }

        /*
        * Request new relative card address. This moves the card from
        * identification mode to data transfer mode
        */
        if self.rca_send().is_err() { /* CMD3 */
            return Err(MCIHostError::SendRelativeAddressFailed);
        }

        /* Card has entered data transfer mode. Get card specific data register */
        if self.csd_send().is_err() { /* CMD9 */
            return Err(MCIHostError::SendCsdFailed);
        }

        /* Move the card to transfer state (with CMD7) to run remaining commands */
        if self.card_select(true).is_err() { /* CMD7 */
            return Err(MCIHostError::SelectCardFailed);
        }

        /* Set to max frequency in non-high speed mode. */
        /*
        * With card in data transfer state, we can set SD clock to maximum
        * frequency for non high speed mode (25Mhz)
        */
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        self.base.bus_clk_hz = host.dev.card_clock_set(SD_CLOCK_25MHZ,host);

        /* Read SD SCR (SD configuration register),
        * to get supported bus width
        */
        if self.scr_send().is_err() { /* ACMD51 */
            return Err(MCIHostError::SendScrFailed);
        }

        /*
        * Init UHS capable SD card. Follows figure 3-16 in physical layer specification.
        */
        /* Set to 4-bit data bus mode. */
        if self.flags.contains(SdCardFlag::Support4BitWidth) {
            /* Raise bus width to 4 bits */
            warn!("card support 4 bit width");
            if self.data_bus_width_set(MCIHostBusWdith::Bit4).is_err() { /* ACMD6 */
                return Err(MCIHostError::SetDataBusWidthFailed);
            }
            let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
            host.dev.card_bus_width_set(MCIHostBusWdith::Bit4);
        }

        /* try to get card current status */
        if self.status_read().is_err() { /* ACMD13 */
            return Err(MCIHostError::SendScrFailed);
        }
        
        /* set block size */
        if self.block_size_set(self.base.block_size).is_err() { /* CMD16 */
            return Err(MCIHostError::SetCardBlockSizeFailed);
        }

        /* SDR104, SDR50, and DDR50 mode need tuning */
        if self.bus_timing_select().is_err() {
            return Err(MCIHostError::SwitchBusTimingFailed);
        }

        self.card_dump();

        Ok(())
    }

    fn bus_voltage_prob(&mut self) -> MCIHostStatus {
        info!("probing bus voltage");
        /* 3.3V voltage should be supported as default */
        let mut acmd41_argument = {
            MCIHostOCR::VDD_29_30 | MCIHostOCR::VDD_32_33 | MCIHostOCR::VDD_33_34
        };

        /*
        * If card is high capacity (SDXC or SDHC), and supports 1.8V signaling,
        * switch to new signal voltage using "signal voltage switch procedure"
        * described in SD specification
        */
        if let Some(io_voltage) = self.usr_param.io_voltage.as_ref() {
            match io_voltage.typ() {
                SdIoVoltageCtrlType::NotSupport => {
                    /* do nothing */
                },
                _ => {
                    let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
                    let capability = host.capability;
                    if capability.contains(MCIHostCapability::VOLTAGE_1V8) && 
                      (capability.contains(MCIHostCapability::SDR104) || 
                        capability.contains(MCIHostCapability::SDR50) ||
                        capability.contains(MCIHostCapability::DDR_MODE)) {

                        info!("Support 1.8v (with SDR50/SDR104/DDR mode)");
                        
                        /* allow user select the work voltage, if not select, sdmmc will handle it automatically */
                        acmd41_argument |= MCIHostOCR::SWITCH_18_REQUEST_FLAG;
                        /* reset to 3v3 signal voltage */
                        if self.switch_io_voltage(MCIHostOperationVoltage::Voltage330V).is_ok() {
                            /* Host changed the operation signal voltage successfully, then card need power reset */
                            self.card_power_set(false)?;
                            self.card_power_set(true)?;
                        }

                    }
                },
            }
        }

        self.operation_voltage = MCIHostOperationVoltage::Voltage330V;

        /* send card active */
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        host.dev.card_active_send();
        loop {
            /* card go idle */
            if self.go_idle().is_err() { /* CMD0 */
                return Err(MCIHostError::GoIdleFailed);
            }
            /* Check card's supported interface condition. */
            if self.interface_condition_send().is_ok() { /* CMD8 */
                /* SDHC or SDXC card */
                acmd41_argument |= MCIHostOCR::CARD_CAPACITY_SUPPORT_FLAG;
                self.flags |= SdCardFlag::SupportSdhc;
            } else {
                /* SDSC card */
                if self.go_idle().is_err() { /* make up for legacy card which do not support CMD8 */
                    return Err(MCIHostError::GoIdleFailed);
                }
            }

            /* Set card interface condition according to SDHC capability and card's supported interface condition. */
            if self.application_opration_condition_send(acmd41_argument.bits()).is_err() { /* ACMD41 */
                return Err(MCIHostError::HandShakeOperationConditionFailed);
            }

            /* check if card support 1.8V */
            if self.flags.contains(SdCardFlag::SupportVoltage180v) {
                
                if let Some(io_voltage) = self.usr_param.io_voltage.as_ref() {
                    if io_voltage.typ() == SdIoVoltageCtrlType::NotSupport {
                        break;
                    }
                }

                match self.voltage_switch(MCIHostOperationVoltage::Voltage180V) {
                    Err(MCIHostError::SwitchVoltageFail) => {
                        break;
                    },
                    /* card enters UHS-I mode and input/ouput timings are changed to SDR12 by default */
                    Err(MCIHostError::SwitchVoltage18VFail33VSuccess) => {
                        acmd41_argument &= !MCIHostOCR::SWITCH_18_REQUEST_FLAG;
                        self.flags &= !SdCardFlag::SupportVoltage180v;
                        continue;
                    },
                    _ => {
                        info!("Select 1.8v");
                        self.operation_voltage = MCIHostOperationVoltage::Voltage180V;
                        break;
                    },
                }
            }
            break;
        }
    
        Ok(())
    }

    fn switch_io_voltage(&mut self,voltage:MCIHostOperationVoltage) -> MCIHostStatus {
        let io_voltage = self.usr_param.io_voltage.as_ref().ok_or(MCIHostError::Fail)?;
        let typ = io_voltage.typ();

        if typ == SdIoVoltageCtrlType::NotSupport {
            return Err(MCIHostError::NotSupportYet);
        }

        if typ == SdIoVoltageCtrlType::ByGpio {
            /* make sure card signal line voltage is 3.3v before initalization */
            if let Some(func) = io_voltage.func() {
                func(voltage);
            }
        } else if typ == SdIoVoltageCtrlType::ByHost {
            let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
            let _ = host.dev.switch_to_voltage(voltage,host);
        } else {
            return Err(MCIHostError::NotSupportYet);
        }

        Ok(())
    }
    
    fn host_init(&mut self,addr:NonNull<u8>) -> MCIHostStatus {
        info!("host init");
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        if !self.base.is_host_ready {
            if let Err(err) = host.dev.init(addr,host) {
                info!("SD host driver init failed, error = {:?}", err);
                return Err(MCIHostError::Fail);
            }
        }

        let cd = self.usr_param.cd.as_ref().ok_or(MCIHostError::HostNotReady)?;
        if cd.typ == MCIHostDetectCardType::ByGpioCD || cd.typ == MCIHostDetectCardType::ByHostDATA3 {
            info!("SD card init start");
            let _ = host.dev.card_detect_init(cd);
        }

        /* set the host status flag, after the card re-plug in, don't need init host again */
        self.base.is_host_ready = true;

        info!("host init ok");
        Ok(())
    }

    fn host_do_reset(&self) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        host.dev.reset()
    }

    fn card_power_set(&self, enable: bool) -> MCIHostStatus {

        if self.usr_param.sd_pwr.is_some() {
            let sd_pwr = self.usr_param.sd_pwr.unwrap();
            sd_pwr(enable);
        } else {
            let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
            host.dev.card_power_set(enable);
        }

        let power_delay =  if enable {
            if self.usr_param.power_on_delay_ms == 0{
                SD_POWER_ON_DELAY_MS
            } else {
                self.usr_param.power_on_delay_ms
            }
        }else {
            if self.usr_param.power_off_delay_ms == 0 {
                SD_POWER_OFF_DELAY_MS
            } else {
                self.usr_param.power_off_delay_ms
            }
        };

        sleep(Duration::from_millis(power_delay as u64));
        Ok(())
    }

    fn polling_card_insert(&self,status:SDStatus) -> MCIHostStatus {
        let cd = self.usr_param.cd.as_ref().ok_or(MCIHostError::HostNotReady)?;

        if cd.typ == MCIHostDetectCardType::ByGpioCD {
            let card_detect = cd.card_detected.ok_or(MCIHostError::Fail)?;

            loop {
                if card_detect() && status == SDStatus::Inserted {
                    let cd_debounce_ms = cd.cd_debounce_ms;
                    sleep(Duration::from_millis(cd_debounce_ms as u64));
                    if card_detect() {
                        break;
                    }
                }

                if !card_detect() && status == SDStatus::Removed {
                    break;
                }
            }

        } else {
            /* mostly advanced host not detect card by gpio, therefore follow this branch */
            if self.base.is_host_ready == false {
                info!("SD host not ready !!!");
                return Err(MCIHostError::Fail);
            }

            /* polling wait until card presented or timeout */
            let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
            if host.dev.card_detect_status_polling(status, u32::MAX, host).is_err() {
                info!("Polling SD card status failed !!!");
                return Err(MCIHostError::Fail);
            }
        }
        Ok(())
    }

    fn polling_card_status_busy(&mut self,timeout_ms:u32) -> MCIHostStatus {
        let mut status_timeout_us = timeout_ms * 1000;

        while status_timeout_us > 0 {
            let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
            if !host.dev.card_is_busy() {
                if Err(MCIHostError::CardStatusIdle) == self.card_status_send() {
                    return Err(MCIHostError::CardStatusIdle);
                }
            } else {
                /* Delay 125us to throttle the polling rate */
                sleep(Duration::from_micros(125));
                status_timeout_us -= 125;
            }
        }
        Err(MCIHostError::CardStatusBusy)
    }

    fn write_successful_block_send(&mut self, blocks: &mut u32) -> MCIHostStatus {
        if Err(MCIHostError::CardStatusIdle) != self.polling_card_status_busy(SD_CARD_ACCESS_WAIT_IDLE_TIMEOUT) {
            return Err(MCIHostError::WaitWriteCompleteFailed);
        }

        if self.application_cmd_send(self.base.relative_address).is_err() {
            return Err(MCIHostError::SendApplicationCommandFailed);
        }

        let mut command = MCIHostCmd::new();
        command.index_set(SdAppCmd::SendNumberWriteBlocks as u32);
        command.response_type_set(MCIHostResponseType::R1);

        let mut data = MCIHostData::new();
        data.block_size_set(4);
        data.block_count_set(1);
        let tmp_buf = vec![0; 4];
        data.rx_data_set(Some(tmp_buf));

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
        content.set_data(Some(data));

        let result = self.transfer(&mut content, 3);
        let response = content.cmd().unwrap().response();
        debug!("in write successful blk send response is {:?}", response);
        if result.is_err() || response[0] & MCIHostCardStatusFlag::ALL_ERROR_FLAG.bits() != 0 {
            error!("\r\n\r\nError: send ACMD22 failed with host error {:?}, response {:x}\r\n", result, response[0]);
            return result;
        } else {
            *blocks = swap_word_byte_sequence_u32(response[0]);
        }

        Ok(())
    }

    fn bus_timing_select(&mut self) -> MCIHostStatus {
        
        if self.operation_voltage != MCIHostOperationVoltage::Voltage180V {
            /* group 1, function 1 ->high speed mode*/
            debug!("group1");
            match self.func_select(SdGroupNum::TimingMode, SdTimingFuncNum::SDR25HighSpeed) {
                Ok(_) => {
                    /* If the result isn't "switching to high speed mode(50MHZ) successfully or card doesn't support high speed
                    * mode". Return failed status. */
                    let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
                    
                    self.current_timing = SdTimingMode::SDR25HighSpeedMode;
                    self.base.bus_clk_hz = host.dev.card_clock_set(max(self.usr_param.max_freq, SD_CLOCK_50MHZ),host);
                } ,
                Err(err) => {
                    if err == MCIHostError::NotSupportYet {
                        /* if not support high speed, keep the card work at default mode */
                        info!("\r\nNote: High speed mode is not supported by card\r\n");
                        return Ok(());
                    }
                    return Err(err);
                },
            }
        } else { /* card is in UHS_I mode */
            loop {

                if self.current_timing == SdTimingMode::SDR12DefaultMode {
                    /* if timing not specified, probe card capability from SDR104 mode */
                    self.current_timing = SdTimingMode::SDR104Mode;
                }

                if self.current_timing == SdTimingMode::SDR104Mode {
                    let host_capability = {
                        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
                        host.capability
                    };
                    if host_capability.contains(MCIHostCapability::SDR104) {
                        match self.func_select(SdGroupNum::TimingMode, SdTimingFuncNum::SDR104) {
                            Ok(_) => {
                                let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
                                self.current_timing = SdTimingMode::SDR104Mode;
                                self.base.bus_clk_hz = host.dev.card_clock_set(SD_CLOCK_208MHZ,host);
                                break;
                            },
                            _=> {
                                info!("\r\nNote: SDR104 mode is not supported\r\n");
                                self.current_timing = SdTimingMode::SDR50Mode;
                            }
                        }
                    } 
                }

                if self.current_timing == SdTimingMode::SDR50Mode {
                    let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
                    if host.capability.contains(MCIHostCapability::SDR50) {
                        match self.func_select(SdGroupNum::TimingMode, SdTimingFuncNum::SDR50) {
                            Ok(_) => {
                                let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
                                self.current_timing = SdTimingMode::SDR50Mode;
                                self.base.bus_clk_hz = host.dev.card_clock_set(SD_CLOCK_100MHZ,host);
                                break;
                            },
                            _=> {
                                info!("\r\nNote: SDR50 mode is not supported\r\n");
                                self.current_timing = SdTimingMode::SDR25HighSpeedMode;
                            }
                        }
                    }
                }

                if self.current_timing == SdTimingMode::SDR25HighSpeedMode {
                    match self.func_select(SdGroupNum::TimingMode, SdTimingFuncNum::SDR25HighSpeed) {
                        Ok(_) => {
                            let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
                            self.current_timing = SdTimingMode::SDR25HighSpeedMode;
                            self.base.bus_clk_hz = host.dev.card_clock_set(SD_CLOCK_50MHZ,host);
                            break;
                        },
                        _=> {
                            info!("\r\nNote: SDR25 high speed mode is not supported\r\n");
                            self.current_timing = SdTimingMode::SDR12DefaultMode;
                        }
                    }
                }

                info!("\r\nWarning: unknown timing mode\r\n");
                break;
            }
        }

        /* Update io strength according to different bus frequency */
        if self.usr_param.io_strength.is_some() {
            let io_strength = self.usr_param.io_strength.unwrap();
            io_strength(self.current_timing);
        }

        /* SDR50 and SDR104 mode need tuning */
        if self.current_timing == SdTimingMode::SDR50Mode || self.current_timing == SdTimingMode::SDR104Mode {
            /* execute tuning */
            if self.execute_tuning().is_err() {
                info!("\r\nError: tuning failed for mode {}\r\n",self.current_timing as u32);
                return Err(MCIHostError::TuningFail);
            }
        }

        Ok(())
    }

    fn func_select(&mut self,group:SdGroupNum,func:SdTimingFuncNum) -> MCIHostStatus {

        /* check if card support CMD6 */
        if (self.version as u32 <= SdSpecificationVersion::Version1_0 as u32) || 
            (self.csd.card_command_classes & SdCardCmdClass::Switch.bits() == 0) {
            info!("\r\nError: current card not support CMD6\r\n");
            return Err(MCIHostError::CardNotSupport);
        }

        /* Check if card support high speed mode. */
        let mut func_status =  match self.func_swtich(SdSwitchMode::Check, group, func) {
            Some(status) => status,
            None => return Err(MCIHostError::TransferFailed),
        };
        

        /* convert to little endian sequence */
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        let _ = host.dev.convert_data_to_little_endian(&mut func_status, 5, MCIHostDataPacketFormat::MSBFirst,host);

        /*  
            -functionStatus[0U]---bit511~bit480;
            -functionStatus[1U]---bit479~bit448;
            -functionStatus[2U]---bit447~bit416;
            -functionStatus[3U]---bit415~bit384;
            -functionStatus[4U]---bit383~bit352;
            According to the "switch function status[bits 511~0]" return by switch command in mode "check function":
            -Check if function 1(high speed) in function group 1 is supported by checking if bit 401 is set;
            -check if function 1 is ready and can be switched by checking if bits 379~376 equal value 1;
        */
        let mut func_group_info = [0u16;6];
        func_group_info[5] = func_status[0] as u16;
        func_group_info[4] = (func_status[1] >> 16) as u16;
        func_group_info[3] = (func_status[1]) as u16;
        func_group_info[2] = (func_status[2] >> 16) as u16;
        func_group_info[1] = (func_status[2]) as u16;
        func_group_info[0] = (func_status[3] >> 16) as u16;
        info!("func_group_info: {:?}", func_group_info);
        
        let current_func_status = ((func_status[3] & 0xff) << 8) | (func_status[4] >> 24);

        /* check if function is support */
        if (func_group_info[group as usize] & (1 << (func as u16)) == 0) ||
            (((current_func_status >> (group as u32)*4) & 0xf) != (func as u32)) {
            info!("\r\nError: function {} in group {} not support\r\n",func as u32,group as u32);
            return Err(MCIHostError::CardNotSupport);
        }

        let func_status =  match self.func_swtich(SdSwitchMode::Set, group, func) {
            Some(status) => status,
            None => return Err(MCIHostError::TransferFailed),
        };

        /* convert to little endian sequence */
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        let mut func_status_need_convert = func_status[3..].to_vec();
        let _ = host.dev.convert_data_to_little_endian(&mut func_status_need_convert, 2, MCIHostDataPacketFormat::MSBFirst,host);
        let mut func_status = func_status[0..3].to_vec();
        func_status.extend_from_slice(&func_status_need_convert);

        /* According to the "switch function status[bits 511~0]" return by switch command in mode "set function":
            -check if group 1 is successfully changed to function 1 by checking if bits 379~376 equal value 1;
        */
        let current_func_status = ((func_status[3] & 0xff) << 8) | (func_status[4] >> 24);

        if ((current_func_status >> (group as u32)*4) & 0xf) != (func as u32) {
            info!("\r\nError: switch to function {} failed\r\n",func as u32);
            return Err(MCIHostError::SwitchFailed);
        }

        Ok(())
    }

    fn execute_tuning(&mut self,) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        let mut buffer = vec![0u32;64];
        host.dev.execute_tuning(SdCmd::SendTuningBlock as u32, &mut buffer, 64)
    } 
    
    /// will clear buffer passed to this method
    pub fn read_blocks(&mut self,buffer:&mut Vec<u32>,start_block:u32,block_count:u32) -> MCIHostStatus {
        buffer.clear();
        let mut block_left = block_count;
        let mut block_count_one_time:u32;

        while block_left != 0 {
            // todo如果修正当前的性能问题,则需要考虑对齐问题
            let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
            if block_left > host.max_block_count.get() {
                block_left -= host.max_block_count.get();
                block_count_one_time = host.max_block_count.get();
            } else {
                block_count_one_time = block_left;
                block_left = 0;
            }

            let len = block_count_one_time * MCI_HOST_DEFAULT_BLOCK_SIZE / 4;
            let mut once_buffer = vec![0u32; len as usize];
            if self.read(&mut once_buffer, 
                        start_block, 
                        MCI_HOST_DEFAULT_BLOCK_SIZE,
                        block_count_one_time).is_err() {
                return Err(MCIHostError::TransferFailed);
            }

            buffer.extend(once_buffer.iter());
        }
        
        Ok(())
    }

    pub fn write_blocks(&mut self, buffer: &mut Vec<u32>, start_block: u32, block_count: u32) -> MCIHostStatus {
        let mut block_left = block_count;
        let mut block_count_one_time: u32;
        let mut block_written_one_time = 0; // 一次写操作写成功的块数

        while block_left != 0 {
            let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
            if block_left > host.max_block_count.get() {
                block_count_one_time = host.max_block_count.get();
            } else {
                block_count_one_time = block_left;
            }

            let len = MCI_HOST_DEFAULT_BLOCK_SIZE * block_count_one_time / 4;
            let mut once_buffer = vec![0u32; len as usize];
            let start_addr = (block_count - block_left) * MCI_HOST_DEFAULT_BLOCK_SIZE / 4;
            let end_addr = start_addr + block_count_one_time * MCI_HOST_DEFAULT_BLOCK_SIZE / 4;
            debug!("write block(s) one time, relative addr(u32) from {} - {}, block count {}", start_addr, end_addr, block_count_one_time);
            once_buffer.copy_from_slice(&buffer[start_addr as usize..end_addr as usize]);
            if self.write(&mut once_buffer, 
                start_block + block_count - block_left, 
                MCI_HOST_DEFAULT_BLOCK_SIZE, 
                block_count_one_time, 
                &mut block_written_one_time
            ).is_err() {
                error!("write block(s) failed!");
                return Err(MCIHostError::TransferFailed);
            }

            block_left -= block_count_one_time;
        }

        Ok(())
    }

    fn transfer(&mut self,content:&mut MCIHostTransfer,retry: u32) -> MCIHostStatus {
        let mut retry = retry;
        let mut retuning_count = 3;
        loop {
            let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
            let status = host.dev.transfer_function(content,host);
            if status.is_ok() {
               break;
            } 

            /* if transfer data failed, send cmd12 to abort current transfer */
            if content.data().is_some() {
                let _ = self.transmission_stop();
                /* when transfer error occur, polling card status until it is ready for next data transfer, otherwise the
                * retry transfer will fail again */
                if Err(MCIHostError::CardStatusIdle) != self.polling_card_status_busy(SD_CARD_ACCESS_WAIT_IDLE_TIMEOUT) {
                    return Err(MCIHostError::TransferFailed);
                }
            }

            if retry == 0 || status == Err(MCIHostError::ReTuningRequest) {
                if self.current_timing == SdTimingMode::SDR104Mode || self.current_timing == SdTimingMode::SDR50Mode {
                    if retuning_count == 0 {
                        break;
                    }
                    retuning_count -= 1;
                    /* Perform retuning, CMD19 sends a tuning block to the host to determine sampling point. 
                        UHS50 and UHS104 cards support CMD19 in 1.8V signaling. Sampling
                        clock tuning is required for UHS104 host and optional for UHS50 host. */
                    if self.execute_tuning().is_err() {
                        info!("\r\nError: retuning failed.\r\n");
                        return Err(MCIHostError::TuningFail);
                    } else {
                        info!("\r\nlog: retuning successfully.\r\n");
                        continue;
                    }
                }
            } else {
                break;
            }

            if retry != 0 {
                retry -= 1;
            } else {
                break;
            }
        }
        Ok(())
    }

}

/// SDIO规范CMD指令
impl SdCard {
    /// CMD 0 
    fn go_idle(&self) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        host.go_idle()
    }

    /// CMD 2 
    fn all_cid_send(&mut self) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        
        let mut command = MCIHostCmd::new();

        command.index_set(MCIHostCommonCmd::AllSendCid as u32);
        command.argument_set(0);
        command.response_type_set(MCIHostResponseType::R2);

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));

        host.dev.transfer_function(&mut content,host)?;

        let command = content.cmd().unwrap();
        let response = command.response();

        self.base.internal_buffer.clear();
        // self.base.internal_buffer.extend(response.iter().flat_map(|&val| val.to_ne_bytes()));
        if self.base.internal_buffer.copy_from_slice(response).is_err() {
            return Err(MCIHostError::Fail);
        }

        self.decode_cid();

        Ok(())
    }

    /// CMD 3 
    fn rca_send(&mut self) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        
        let mut command = MCIHostCmd::new();

        command.index_set(SdCmd::SendRelativeAddress as u32);
        command.argument_set(0);
        command.response_type_set(MCIHostResponseType::R6);

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));

        if let Err(err) = host.dev.transfer_function(&mut content,host){
            let command = content.cmd().unwrap();
            let response = command.response();

            info!("\r\nError: send CMD3 failed with host error {:?}, response 0x{:x}\r\n"
                ,err
                ,response[0]);

            return Err(err);
        }else {

            let command = content.cmd().unwrap();
            let response = command.response();

            self.base.relative_address = response[0] >> 16;
        }

        Ok(())
    }

    /// CMD 6 
    fn func_swtich(&mut self,mode: SdSwitchMode,group: SdGroupNum,num: SdTimingFuncNum) -> Option<Vec<u32>> {
        let host = self.base.host.as_ref()?;

        let mut command = MCIHostCmd::new();

        command.index_set(SdCmd::Switch as u32);
        command.argument_set({
            let mut arg = (mode as u32) << 31 | 0x00FFFFFF;
            arg &=  !(0xf << ((group as u32)*4));
            arg |= (num as u32) << ((group as u32)*4);
            arg
        });
        command.response_type_set(MCIHostResponseType::R1);

        let mut data = MCIHostData::new();

        data.block_size_set(64);
        data.block_count_set(1);
        let tmp_buf = vec![0; 64];
        data.rx_data_set(Some(tmp_buf)); //todo 似乎影响性能 DMA 似乎是最好不要往栈上读写的? 从内存池分配

        let mut content = MCIHostTransfer::new();

        content.set_cmd(Some(command));
        content.set_data(Some(data));

        if let Err(err) = host.dev.transfer_function(&mut content,host) {
            let command = content.cmd().unwrap();
            let response = command.response()[0];

            info!("\r\nError: send CMD6 failed with host error {:?}, response 0x{:x}\r\n"
                ,err
                ,response);
            
            return None;
        }

        let command = content.cmd().unwrap();
        let response = command.response()[0];

        if MCIHostCardStatusFlag::ALL_ERROR_FLAG.bits() & response != 0 {
            info!("\r\nError: CMD6 response error, response 0x{:x}\r\n", response);
        }

        let data = content.data_mut().unwrap();

        data.rx_data_take()
    }

    /// CMD 7 
    fn card_select(&mut self,is_selected:bool) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        host.card_select(self.base.relative_address, is_selected)
    }


    /// CMD 8 
    fn interface_condition_send(&mut self) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        
        let mut command = MCIHostCmd::new();

        command.index_set(SdCmd::SendInterfaceCondition as u32);
        command.argument_set(0x1AA);
        command.response_type_set(MCIHostResponseType::R7);

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));

        let mut i = MCI_HOST_MAX_CMD_RETRIES;
        loop {
            if let Err(err) = host.dev.transfer_function(&mut content,host) {
                info!("\r\nError: send CMD8 failed with host error {:?}, response {}\r\n",
                    err,
                    {
                        let command = content.cmd().unwrap();
                        let response = command.response();
                        response[0]
                    });
                if i == 0 {
                    return Err(err);
                }
            } else {
                let command = content.cmd().unwrap();
                let response = command.response();
                if response[0] & 0xFF != 0xAA {
                    info!("\r\nError: CMD8 response error, response 0x{:x}\r\n", response[0]);
                    return Err(MCIHostError::CardNotSupport);
                } else {
                    break;
                }
            }

            i -= 1;
        }
        
        Ok(())
    }

    /// CMD 9 
    fn csd_send(&mut self) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        
        let mut command = MCIHostCmd::new();

        command.index_set(MCIHostCommonCmd::SendCsd as u32);
        command.argument_set(self.base.relative_address << 16);
        command.response_type_set(MCIHostResponseType::R2);

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));

        if let Err(err) = host.dev.transfer_function(&mut content,host) {
            let command = content.cmd().unwrap();
            let response = command.response();
            
            info!("Error: send CMD9 failed with host error {:?}, response 0x{:x}\r\n",
                err,
                response[0]);

            return Err(err);
        }

        let command = content.cmd().unwrap();
        let response = command.response();

        self.base.internal_buffer.clear();
        // self.base.internal_buffer.extend(response.iter().flat_map(|&val| val.to_ne_bytes()));
        if let Err(e) = self.base.internal_buffer.copy_from_slice(response) {
            error!("copy to PoolBuffer failed! err: {:?}", e);
            return Err(MCIHostError::Fail);
        }

        self.decode_csd();

        Ok(())
        }

    /// CMD 11
    fn voltage_switch(&mut self,voltage: MCIHostOperationVoltage) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        
        let mut command = MCIHostCmd::new();

        command.index_set(SdCmd::VoltageSwitch as u32);
        command.argument_set(0);
        command.response_type_set(MCIHostResponseType::R1);

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));

        if host.dev.transfer_function(&mut content,host).is_err() {
            return Err(MCIHostError::TransferFailed);
        }

        /*
        * Card should drive CMD and DAT[3:0] signals low at the next clock
        * cycle. Some cards will only drive these
        * lines low briefly, so we should check as soon as possible
        */
        if !host.dev.card_is_busy() {
            /* Delay 1ms to allow card to drive lines low */
            sleep(Duration::from_millis(1));
            if !host.dev.card_is_busy() {
                /* Card did not drive CMD and DAT lines low */
                info!("\r\nError: card not drive lines low\r\n");
                return Err(MCIHostError::CardStatusBusy);
            }
        }

        /*
        * Per SD spec (section "Timing to Switch Signal Voltage"),
        * host must gate clock at least 5ms.
        */
        host.dev.card_clock_set(0,host);

        /* switch io voltage */
        if self.switch_io_voltage(voltage) == Err(MCIHostError::NotSupportYet) {
            info!("Failed to switch SD host to 1.8V");
            return Err(MCIHostError::SwitchVoltageFail);
        }

        /* Gate for 10ms, even though spec requires 5 */
        sleep(Duration::from_millis(10));

        /* 重新获取 host 实例 */
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;

        /* Restart the clock */
        host.dev.card_clock_set(self.base.bus_clk_hz,host);

        /*
        * If SD does not drive at least one of
        * DAT[3:0] high within 1ms, switch failed
        */
        sleep(Duration::from_millis(1));

        if host.dev.card_is_busy() {
            info!("Card failed to switch voltages");
            return Err(MCIHostError::SwitchVoltageFail);
        }

        info!("Card switched to 1.8V signaling");
        Ok(())
    }

    /// CMD 12 
    fn transmission_stop(&mut self) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        
        let mut command = MCIHostCmd::new();

        command.index_set(MCIHostCommonCmd::StopTransmission as u32);
        command.argument_set(0);
        command.cmd_type_set(MCIHostCmdType::Abort);
        command.response_type_set(MCIHostResponseType::R1b);

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));

        if let Err(err) = host.dev.transfer_function(&mut content,host) {
            let command = content.cmd().unwrap();
            let response = command.response();
            info!("\r\nError: send CMD12 failed with host error {:?}, reponse 0x{:x}\r\n",err,response[0]);

            return Err(MCIHostError::TransferFailed);
        }
        Ok(())
    }

    /// CMD 13 
    fn card_status_send(&mut self) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        
        let mut command = MCIHostCmd::new();

        command.index_set(MCIHostCommonCmd::SendStatus as u32);
        command.argument_set(self.base.relative_address << 16);
        command.response_type_set(MCIHostResponseType::R1);

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));

        let mut retry = SD_CMD13_RETRY_TIMES;
        while retry > 0 {
            if let Err(err) = host.dev.transfer_function(&mut content,host) {
                let command = content.cmd().unwrap();
                let response = command.response();
                
                info!("\r\nError: send CMD13 failed with host error {:?}, response 0x{:x}\r\n",
                    err,
                    response[0]);
    
                retry -= 1;
                continue;
            } else {
                let command = content.cmd().unwrap();
                let response = command.response();

                if (response[0] & MCIHostCardStatusFlag::READY_FOR_DATA.bits() != 0) &&
                   (MCIHostCurrentState::current_state(response[0]) != MCIHostCurrentState::Programming) {
                    return Err(MCIHostError::CardStatusIdle);
                } else {
                    return Err(MCIHostError::CardStatusBusy);
                }
            }
        }
        Ok(())
    }

    /// CMD 16 
    fn block_size_set(&mut self,block_size:u32) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        host.block_size_set(block_size)
    }

    /// CMD 17/18 
    fn read(&mut self,buffer:&mut Vec<u32>,start_block:u32,block_size:u32,block_count:u32) -> MCIHostStatus {
        if (self.flags.contains(SdCardFlag::SupportHighCapacity) && block_size != 512) ||
           (block_size > self.base.block_size) ||
           ({
                let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
                block_size > host.max_block_size
           }) ||
           (block_size % 4 != 0) {
            info!("\r\nError: read with parameter, block size {} is not support\r\n",block_size);
            return Err(MCIHostError::CardNotSupport);
        }

        /* read command are not allowed while card is programming */
        if Err(MCIHostError::CardStatusIdle) != self.polling_card_status_busy(SD_CARD_ACCESS_WAIT_IDLE_TIMEOUT) {
            info!("Error : read failed with wrong card busy\r\n");
            return Err(MCIHostError::PollingCardIdleFailed);
        }

        let mut command = MCIHostCmd::new();
        
        info!("read block(s), block_size = {}, block_count = {}", block_size, block_count);
        command.index_set({
            if block_count == 1 {
                MCIHostCommonCmd::ReadSingleBlock as u32 
            } else {
                MCIHostCommonCmd::ReadMultipleBlock as u32 
            }
        }); 

        command.argument_set({
            if self.flags.contains(SdCardFlag::SupportHighCapacity) {
                start_block
            } else {
                start_block*block_size
            }
        });

        command.response_type_set(MCIHostResponseType::R1);
        command.response_error_flags_set(MCIHostCardStatusFlag::ALL_ERROR_FLAG);

        let mut data = MCIHostData::new();
        data.block_size_set(block_size as usize);
        data.block_count_set(block_count);

        let len = block_size * block_count / 4;
        let tmp_buf = vec![0; len as usize]; // todo 从内存池分配
        data.rx_data_set(Some(tmp_buf));
        data.enable_auto_command12_set(true);


        let mut context = MCIHostTransfer::new();
        context.set_cmd(Some(command));
        context.set_data(Some(data));

        if let Err(err) = self.transfer(&mut context, 3) {
            return Err(err);
        }

        let data = context.data_mut().unwrap();
        let rx_data = data.rx_data().unwrap();
        buffer.clear();
        buffer.extend(rx_data);

        Ok(())
    }

    /// CMD 19 
    fn tuning_execute(&mut self) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        let mut buffer = vec![0u32;64];
        let status = host.dev.execute_tuning(SdCmd::SendTuningBlock as u32, &mut buffer, 64);

        // todo 性能问题
        self.base.internal_buffer.clear();
        // self.base.internal_buffer.extend(buffer.iter().flat_map(|&val| val.to_ne_bytes()));
        let buffer = buffer.iter().flat_map(|&val| val.to_ne_bytes()).collect::<Vec<u8>>();
        if let Err(e) = self.base.internal_buffer.copy_from_slice(&buffer[..]) {
            error!("copy to PoolBuffer failed! err: {:?}", e);
            return Err(MCIHostError::Fail)
        }

        status
    }

    /// CMD 24/25
    pub fn write(&mut self, 
        buffer: &mut Vec<u32>, 
        start_block: u32, 
        block_size: u32, 
        block_count: u32, 
        written_blocks: &mut u32
    ) -> MCIHostStatus {
        if (self.flags.contains(SdCardFlag::SupportHighCapacity) && block_size != 512) ||
            (block_size > self.base.block_size) || 
            ({
                let host  = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
                block_size > host.max_block_size
            }) ||
            (block_size % 4 != 0) 
        {
            error!("\r\nError: write with parameter, block size {} is not support\r\n", block_size);
            return Err(MCIHostError::CardNotSupport);
        }

        if Err(MCIHostError::CardStatusIdle) != self.polling_card_status_busy(SD_CARD_ACCESS_WAIT_IDLE_TIMEOUT) {
            error!("Error : read failed with wrong card busy\r\n");
            return Err(MCIHostError::PollingCardIdleFailed);
        }

        let mut command = MCIHostCmd::new();
        command.response_type_set(MCIHostResponseType::R1);
        command.response_error_flags_set(MCIHostCardStatusFlag::ALL_ERROR_FLAG);
        command.index_set( 
            if block_count == 1 {
                MCIHostCommonCmd::WriteSingleBlock as u32
            } else {
                debug!("write multiple blocks! block count {}", block_count);
                MCIHostCommonCmd::WriteMultipleBlock as u32
            }
        );
        command.argument_set(
            if self.flags.contains(SdCardFlag::SupportHighCapacity) {
                start_block
            } else {
                start_block * block_size
            }
        );

        let mut data = MCIHostData::new();
        data.enable_auto_command12_set(false);
        data.block_size_set(block_size as usize);
        data.block_count_set(block_count);
        data.tx_data_set(Some(buffer.to_vec()));

        *written_blocks = block_count;

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
        content.set_data(Some(data));

        if let Err(e) = self.transfer(&mut content, 3) {
            return Err(e);
        } else {
            if let Err(e) = self.write_successful_block_send(written_blocks) {
                return Err(e);
            } else if *written_blocks == 0 {
                return Err(MCIHostError::TransferFailed);
            }
            debug!("written blocks this time is {}", written_blocks);
        }

        Ok(())
    }

    /// CMD 55 
    fn application_cmd_send(&mut self,relative_address:u32) -> MCIHostStatus {
        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        host.application_command_send(relative_address)
    }


}

impl SdCard {

    /// ACMD 6 
    fn data_bus_width_set(&mut self,width: MCIHostBusWdith) -> MCIHostStatus {

        /*
        * The specification strictly requires card interrupts to be masked, but
        * Linux does not do so, so we won't either.
        */
        /* Send ACMD6 to change bus width */
        if self.application_cmd_send(self.base.relative_address).is_err() {
            info!("SD app command failed for ACMD6");
            return Err(MCIHostError::SendApplicationCommandFailed);
        }

        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        
        let mut command = MCIHostCmd::new();

        command.index_set(SdAppCmd::SetBusWdith as u32);
        command.response_type_set(MCIHostResponseType::R1);

        match width {
            MCIHostBusWdith::Bit1 => {
                command.argument_set(0);
            },
            MCIHostBusWdith::Bit4 => {
                command.argument_set(2);
            },
            _ => {
                return Err(MCIHostError::InvalidArgument);
            },
        }

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));

        if let Err(err) = host.dev.transfer_function(&mut content,host) {
            let command = content.cmd().unwrap();
            let response = command.response();

            info!("\r\nError: send ACMD6 failed with host error {:?}, response 0x{:x}\r\n"
                ,err
                ,response[0]);
            return Err(MCIHostError::TransferFailed);
        }

        Ok(())
    }

    /// ACMD 13 
    fn status_read(&mut self) -> MCIHostStatus {

        // todo polling card status

        Ok(())
    }

    /// ACMD 41 
    fn application_opration_condition_send(&mut self,argument: u32) -> MCIHostStatus {
        
        let mut command = MCIHostCmd::new();

        command.index_set(SdAppCmd::SendOperationCondition as u32);
        command.argument_set(argument);
        command.response_type_set(MCIHostResponseType::R3);

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));

        let mut i = MCI_HOST_MAX_CMD_RETRIES;
        while i > 0 {

            if self.application_cmd_send(0).is_err() {
                continue;
            }

            let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
            
            if let Err(err) = host.dev.transfer_function(&mut content,host) {
                info!("\r\nError: send CMD8 failed with host error {:?}, response {}\r\n",
                    err,
                    {
                        let command = content.cmd().unwrap();
                        let response = command.response();
                        response[0]
                    });
                return Err(MCIHostError::TransferFailed);
            } 

            /* Wait until card exit busy state. */
            let command = content.cmd().unwrap();
            let response = command.response()[0];
            if response & MCIHostOCR::POWER_UP_BUSY_FLAG.bits() != 0 {
                /* high capacity check */
                if response & MCIHostOCR::HOST_CAPACITY_SUPPORT_FLAG.bits() != 0 {
                    self.flags |= SdCardFlag::SupportHighCapacity;
                    info!("capcity > 2GB, is high capacity card")
                }

                /* 1.8V support */
                if response & MCIHostOCR::SWITCH_18_ACCEPT_FLAG.bits() != 0 {
                    self.flags |= SdCardFlag::SupportVoltage180v;
                    info!("supports 1.8v, card is UHS card")
                } else {
                    info!("only supports 3.3v, card is not UHS card")
                }
                self.base.ocr = response;
                return Ok(());
            }

            i -= 1;
            sleep(Duration::from_millis(10));
        }

        info!("\r\nError: send ACMD41 timeout\r\n");
        Ok(())
    }

    /// ACMD 51 
    fn scr_send(&mut self) -> MCIHostStatus {
        if self.application_cmd_send(self.base.relative_address).is_err() {
            return Err(MCIHostError::SendApplicationCommandFailed);
        }

        let host = self.base.host.as_ref().ok_or(MCIHostError::HostNotReady)?;
        
        let mut command = MCIHostCmd::new();

        command.index_set(SdAppCmd::SendScr as u32);
        command.argument_set(0);
        command.response_type_set(MCIHostResponseType::R1);

        let mut data = MCIHostData::new();

        data.block_size_set(8);
        data.block_count_set(1);
        let tmp_buf = vec![0; 8];
        data.rx_data_set(Some(tmp_buf)); //todo 似乎影响性能 DMA 似乎是最好不要往栈上读写的?

        let mut content = MCIHostTransfer::new();
        content.set_cmd(Some(command));
        content.set_data(Some(data));

        if let Err(err) = host.dev.transfer_function(&mut content,host) {
            info!("\r\nError: send CMD51 failed with host error {:?}\r\n", err);
            return Err(err);
        }

        let raw_src = content.data_mut().unwrap().rx_data_mut().unwrap();
        debug!("src from transfer {:?}", raw_src);
        /* according to spec. there are two types of Data packet format for SD card
            1. Usual data (8-bit width), are sent in LSB first
            2. Wide width data (SD Memory register), are shifted from the MSB bit, 
                e.g. ACMD13 (SD Status), ACMD51 (SCR) */
        let _ = host.dev.convert_data_to_little_endian(raw_src, 2, MCIHostDataPacketFormat::MSBFirst,host);
        
        /* decode scr */
        self.decode_scr(&raw_src);

        Ok(())
    }
}

impl SdCard {
    fn decode_cid(&mut self) {

        let cid = &mut self.cid;
        // todo 可能存在性能问题
        let rawcid = match self.base.internal_buffer.to_vec::<u32>() {
            Err(e) => {
                error!("Construct Vec<u32> from internal_buffer failed! err: {:?}", e);
                panic!();
            },
            Ok(rawcid) => rawcid,
        };

        cid.manufacturer_id = ((rawcid[3] & 0xFF000000) >> 24) as u8;
        cid.application_id = ((rawcid[3] & 0xFFFF00) >> 8) as u16;

        cid.product_name[0] = (rawcid[3] & 0xFF) as u8;
        cid.product_name[1] = ((rawcid[2] & 0xFF000000) >> 24) as u8;
        cid.product_name[2] = ((rawcid[2] & 0xFF0000) >> 16) as u8;
        cid.product_name[3] = ((rawcid[2] & 0xFF00) >> 8) as u8;
        cid.product_name[4] = (rawcid[2] & 0xFF) as u8;

        cid.product_version = ((rawcid[1] & 0xFF000000) >> 24) as u8;
        cid.serial_number =
            ((rawcid[1] & 0xFFFFFF) << 8) | ((rawcid[0] & 0xFF000000) >> 24);

        cid.manufacturing_data = ((rawcid[0] & 0xFFF00) >> 8) as u16;
    }

    fn decode_csd(&mut self) {

        let csd = &mut self.csd;
        // todo 可能存在性能问题
        // let rawcsd = u8_to_u32_slice(&self.base.internal_buffer);
        let rawcsd = match self.base.internal_buffer.to_vec::<u32>() {
            Err(e) => {
                error!("Construct Vec<u32> from internal_buffer failed! err: {:?}", e);
                panic!();
            },
            Ok(rawcsd) => rawcsd,
        };

        csd.csd_structure = ((rawcsd[3] & 0xC0000000) >> 30) as u8;
        csd.data_read_access_time1 = ((rawcsd[3] & 0xFF0000) >> 16) as u8;
        csd.data_read_access_time2 = ((rawcsd[3] & 0xFF00) >> 8) as u8;
        csd.transfer_speed = (rawcsd[3] & 0xFF) as u8;
        csd.card_command_classes = ((rawcsd[2] & 0xFFF00000) >> 20) as u16;
        csd.read_block_length = ((rawcsd[2] & 0xF0000) >> 16) as u8;

        if rawcsd[2] & 0x8000 != 0 {
            csd.flags |= CsdFlags::READ_BLOCK_PARTIAL.bits();
        }
        if rawcsd[2] & 0x4000 != 0 {
            csd.flags |= CsdFlags::READ_BLOCK_PARTIAL.bits();
        }
        if rawcsd[2] & 0x2000 != 0 {
            csd.flags |= CsdFlags::READ_BLOCK_MISALIGN.bits();
        }
        if rawcsd[2] & 0x1000 != 0 {
            csd.flags |= CsdFlags::DSR_IMPLEMENTED.bits();
        }
        if  csd.csd_structure == 0 {
            info!("   csd structure: 1.0");
            csd.device_size =
                ((rawcsd[2] & 0x3FF) << 2) | ((rawcsd[1] & 0xC0000000) >> 30);
            csd.read_current_vdd_min = ((rawcsd[1] & 0x38000000) >> 27) as u8;
            csd.read_current_vdd_max = ((rawcsd[1] & 0x7000000) >> 24) as u8;
            csd.write_current_vdd_min = ((rawcsd[1] & 0xE00000) >> 20) as u8;
            csd.write_current_vdd_max = ((rawcsd[1] & 0x1C0000) >> 18) as u8;
            csd.device_size_multiplier = ((rawcsd[1] & 0x38000) >> 15) as u8;
            /* Get card total block count and block size. */
            self.block_count =
                (csd.device_size + 1) << (csd.device_size_multiplier + 2);
            self.base.block_size = 1 << csd.read_block_length;
            if self.base.block_size > MCI_HOST_DEFAULT_BLOCK_SIZE {
                self.block_count = self.block_count * self.base.block_size;
                self.base.block_size = MCI_HOST_DEFAULT_BLOCK_SIZE;
                self.block_count = self.block_count / self.base.block_size;
            }
        } else if csd.csd_structure == 1 {
            info!("   csd structure: 2.0");
            self.base.block_size = MCI_HOST_DEFAULT_BLOCK_SIZE;
            csd.device_size =
                ((rawcsd[2] & 0x3F) << 16) | ((rawcsd[1] & 0xFFFF0000) >> 16);
            if csd.device_size >= 0xFFFF {
                info!("device size is {}, supports sdxc", csd.device_size);
                self.flags |= SdCardFlag::SupportSdxc;
            }
            self.block_count = (csd.device_size + 1) * 1024;
        } else {
            info!(
                "unknown SD CSD structure version 0x{:x}",
                csd.csd_structure
            );
            /* not support csd version */
        }

        if ((rawcsd[1] & 0x4000) >> 14) as u8 != 0 {
            csd.flags |= CsdFlags::ERASE_BLOCK_ENABLED.bits();
        }

        csd.erase_sector_size = ((rawcsd[1] & 0x3F80) >> 7) as u8;
        csd.write_protect_group_size = (rawcsd[1] & 0x7F) as u8;

        if (rawcsd[0] & 0x80000000) as u8 != 0 {
            csd.flags |= CsdFlags::WRITE_PROTECT_GROUP_ENABLED.bits();
        }

        csd.write_speed_factor = ((rawcsd[0] & 0x1C000000) >> 26) as u8;
        csd.write_block_length = ((rawcsd[0] & 0x3C00000) >> 22) as u8;

        if ((rawcsd[0] & 0x200000) >> 21) as u8 != 0 {
            csd.flags |= CsdFlags::WRITE_BLOCK_PARTIAL.bits();
        }
        if ((rawcsd[0] & 0x8000) >> 15) as u8 != 0 {
            csd.flags |= CsdFlags::FILE_FORMAT_GROUP.bits();
        }
        if ((rawcsd[0] & 0x4000) >> 14) as u8 != 0 {
            csd.flags |= CsdFlags::COPY.bits();
        }
        if ((rawcsd[0] & 0x2000) >> 13) as u8 != 0 {
            csd.flags |= CsdFlags::PERMANENT_WRITE_PROTECT.bits();
        }
        if ((rawcsd[0] & 0x1000) >> 12) as u8 != 0 {
            csd.flags |= CsdFlags::TEMPORARY_WRITE_PROTECT.bits();
        }
        csd.file_format = ((rawcsd[0] & 0xC00) >> 10) as u8;

        info!(
            "Card block count {}, block size {}",
            self.block_count, self.base.block_size
        );
    }

    fn decode_scr(&mut self, rawscr: &Vec<u32>) {
        let scr = &mut self.scr;

        scr.scr_structure = ((rawscr[0] & 0xF0000000) >> 28) as u8;
        scr.sd_specification = ((rawscr[0] & 0xF000000) >> 24) as u8;
        if ((rawscr[0] & 0x800000) >> 23) as u8 != 0 {
            scr.flags |= ScrFlags::DATA_STATUS_AFTER_ERASE.bits();
        }
        scr.sd_security = ((rawscr[0] & 0x700000) >> 20) as u8;
        scr.sd_bus_widths = ((rawscr[0] & 0xF0000) >> 16) as u8;
        if ((rawscr[0] & 0x8000) >> 15) as u8 != 0 {
            scr.flags |= ScrFlags::SD_SPECIFICATION3.bits();
        }
        scr.extended_security = ((rawscr[0] & 0x7800) >> 10) as u8;
        scr.command_support = (rawscr[0] & 0x3) as u8;
        scr.reserved_for_manufacturer = rawscr[1];
        /* Get specification version. */
        if scr.sd_specification == 0 {
            info!("   SCR version: 1.0");
            self.version = SdSpecificationVersion::Version1_0;
        } else if scr.sd_specification == 1 {
            info!("   SCR version: 1.1");
            self.version = SdSpecificationVersion::Version1_1;
        } else if scr.sd_specification == 2 {
            info!("   SCR version: 2.0");
            self.version = SdSpecificationVersion::Version2_0;
            if scr.flags & ScrFlags::SD_SPECIFICATION3.bits() != 0 {
                info!("   SCR version: 3.0");
                self.version = SdSpecificationVersion::Version3_0;
            }
        } else {
            info!("   SCR version: unknown");
        }
        /* Check card supported bus width */
        if scr.sd_bus_widths & 0x4 != 0 {
            info!("   Card support 4-bit bus width");
            self.flags |= SdCardFlag::Support4BitWidth;
        }
        /* Check if card supports speed class command (CMD20) */
        if scr.command_support & 0x1 != 0 {
            info!("   Card support speed class control command");
            self.flags |= SdCardFlag::SupportSpeedClassControlCmd;
        }
        /* Check if card supports set block count command (CMD23) */
        if scr.command_support & 0x2 != 0 {
            info!("   Card support set block count command");
            self.flags |= SdCardFlag::SupportSetBlockCountCmd;
        }
    }
}

impl SdCard {
    fn card_dump(&self) {
        let mut card_name = [0u8;SD_PRODUCT_NAME_BYTES];
        card_name.copy_from_slice(self.cid.product_name.as_slice());
        info!("Card Name: {}",str::from_utf8(&card_name).unwrap());

        match self.version {
            SdSpecificationVersion::Version1_0 => {
                info!("Card Version: 1.0");
            },
            SdSpecificationVersion::Version1_1 => {
                info!("Card Version: 1.1");
            },
            SdSpecificationVersion::Version2_0 => {
                info!("Card Version: 2.0");
            },
            SdSpecificationVersion::Version3_0 => {
                info!("Card Version: 3.0");
            },
        }

        if self.flags.contains(SdCardFlag::SupportSdhc) {
            info!(" SDHC ");
        } 

        if self.flags.contains(SdCardFlag::SupportSdxc) {
            info!(" SDXC ");
        }

        info!("\r\n");

        info!("  Size: {} GB\r\n",(self.block_count as u64 *self.base.block_size as u64) / SZ_1G);
        
        if self.base.bus_clk_hz >(1000*1000) {
            info!("  Bus-Speed: {} MHz\r\n",self.base.bus_clk_hz/(1000*1000));
        } else if self.base.bus_clk_hz > 1000 {
            info!("  Bus-Speed: {} KHz\r\n",self.base.bus_clk_hz/1000);
        } else {
            info!("  Bus-Speed: {} Hz\r\n",self.base.bus_clk_hz);
        }

        match self.operation_voltage {
            MCIHostOperationVoltage::Voltage330V => {
                info!("  Voltage: 3.3v\r\n");
            },
            MCIHostOperationVoltage::Voltage300V => {
                info!("  Voltage: 3.0v\r\n");
            },
            MCIHostOperationVoltage::Voltage180V => {
                info!("  Voltage: 1.8v\r\n");
            },
            _ => {
                info!("  Voltage: unknown\r\n");
            }
        }

        match self.current_timing {
            SdTimingMode::SDR12DefaultMode => {
                if self.operation_voltage == MCIHostOperationVoltage::Voltage330V {
                    info!("  Timing: Default-Speed\r\n");
                } else if self.operation_voltage == MCIHostOperationVoltage::Voltage180V {
                    info!("  Timing: SDR12\r\n");
                }
            },
            SdTimingMode::SDR25HighSpeedMode => {
                if self.operation_voltage == MCIHostOperationVoltage::Voltage330V {
                    info!("  Timing: High-Speed\r\n");
                } else if self.operation_voltage == MCIHostOperationVoltage::Voltage180V {
                    info!("  Timing: SDR25\r\n");
                }
            },
            SdTimingMode::SDR50Mode => {
                info!("  Timing: SDR50 Mode\r\n");
            },
            SdTimingMode::SDR104Mode => {
                info!("  Timing: SDR104 Mode\r\n");
            },
            SdTimingMode::DDR50Mode => {
                info!("  Timing: DDR50 Mode\r\n");
            },
        }

        match self.max_current {
            SdMaxCurrent::Limit200mA => {
                info!("  Max. Current: 200mA\r\n");
            },
            SdMaxCurrent::Limit400mA => {
                info!("  Max. Current: 400mA\r\n");
            },
            SdMaxCurrent::Limit600mA => {
                info!("  Max. Current: 600mA\r\n");
            },
            SdMaxCurrent::Limit800mA => {
                info!("  Max. Current: 800mA\r\n");
            },
        }

        match self.driver_strength {
            SdDriverStrength::TypeA => {
                info!("  Drv. Type: A\r\n");
            },
            SdDriverStrength::TypeB => {
                info!("  Drv. Type: B\r\n");
            },
            SdDriverStrength::TypeC => {
                info!("  Drv. Type: C\r\n");
            },
            SdDriverStrength::TypeD => {
                info!("  Drv. Type: D\r\n");
            },
        }
    }
}