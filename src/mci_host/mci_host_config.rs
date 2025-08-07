#[cfg(all(feature = "dma", feature = "pio"))]
compile_error!("can't enable feature dma and pio at the same time!");
#[cfg(all(feature = "irq", feature = "poll"))]
compile_error!("can't enable feature irq and poll at the same time!");

use crate::mci::consts::MCIId;

use super::sd::consts::{SD_BLOCK_SIZE, SD_CLOCK_50MHZ, SD_MAX_RW_BLK};

#[allow(unused)]
#[derive(Clone, Copy)]
pub struct MCIHostConfig {
    pub(crate) host_id: MCIId,                 // 主机 ID
    pub(crate) host_type: MCIHostType,         // 主机类型
    pub(crate) card_type: MCIHostCardType,     // 卡类型
    pub(crate) enable_irq: bool,               // 是否启用中断
    pub(crate) enable_dma: bool,               // 是否启用 DMA
    pub(crate) endian_mode: MCIHostEndianMode, // 字节序模式
    pub(crate) max_trans_size: usize,          // 最大传输大小
    pub(crate) def_block_size: usize,          // 默认块大小
    pub(crate) card_clock: u32,                // 卡时钟频率
    pub(crate) is_uhs_card: bool,              // 是否为 UHS 卡
                                               /* for SDIO card, to support card customized interrupt handling */ // todo 暂时没实现这部分功能
                                               // todo timeTuner
}

#[allow(unused)]
impl MCIHostConfig {
    /// 目前默认为DMA POLL模式
    pub fn new() -> Self {
        let mut config = Self {
            host_id: MCIId::MCI1,
            host_type: MCIHostType::SDIF,
            card_type: MCIHostCardType::MicroSD,
            enable_irq: false,
            enable_dma: true,
            endian_mode: MCIHostEndianMode::Little,
            max_trans_size: SD_MAX_RW_BLK * SD_BLOCK_SIZE,
            def_block_size: SD_BLOCK_SIZE,
            card_clock: SD_CLOCK_50MHZ,
            is_uhs_card: false,
        };

        if cfg!(feature = "dma") {
            config.host_id = MCIId::MCI1;
            config.enable_dma = true;
        } else if cfg!(feature = "pio") {
            config.host_id = MCIId::MCI0;
            config.enable_dma = false;
        }

        if cfg!(feature = "irq") {
            config.enable_irq = true;
        } else if cfg!(feature = "poll") {
            config.enable_irq = false;
        }

        config
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MCIHostType {
    SDMMC,
    SDIF,
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MCIHostCardType {
    StandardSD,
    MicroSD,
    EMMC,
    SDIO,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MCIHostEndianMode {
    Big = 0,         /* Big endian mode */
    HalfWordBig = 1, /* Half word big endian mode */
    Little = 2,      /* Little endian mode */
}
