use log::*;
use super::mci_data::MCIData;
use super::MCI;
use super::err::*;
use super::consts::*;
use super::regs::*;

impl MCI {
    pub(crate) fn pio_write_data(&self, data: &MCIData) -> MCIResult {
        let reg = self.config.reg();
        let wr_times: usize = (data.datalen() / 4) as usize; /* u8 --> u32 */
        let buf = if let Some(buf) = data.buf() {
            buf
        }else {
            return Err(MCIError::NotInit);
        };

        /* write fifo data */
        reg.write_reg(MCICmd::DAT_WRITE);
        for i in 0..wr_times {
            reg.write_reg(MCIDataReg::from_bits_truncate(buf[i]));
        }
        Ok(())
    }

    pub(crate) fn pio_read_data(&self, data: &mut MCIData) -> MCIResult {
        let reg = self.config.reg();
        let datalen = data.datalen();
        let rd_times = (datalen / 4) as usize; /* u8 --> u32 */
        let buf = if let Some(buf) = data.buf_mut(){
            buf
        }else {
            return Err(MCIError::NotInit);
        };

        buf.clear();

        if datalen > MCI_MAX_FIFO_CNT {
            error!("Fifo do not support writing more than 0x{:x}.",MCI_MAX_FIFO_CNT);
            return Err(MCIError::NotSupport);
        }
        for _i in 0..rd_times {
            buf.push(reg.read_reg::<MCIDataReg>().bits());
        }
        Ok(())
    }
    
}