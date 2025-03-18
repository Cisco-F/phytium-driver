use bitflags::bitflags;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MCIId {
    MCI0,
    MCI1,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MCIFifoDepth {
    Depth8 = 23,
    Depth16 = 24,
    Depth32 = 25,
    Depth64 = 26,
    Depth128 = 27,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct MCICmdFlag: u32 {
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

// 定义传输模式枚举
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MCITransMode {
    DMA,      // DMA传输模式
    PIO,      // PIO传输模式（通过读/写Fifo）
}

// 定义中断类型枚举
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MCIIntrType {
    GeneralIntr,       // 属于控制器的中断状态
    DmaIntr,           // 属于DMA的中断状态
}

// 定义事件类型枚举
#[derive(Debug, PartialEq)]
pub enum FsDifEvtType {
    CardDetected = 0,  // 卡检测事件
    CmdDone,           // 命令传输完成事件
    DataDone,          // 包含数据的命令传输完成事件
    SdioIrq,           // SDIO卡自定义事件
    ErrOccured,        // 传输中出现错误
    NumOfEvt,          // 事件数量
}

// 定义时钟速度枚举
#[derive(Debug, PartialEq)]
pub enum MCIClkSpeed {
    ClkSpeed400KHz = 400_000,
    ClkSpeed25Mhz = 25_000_000,
    ClkSpeed26Mhz = 26_000_000, // mmc
    ClkSpeed50Mhz = 50_000_000,
    ClkSpeed52Mhz = 52_000_000, // mmc
    ClkSpeed66Mhz = 66_000_000, // mmc
    ClkSpeed100Mhz = 100_000_000,
}

impl From<u32> for MCIClkSpeed {
    fn from(value: u32) -> Self {
        match value {
            400_000 => MCIClkSpeed::ClkSpeed400KHz,
            25_000_000 => MCIClkSpeed::ClkSpeed25Mhz,
            26_000_000 => MCIClkSpeed::ClkSpeed26Mhz,
            50_000_000 => MCIClkSpeed::ClkSpeed50Mhz,
            52_000_000 => MCIClkSpeed::ClkSpeed52Mhz,
            66_000_000 => MCIClkSpeed::ClkSpeed66Mhz,
            100_000_000 => MCIClkSpeed::ClkSpeed100Mhz,
            _ => panic!("Invalid clock speed"),
        }
    }
}

#[inline(always)]
pub unsafe fn dsb() {
    core::arch::asm!("dsb sy", options(nostack, preserves_flags));
}

/** @name Register Map
 *
 * Register offsets from the base address of an SD device.
 * @{
 */
pub const FSDIF_CNTRL_OFFSET: u32 = 0x00; // the controller config reg
pub const FSDIF_PWREN_OFFSET: u32 = 0x04; // the power enable reg
pub const FSDIF_CLKDIV_OFFSET: u32 = 0x08; // the clock divider reg
pub const FSDIF_CLKENA_OFFSET: u32 = 0x10; // the clock enable reg
pub const FSDIF_TMOUT_OFFSET: u32 = 0x14; // the timeout reg
pub const FSDIF_CTYPE_OFFSET: u32 = 0x18; // the card type reg
pub const FSDIF_BLK_SIZ_OFFSET: u32 = 0x1C; // the block size reg
pub const FSDIF_BYT_CNT_OFFSET: u32 = 0x20; // the byte count reg
pub const FSDIF_INT_MASK_OFFSET: u32 = 0x24; // the interrupt mask reg
pub const FSDIF_CMD_ARG_OFFSET: u32 = 0x28; // the command argument reg
pub const FSDIF_CMD_OFFSET: u32 = 0x2C; // the command reg
pub const FSDIF_RESP0_OFFSET: u32 = 0x30; // the response reg0
pub const FSDIF_RESP1_OFFSET: u32 = 0x34; // the response reg1
pub const FSDIF_RESP2_OFFSET: u32 = 0x38; // the response reg2
pub const FSDIF_RESP3_OFFSET: u32 = 0x3C; // the response reg3
pub const FSDIF_MASKED_INTS_OFFSET: u32 = 0x40; // the masked interrupt status reg
pub const FSDIF_RAW_INTS_OFFSET: u32 = 0x44; // the raw interrupt status reg
pub const FSDIF_STATUS_OFFSET: u32 = 0x48; // the status reg
pub const FSDIF_FIFOTH_OFFSET: u32 = 0x4C; // the FIFO threshold watermark reg
pub const FSDIF_CARD_DETECT_OFFSET: u32 = 0x50; // the card detect reg
pub const FSDIF_CARD_WRTPRT_OFFSET: u32 = 0x54; // the card write protect reg
pub const FSDIF_CKSTS_OFFSET: u32 = 0x58; // the ciu ready
pub const FSDIF_TRAN_CARD_CNT_OFFSET: u32 = 0x5C; // the transferred CIU card byte count reg
pub const FSDIF_TRAN_FIFO_CNT_OFFSET: u32 = 0x60; // the transferred host to FIFO byte count reg
pub const FSDIF_DEBNCE_OFFSET: u32 = 0x64; // the debounce count reg
pub const FSDIF_UID_OFFSET: u32 = 0x68; // the user ID reg
pub const FSDIF_VID_OFFSET: u32 = 0x6C; // the controller version ID reg
pub const FSDIF_HWCONF_OFFSET: u32 = 0x70; // the hardware configuration reg
pub const FSDIF_UHS_REG_OFFSET: u32 = 0x74; // the UHS-I reg
pub const FSDIF_CARD_RESET_OFFSET: u32 = 0x78; // the card reset reg
pub const FSDIF_BUS_MODE_OFFSET: u32 = 0x80; // the bus mode reg
pub const FSDIF_DESC_LIST_ADDRL_OFFSET: u32 = 0x88; // the descriptor list low base address reg
pub const FSDIF_DESC_LIST_ADDRH_OFFSET: u32 = 0x8C; // the descriptor list high base address reg
pub const FSDIF_DMAC_STATUS_OFFSET: u32 = 0x90; // the internal DMAC status reg
pub const FSDIF_DMAC_INT_EN_OFFSET: u32 = 0x94; // the internal DMAC interrupt enable reg
pub const FSDIF_CUR_DESC_ADDRL_OFFSET: u32 = 0x98; // the current host descriptor low address reg
pub const FSDIF_CUR_DESC_ADDRH_OFFSET: u32 = 0x9C; // the current host descriptor high address reg
pub const FSDIF_CUR_BUF_ADDRL_OFFSET: u32 = 0xA0; // the current buffer low address reg
pub const FSDIF_CUR_BUF_ADDRH_OFFSET: u32 = 0xA4; // the current buffer high address reg
pub const FSDIF_CARD_THRCTL_OFFSET: u32 = 0x100; // the card threshold control reg
pub const FSDIF_CLK_SRC_OFFSET: u32 = 0x108; // the UHS register extension
pub const FSDIF_EMMC_DDR_REG_OFFSET: u32 = 0x10C; // the EMMC DDR reg
pub const FSDIF_ENABLE_SHIFT_OFFSET: u32 = 0x110; // the enable phase shift reg
pub const FSDIF_DATA_OFFSET: u32 = 0x200; // the data FIFO access

pub const RETRIES_TIMEOUT:usize = 50000; /* timeout for retries */
pub const FSDIF_DELAY_US:u32 = 5;
pub const MCI_MAX_FIFO_CNT:u32 = 0x800;

pub const FSL_SDMMC_MAX_CMD_RETRIES:u32 = 10;

pub const FSDIF0_ID: u32 = 0;
pub const FSDIF1_ID: u32 = 1;

pub const FT_COMPONENT_IS_READY: u32 = 0x11111111;

// DMA相关
pub const FSDIF_IDMAC_DES0_DIC: u32 = 1 << 1;    /* 内部描述表不触发TI/RI中断 */
pub const FSDIF_IDMAC_DES0_LD: u32 = 1 << 2;     /* 数据的最后一个描述符 */
pub const FSDIF_IDMAC_DES0_FD: u32 = 1 << 3;     /* 数据的第一个描述符 */
pub const FSDIF_IDMAC_DES0_CH: u32 = 1 << 4;     /* 链接下一个描述符地址 */
pub const FSDIF_IDMAC_DES0_ER: u32 = 1 << 5;     /* 链表已经到达最后一个链表 */
pub const FSDIF_IDMAC_DES0_CES: u32 = 1 << 30;   /* RINTSTS寄存器错误汇总 */
pub const FSDIF_IDMAC_DES0_OWN: u32 = 1 << 31;   /* 描述符关联DMA，完成传输后该位置置0 */
pub const FSDIF_IDMAC_MAX_BUF_SIZE: usize = 0x1000; // 每个desc在chained mode最多传输的字节数

pub const FSDIF_INT_RE_BIT: u32 = 1 << 1;
pub const FSDIF_INT_CMD_BIT: u32 = 1 << 2;
pub const FSDIF_INT_DTO_BIT: u32 = 1 << 3;
pub const FSDIF_INT_RCRC_BIT: u32 = 1 << 6;
pub const FSDIF_INT_DCRC_BIT: u32 = 1 << 7;
pub const FSDIF_INT_RTO_BIT: u32 = 1 << 8;
pub const FSDIF_INT_DRTO_BIT: u32 = 1 << 9;
pub const FSDIF_INT_HTO_BIT: u32 = 1 << 10;
pub const FSDIF_INT_HLE_BIT: u32 = 1 << 12;
pub const FSDIF_INT_SBE_BCI_BIT: u32 = 1 << 13;

pub const FSDIF_INTS_CMD_MASK: u32 = (FSDIF_INT_RE_BIT | FSDIF_INT_CMD_BIT | FSDIF_INT_RCRC_BIT |
                                      FSDIF_INT_RTO_BIT | FSDIF_INT_HTO_BIT | FSDIF_INT_HLE_BIT);

pub const FSDIF_DMAC_INT_ENA_TI: u32 = 1;    /* RW 发送完成中断使能 */
pub const FSDIF_DMAC_INT_ENA_FBE: u32 = 1 << 2;   /* RW 总线错误中断使能 */
pub const FSDIF_DMAC_INT_ENA_DU: u32 = 1 << 4;     /* RW 描述符不可读中断使能 */
pub const FSDIF_DMAC_INT_ENA_NIS: u32 = 1 << 8;    /* RW 正常中断汇总使能 */
pub const FSDIF_DMAC_INT_ENA_AIS: u32 = 1 << 9;    /* RW 异常中断汇总使能 */

pub const FSDIF_DMAC_INTS_MASK: u32 = (FSDIF_DMAC_INT_ENA_FBE | FSDIF_DMAC_INT_ENA_DU |
                                       FSDIF_DMAC_INT_ENA_NIS | FSDIF_DMAC_INT_ENA_AIS);