
//* 包内共用的一些子包 可能其它模块有同名的子包 不要pub出来 */
pub mod constants;
pub mod regs;
mod err;

//* 功能相关的子包 */
mod mci_timing;
mod mci_config;
mod mci_cmd;
mod mci_hardware;
mod mci_intr;
pub mod mci_data;
mod mci_cmddata;
mod mci_pio;
pub mod mci_dma;

//* 包内的引用 */
use err::*;
use constants::*;
use mci_dma::{FSdifIDmaDescList, FSdifIDmaDesc};
use regs::*;
use log::*;

pub use mci_cmddata::*;
pub use mci_config::*;
pub use mci_timing::*;


//* 包外的引用 */
use crate::{regs::*, sleep, IoPad};
use core::time::Duration;

//* MCI */
pub struct MCI {
    config: MCIConfig,
    is_ready: bool,
    prev_cmd: u32, // todo 这里需要实现成一个实现了Command的enum
    curr_timing: MCITiming,
    cur_cmd: Option<MCICmdData>,
    io_pad: Option<IoPad>,
    desc_list: FSdifIDmaDescList,
}

//* MCI constance */
impl MCI {
    const SWITCH_VOLTAGE: u32 = 11;
    const EXT_APP_CMD: u32 = 55;
    
    pub(crate) fn relax_handler() {
        sleep(Duration::from_micros(10));
    }

    pub(crate) fn new(config: MCIConfig) -> Self {
        MCI {
            config,
            is_ready: false,
            prev_cmd: 0,
            curr_timing: MCITiming::new(),
            cur_cmd: None,
            io_pad: None,
            desc_list: FSdifIDmaDescList::new(),
        }
    }

    pub(crate) fn new_restart(config: MCIConfig) -> Self {
        MCI {
            config,
            is_ready: true,
            prev_cmd: 0,
            curr_timing: MCITiming::new(),
            cur_cmd: None,
            io_pad: None,
            desc_list: FSdifIDmaDescList::new(),
        }
    }
}

//* MCI pub API */
impl MCI {

    pub fn iopad_set(&mut self, iopad: IoPad) {
        self.io_pad = Some(iopad);
    }

    pub fn iopad_take(&mut self) -> Option<IoPad> {
        self.io_pad.take()
    }

    // todo 避免所有权问题先用了clone
    pub fn cur_cmd_set(&mut self, cmd: &MCICmdData) {
        self.cur_cmd = Some(cmd.clone());
    }

    /* initialization SDIF controller instance */
    pub fn config_init(&mut self,config: &MCIConfig) -> MCIResult {
        if self.is_ready {
            warn!("Device is already initialized!!!");
        }
        if *config != self.config {
            self.config = config.clone();
        }
        if let Ok(_) = self.reset() {
            self.is_ready = true;
            info!("Device initialize success !!!");
        }
        Ok(())
    }

    /* deinitialization SDIF controller instance */
    pub fn config_deinit(&mut self) -> MCIResult {
        self.interrupt_mask_set(MCIIntrType::GeneralIntr, MCIIntMask::ALL_BITS.bits(), false); /* 关闭控制器中断位 */
        self.interrupt_mask_set(MCIIntrType::DmaIntr, MCIDMACIntEn::ALL_BITS.bits(), false); /* 关闭DMA中断位 */
        
        self.raw_status_clear(); /* 清除中断状态 */
        self.dma_status_clear();

        self.power_set(false); /* 关闭电源 */
        self.clock_set(false); /* 关闭卡时钟 */

        let reg = self.config.reg();
        reg.clear_reg(MCIClkSrc::UHS_EXT_CLK_ENA); /* 关闭外部时钟 */
        reg.clear_reg(MCIUhsReg::VOLT_180);/* 恢复为3.3v默认电压 */

        self.is_ready = false;
        Ok(())
    }

    // pub fn desc_list_get(&self) -> FSdifIDmaDescList {
    //     self.desc_list
    // }

    /* Setup DMA descriptor for SDIF controller instance */
    // 暂时修改报错类型
    pub fn set_idma_list(&mut self, desc: *mut FSdifIDmaDesc, desc_dma: u32, desc_num: u32) {
        if !self.is_ready {
            error!("Device is not yet initialized!");
            // return Err(MCIHostError::NotInit);
        }

        if self.config.trans_mode() != MCITransMode::DMA {
            error!("Device is not configured in DMA transfer mode!");
            // return Err(MCIError::InvalidState);
        }

        self.desc_list.first_desc = desc;
        self.desc_list.first_desc_dma = desc_dma;
        self.desc_list.desc_num = desc_num;
        self.desc_list.desc_trans_sz = FSDIF_IDMAC_MAX_BUF_SIZE;

        // Ok(())
    }

    /* Set the Card clock freqency */
    pub fn clk_freq_set(&mut self, clk_hz: u32) -> MCIResult {
        let reg = self.config.reg();
        let mut reg_val = MCICmd::UPD_CLK;

        let cmd_reg = reg.read_reg::<MCICmd>();
        let cur_cmd_index =  cmd_reg.index_get();

        info!("Set clk as {}",clk_hz);
        if cur_cmd_index == Self::SWITCH_VOLTAGE {
            reg_val |= MCICmd::VOLT_SWITCH;
        }

        if clk_hz > 0 {
            /* select board-related time-tuning configurations */
            let target_timing = 
                MCIConfig::get_tuning(
                    clk_hz.into(),
                    self.config.non_removable()).ok_or_else(|| {
                        error!("No available timing !!!");
                        MCIError::InvalidTiming
                    })?;
            /* update pad delay */
            target_timing.pad_delay(self.io_pad.as_mut().unwrap(), self.config.instance_id());

            /* update clock source setting */
            self.update_exteral_clk(MCIClkSrc::from_bits_retain(target_timing.clk_src()))?;
           
            self.clock_set(false);

            /* update clock for clock source */
            if let Err(err) = 
                if cur_cmd_index == Self::SWITCH_VOLTAGE as u32 {
                    self.private_cmd11_send(reg_val | cmd_reg)
                } else {
                    self.private_cmd_send(reg_val, 0)
                }{
                error!("update ext clock failed !!!");
                return Err(err);
            }

            /* set clock divider */
            reg.write_reg(MCIClkDiv::from_bits_truncate(target_timing.clk_div()));
            reg.write_reg(MCIEnableShift::from_bits_truncate(target_timing.shift()));
            info!("clk_src: 0x{:x} clk_div: 0x{:x}, shift: 0x{:x}",
                reg.read_reg::<MCIClkSrc>(),
                reg.read_reg::<MCIClkDiv>(),
                reg.read_reg::<MCIEnableShift>());

            self.clock_set(true);

            /* update clock for clock divider */
            if cur_cmd_index == Self::SWITCH_VOLTAGE as u32 {
                self.private_cmd11_send(reg_val | cmd_reg)?;
            } else {
                self.private_cmd_send(reg_val, 0)?;
            }

            self.curr_timing = target_timing;
        } else {
            /* close bus clock in case target clock is 0 */
            self.clock_set(false);

            if cur_cmd_index == Self::SWITCH_VOLTAGE as u32 {
                self.private_cmd11_send(reg_val | cmd_reg)?;
            } else {
                self.private_cmd_send(reg_val, 0)?;
            }

            reg.clear_reg(MCIClkSrc::UHS_EXT_CLK_ENA);

            self.curr_timing = MCITiming::new();
        }
        Ok(())
    }

    /// Start command and data transfer in DMA mode
    pub fn dma_transfer(&mut self, cmd_data: &mut MCICmdData) -> MCIResult {
        cmd_data.success_set(false);
        self.cur_cmd_set(&cmd_data);

        if !self.is_ready {
            error!("Device is not yet initialized!");
            return Err(MCIError::NotInit);
        }

        if self.config.trans_mode() != MCITransMode::DMA {
            error!("Device is not configured in DMA transfer mode!");
            return Err(MCIError::InvalidState);
        }

        // for removable media, check if card exists
        if !self.config.non_removable() && !self.check_if_card_exist() {
            error!("card is not detected !!!");
            return Err(MCIError::NoCard);
        }

        // wait previous command finished and card not busy
        self.poll_wait_busy_card()?;

        // 清除原始中断寄存器
        self.config.reg().write_reg(MCIRawInts::from_bits_truncate((1 << 16) - 2));

        /* reset fifo and DMA before transfer */
        self.ctrl_reset(MCICtrl::FIFO_RESET | MCICtrl::DMA_RESET)?;

        // enable use of DMA
        self.config.reg().modify_reg(|reg| { MCICtrl::USE_INTERNAL_DMAC | reg });
        self.config.reg().modify_reg(|reg| { MCIBusMode::DE | reg });

        // transfer data
        if cmd_data.get_data().is_some() {
            self.dma_transfer_data(cmd_data.get_data().unwrap())?;
        }

        // transfer command
        self.cmd_transfer(&cmd_data)?;

        Ok(())
    }

    /* Wait DMA transfer finished by poll */
    pub fn poll_wait_dma_end(&mut self, cmd_data: &mut MCICmdData) -> MCIResult {
        let wait_bits = if cmd_data.get_data().unwrap().blkcnt() == 0 { FSDIF_INT_CMD_BIT } else { FSDIF_INT_CMD_BIT | FSDIF_INT_DTO_BIT };
        let mut reg_val;

        if !self.is_ready {
            error!("Device is not yet initialized!");
            return Err(MCIError::NotInit);
        }

        if self.config.trans_mode() != MCITransMode::DMA {
            error!("Device is not configured in DMA transfer mode!");
            return Err(MCIError::InvalidState);
        }

        /* wait command done or data timeout */
        let mut delay = RETRIES_TIMEOUT;
        loop {
            reg_val = self.config.reg().read_reg::<MCIRawInts>().bits();
            if delay % 1000 == 0 {
                debug!("reg_val = 0x{:x}", reg_val);
            }
            // todo relax handler? 

            delay -= 1;
            if wait_bits & reg_val == wait_bits || delay == 0 {
                break;
            }
        }

        /* clear status to ack data done */
        self.raw_status_clear();

        if wait_bits & reg_val != wait_bits && delay <= 0 {
            error!("Wait command done timeout, raw ints: 0x{:x}!", reg_val);
            return Err(MCIError::CmdTimeout);
        }

        if cmd_data.get_data().unwrap().buf().unwrap().len() == 0 {
            let read = cmd_data.flag().bits() & FSDIF_CMD_FLAG_READ_DATA;
            if read != 0 {
                unsafe { dsb(); }
            }
        }

        self.cmd_response_get(cmd_data)?;

        Ok(())
    }

    /* Start command and data transfer in PIO mode */
    pub fn pio_transfer(&self, cmd_data: &mut MCICmdData) -> MCIResult {
        let read = cmd_data.flag().contains(MCICmdFlag::READ_DATA);
        let reg = self.config.reg();

        cmd_data.success_set(false);

        if !self.is_ready{
            error!("device is not yet initialized!!!");
            return Err(MCIError::NotInit);
        }
        if self.config.trans_mode() != MCITransMode::PIO {
            error!("device is not configure in PIO transfer mode.");
            return Err(MCIError::InvalidState);
        }

        /* for removable media, check if card exists */
        if !self.config.non_removable() && !self.check_if_card_exist() {
            error!("card is not detected !!!");
            return Err(MCIError::NoCard);
        }

        /* wait previous command finished and card not busy */
        self.poll_wait_busy_card()?;

        /* reset fifo and not use DMA */
        reg.clear_reg(MCICtrl::USE_INTERNAL_DMAC);
        self.ctrl_reset(MCICtrl::FIFO_RESET)?;
        reg.clear_reg(MCIBusMode::DE);
  
        /* transfer data */
        if let Some(data) = cmd_data.get_mut_data() {
            /* while in PIO mode, max data transferred is 0x800 */
            if data.datalen() > MCI_MAX_FIFO_CNT {
                error!("Fifo do not support writing more than {:x}.",MCI_MAX_FIFO_CNT);
                return Err(MCIError::NotSupport);
            }

            /* set transfer data length and block size */
            self.trans_bytes_set(data.datalen() as u32);
            self.blksize_set(data.blksz());

            /* if need to write, write to fifo before send command */
            if !read { 
                /* invalide buffer for data to write */
                // unsafe { dsb() };
                self.pio_write_data(data)?;
            }
        }
        self.cmd_transfer(cmd_data)?;
        Ok(())
    }

    /* Wait PIO transfer finished by poll */
    pub fn poll_wait_pio_end(&mut self,cmd_data: &mut MCICmdData) -> MCIResult{
        let read = cmd_data.flag().contains(MCICmdFlag::READ_DATA);
        let reg = self.config.reg();

        if !self.is_ready {
            error!("device is not yet initialized!!!");
            return Err(MCIError::NotInit);
        }

        if MCITransMode::PIO != self.config.trans_mode() {
            error!("device is not configure in PIO transfer mode.");
            return Err(MCIError::InvalidState);
        }

        info!("wait for PIO cmd to finish ...");
        if let Err(err) = reg.retry_for(|reg: MCIRawInts|{
            let result = reg.contains(MCIRawInts::CMD_BIT);
            MCI::relax_handler();
            result
        }, Some(RETRIES_TIMEOUT)){
            error!("wait cmd done timeout, raw ints: 0x{:x}",
                    self.raw_status_get());
            return Err(err);
        }

        /* if need to read data, read fifo after send command */
        if cmd_data.get_data().is_some() && read {
            info!("wait for PIO data to read ...");
            if let Err(err)=reg.retry_for(|reg|{
                MCI::relax_handler();
                (MCIRawInts::DTO_BIT & reg).bits() != 0
            }, Some(RETRIES_TIMEOUT)){
                self.raw_status_clear();
                return Err(err);
            }

            /* clear status to ack */
            self.raw_status_clear();
            info!("card cnt: 0x{:x}, fifo cnt: 0x{:x}",
                   reg.read_reg::<MCITranCardCnt>(),
                   reg.read_reg::<MCITranFifoCnt>());
        }
        /* clear status to ack cmd done */
        self.raw_status_clear();
        self.cmd_response_get(cmd_data)?;
        Ok(())
    }

    /* Read PIO data, it works in IRQ mode */ // todo 不知道协议栈层需要不需要调用,已经实现.
    /* Get cmd response and received data after wait poll status or interrupt signal */ // todo 不知道协议栈层需要不需要调用,已经实现.

    /* Interrupt handler for SDIF instance */ //todo 在中断模式下会使用到
    /* Register event call-back function as handler for interrupt events */ //todo 在中断模式下会使用到

    /* Reset controller from error state */
    pub fn restart(&self) -> MCIResult {

        if false == self.is_ready {
            error!("Device is not yet initialized!!!");
            return Err(MCIError::NotInit);
        }

        /* reset controller */
        self.ctrl_reset(MCICtrl::FIFO_RESET)?;

        /* reset controller if in busy state */
        self.busy_card_reset()?;

        /* reset clock */
        self.clk_restart()?;

        /* reset internal DMA */
        if self.config.trans_mode() == MCITransMode::DMA {
            self.idma_reset();
        }
        Ok(())
    }


    /* Dump all register value of SDIF instance */
    pub fn register_dump(&self) {
        let reg = self.config.reg();
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

    /* Dump command and data info */
    pub fn cmd_info_dump(cmd_data: &MCICmdData){
        debug!("cmd struct @{:p}",cmd_data);
        debug!("   opcode: {}",cmd_data.cmdidx());
        debug!("   arg: 0x{:x}",cmd_data.cmdarg());
        let response = cmd_data.get_response();
        debug!("   resp@{:p}: 0x{:x} 0x{:x} 0x{:x} 0x{:x}",
                response,
                response[0],
                response[1],
                response[2],
                response[3]);
        debug!("   flag: 0x{:x}",cmd_data.flag());
        debug!("   data @{:p}",cmd_data.get_data().unwrap());
        
        if let Some(data) = cmd_data.get_data() {
            debug!("   buf: {:p}, len: {}",data,data.datalen());
            debug!("   blk sz: {}",data.blksz());
            debug!("   blk cnt: {}",data.blkcnt());
        }
    }
}


//* MCI private API */
impl MCI {
    fn reset(&self) -> MCIResult {
        /* set fifo */
        self.fifoth_set(
            MCIFifoThDMATransSize::DMATrans8, 
            MCIFifoTh::RX_WMARK, 
            MCIFifoTh::TX_WMARK);  
        
        /* set card threshold */
        self.cardthr_set(MCIFifoDepth::Depth8);
    
        /* disable clock and update ext clk */
        self.clock_set(false);

        /* set 1st clock */
        if let Err(err) = self.init_external_clk() {
            error!("Update extern clock failed: {:?}", err);
            return Err(err);
        }

        /* power on */
        self.power_set(true);  
        self.clock_set(true); 
        self.clock_src_set(true);

        /* set voltage as 3.3v */
        self.voltage_1_8v_set(false);  

        /* set bus width as 1 */
        self.bus_width_set(1);

        /* reset controller and card */
        if let Err(err) = 
            if self.config.trans_mode() == MCITransMode::DMA {
                self.ctrl_reset(MCICtrl::FIFO_RESET | MCICtrl::DMA_RESET)
            } else {
                self.ctrl_reset(MCICtrl::FIFO_RESET)
            } 
        {
            error!("Reset controller failed: {:?}", err);
            return Err(err);
        }

        /* send private command to update clock */
        self.private_cmd_send(MCICmd::UPD_CLK, 0)?;

        /* reset card for no-removeable media, e.g. eMMC */
        self.cardreset_config();

        /* clear interrupt status */  
        self.clear_interrupt_status();

        /* get base address */
        let reg = self.config.reg();

        /* enable card detect interrupt */
        if !self.config.non_removable() {
            reg.set_reg(MCIIntMask::CD_BIT);
        }
        
        /* enable controller and internal DMA */
        reg.set_reg(MCICtrl::INT_ENABLE | MCICtrl::USE_INTERNAL_DMAC);
        
        /* set data and resp timeout */
        reg.write_reg(MCITimeout::timeout_data(
            MCITimeout::MAX_DATA_TIMEOUT, 
            MCITimeout::MAX_RESP_TIMEOUT));
        
        /* reset descriptors and dma */
        if self.config.trans_mode() == MCITransMode::DMA {
            self.descriptor_set(0);
            self.idma_reset();
        }

        info!("Reset hardware done !!!");
        Ok(())
    }

    fn poll_wait_busy_card(&self) -> MCIResult {
        let busy_bits = MCIStatus::DATA_BUSY | MCIStatus::DATA_STATE_MC_BUSY;
        let reg = self.config.reg();
        let reg_val = reg.read_reg::<MCIStatus>();
        if reg_val.contains(busy_bits.clone()) {
           warn!("Card is busy, waiting ...");
        }
        if let Err(err) =reg.retry_for(|reg: MCIStatus|{
            let result = !reg.contains(busy_bits);
            MCI::relax_handler();
            result
        }, Some(RETRIES_TIMEOUT)){
            error!("Wait card busy timeout !!!");
            return Err(err);
        }
        Ok(())
    }

    fn busy_card_reset(&self) -> MCIResult {
        let reg = self.config.reg();

        reg.set_reg(MCICtrl::CONTROLLER_RESET);

        reg.retry_for(|reg_val: MCIStatus|{
            reg.set_reg(MCICtrl::CONTROLLER_RESET);
            !reg_val.contains(MCIStatus::DATA_BUSY)
        }, Some(RETRIES_TIMEOUT))?;

        Ok(())
    }

    fn clk_restart(&self) -> MCIResult {
        let reg = self.config.reg();

        /* wait command finish if previous command is in error state */
        reg.retry_for(|reg|{
            (MCICmd::START & reg).bits() == 0
        }, Some(RETRIES_TIMEOUT))?;

        /* update clock */
        self.clock_set(false);

        let clk_div = reg.read_reg::<MCIClkDiv>();
        let uhs = reg.read_reg::<MCIClkSrc>();

        self.update_exteral_clk(uhs)?;

        reg.write_reg(clk_div);

        self.clock_set(true);

        self.private_cmd_send(MCICmd::UPD_CLK, 0)?;
        Ok(())
    }
}