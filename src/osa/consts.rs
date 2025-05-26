/// Size of 1 MiB
pub const SZ_1M: usize = 1024 * 1024;
/// Max size can be managed by Tlsf pool
pub const MAX_POOL_SIZE: usize = SZ_1M;
/// OSA semaphore handle size
pub const OSA_SEM_HANDLE_SIZE: usize = 8;

/// transfer event
pub const SDMMC_OSA_EVENT_TRANSFER_CMD_SUCCESS: u32 = 1 << 0;
pub const SDMMC_OSA_EVENT_TRANSFER_CMD_FAIL: u32 = 1 << 1;
pub const SDMMC_OSA_EVENT_TRANSFER_DATA_SUCCESS: u32 = 1 << 2;
pub const SDMMC_OSA_EVENT_TRANSFER_DATA_FAIL: u32 = 1 << 3;
pub const SDMMC_OSA_EVENT_TRANSFER_DMA_COMPLETE: u32 = 1 << 4;
pub const SDMMC_OSA_EVENT_CARD_REMOVED: u32 = 1 << 9;

pub const FSDIF_TRANS_ERR_EVENTS: u32 = SDMMC_OSA_EVENT_TRANSFER_CMD_FAIL |
                                        SDMMC_OSA_EVENT_TRANSFER_DATA_FAIL |
                                        SDMMC_OSA_EVENT_CARD_REMOVED;

pub const SDMMC_OSA_EVENT_FLAG_AND: u32 = 1 << 0;
pub const SDMMC_OSA_EVENT_FLAG_OR: u32 = 1 << 1;