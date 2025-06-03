use log::*;

use crate::aarch::dsb;

use super::mci_cmddata::MCICmdData;
use super::MCI;
use super::err::*;
use super::consts::*;
use super::regs::*;

impl MCI {
    pub(crate) fn private_cmd_send(&self,cmd:MCICmd, arg: u32) -> MCIResult{
        // self.interrupt_mask_set(MCIIntrType::GeneralIntr, MCIIntMask::INTS_DATA_MASK.bits(), false);
        // self.clear_interrupt_status();
        let reg = self.config.reg();

        warn!("raw ints: 0x{:x}", reg.read_reg::<MCIRawInts>());
        warn!("int mask: 0x{:b}", reg.read_reg::<MCIIntMask>());

        reg.retry_for(|reg: MCIStatus| {
            !reg.contains(MCIStatus::DATA_BUSY)
        }, Some(RETRIES_TIMEOUT))?;
        reg.write_reg(MCICmdArg::from_bits_truncate(arg));

        unsafe { dsb() };/* drain writebuffer */

        let int_mask = MCIIntMask::INTS_CMD_MASK | MCIIntMask::INTS_DATA_MASK;

        info!("raw ints 0x{:x}", reg.read_reg::<MCIRawInts>().bits());
        let cmd_reg = MCICmd::START | cmd;
        info!("writing cmd reg");
        reg.write_reg(cmd_reg);
        // self.interrupt_mask_set(MCIIntrType::GeneralIntr, int_mask.bits(), true);
        // self.interrupt_mask_set(MCIIntrType::DmaIntr, MCIDMACIntEn::INTS_MASK.bits(), true);
        info!("after write reg");

        reg.retry_for(|reg:MCICmd|{
            !reg.contains(MCICmd::START)
        }, Some(RETRIES_TIMEOUT))?;
        warn!("after write cmd reg raw ints: 0x{:x}", reg.read_reg::<MCIRawInts>());

        info!("private cmd ok");
        Ok(())
    }

    pub(crate) fn private_cmd11_send(&self,cmd:MCICmd) -> MCIResult{
        let reg = self.config.reg();
        unsafe { dsb() };/* drain writebuffer */
        reg.write_reg(MCICmd::START | cmd);
        reg.retry_for(|reg|{
            (MCICmd::START & reg).bits() == 0
        },  Some(RETRIES_TIMEOUT))?;
        Ok(())
    }

    pub(crate) fn cmd_transfer<'a>(&self, cmd_data: &'a MCICmdData) -> MCIResult {
        let mut raw_cmd = MCICmd::empty();
        let reg = self.config.reg();

        if self.curr_timing.use_hold() {
            raw_cmd |= MCICmd::USE_HOLD_REG;
        }

        let flag = cmd_data.flag();
        if flag.contains(MCICmdFlag::ABORT) {
            raw_cmd |= MCICmd::STOP_ABORT;
        }
        /* 命令需要进行卡初始化，如CMD-0 */
        if flag.contains(MCICmdFlag::NEED_INIT) {
            raw_cmd |= MCICmd::INIT;
        }
        /* 命令涉及电压切换 */
        if flag.contains(MCICmdFlag::SWITCH_VOLTAGE) {
            raw_cmd |= MCICmd::VOLT_SWITCH;
        }
        /* 命令传输过程伴随数据传输 */
        if flag.contains(MCICmdFlag::EXP_DATA) {
            raw_cmd |= MCICmd::DAT_EXP;
            if flag.contains(MCICmdFlag::WRITE_DATA) {
                raw_cmd |= MCICmd::DAT_WRITE;
            }
        }
        /* 命令需要进行CRC校验 */
        if flag.contains(MCICmdFlag::NEED_RESP_CRC) {
            raw_cmd |= MCICmd::RESP_CRC;
        }
        /* 命令需要响应回复 */
        if flag.contains(MCICmdFlag::EXP_RESP) {
            raw_cmd |= MCICmd::RESP_EXP;
            if flag.contains(MCICmdFlag::EXP_LONG_RESP) {
                raw_cmd |= MCICmd::RESP_LONG;
            }
        }
        raw_cmd |= MCICmd::from_bits_truncate(set_reg32_bits!(cmd_data.cmdidx(), 5, 0));
        debug!("============[{}-{}]@0x{:x} begin ============",
        {
            if self.prev_cmd == Self::EXT_APP_CMD {
                "ACMD"
            } else {
                "CMD"
            }
        },
        cmd_data.cmdidx(),
        reg.addr.as_ptr() as usize );
        debug!("    cmd: 0x{:x}", raw_cmd.bits());
        debug!("    arg: 0x{:x}", cmd_data.cmdarg());
        /* enable related interrupt */
        self.interrupt_mask_set(MCIIntrType::GeneralIntr, MCIIntMask::INTS_CMD_MASK.bits(), true);
        info!("sending private cmd");
        self.private_cmd_send(raw_cmd, cmd_data.cmdarg())?;
        Ok(())
    }

    pub(crate) fn cmd_response_get(&mut self,cmd_data: &mut MCICmdData) -> MCIResult{
        let read = cmd_data.flag().contains(MCICmdFlag::READ_DATA);

        if !self.is_ready {
            error!("device is not yet initialized!!!");
            return Err(MCIError::NotInit);
        }

        if let Some(data) = cmd_data.get_mut_data(){
            if read {
                if MCITransMode::PIO == self.config.trans_mode() {
                    self.pio_read_data(data)?;
                }
            }
        }

        /* check response of cmd */
        let flag = cmd_data.flag().clone();
        let reg = self.config.reg();
        if flag.contains(MCICmdFlag::EXP_RESP) {
            let response = cmd_data.get_mut_response();
            if flag.contains(MCICmdFlag::EXP_LONG_RESP) {
                response[0] = reg.read_reg::<MCIResp0>().bits();
                response[1] = reg.read_reg::<MCIResp1>().bits();
                response[2] = reg.read_reg::<MCIResp2>().bits();
                response[3] = reg.read_reg::<MCIResp3>().bits();
                debug!("    resp: 0x{:x}-0x{:x}-0x{:x}-0x{:x}",
                response[0],response[1],response[2],response[3]);
            }else {
                response[0] = reg.read_reg::<MCIResp0>().bits();
                response[1] = 0;
                response[2] = 0;
                response[3] = 0;
                debug!("    resp: 0x{:x}",response[0]);
            }
        }

        cmd_data.success_set(true); /* cmd / data transfer finished successful */
        debug!("============[{}-{}]@0x{:x} end ============",
        {
            if self.prev_cmd == Self::EXT_APP_CMD {
                "ACMD"
            } else {
                "CMD"
            }
        },
        cmd_data.cmdidx(),
        reg.addr.as_ptr() as usize );

        /* disable related interrupt */
        self.interrupt_mask_set(MCIIntrType::GeneralIntr,(MCIIntMask::INTS_CMD_MASK | MCIIntMask::INTS_DATA_MASK).bits(),false);
        self.interrupt_mask_set(MCIIntrType::DmaIntr,MCIDMACIntEn::INTS_MASK.bits(),false);
        info!("cmd send done ...");

        self.prev_cmd = cmd_data.cmdidx();

        Ok(())
    }

}