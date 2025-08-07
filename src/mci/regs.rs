use core::ops;

use crate::mci::{consts::*, err::MCIError};
use bitflags::bitflags;

use super::{FlagReg, Reg};

pub type MCIReg = Reg<MCIError>;

impl Clone for MCIReg {
    fn clone(&self) -> Self {
        Self::new(self.addr)
    }
}

// FSDIF_CNTRL_OFFSET x0 Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCICtrl: u32 {
        const CONTROLLER_RESET = 1 << 0; // RW 复位控制器，除 DMA，FIFO
        const FIFO_RESET = 1 << 1; // RW 复位 FIFO, 1 有效
        const DMA_RESET = 1 << 2; // RW 复位内部 DMA, 1 有效
        const INT_ENABLE = 1 << 4; // RW 全局中断使能配置, 1 使能
        const DMA_ENABLE = 1 << 5; // RW 外部 DMA 模式使能
        const READ_WAIT = 1 << 6; // RW SDIF 读等待 1 有效
        const SEND_IRQ_RESPONSE = 1 << 7; // RW MMC 中断自动响应配置 1 有效
        const ABORT_READ_DATA = 1 << 8; // RW 读暂停异常清除
        const SEND_CCSD = 1 << 9; // RW 发送CCD (NOT USED)
        const SEND_AUTO_STOP_CCSD = 1 << 10; // RW 发送CCD，自动STOP (NOT USED)
        const ENDIAN = 1 << 11; // RW 0：小端，1：大端
        const CARD_VOLTAGE_A_MASK = 0xf << 16; // RW A电压选择
        const CARD_VOLTAGE_B_MASK = 0xf << 20; // RW B电压选择
        const ENABLE_OD_PULLUP = 1 << 24; // RW 外部开漏输出
        const USE_INTERNAL_DMAC = 1 << 25; // RW 使用内部DMA
    }
}

impl FlagReg for MCICtrl {
    const REG: u32 = FSDIF_CNTRL_OFFSET;
}

// FSDIF_PWREN_OFFSET 0x4 Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCIPwrEn: u32 {
        const ENABLE = 1 << 0; // RW 卡供电开关, 0：关；1：开
    }
}

impl FlagReg for MCIPwrEn {
    const REG: u32 = FSDIF_PWREN_OFFSET;
}

// FSDIF_CLKDIV_OFFSET 0x8 Register
bitflags! {
    pub struct MCIClkDiv: u32 {
        /* CLK_SAMPLE 和 CLK_SAMPLE 必须小于 CLK_DIVIDER */
        const CLK_DIVDER_BIT0 = 1 << 0; /* 时钟分频参数设置，分频参数=2*CLK_DIVIDER */
        const CLK_DIVDER_BIT1 = 1 << 1;
        const CLK_DIVDER_BIT2 = 1 << 2;
        const CLK_DIVDER_BIT3 = 1 << 3;
        const CLK_DIVDER_BIT4 = 1 << 4;
        const CLK_DIVDER_BIT5 = 1 << 5;
        const CLK_DIVDER_BIT6 = 1 << 6;
        const CLK_DIVDER_BIT7 = 1 << 7;
        const CLK_DRV_BIT0 = 1 << 8; /* 输出相位区间设置 */
        const CLK_DRV_BIT1 = 1 << 9;
        const CLK_DRV_BIT2 = 1 << 10;
        const CLK_DRV_BIT3 = 1 << 11;
        const CLK_DRV_BIT4 = 1 << 12;
        const CLK_DRV_BIT5 = 1 << 13;
        const CLK_DRV_BIT6 = 1 << 14;
        const CLK_DRV_BIT7 = 1 << 15;
        const CLK_SAMPLE_BIT0 = 1 << 16; /* 采样相位区间设置 */
        const CLK_SAMPLE_BIT1 = 1 << 17;
        const CLK_SAMPLE_BIT2 = 1 << 18;
        const CLK_SAMPLE_BIT3 = 1 << 19;
        const CLK_SAMPLE_BIT4 = 1 << 20;
        const CLK_SAMPLE_BIT5 = 1 << 21;
        const CLK_SAMPLE_BIT6 = 1 << 22;
        const CLK_SAMPLE_BIT7 = 1 << 23;
    }
}

impl FlagReg for MCIClkDiv {
    const REG: u32 = FSDIF_CLKDIV_OFFSET;
}

impl MCIClkDiv {
    pub fn clk_sample_set(x: u32) -> Self {
        Self::from_bits_truncate(set_reg32_bits!(x, 23, 16))
    }
    pub fn clk_drv_set(x: u32) -> Self {
        Self::from_bits_truncate(set_reg32_bits!(x, 15, 8))
    }
    pub fn clk_divider_set(x: u32) -> Self {
        Self::from_bits_truncate(set_reg32_bits!(x, 7, 0))
    }
    pub fn clk_div(samp: u32, drv: u32, div: u32) -> Self {
        Self::clk_sample_set(samp) | Self::clk_drv_set(drv) | Self::clk_divider_set(div)
    }
    pub fn clk_divider_get(div_reg: u32) -> Self {
        MCIClkDiv::from_bits_truncate(get_reg32_bits!(div_reg, 7, 0))
    }
}

// FSDIF_CLKENA_OFFSET Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCIClkEn: u32 {
        const CCLK_ENABLE = 1 << 0; /* RW 0：Clock disabled；1：Clock enabled */
        const CLKENA_CCLK_LOW_POWER = 1<<16; /* RW 0x0：非低功耗；0x1：低功耗 */
    }
}

impl FlagReg for MCIClkEn {
    const REG: u32 = FSDIF_CLKENA_OFFSET;
}

// FSDIF_TMOUT_OFFSET Register
bitflags! {
    pub struct MCITimeout: u32 {
        const MAX_DATA_TIMEOUT = 0xffffff; /* RW 读卡超时（以卡时钟为单位） */
        const MAX_RESP_TIMEOUT = 0xff; /* RW 响应超时（以卡时钟为单位） */
        const RESP_TIMEOUT_BIT0 = 1 << 0; /* RW 响应超时的第0位 */
        const RESP_TIMEOUT_BIT1 = 1 << 1; /* RW 响应超时的第1位 */
        const RESP_TIMEOUT_BIT2 = 1 << 2; /* RW 响应超时的第2位 */
        const RESP_TIMEOUT_BIT3 = 1 << 3; /* RW 响应超时的第3位 */
        const RESP_TIMEOUT_BIT4 = 1 << 4; /* RW 响应超时的第4位 */
        const RESP_TIMEOUT_BIT5 = 1 << 5; /* RW 响应超时的第5位 */
        const RESP_TIMEOUT_BIT6 = 1 << 6; /* RW 响应超时的第6位 */
        const RESP_TIMEOUT_BIT7 = 1 << 7; /* RW 响应超时的第7位 */
        const DATA_TIMEOUT_BIT0 = 1 << 8; /* RW 读卡超时的第0位 */
        const DATA_TIMEOUT_BIT1 = 1 << 9; /* RW 读卡超时的第1位 */
        const DATA_TIMEOUT_BIT2 = 1 << 10; /* RW 读卡超时的第2位 */
        const DATA_TIMEOUT_BIT3 = 1 << 11; /* RW 读卡超时的第3位 */
        const DATA_TIMEOUT_BIT4 = 1 << 12; /* RW 读卡超时的第4位 */
        const DATA_TIMEOUT_BIT5 = 1 << 13; /* RW 读卡超时的第5位 */
        const DATA_TIMEOUT_BIT6 = 1 << 14; /* RW 读卡超时的第6位 */
        const DATA_TIMEOUT_BIT7 = 1 << 15; /* RW 读卡超时的第7位 */
        const DATA_TIMEOUT_BIT8 = 1 << 16; /* RW 读卡超时的第8位 */
        const DATA_TIMEOUT_BIT9 = 1 << 17; /* RW 读卡超时的第9位 */
        const DATA_TIMEOUT_BIT10 = 1 << 18; /* RW 读卡超时的第10位 */
        const DATA_TIMEOUT_BIT11 = 1 << 19; /* RW 读卡超时的第11位 */
        const DATA_TIMEOUT_BIT12 = 1 << 20; /* RW 读卡超时的第12位 */
        const DATA_TIMEOUT_BIT13 = 1 << 21; /* RW 读卡超时的第13位 */
        const DATA_TIMEOUT_BIT14 = 1 << 22; /* RW 读卡超时的第14位 */
        const DATA_TIMEOUT_BIT15 = 1 << 23; /* RW 读卡超时的第15位 */
        const DATA_TIMEOUT_BIT16 = 1 << 24; /* RW 读卡超时的第16位 */
        const DATA_TIMEOUT_BIT17 = 1 << 25; /* RW 读卡超时的第17位 */
        const DATA_TIMEOUT_BIT18 = 1 << 26; /* RW 读卡超时的第18位 */
        const DATA_TIMEOUT_BIT19 = 1 << 27; /* RW 读卡超时的第19位 */
        const DATA_TIMEOUT_BIT20 = 1 << 28; /* RW 读卡超时的第20位 */
        const DATA_TIMEOUT_BIT21 = 1 << 29; /* RW 读卡超时的第21位 */
        const DATA_TIMEOUT_BIT22 = 1 << 30; /* RW 读卡超时的第22位 */
        const DATA_TIMEOUT_BIT23 = 1 << 31; /* RW 读卡超时的第23位 */
    }
}

impl FlagReg for MCITimeout {
    const REG: u32 = FSDIF_TMOUT_OFFSET;
}

impl MCITimeout {
    pub fn timeout_data(data_timeout: MCITimeout, resp_timeout: MCITimeout) -> MCITimeout {
        MCITimeout::from_bits_truncate(
            (genmask!(31, 8) & (data_timeout.bits() << 8)) | (genmask!(7, 0) & resp_timeout.bits()),
        )
    }
}

// FSDIF_CTYPE_OFFSET Register
bitflags! {
    pub struct MCICType: u32 {
        const CARD0_WIDTH1_8BIT = 1 << 16; /* 1: 8-bit mode */
        const CARD0_WIDTH2_4BIT = 1 << 0; /* 1: 4-bit mode */
        const CARD0_WIDTH2_1BIT = 0; /* 0: 1-bit mode */
    }
}

impl From<u32> for MCICType {
    fn from(val: u32) -> Self {
        match val {
            1 => Self::CARD0_WIDTH2_1BIT,
            4 => Self::CARD0_WIDTH2_4BIT,
            8 => Self::CARD0_WIDTH1_8BIT,
            _ => panic!("Invalid card width: {}", val),
        }
    }
}

impl FlagReg for MCICType {
    const REG: u32 = FSDIF_CTYPE_OFFSET;
}

// FSDIF_INT_MASK_OFFSET Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCIIntMask: u32 {
        const CD_BIT = 1 << 0;       /* RW Card detect (CD) */
        const RE_BIT = 1 << 1;       /* RW Response error (RE) */
        const CMD_BIT = 1 << 2;      /* RW Command done (CD) */
        const DTO_BIT = 1 << 3;      /* RW Data transfer over (DTO) */
        const TXDR_BIT = 1 << 4;     /* RW Transmit FIFO data request (TXDR) */
        const RXDR_BIT = 1 << 5;     /* RW Receive FIFO data request (RXDR) */
        const RCRC_BIT = 1 << 6;     /* RW Response CRC error (RCRC) */
        const DCRC_BIT = 1 << 7;     /* RW Data CRC error (DCRC) */
        const RTO_BIT = 1 << 8;      /* RW Response timeout (RTO) */
        const DRTO_BIT = 1 << 9;     /* RW Data read timeout (DRTO) */
        const HTO_BIT = 1 << 10;     /* RW Data starvation-by-host timeout (HTO) */
        const FRUN_BIT = 1 << 11;    /* RW FIFO underrun/overrun error (FRUN) */
        const HLE_BIT = 1 << 12;     /* RW Hardware locked write error (HLE) */
        const SBE_BCI_BIT = 1 << 13; /* RW Start-bit error (SBE) */
        const ACD_BIT = 1 << 14;     /* RW Auto command done (ACD) */
        const EBE_BIT = 1 << 15;     /* RW End-bit error (read)/Write no CRC (EBE) */
        const SDIO_BIT = 1 << 16;    /* RW SDIO interrupt for card */
        const ALL_BITS = 0x1FFFF;    /* RW All bits */
        // const INTS_DATA_MASK = DTO_BIT | DCRC_BIT | DRTO_BIT | SBE_BCI_BIT;
        const INTS_DATA_MASK = 0x2288;
        // const INTS_CMD_MASK = RE_BIT | CMD_BIT | RCRC_BIT | RTO_BIT | HTO_BIT | HLE_BIT;
        const INTS_CMD_MASK = 0x1546;

    }
}

impl FlagReg for MCIIntMask {
    const REG: u32 = FSDIF_INT_MASK_OFFSET; // 假设FSDIF_INT_OFFSET是对应的寄存器偏移量
}

// FSDIF_MASKED_INTS_OFFSET Register
bitflags! {
    pub struct MCIMaskedInts: u32 {
        const CD_BIT = 1 << 0;       /* RW Card detect (CD) */
        const RE_BIT = 1 << 1;       /* RW Response error (RE) */
        const CMD_BIT = 1 << 2;      /* RW Command done (CD) */
        const DTO_BIT = 1 << 3;      /* RW Data transfer over (DTO) */
        const TXDR_BIT = 1 << 4;     /* RW Transmit FIFO data request (TXDR) */
        const RXDR_BIT = 1 << 5;     /* RW Receive FIFO data request (RXDR) */
        const RCRC_BIT = 1 << 6;     /* RW Response CRC error (RCRC) */
        const DCRC_BIT = 1 << 7;     /* RW Data CRC error (DCRC) */
        const RTO_BIT = 1 << 8;      /* RW Response timeout (RTO) */
        const DRTO_BIT = 1 << 9;     /* RW Data read timeout (DRTO) */
        const HTO_BIT = 1 << 10;     /* RW Data starvation-by-host timeout (HTO) */
        const FRUN_BIT = 1 << 11;    /* RW FIFO underrun/overrun error (FRUN) */
        const HLE_BIT = 1 << 12;     /* RW Hardware locked write error (HLE) */
        const SBE_BCI_BIT = 1 << 13; /* RW Start-bit error (SBE) */
        const ACD_BIT = 1 << 14;     /* RW Auto command done (ACD) */
        const EBE_BIT = 1 << 15;     /* RW End-bit error (read)/Write no CRC (EBE) */
        const SDIO_BIT = 1 << 16;    /* RW SDIO interrupt for card */
        const ALL_BITS = 0x1FFFF;    /* RW All bits */
        // const INTS_DATA_MASK = DTO_BIT | DCRC_BIT | DRTO_BIT | SBE_BCI_BIT;
        const INTS_DATA_MASK = 0x2288;
        // const INTS_CMD_MASK = RE_BIT | CMD_BIT | RCRC_BIT | RTO_BIT | HTO_BIT | HLE_BIT;
        const INTS_CMD_MASK = 0x5446;
    }
}

impl FlagReg for MCIMaskedInts {
    const REG: u32 = FSDIF_MASKED_INTS_OFFSET;
}

// FSDIF_RAW_INTS_OFFSET Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCIRawInts: u32 {
        const CD_BIT = 1 << 0;       /* RW Card detect (CD) */
        const RE_BIT = 1 << 1;       /* RW Response error (RE) */
        const CMD_BIT = 1 << 2;      /* RW Command done (CD) */
        const DTO_BIT = 1 << 3;      /* RW Data transfer over (DTO) */
        const TXDR_BIT = 1 << 4;     /* RW Transmit FIFO data request (TXDR) */
        const RXDR_BIT = 1 << 5;     /* RW Receive FIFO data request (RXDR) */
        const RCRC_BIT = 1 << 6;     /* RW Response CRC error (RCRC) */
        const DCRC_BIT = 1 << 7;     /* RW Data CRC error (DCRC) */
        const RTO_BIT = 1 << 8;      /* RW Response timeout (RTO) */
        const DRTO_BIT = 1 << 9;     /* RW Data read timeout (DRTO) */
        const HTO_BIT = 1 << 10;     /* RW Data starvation-by-host timeout (HTO) */
        const FRUN_BIT = 1 << 11;    /* RW FIFO underrun/overrun error (FRUN) */
        const HLE_BIT = 1 << 12;     /* RW Hardware locked write error (HLE) */
        const SBE_BCI_BIT = 1 << 13; /* RW Start-bit error (SBE) */
        const ACD_BIT = 1 << 14;     /* RW Auto command done (ACD) */
        const EBE_BIT = 1 << 15;     /* RW End-bit error (read)/Write no CRC (EBE) */
        const SDIO_BIT = 1 << 16;    /* RW SDIO interrupt for card */
        const ALL_BITS = 0x1FFFF;    /* RW All bits */
        const INTS_CMD_MASK = 0x1546;
        const INTS_DATA_MASK = 0x2288;
        // const CMD_ERR_INTS_MASK = RTO_BIT | RCRC_BIT | RE_BIT | DCRC_BIT | DRTO_BIT | SBE_BCI_BIT;
        const CMD_ERR_INTS_MASK = 0x23C2;
    }
}

impl FlagReg for MCIRawInts {
    const REG: u32 = FSDIF_RAW_INTS_OFFSET;
}

// FSDIF_CMD_OFFSET Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCICmd: u32 {
        const START = 1 << 31;                /* 启动命令 */
        const USE_HOLD_REG = 1 << 29;         /* 0: 旁路HOLD寄存器，1: 使能HOLD寄存器 */
        const VOLT_SWITCH = 1 << 28;          /* 0: 无电压转换，1: 有电压转换 */
        const BOOT_MODE = 1 << 27;            /* 0: Mandatory boot, 1: Alternate boot */
        const DISABLE_BOOT = 1 << 26;         /* 中止boot进程 */
        const EXPECT_BOOT_ACK = 1 << 25;      /* 1: Expect boot ack */
        const ENABLE_BOOT = 1 << 24;          /* 1: 使能 boot for mandatory */
        const UPD_CLK = 1 << 21;              /* 1：不发送指令，仅更新时钟寄存器的值到卡时钟域内 */
        const INIT = 1 << 15;                  /* 0：在发送指令前不发送初始化序列（80 个周期） 1: 发送 */
        const STOP_ABORT = 1 << 14;           /* 1：停止或中止命令，用于停止当前的数据传输 */
        const WAIT_PRVDATA_COMPLETE = 1 << 13; /* 1：等待前面的数据传输完成后再发送指令 0: 立即发送命令 */
        const SEND_AUTO_STOP = 1 << 12;       /* 1：在数据传送结束时发送停止命令 */
        const DAT_WRITE = 1 << 10;            /* 0：读卡 1：写卡 */
        const DAT_EXP = 1 << 9;                /* 0：不等待数据传输, 1：等待数据传输 */
        const RESP_CRC = 1 << 8;               /* 1：检查响应 CRC */
        const RESP_LONG = 1 << 7;              /* 0：等待卡的短响应 1：等待卡的长响应 */
        const RESP_EXP = 1 << 6;               /* 1：等待卡的响应，0：命令不需要卡响应 */
        const CMD_INDEX_BIT5 = 1 << 5;         /* 命令索引号的第5位 */
        const CMD_INDEX_BIT4 = 1 << 4;         /* 命令索引号的第4位 */
        const CMD_INDEX_BIT3 = 1 << 3;         /* 命令索引号的第3位 */
        const CMD_INDEX_BIT2 = 1 << 2;         /* 命令索引号的第2位 */
        const CMD_INDEX_BIT1 = 1 << 1;         /* 命令索引号的第1位 */
        const CMD_INDEX_BIT0 = 1 << 0;         /* 命令索引号的第0位 */
    }
}

impl FlagReg for MCICmd {
    const REG: u32 = FSDIF_CMD_OFFSET; // 假设FSDIF_CMD_OFFSET是对应的寄存器偏移量
}

impl MCICmd {
    pub fn index_set(x: u32) -> Self {
        Self::from_bits_truncate(set_reg32_bits!(x, 5, 0))
    }

    pub fn index_get(&self) -> u32 {
        self.bits() & genmask!(5, 0)
    }
}

// FSDIF_STATUS_OFFSET Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCIStatus: u32 {
        const FIFO_RX = 1 << 0;     /* RO, 达到 FIFO_RX 标记 */
        const FIFO_TX = 1 << 1;     /* RO, 达到 FIFO_TX 标记 */
        const FIFO_EMPTY = 1 << 2;  /* RO, FIFO empty */
        const FIFO_FULL = 1 << 3;   /* RO, FIFO full */
        const CMD_FSM_BIT0 = 1 << 4; /* RO CMD FSM 状态 */
        const CMD_FSM_BIT1 = 1 << 5; /* RO CMD FSM 状态 */
        const CMD_FSM_BIT2 = 1 << 6; /* RO CMD FSM 状态 */
        const CMD_FSM_BIT3 = 1 << 7; /* RO CMD FSM 状态 */
        const DATA3_STATUS = 1 << 8; /* RO DATA[3] 卡在位检测，1：在位 */
        const DATA_BUSY = 1 << 9;   /* RO 1: 卡 busy */
        const DATA_STATE_MC_BUSY = 1 << 10;  /* RO DATA TX|RX FSM busy  */
        const RESP_INDEX_BIT0 = 1 << 11; /* RO 响应索引号的第0位 */
        const RESP_INDEX_BIT1 = 1 << 12; /* RO 响应索引号的第1位 */
        const RESP_INDEX_BIT2 = 1 << 13; /* RO 响应索引号的第2位 */
        const RESP_INDEX_BIT3 = 1 << 14; /* RO 响应索引号的第3位 */
        const RESP_INDEX_BIT4 = 1 << 15; /* RO 响应索引号的第4位 */
        const RESP_INDEX_BIT5 = 1 << 16; /* RO 响应索引号的第5位 */
        const FIFO_CNT_BIT0 = 1 << 17;   /* RO FIFO 中的数据计数的第0位 */
        const FIFO_CNT_BIT1 = 1 << 18;   /* RO FIFO 中的数据计数的第1位 */
        const FIFO_CNT_BIT2 = 1 << 19;   /* RO FIFO 中的数据计数的第2位 */
        const FIFO_CNT_BIT3 = 1 << 20;   /* RO FIFO 中的数据计数的第3位 */
        const FIFO_CNT_BIT4 = 1 << 21;   /* RO FIFO 中的数据计数的第4位 */
        const FIFO_CNT_BIT5 = 1 << 22;   /* RO FIFO 中的数据计数的第5位 */
        const FIFO_CNT_BIT6 = 1 << 23;   /* RO FIFO 中的数据计数的第6位 */
        const FIFO_CNT_BIT7 = 1 << 24;   /* RO FIFO 中的数据计数的第7位 */
        const FIFO_CNT_BIT8 = 1 << 25;   /* RO FIFO 中的数据计数的第8位 */
        const FIFO_CNT_BIT9 = 1 << 26;   /* RO FIFO 中的数据计数的第9位 */
        const FIFO_CNT_BIT10 = 1 << 27;  /* RO FIFO 中的数据计数的第10位 */
        const FIFO_CNT_BIT11 = 1 << 28;  /* RO FIFO 中的数据计数的第11位 */
        const FIFO_CNT_BIT12 = 1 << 29;  /* RO FIFO 中的数据计数的第12位 */
        const DMA_ACK = 1 << 30;    /* RO DMA 确认 */
        const DMA_REQ = 1 << 31;    /* RO DMA 请求 */
    }
}

impl FlagReg for MCIStatus {
    const REG: u32 = FSDIF_STATUS_OFFSET;
}

// FSDIF_FIFOTH_OFFSET Register
bitflags! {
    pub struct MCIFifoTh: u32 {
        const DMA_TRANS_MASK = genmask!(30, 28); /* 多次传输的突发大小 */
        const RX_WMARK_MASK = genmask!(27, 16);  /* 当接收数据给卡时FIFO的阈值 */
        const TX_WMARK_MASK = genmask!(11, 0);   /* 当发送数据给卡时FIFO的阈值 */
        const TX_WMARK_BIT0 = 1 << 0;            /* TX_WMARK 的第0位 */
        const TX_WMARK_BIT1 = 1 << 1;            /* TX_WMARK 的第1位 */
        const TX_WMARK_BIT2 = 1 << 2;            /* TX_WMARK 的第2位 */
        const TX_WMARK_BIT3 = 1 << 3;            /* TX_WMARK 的第3位 */
        const TX_WMARK_BIT4 = 1 << 4;            /* TX_WMARK 的第4位 */
        const TX_WMARK_BIT5 = 1 << 5;            /* TX_WMARK 的第5位 */
        const TX_WMARK_BIT6 = 1 << 6;            /* TX_WMARK 的第6位 */
        const TX_WMARK_BIT7 = 1 << 7;            /* TX_WMARK 的第7位 */
        const TX_WMARK_BIT8 = 1 << 8;            /* TX_WMARK 的第8位 */
        const TX_WMARK_BIT9 = 1 << 9;            /* TX_WMARK 的第9位 */
        const TX_WMARK_BIT10 = 1 << 10;          /* TX_WMARK 的第10位 */
        const TX_WMARK_BIT11 = 1 << 11;          /* TX_WMARK 的第11位 */
        const RX_WMARK_BIT0 = 1 << 16;           /* RX_WMARK 的第0位 */
        const RX_WMARK_BIT1 = 1 << 17;           /* RX_WMARK 的第1位 */
        const RX_WMARK_BIT2 = 1 << 18;           /* RX_WMARK 的第2位 */
        const RX_WMARK_BIT3 = 1 << 19;           /* RX_WMARK 的第3位 */
        const RX_WMARK_BIT4 = 1 << 20;           /* RX_WMARK 的第4位 */
        const RX_WMARK_BIT5 = 1 << 21;           /* RX_WMARK 的第5位 */
        const RX_WMARK_BIT6 = 1 << 22;           /* RX_WMARK 的第6位 */
        const RX_WMARK_BIT7 = 1 << 23;           /* RX_WMARK 的第7位 */
        const RX_WMARK_BIT8 = 1 << 24;           /* RX_WMARK 的第8位 */
        const RX_WMARK_BIT9 = 1 << 25;           /* RX_WMARK 的第9位 */
        const RX_WMARK_BIT10 = 1 << 26;          /* RX_WMARK 的第10位 */
        const RX_WMARK_BIT11 = 1 << 27;          /* RX_WMARK 的第11位 */
        const DMA_TRANS_BIT0 = 1 << 28;          /* DMA */
        const DMA_TRANS_BIT1 = 1 << 29;          /* DMA */
        const DMA_TRANS_BIT2 = 1 << 30;          /* DMA */
    }
}

impl MCIFifoTh {
    pub const RX_WMARK: u32 = 0x7;
    pub const TX_WMARK: u32 = 0x100;
    /*
    trans_size: Burst size of multiple transaction;
    rx_wmark: FIFO threshold watermark level when receiving data to card.
    tx_wmark: FIFO threshold watermark level when transmitting data to card
    */
    pub fn fifoth(trans_size: MCIFifoThDMATransSize, rx_wmark: u32, tx_wmark: u32) -> Self {
        let trans_size: u32 = trans_size.into();
        (MCIFifoTh::DMA_TRANS_MASK & (trans_size << 28))
            | (MCIFifoTh::RX_WMARK_MASK & (rx_wmark << 16))
            | (MCIFifoTh::TX_WMARK_MASK & tx_wmark)
    }
}

BitsOpsForU32!(MCIFifoTh);

impl FlagReg for MCIFifoTh {
    const REG: u32 = FSDIF_FIFOTH_OFFSET;
}

pub enum MCIFifoThDMATransSize {
    DMATrans1 = 0b000,
    DMATrans4 = 0b001,
    DMATrans8 = 0b010,
    DMATrans16 = 0b011,
    DMATrans32 = 0b100,
    DMATrans64 = 0b101,
    DMATrans128 = 0b110,
    DMATrans256 = 0b111,
}

impl From<MCIFifoThDMATransSize> for u32 {
    fn from(val: MCIFifoThDMATransSize) -> Self {
        val as u32
    }
}

// FSDIF_CARD_DETECT_OFFSET Register
bitflags! {
    pub struct MCICardDetect: u32 {
        const DETECTED = 1 << 0; /* 1：卡不在位；0：卡在位 */
    }
}

impl FlagReg for MCICardDetect {
    const REG: u32 = FSDIF_CARD_DETECT_OFFSET; // 假设 FSDIF_CARD_DETECT_OFFSET 是对应的寄存器偏移量
}

// FSDIF_CARD_WRTPRT_OFFSET Register
bitflags! {
    pub struct MCICardWrtp: u32 {
        const WRITE_PROTECTED = 1 << 0; /* 1：写保护；0：无写保护 */
    }
}

impl FlagReg for MCICardWrtp {
    const REG: u32 = FSDIF_CARD_WRTPRT_OFFSET; // 假设 FSDIF_CARD_WRTPRT_OFFSET 是对应的寄存器偏移量
}

// FSDIF_CKSTS_OFFSET Register
bitflags! {
    pub struct MCIClkSts: u32 {
        const READY = 1 << 0; /* CIU 时钟 ready */
    }
}

impl FlagReg for MCIClkSts {
    const REG: u32 = FSDIF_CKSTS_OFFSET; // 假设 FSDIF_CKSTS_OFFSET 是对应的寄存器偏移量
}

// FSDIF_UHS_REG_OFFSET Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCIUhsReg: u32 {
        const VOLT_180 = 1 << 0; /* RW 外部调压器接口电压 0: 3.3v, 1: 1.8v */
        const DDR = 1 << 16;     /* RW DDR 模式 */
    }
}

impl FlagReg for MCIUhsReg {
    const REG: u32 = FSDIF_UHS_REG_OFFSET; // 假设 FSDIF_UHS_REG_OFFSET 是对应的寄存器偏移量
}

// FSDIF_CARD_RESET_OFFSET Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCICardReset: u32 {
        const ENABLE = 1 << 0; /* RW 1：运行；0：复位 */
    }
}

impl FlagReg for MCICardReset {
    const REG: u32 = FSDIF_CARD_RESET_OFFSET; // 假设 FSDIF_CARD_RESET_OFFSET 是对应的寄存器偏移量
}

// FSDIF_BUS_MODE_OFFSET Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCIBusMode: u32 {
        const SWR = 1 << 0; /* RW 软复位，复位idma内部寄存器 */
        const FB = 1 << 1;  /* RW 固定burst */
        const DE = 1 << 7;  /* RW idma使能 */
        const PBL_BIT0 = 1 << 8; /* R0 传输突发长度 */
        const PBL_BIT1 = 1 << 9; /* R0 传输突发长度 */
        const PBL_BIT2 = 1 << 10; /* R0 传输突发长度 */
    }
}

impl FlagReg for MCIBusMode {
    const REG: u32 = FSDIF_BUS_MODE_OFFSET; // 假设 FSDIF_BUS_MODE_OFFSET 是对应的寄存器偏移量
}

// FSDIF_DMAC_STATUS_OFFSET Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCIDMACStatus: u32 {
        const TI = 1 << 0;  /* RW 发送中断。表示链表的数据发送完成 */
        const RI = 1 << 1;  /* RW 接收中断。表示链表的数据接收完成 */
        const FBE = 1 << 2; /* RW 致命总线错误中断 */
        const DU_BIT0 = 1 << 3;  /* RW 链表不可用中断 */
        const DU_BIT1 = 1 << 4;  /* RW 链表不可用中断 */
        const CES = 1 << 5; /* RW 卡错误汇总 */
        const NIS = 1 << 8; /* RW 正常中断汇总 */
        const AIS = 1 << 9; /* RW 异常中断汇总 */
        const EB_BIT0 = 1 << 10;
        const EB_BIT1 = 1 << 11;
        const EB_BIT2 = 1 << 12;
        const FSM_BIT0 = 1 << 13;
        const FSM_BIT1 = 1 << 14;
        const FSM_BIT2 = 1 << 15;
        const FSM_BIT3 = 1 << 16;
        const FSM_BIT4 = 1 << 17;
        const FSM_BIT5 = 1 << 18;
        const FSM_BIT6 = 1 << 19;
        const FSM_BIT7 = 1 << 20;
        const FSM_BIT8 = 1 << 21;
        const FSM_BIT9 = 1 << 22;
        const FSM_BIT10 = 1 << 23;
        const FSM_BIT11 = 1 << 24;
        const FSM_BIT12 = 1 << 25;
        const FSM_BIT13 = 1 << 26;
        const FSM_BIT14 = 1 << 27;
        const FSM_BIT15 = 1 << 28;
        const FSM_BIT16 = 1 << 29;
        const FSM_BIT17 = 1 << 30;
        const FSM_BIT18 = 1 << 31;
        const ALL_BITS = 0x3ff;
        const STATUS_EB_TX = 0b001;
        const STATUS_EB_RX = 0b010;
        // const DMAC_ERR_INTS_MASK = FBE | DU_BIT1 | NIS | AIS
        const DMAC_ERR_INTS_MASK = 0x314;
    }
}

impl FlagReg for MCIDMACStatus {
    const REG: u32 = FSDIF_DMAC_STATUS_OFFSET; // 假设 FSDIF_DMAC_STATUS_OFFSET 是对应的寄存器偏移量
}

// FSDIF_DMAC_INT_EN_OFFSET Register
bitflags! {
    pub struct MCIDMACIntEn: u32 {
        const TI = 1 << 0;  /* RW 发送完成中断使能 */
        const RI = 1 << 1;  /* RW 接收完成中断使能 */
        const FBE = 1 << 2; /* RW 总线错误中断使能 */
        const DU = 1 << 4;  /* RW 描述符不可读中断使能 */
        const CES = 1 << 5; /* RW 卡错误中断使能 */
        const NIS = 1 << 8; /* RW 正常中断汇总使能 */
        const AIS = 1 << 9; /* RW 异常中断汇总使能 */
        const ALL_BITS = 0x3ff;
        // const INTS_MASK = FBE | DU | NIS | AIS;
        const INTS_MASK = 0x314;
    }
}

impl FlagReg for MCIDMACIntEn {
    const REG: u32 = FSDIF_DMAC_INT_EN_OFFSET; // 假设 FSDIF_DMAC_INT_EN_OFFSET 是对应的寄存器偏移量
}

// FSDIF_CARD_THRCTL_OFFSET Register
bitflags! {
    pub struct MCICardThrctl: u32 {
        const CARDRD = 1 << 0;   /* RW 读卡threshold使能 */
        const BUSY_CLR = 1 << 1; /* RW busy清中断 */
        const CARDWR = 1 << 2;   /* RO 写卡threshold使能 */
        const FIFO_DEPTH_BIT0 = 1 << 16; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT1 = 1 << 17; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT2 = 1 << 18; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT3 = 1 << 19; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT4 = 1 << 20; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT5 = 1 << 21; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT6 = 1 << 22; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT7 = 1 << 23; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT8 = 1 << 24; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT9 = 1 << 25; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT10 = 1 << 26; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT11 = 1 << 27; /* RW FIFO深度 */
        const FIFO_DEPTH_BIT12 = 1 << 28; /* RW FIFO深度 */
    }
}

impl FlagReg for MCICardThrctl {
    const REG: u32 = FSDIF_CARD_THRCTL_OFFSET; // 假设 FSDIF_CARD_THRCTL_OFFSET 是对应的寄存器偏移量
}

impl From<MCIFifoDepth> for MCICardThrctl {
    fn from(value: MCIFifoDepth) -> Self {
        let value: u32 = value as u32;
        let value: u32 = 1 << value;
        MCICardThrctl::from_bits_truncate(value)
    }
}

// FSDIF_CLK_SRC_OFFSET Register
bitflags! {
    #[derive(Clone, Copy)]
    pub struct MCIClkSrc: u32 {
        const UHS_EXT_MMC_VOLT = 1 << 0;         /* RW 1.2V供电选择 */
        const UHS_EXT_CLK_ENA = 1 << 1;          /* RW 外部时钟，CIU时钟使能 */
        const UHS_EXT_CLK_MUX = 1 << 31;         /* RW 外部时钟选择 */
        const UHS_CLK_DIV_MASK = genmask!(14, 8); /* RW 分频系数，ciu_f = clk_div_ctrl + 1, min=1*/
        const UHS_CLK_DIV_BIT0 = 1 << 8;         /* RW 分频系数，ciu_f = clk_div_ctrl + 1, min=1*/
        const UHS_CLK_DIV_BIT1 = 1 << 9;         /* RW 分频系数，ciu_f = clk_div_ctrl + 1, min=1*/
        const UHS_CLK_DIV_BIT2 = 1 << 10;        /* RW 分频系数，ciu_f = clk_div_ctrl + 1, min=1*/
        const UHS_CLK_DIV_BIT3 = 1 << 11;        /* RW 分频系数，ciu_f = clk_div_ctrl + 1, min=1*/
        const UHS_CLK_DIV_BIT4 = 1 << 12;        /* RW 分频系数，ciu_f = clk_div_ctrl + 1, min=1*/
        const UHS_CLK_DIV_BIT5 = 1 << 13;        /* RW 分频系数，ciu_f = clk_div_ctrl + 1, min=1*/
        const UHS_CLK_DIV_BIT6 = 1 << 14;        /* RW 分频系数，ciu_f = clk_div_ctrl + 1, min=1*/
        const UHS_CLK_SAMP_MASK = genmask!(22, 16); /* RW 采样相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_SAMP_BIT0 = 1 << 16;         /* RW 采样相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_SAMP_BIT1 = 1 << 17;         /* RW 采样相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_SAMP_BIT2 = 1 << 18;         /* RW 采样相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_SAMP_BIT3 = 1 << 19;         /* RW 采样相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_SAMP_BIT4 = 1 << 20;         /* RW 采样相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_SAMP_BIT5 = 1 << 21;         /* RW 采样相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_SAMP_BIT6 = 1 << 22;         /* RW 采样相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_DRV_MASK = genmask!(30, 24); /* RW 输出相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_DRV_BIT0 = 1 << 24;         /* RW 输出相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_DRV_BIT1 = 1 << 25;         /* RW 输出相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_DRV_BIT2 = 1 << 26;         /* RW 输出相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_DRV_BIT3 = 1 << 27;         /* RW 输出相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_DRV_BIT4 = 1 << 28;         /* RW 输出相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_DRV_BIT5 = 1 << 29;         /* RW 输出相位参数，相对于ciu时钟相位点 */
        const UHS_CLK_DRV_BIT6 = 1 << 30;         /* RW 输出相位参数，相对于ciu时钟相位点 */
    }
}

impl FlagReg for MCIClkSrc {
    const REG: u32 = FSDIF_CLK_SRC_OFFSET; // 假设 FSDIF_CLK_SRC_OFFSET 是对应的寄存器偏移量
}

impl MCIClkSrc {
    pub fn uhs_clk_div(x: u32) -> Self {
        Self::UHS_CLK_DIV_MASK & Self::from_bits_truncate(x << 8)
    }

    pub fn uhs_clk_samp(x: u32) -> Self {
        Self::UHS_CLK_SAMP_MASK & Self::from_bits_truncate(x << 16)
    }

    pub fn uhs_clk_drv(x: u32) -> Self {
        Self::UHS_CLK_DRV_MASK & Self::from_bits_truncate(x << 24)
    }

    pub fn uhs_reg(drv_phase: u32, samp_phase: u32, clk_div: u32) -> Self {
        Self::uhs_clk_div(clk_div) | Self::uhs_clk_samp(samp_phase) | Self::uhs_clk_drv(drv_phase)
    }
}

// FSDIF_EMMC_DDR_REG_OFFSET Register
bitflags! {
    pub struct MCIEmmcDdrReg: u32 {
        const CYCLE = 1 << 0; /* RW 1: start bit小于一个周期，0：start bit 为一个周期 */
    }
}

impl FlagReg for MCIEmmcDdrReg {
    const REG: u32 = FSDIF_EMMC_DDR_REG_OFFSET; // 假设 FSDIF_EMMC_DDR_REG_OFFSET 是对应的寄存器偏移量
}

// FSDIF_DESC_LIST_ADDRH_OFFSET Register
bitflags! {
    pub struct MCIDescListAddrH: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}

impl FlagReg for MCIDescListAddrH {
    const REG: u32 = FSDIF_DESC_LIST_ADDRH_OFFSET; // 假设 FSDIF_DESC_LIST_ADDRH_OFFSET 是对应的寄存器偏移量
}

// FSDIF_DESC_LIST_ADDRL_OFFSET Register
bitflags! {
    pub struct MCIDescListAddrL: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}

impl FlagReg for MCIDescListAddrL {
    const REG: u32 = FSDIF_DESC_LIST_ADDRL_OFFSET; // 假设 FSDIF_DESC_LIST_ADDRL_OFFSET 是对应的寄存器偏移量
}

// FSDIF_DATA_OFFSET Register
bitflags! {
    pub struct MCIDataReg: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIDataReg {
    const REG: u32 = FSDIF_DATA_OFFSET; // 假设 FSDIF_DATA_OFFSET 是对应的寄存器偏移量
}

// FSDIF_BYT_CNT_OFFSET Register
bitflags! {
    pub struct MCIBytCnt: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIBytCnt {
    const REG: u32 = FSDIF_BYT_CNT_OFFSET; // 假设 FSDIF_BYT_CNT_OFFSET 是对应的寄存器偏移量
}

// FSDIF_BLK_SIZ_OFFSET Register
bitflags! {
    pub struct MCIBlkSiz: u32 {
        const BIT0 = 1 << 0; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT1 = 1 << 1; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT2 = 1 << 2; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT3 = 1 << 3; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT4 = 1 << 4; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT5 = 1 << 5; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT6 = 1 << 6; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT7 = 1 << 7; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT8 = 1 << 8; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT9 = 1 << 9; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT10 = 1 << 10; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT11 = 1 << 11; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT12 = 1 << 12; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT13 = 1 << 13; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT14 = 1 << 14; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT15 = 1 << 15; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT16 = 1 << 16; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT17 = 1 << 17; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT18 = 1 << 18; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT19 = 1 << 19; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT20 = 1 << 20; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT21 = 1 << 21; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT22 = 1 << 22; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT23 = 1 << 23; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT24 = 1 << 24; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT25 = 1 << 25; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT26 = 1 << 26; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT27 = 1 << 27; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT28 = 1 << 28; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT29 = 1 << 29; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT30 = 1 << 30; /* RW 1: 512字节块大小，0：512字节块大小 */
        const BIT31 = 1 << 31; /* RW 1: 512字节块大小，0：512字节块大小 */
        const ALL_BITS = 0xFFFFFFFF;
    }
}
impl FlagReg for MCIBlkSiz {
    const REG: u32 = FSDIF_BLK_SIZ_OFFSET; // 假设 FSDIF_BLK_SIZ_OFFSET 是对应的寄存器偏移量
}

// FSDIF_TRAN_CARD_CNT_OFFSET Register
bitflags! {
    pub struct MCITranCardCnt:u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCITranCardCnt {
    const REG: u32 = FSDIF_TRAN_CARD_CNT_OFFSET;
}

// FSDIF_TRAN_FIFO_CNT_OFFSET Register
bitflags! {
    pub struct MCITranFifoCnt:u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCITranFifoCnt {
    const REG: u32 = FSDIF_TRAN_FIFO_CNT_OFFSET;
}

// FSDIF_RESP0_OFFSET Register
bitflags! {
    pub struct MCIResp0:u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIResp0 {
    const REG: u32 = FSDIF_RESP0_OFFSET;
}

// FSDIF_RESP1_OFFSET Register
bitflags! {
    pub struct MCIResp1:u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIResp1 {
    const REG: u32 = FSDIF_RESP1_OFFSET;
}

// FSDIF_RESP2_OFFSET Register
bitflags! {
    pub struct MCIResp2:u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIResp2 {
    const REG: u32 = FSDIF_RESP2_OFFSET;
}

// FSDIF_RESP3_OFFSET Register
bitflags! {
    pub struct MCIResp3:u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIResp3 {
    const REG: u32 = FSDIF_RESP3_OFFSET;
}

// FSDIF_CMD_ARG_OFFSET Register
bitflags! {
    pub struct MCICmdArg: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}

impl FlagReg for MCICmdArg {
    const REG: u32 = FSDIF_CMD_ARG_OFFSET;
}

// FSDIF_DEBNCE_OFFSET Register
bitflags! {
    pub struct MCIDebnce: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIDebnce {
    const REG: u32 = FSDIF_DEBNCE_OFFSET;
}

// FSDIF_UID_OFFSET Register
bitflags! {
    pub struct MCIUid: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIUid {
    const REG: u32 = FSDIF_UID_OFFSET;
}

// FSDIF_VID_OFFSET Register
bitflags! {
    pub struct MCIVid: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIVid {
    const REG: u32 = FSDIF_VID_OFFSET;
}

// FSDIF_HWCONF_OFFSET Register
bitflags! {
    pub struct MCIHwconf: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIHwconf {
    const REG: u32 = FSDIF_HWCONF_OFFSET;
}
// FSDIF_CUR_DESC_ADDRL_OFFSET Register
bitflags! {
    pub struct MCICurDescAddrL: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCICurDescAddrL {
    const REG: u32 = FSDIF_CUR_DESC_ADDRL_OFFSET;
}
// FSDIF_CUR_DESC_ADDRH_OFFSET Register
bitflags! {
    pub struct MCIDescAddrH: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIDescAddrH {
    const REG: u32 = FSDIF_CUR_DESC_ADDRH_OFFSET;
}
// FSDIF_CUR_BUF_ADDRL_OFFSET Register
bitflags! {
    pub struct MCICurBufAddrL: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCICurBufAddrL {
    const REG: u32 = FSDIF_CUR_BUF_ADDRL_OFFSET;
}
// FSDIF_CUR_BUF_ADDRH_OFFSET Register
bitflags! {
    pub struct MCIBufAddrH: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIBufAddrH {
    const REG: u32 = FSDIF_CUR_BUF_ADDRH_OFFSET;
}
// FSDIF_ENABLE_SHIFT_OFFSET Register
bitflags! {
    pub struct MCIEnableShift: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for MCIEnableShift {
    const REG: u32 = FSDIF_ENABLE_SHIFT_OFFSET;
}

bitflags! {
    pub struct IrqTempRegister: u32 {
        const BIT0 = 1 << 0;
        const BIT1 = 1 << 1;
        const BIT2 = 1 << 2;
        const BIT3 = 1 << 3;
        const BIT4 = 1 << 4;
        const BIT5 = 1 << 5;
        const BIT6 = 1 << 6;
        const BIT7 = 1 << 7;
        const BIT8 = 1 << 8;
        const BIT9 = 1 << 9;
        const BIT10 = 1 << 10;
        const BIT11 = 1 << 11;
        const BIT12 = 1 << 12;
        const BIT13 = 1 << 13;
        const BIT14 = 1 << 14;
        const BIT15 = 1 << 15;
        const BIT16 = 1 << 16;
        const BIT17 = 1 << 17;
        const BIT18 = 1 << 18;
        const BIT19 = 1 << 19;
        const BIT20 = 1 << 20;
        const BIT21 = 1 << 21;
        const BIT22 = 1 << 22;
        const BIT23 = 1 << 23;
        const BIT24 = 1 << 24;
        const BIT25 = 1 << 25;
        const BIT26 = 1 << 26;
        const BIT27 = 1 << 27;
        const BIT28 = 1 << 28;
        const BIT29 = 1 << 29;
        const BIT30 = 1 << 30;
        const BIT31 = 1 << 31;
    }
}
impl FlagReg for IrqTempRegister {
    const REG: u32 = TEMP_REGISTER_OFFSET;
}
