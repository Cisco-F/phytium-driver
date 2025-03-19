
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

//* 包内的引用 */
use err::*;
use constants::*;
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
    //todo cur_cmd needed
    io_pad: Option<IoPad>,
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
            io_pad: None,
        }
    }

    pub(crate) fn new_restart(config: MCIConfig) -> Self {
        MCI {
            config,
            is_ready: true,
            prev_cmd: 0,
            curr_timing: MCITiming::new(),
            io_pad: None,
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

    /* Setup DMA descriptor for SDIF controller instance */ // TODO

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

    /* Start command and data transfer in DMA mode */ // TODO
    /* Wait DMA transfer finished by poll */ // TODO

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

    pub fn transfer(&self,cmd_data: &mut FSdifCmdData,retry: u32) -> FsdifResult {
        let mut retry = retry;
        loop {
            if let Err(_) = self.pio_transfer(cmd_data) {
                retry -= 1;
                continue;
            }
            if let Err(_) = self.poll_wait_pio_end(cmd_data) {
                /* if transfer data failed, send cmd12 to abort current transfer */
                if cmd_data.data.buf.len() > 0 {
                    /* when transfer error occur, polling card status until it is ready for next data transfer, otherwise the
                    * retry transfer will fail again */
                    let _ = self.stop_transmission();
                    warn!("pio transfer failed, polling card status until it is ready for next data transfer");
                    self.polling_card_status_busy(600)?;
                }
                if retry ==0 {
                    // todo 为选择timing之后进行操作
                }
                retry -= 1;
                continue;
            }
            break;
        }
        Ok(())
    }
}

//* DMA 相关的函数 */
impl MCI {
    // todo dma burst
    pub fn enable_dma(&mut self) {
        self.reg.modify_reg(|reg| {
            FsdifBusMode::DE | reg
        });
    }

    pub fn dma_int_set(&mut self) {
        self.reg.modify_reg(|reg| {
            FsdifDmacIntEn::RI | FsdifDmacIntEn::TI | FsdifDmacIntEn::FBE | reg
        });
    }

    pub fn dump_dma_descriptor(&self, desc_in_use: u32) {
        if !self.desc_list.first_desc.is_null() {
            for i in 0..desc_in_use {
                unsafe {
                    let cur_desc = &*self.desc_list.first_desc.add(i as usize);
                    debug!("descriptor@{:p}", cur_desc);
                    debug!("\tattribute: 0x{:x}", (*cur_desc).attribute);
                    debug!("\tnon1: 0x{:x}", (*cur_desc).non1);
                    debug!("\tlen: 0x{:x}", (*cur_desc).len);
                    debug!("\tnon2: 0x{:x}", (*cur_desc).non2);
                    debug!("\taddr_lo: 0x{:x}", (*cur_desc).addr_lo);
                    debug!("\taddr_hi: 0x{:x}", (*cur_desc).addr_hi);
                    debug!("\tdesc_lo: 0x{:x}", (*cur_desc).desc_lo);
                    debug!("\tdesc_hi: 0x{:x}", (*cur_desc).desc_hi);
                }
            }
        }
    }

    pub fn setup_dma_descriptor(&mut self, data: &FsdifBuf) -> FsdifResult {
        let desc_list = &self.desc_list;
        // 一个desc可以传输的块数
        let desc_blocks = desc_list.desc_trans_sz / data.blksz; 
        let mut remain_blocks = data.blkcnt;
        let mut buf_addr = data.buf_dma;
        let mut trans_blocks: u32; // 本次循环被传输的块
        let mut is_first = true;
        let mut is_last = false;

        let mut desc_num = 1u32;
        let data_len = data.blkcnt * data.blksz;
        // 计算需要多少desc来传输
        if data_len > desc_list.desc_trans_sz {
            desc_num = data_len / desc_list.desc_trans_sz;
            desc_num += match data_len % desc_list.desc_trans_sz {
                0 => 0,
                _ => 1,
            };
        }

        if desc_num > desc_list.desc_num {
            error!("Transfer descriptor are not enough! desc need: {}, desc available: {}", desc_num, desc_list.desc_num);
            return Err(FsdifError::ShortBuf);
        }

        debug!("DMA transfer 0x{:x} use {} desc, total {} available", data.buf_dma, desc_num, desc_list.desc_num);

        unsafe {
            let ptr = desc_list.first_desc as *mut u8;
            let size = core::mem::size_of::<FSdifDmaDesc>() * desc_list.desc_num as usize;
            for i in 0..size {
                *ptr.add(i) = 0;
            }
        }

        for i in 0..desc_num {
            trans_blocks = if remain_blocks <= desc_blocks {
                remain_blocks
            } else {
                desc_blocks
            };
            unsafe {
                let cur_desc = self.desc_list.first_desc.add(i as usize);

                let mut next_desc_addr = desc_list.first_desc_dma as usize + (i + 1) as usize * core::mem::size_of::<FSdifDmaDesc>();

                is_first = i == 0;
                is_last = i == desc_num - 1;

                (*cur_desc).attribute = FSDIF_IDMAC_DES0_CH | FSDIF_IDMAC_DES0_OWN;
                if is_first {
                    (*cur_desc).attribute |= FSDIF_IDMAC_DES0_FD;
                }
                if is_last {
                    (*cur_desc).attribute |= FSDIF_IDMAC_DES0_LD | FSDIF_IDMAC_DES0_ER;
                }

                (*cur_desc).non1 = 0u32;
                (*cur_desc).len = trans_blocks * data.blksz as u32;

                if buf_addr as u32 % data.blksz != 0 {
                    error!("Data buffer 0x{:x} do not align to {}!", buf_addr, data.blksz);
                    return Err(FsdifError::DmaBufUnalign);
                }

                // aarch 架构
                (*cur_desc).addr_hi = (buf_addr >> 32) as u32;
                (*cur_desc).addr_lo = buf_addr as u32;

                next_desc_addr = if is_last { 0 } else { next_desc_addr };
                if next_desc_addr % core::mem::size_of::<FSdifDmaDesc>() != 0 {
                    error!("DMA descriptor 0x{:x} do not align!", next_desc_addr);
                    return Err(FsdifError::DmaBufUnalign);
                }

                (*cur_desc).desc_hi = (next_desc_addr >> 32) as u32;
                (*cur_desc).desc_lo = next_desc_addr as u32;

                buf_addr += (*cur_desc).len as usize;
                remain_blocks -= trans_blocks;
            }
        }
        unsafe{ dsb(); }
        self.dump_dma_descriptor(desc_num);

        Ok(())
    }

    /// start DMA transfers for data
    pub fn dma_transfer_data(&mut self, data: &FsdifBuf) -> FsdifResult {
        self.interrupt_mask_set(FsDifIntrType::GeneralIntr, FSDIF_INTS_CMD_MASK, true);
        self.interrupt_mask_set(FsDifIntrType::DmaIntr, FSDIF_DMAC_INTS_MASK, true);

        self.setup_dma_descriptor(&data)?;

        let data_len = data.blkcnt * data.blksz;
        info!("Descriptor@{:p}, trans bytes: {}, block size: {}", self.desc_list.first_desc, data_len, data.blksz);

        self.descriptor_set(self.desc_list.first_desc_dma as u32);
        self.trans_bytes_set(data_len);
        self.block_size_set(data.blksz);

        Ok(())
    }

    /// start command and data transfer in DMA mode
    pub fn dma_transfer(&mut self, cmd_data: &mut FSdifCmdData) -> FsdifResult {
        cmd_data.success = false;
        // todo 将self的cur_cmd指向传入的cmd_data

        if !self.is_ready {
            error!("Device is not yet initialized!");
            return Err(FsdifError::NotInit);
        }

        if self.config.trans_mode != FsDifTransMode::DmaTransMode {
            error!("Device is not configured in DMA transfer mode!");
            return Err(FsdifError::InvalidState);
        }

        /* for removable media, check if card exists */
        if !self.config.non_removable && !self.card_detected() {
            error!("card is not detected !!!");
            return Err(FsdifError::NoCard);
        }

        /* wait previous command finished and card not busy */
        self.poll_wait_busy_card()?;

        // 写0xffffe ？
        self.reg.write_reg(FsdifRawInts::from_bits_truncate((1 << 16) - 2));

        /* reset fifo and DMA before transfer */
        self.ctrl_reset(FsdifCtrl::FIFO_RESET | FsdifCtrl::DMA_RESET)?;

        /* enable use of DMA */
        let reg = self.reg.read_reg::<FsdifCtrl>();
        self.reg.write_reg(reg | FsdifCtrl::USE_INTERNAL_DMAC);
        let reg = self.reg.read_reg::<FsdifBusMode>();
        self.reg.write_reg(reg | FsdifBusMode::DE);

        // 传输数据为空，C SDK写法为指针为NULL，这里暂且这样写
        if cmd_data.data.buf.len() != 0 {
            self.dma_transfer_data(&cmd_data.data)?;
        }

        Ok(())
    }

    pub fn poll_wait_dma_end(&mut self, cmd_data: &mut FSdifCmdData) -> FsdifResult {
        let base_addr = self.iopad.get_base_addr();
        let wait_bits = if cmd_data.data.blkcnt == 0 { FSDIF_INT_CMD_BIT } else { FSDIF_INT_CMD_BIT | FSDIF_INT_DTO_BIT };
        let mut reg_val;

        if !self.is_ready {
            error!("Device is not yet initialized!");
            return Err(FsdifError::NotInit);
        }

        if self.config.trans_mode != FsDifTransMode::DmaTransMode {
            error!("Device is not configured in DMA transfer mode!");
            return Err(FsdifError::InvalidState);
        }

        /* wait command done or data timeout */
        let mut delay = FSDIF_TIMEOUT;
        loop {
            reg_val = self.reg.read_reg::<FsdifRawInts>().bits();
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
            return Err(FsdifError::CmdTimeout);
        }

        if cmd_data.data.buf.len() == 0 {
            let read = cmd_data.flag.bits() & FSDIF_CMD_FLAG_READ_DATA;
            if read != 0 {
                unsafe { dsb(); }
            }
        }

        self.get_cmd_response(cmd_data)?;

        Ok(())
    }

    /// Setup DMA descriptor for SDIF controller instance
    pub fn set_idma_list(&mut self, desc: *mut FSdifDmaDesc, desc_dma: u32, desc_num: u32) -> FsdifResult {
        if !self.is_ready {
            error!("Device is not yet initialized!");
            return Err(FsdifError::NotInit);
        }

        if self.config.trans_mode != FsDifTransMode::DmaTransMode {
            error!("Device is not configured in DMA transfer mode!");
            return Err(FsdifError::InvalidState);
        }

        self.desc_list.first_desc = desc;
        self.desc_list.first_desc_dma = desc_dma;
        self.desc_list.desc_num = desc_num;
        self.desc_list.desc_trans_sz = FSDIF_IDMAC_MAX_BUF_SIZE;

        Ok(())
    }
}

pub struct FsdifBuf<'a> {
    pub buf: &'a mut [u32],
    pub buf_dma: usize,
    pub blksz: u32,
    pub blkcnt: u32,
}

pub struct FSdifCmdData<'a> {
    pub cmdidx: u32,
    pub cmdarg: u32,
    pub response: [u32; 4],
    pub flag: CmdFlag,
    pub data: FsdifBuf<'a>,
    pub success: bool
}

bitflags! {
    pub struct CmdFlag: u32 {
        const NEED_INIT = 0x1;
        const EXP_RESP = 0x2;
        const EXP_LONG_RESP = 0x4;
        const NEED_RESP_CRC = 0x8;
        const EXP_DATA = 0x10;
        const WRITE_DATA = 0x20;
        const READ_DATA = 0x40;
        const NEED_AUTO_STOP = 0x80;
        const ADTC = 0x100;
        const SWITCH_VOLTAGE = 0x200;
        const ABORT = 0x400;
        const AUTO_CMD12 = 0x800;
    }
}

//* CMD 相关的方法 */
impl MCI {
    const FSDIF_EXT_APP_CMD: u32 = 55;
    const FSDIF_SWITCH_VOLTAGE:u32 = 11;
    pub fn transfer_cmd(&self, cmd_data: &FSdifCmdData) -> FsdifResult {
        let mut raw_cmd = FsdifCmd::empty();
        if self.curr_timing.use_hold {
            raw_cmd |= FsdifCmd::USE_HOLD_REG;
        }
        if cmd_data.flag.contains(CmdFlag::ABORT) {
            raw_cmd |= FsdifCmd::STOP_ABORT;
        }
        /* 命令需要进行卡初始化，如CMD-0 */
        if cmd_data.flag.contains(CmdFlag::NEED_INIT) {
            raw_cmd |= FsdifCmd::INIT;
        }
        /* 命令涉及电压切换 */
        if cmd_data.flag.contains(CmdFlag::SWITCH_VOLTAGE) {
            raw_cmd |= FsdifCmd::VOLT_SWITCH;
        }
        /* 命令传输过程伴随数据传输 */
        if cmd_data.flag.contains(CmdFlag::EXP_DATA) {
            raw_cmd |= FsdifCmd::DAT_EXP;
            if cmd_data.flag.contains(CmdFlag::WRITE_DATA) {
                raw_cmd |= FsdifCmd::DAT_WRITE;
            }
        }
        /* 命令需要进行CRC校验 */
        if cmd_data.flag.contains(CmdFlag::NEED_RESP_CRC) {
            raw_cmd |= FsdifCmd::RESP_CRC;
        }
        /* 命令需要响应回复 */
        if cmd_data.flag.contains(CmdFlag::EXP_RESP) {
            raw_cmd |= FsdifCmd::RESP_EXP;
            if cmd_data.flag.contains(CmdFlag::EXP_LONG_RESP) {
                raw_cmd |= FsdifCmd::RESP_LONG;
            }
        }
        raw_cmd |= FsdifCmd::from_bits_truncate(set_reg32_bits!(cmd_data.cmdidx, 5, 0));
        debug!("============[{}-{}]@0x{:x} begin ============",
        {
            if self.prev_cmd == Self::FSDIF_EXT_APP_CMD {
                "ACMD"
            } else {
                "CMD"
            }
        },
        cmd_data.cmdidx,
        self.reg.addr.as_ptr() as usize );
        debug!("    cmd: 0x{:x}", raw_cmd.bits());
        debug!("    arg: 0x{:x}", cmd_data.cmdarg);
        /* enable related interrupt */
        self.interrupt_mask_set(FsDifIntrType::GeneralIntr, FsdifInt::INTS_CMD_MASK.bits(), true);
        self.send_private_cmd(raw_cmd, cmd_data.cmdarg);
        info!("cmd send done");
        Ok(())
    }
    pub fn get_cmd_response(&self,cmd_data: &mut FSdifCmdData) -> FsdifResult{
        let read = cmd_data.flag.contains(CmdFlag::READ_DATA);
        if !self.is_ready {
            error!("device is not yet initialized!!!");
            return Err(FsdifError::NotInit);
        }
        if read {
            if FsDifTransMode::PioTransMode == self.config.trans_mode {
                self.pio_read_data(cmd_data.data.buf)?;
            }
        }
        /* check response of cmd */
        if cmd_data.flag.contains(CmdFlag::EXP_RESP) {
            if cmd_data.flag.contains(CmdFlag::EXP_LONG_RESP) {
                cmd_data.response[0] = self.reg.read_reg::<FsdifResp0>().bits();
                cmd_data.response[1] = self.reg.read_reg::<FsdifResp1>().bits();
                cmd_data.response[2] = self.reg.read_reg::<FsdifResp2>().bits();
                cmd_data.response[3] = self.reg.read_reg::<FsdifResp3>().bits();
                debug!("    resp: 0x{:x}-0x{:x}-0x{:x}-0x{:x}",
                cmd_data.response[0],cmd_data.response[1],cmd_data.response[2],cmd_data.response[3]);
            }else {
                cmd_data.response[0] = self.reg.read_reg::<FsdifResp0>().bits();
                cmd_data.response[1] = 0;
                cmd_data.response[2] = 0;
                cmd_data.response[3] = 0;
                debug!("    resp: 0x{:x}",cmd_data.response[0]);
            }
        }
        cmd_data.success = true;
        debug!("============[{}-{}]@0x{:x} end ============",
        {
            if self.prev_cmd == Self::FSDIF_EXT_APP_CMD {
                "ACMD"
            } else {
                "CMD"
            }
        },
        cmd_data.cmdidx,
        self.reg.addr.as_ptr() as usize );
        /* disable related interrupt */
        self.interrupt_mask_set(FsDifIntrType::GeneralIntr,(FsdifInt::INTS_CMD_MASK|FsdifInt::INTS_DATA_MASK).bits(),false);
        self.interrupt_mask_set(FsDifIntrType::DmaIntr,FsdifDmacIntEn::INTS_MASK.bits(),false);
        info!("cmd send done ...");
        Ok(())
    }
}

//* Interrupt 相关的方法 */
impl MCI {
    pub fn interrupt_mask_get(&self, tp: FsDifIntrType) -> u32 {
        let mut mask = 0;
        if FsDifIntrType::GeneralIntr == tp {
            mask = self.reg.read_reg::<FsdifInt>().bits();
        } else if FsDifIntrType::DmaIntr == tp {
            mask = self.reg.read_reg::<FsdifDmacIntEn>().bits();
        }
        //? 这里不知道要不要用Some作为返回值 
        mask
    }

    pub fn interrupt_mask_set(&self, tp: FsDifIntrType, set_mask: u32, enable: bool) {
        let mut mask = self.interrupt_mask_get(tp);
        if enable {
            mask |= set_mask;
        } else {
            mask &= !set_mask;
        }
        if FsDifIntrType::GeneralIntr == tp {
            self.reg.write_reg(FsdifInt::from_bits_truncate(mask));
        } else if FsDifIntrType::DmaIntr == tp {
            self.reg.write_reg(FsdifDmacIntEn::from_bits_truncate(mask));
        }
    }
}

impl MCI {
    pub fn show_status(&self) {
        debug!("status: 0x{:x}", self.reg.read_reg::<FsdifStatus>());
    }
    pub fn show_cntrl(&self) {
        debug!("cntrl: 0x{:x}", self.reg.read_reg::<FsdifCtrl>());
    }
    pub fn show_busmode(&self) {
        debug!("busmode: 0x{:x}", self.reg.read_reg::<FsdifBusMode>());
    }
    pub fn show_blksiz(&self) {
        debug!("blksiz: 0x{:x}", self.reg.read_reg::<FsdifBlkSiz>());
    }
    pub fn show_bytcnt(&self) {
        debug!("bytcnt: 0x{:x}", self.reg.read_reg::<FsdifBytCnt>());
    }
    pub fn show_dmacinten(&self) {
        debug!("dmacinten: 0x{:x}", self.reg.read_reg::<FsdifDmacIntEn>());
    }
    pub fn dump_register(&self) {
        warn!("cntrl: 0x{:x}", self.reg.read_reg::<FsdifCtrl>());
        warn!("pwren: 0x{:x}", self.reg.read_reg::<FsdifPwrEn>());
        warn!("clkdiv: 0x{:x}", self.reg.read_reg::<FsdifClkDiv>());
        warn!("clkena: 0x{:x}", self.reg.read_reg::<FsdifClkEn>());
        warn!("tmout: 0x{:x}", self.reg.read_reg::<FsdifTimeout>());
        warn!("ctype: 0x{:x}", self.reg.read_reg::<FsdifCType>());
        warn!("blksz: 0x{:x}", self.reg.read_reg::<FsdifBlkSiz>());
        warn!("blkcnt: 0x{:x}", self.reg.read_reg::<FsdifBytCnt>());
        warn!("intmask: 0x{:x}", self.reg.read_reg::<FsdifInt>());
        warn!("cmdarg: 0x{:x}", self.reg.read_reg::<FsdifCmdArg>());
        warn!("cmd: 0x{:x}", self.reg.read_reg::<FsdifCmd>());
        warn!("resp0: 0x{:x}", self.reg.read_reg::<FsdifResp0>());
        warn!("reps1: 0x{:x}", self.reg.read_reg::<FsdifResp1>());
        warn!("resp2: 0x{:x}", self.reg.read_reg::<FsdifResp2>());
        warn!("resp3: 0x{:x}", self.reg.read_reg::<FsdifResp3>());
        warn!("maskints: 0x{:x}", self.reg.read_reg::<FsdifMaskedInts>());
        warn!("rawints: 0x{:x}", self.reg.read_reg::<FsdifRawInts>());
        warn!("status: 0x{:x}", self.reg.read_reg::<FsdifStatus>());
        warn!("fifoth: 0x{:x}", self.reg.read_reg::<FsdifFifoTh>());
        warn!("carddet: 0x{:x}", self.reg.read_reg::<FsdifCardDetect>());
        warn!("wrtprt: 0x{:x}", self.reg.read_reg::<FsdifCardWrtp>());
        warn!("cksts: 0x{:x}", self.reg.read_reg::<FsdifClkSts>());
        warn!("trans_cardcnt: 0x{:x}", self.reg.read_reg::<FsdifTranCardCnt>());
        warn!("trans_fifocnt: 0x{:x}", self.reg.read_reg::<FsdifTranFifoCnt>());
        warn!("debnce: 0x{:x}", self.reg.read_reg::<FsdifDebnce>());
        warn!("uid: 0x{:x}", self.reg.read_reg::<FsdifUid>());
        warn!("vid: 0x{:x}", self.reg.read_reg::<FsdifVid>());
        warn!("hwconf: 0x{:x}", self.reg.read_reg::<FsdifHwconf>());
        warn!("uhsreg: 0x{:x}", self.reg.read_reg::<FsdifUhsReg>());
        warn!("cardreset: 0x{:x}", self.reg.read_reg::<FsdifCardReset>());
        warn!("busmode: 0x{:x}", self.reg.read_reg::<FsdifBusMode>());
        warn!("descaddrl: 0x{:x}", self.reg.read_reg::<FsdifDescListAddrL>());
        warn!("descaddrh: 0x{:x}", self.reg.read_reg::<FsdifDescListAddrH>());
        warn!("dmacstatus: 0x{:x}", self.reg.read_reg::<FsdifDmacStatus>());
        warn!("dmacinten: 0x{:x}", self.reg.read_reg::<FsdifDmacIntEn>());
        warn!("curdescaddrl: 0x{:x}", self.reg.read_reg::<FsdifCurDescAddrL>());
        warn!("curdescaddrh: 0x{:x}", self.reg.read_reg::<FsdifCurDescAddrH>());
        warn!("curbufaddrl: 0x{:x}", self.reg.read_reg::<FsdifCurBufAddrL>());
        warn!("curbufaddrh: 0x{:x}", self.reg.read_reg::<FsdifCurBufAddrH>());
        warn!("card_thrctl: 0x{:x}", self.reg.read_reg::<FsdifCardThrctl>());
        warn!("clock_src: 0x{:x}", self.reg.read_reg::<FsdifClkSrc>());
        warn!("emmcddr: 0x{:x}", self.reg.read_reg::<FsdifEmmcDdrReg>());
        warn!("enableshift: 0x{:x}", self.reg.read_reg::<FsdifEnableShift>());

    }
}

