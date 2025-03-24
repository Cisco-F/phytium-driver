use crate::mci::constants::MCIId;

use super::sd::constants::{SD_BLOCK_SIZE, SD_CLOCK_50MHZ, SD_MAX_RW_BLK};

#[allow(unused)]
pub struct MCIHostConfig {
    pub(crate) host_id: MCIId,                     // 主机 ID
    pub(crate) host_type: MCIHostType,           // 主机类型
    pub(crate) card_type: MCIHostCardType,       // 卡类型
    pub(crate) enable_irq: bool,                 // 是否启用中断
    pub(crate) enable_dma: bool,                 // 是否启用 DMA
    pub(crate) endian_mode: MCIHostEndianMode,   // 字节序模式
    pub(crate) max_trans_size: usize,            // 最大传输大小
    pub(crate) def_block_size: usize,            // 默认块大小
    pub(crate) card_clock: u32,                  // 卡时钟频率
    pub(crate) is_uhs_card: bool,                // 是否为 UHS 卡
    /* for SDIO card, to support card customized interrupt handling */ // todo 暂时没实现这部分功能
}

#[allow(unused)]
impl MCIHostConfig {
    pub fn mci0_sd_instance() -> Self {
        MCIHostConfig {
            host_id: MCIId::MCI0,
            host_type: MCIHostType::SDIF,
            card_type: MCIHostCardType::MicroSD,
            enable_irq: false, // todo 暂时不支持中断
            enable_dma: false, // todo 暂时不支持 DMA
            endian_mode: MCIHostEndianMode::Little,
            max_trans_size: SD_MAX_RW_BLK*SD_BLOCK_SIZE,
            def_block_size: SD_BLOCK_SIZE,
            card_clock: SD_CLOCK_50MHZ,
            is_uhs_card: false, // todo 需要测试能不能支持UHS模式
        }
    }

    pub fn mci0_sd_dma_instance() -> Self {
        MCIHostConfig {
            host_id: MCIId::MCI0,
            host_type: MCIHostType::SDIF,
            card_type: MCIHostCardType::MicroSD,
            enable_irq: true,
            enable_dma: true,
            endian_mode: MCIHostEndianMode::Little,
            max_trans_size: SD_MAX_RW_BLK*SD_BLOCK_SIZE,
            def_block_size: SD_BLOCK_SIZE,
            card_clock: SD_CLOCK_50MHZ,
            is_uhs_card: false, // todo 需要测试能不能支持UHS模式
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MCIHostType {
    SDMMC,
    SDIF
}

#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MCIHostCardType {
    StandardSD,
    MicroSD,
    EMMC,
    SDIO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MCIHostEndianMode {
    Big = 0, /* Big endian mode */
    HalfWordBig = 1, /* Half word big endian mode */
    Little = 2, /* Little endian mode */
}
