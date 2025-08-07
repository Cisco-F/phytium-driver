//! consts relative of memory pool and events

/// Size of 1 MiB
pub const SZ_2M: usize = 2 * 1024 * 1024;
/// Max size can be managed by Tlsf pool
pub const MAX_POOL_SIZE: usize = SZ_2M;
/// OSA semaphore handle size
pub const OSA_SEM_HANDLE_SIZE: usize = 8;

/// Transfer event flags
/// Command transfer completed successfully
pub const SDMMC_OSA_EVENT_TRANSFER_CMD_SUCCESS: u32 = 1 << 0;
/// Command transfer failed
pub const SDMMC_OSA_EVENT_TRANSFER_CMD_FAIL: u32 = 1 << 1;
/// Data transfer completed successfully
pub const SDMMC_OSA_EVENT_TRANSFER_DATA_SUCCESS: u32 = 1 << 2;
/// Data transfer failed
pub const SDMMC_OSA_EVENT_TRANSFER_DATA_FAIL: u32 = 1 << 3;
/// DMA transfer completed
pub const SDMMC_OSA_EVENT_TRANSFER_DMA_COMPLETE: u32 = 1 << 4;

/// Card insertion detected
pub const SDMMC_OSA_EVENT_CARD_INSERTED: u32 = 1 << 8;
/// Card removal detected
pub const SDMMC_OSA_EVENT_CARD_REMOVED: u32 = 1 << 9;

/// Combined error events mask for transfer
pub const FSDIF_TRANS_ERR_EVENTS: u32 = SDMMC_OSA_EVENT_TRANSFER_CMD_FAIL
    | SDMMC_OSA_EVENT_TRANSFER_DATA_FAIL
    | SDMMC_OSA_EVENT_CARD_REMOVED;

/// Event flag for AND operation
pub const SDMMC_OSA_EVENT_FLAG_AND: u32 = 1 << 0;
/// Event flag for OR operation
pub const SDMMC_OSA_EVENT_FLAG_OR: u32 = 1 << 1;
