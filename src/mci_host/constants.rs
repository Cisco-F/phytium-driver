use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostCmdType {
    Normal = 0,  // 普通命令
    Suspend = 1, // 挂起命令
    Resume = 2,  // 恢复命令
    Abort = 3,   // 中止命令
    Empty = 4,   // 空命令
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostResponseType {
    None = 0, // 无响应
    R1 = 1,   // 响应类型: R1
    R1b = 2,  // 响应类型: R1b
    R2 = 3,   // 响应类型: R2
    R3 = 4,   // 响应类型: R3
    R4 = 5,   // 响应类型: R4
    R5 = 6,   // 响应类型: R5
    R5b = 7,  // 响应类型: R5b
    R6 = 8,   // 响应类型: R6s
    R7 = 9,   // 响应类型: R7
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostDataPacketFormat {
    MSBFirst = 0, // 数据包格式: MSB 优先
    LSBFirst = 1, // 数据包格式: LSB 优先
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostBusWdith {
    Bit1 = 1,
    Bit4 = 4,
    Bit8 = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostCommonCmd {
    GoIdleState = 0,         // Go Idle State
    AllSendCid = 2,          // All Send CID
    SetDsr = 4,              // Set DSR
    SelectCard = 7,          // Select Card
    SendCsd = 9,             // Send CSD
    SendCid = 10,            // Send CID
    StopTransmission = 12,   // Stop Transmission
    SendStatus = 13,         // Send Status
    GoInactiveState = 15,    // Go Inactive State
    SetBlockLength = 16,     // Set Block Length
    ReadSingleBlock = 17,    // Read Single Block
    ReadMultipleBlock = 18,  // Read Multiple Block
    SetBlockCount = 23,      // Set Block Count
    WriteSingleBlock = 24,   // Write Single Block
    WriteMultipleBlock = 25, // Write Multiple Block
    ProgramCsd = 27,         // Program CSD
    SetWriteProtect = 28,    // Set Write Protect
    ClearWriteProtect = 29,  // Clear Write Protect
    SendWriteProtect = 30,   // Send Write Protect
    Erase = 38,              // Erase
    LockUnlock = 42,         // Lock Unlock
    ApplicationCommand = 55, // Send Application Command
    GeneralCommand = 56,     // General Purpose Command
    ReadOcr = 58,            // Read OCR
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCISDIOCommand {
    SendRelativeAddress = 3,    // Send Relative Address
    SendOperationCondition = 5, // Send Operation Condition
    SendInterfaceCondition = 8, // Send Interface Condition
    RWIODirect = 52,            // Read/Write I/O Direct
    RWIODirectExtended = 53,    // Read/Write I/O Direct Extended
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCISDIOCCCRAddr {
    SDIOVer = 0x00,            // CCCR & SDIO version
    SDVersion = 0x01,          // SD version
    IOEnable = 0x02,           // io enable register
    IOReady = 0x03,            // io ready register
    IOIntEnable = 0x04,        // io interrupt enable register
    IOIntPending = 0x05,       // io interrupt pending register
    IOAbort = 0x06,            // io abort register
    BusInterface = 0x07,       // bus interface register
    CardCapability = 0x08,     // card capability register
    CommonCISPointer = 0x09,   // common CIS pointer register
    BusSuspend = 0x0C,         // bus suspend register
    FunctionSelect = 0x0D,     // function select register
    ExecutionFlag = 0x0E,      // execution flag register
    ReadyFlag = 0x0F,          // ready flag register
    FN0BlockSizeLow = 0x10,    // FN0 block size register
    FN0BlockSizeHigh = 0x11,   // FN0 block size register
    PowerControl = 0x12,       // power control register
    BusSpeed = 0x13,           // bus speed register
    UHSITimingSupport = 0x14,  // UHS-I timing support register
    DriverStrength = 0x15,     // Driver strength register
    InterruptExtension = 0x16, // Interrupt extension register
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MCIHostCardStatusFlag: u32 {
        const OUT_OF_RANGE                  = 1 << 31; // Out of range status bit
        const ADDRESS_ERROR                 = 1 << 30; // Address error status bit
        const BLOCK_LENGTH_ERROR            = 1 << 29; // Block length error status bit
        const ERASE_SEQUENCE_ERROR          = 1 << 28; // Erase sequence error status bit
        const ERASE_PARAMETER_ERROR         = 1 << 27; // Erase parameter error status bit
        const WRITE_PROTECT_VIOLATION       = 1 << 26; // Write protection violation status bit
        const CARD_IS_LOCKED                = 1 << 25; // Card locked status bit
        const LOCK_UNLOCK_FAILED            = 1 << 24; // Lock/unlock error status bit
        const COMMAND_CRC_ERROR             = 1 << 23; // CRC error status bit
        const ILLEGAL_COMMAND               = 1 << 22; // Illegal command status bit
        const CARD_ECC_FAILED               = 1 << 21; // Card ECC error status bit
        const CARD_CONTROLLER_ERROR         = 1 << 20; // Internal card controller error status bit
        const ERROR                         = 1 << 19; // A general or an unknown error status bit
        const CID_CSD_OVERWRITE             = 1 << 16; // CID/CSD overwrite status bit
        const WRITE_PROTECT_ERASE_SKIP      = 1 << 15; // Write protection erase skip status bit
        const CARD_ECC_DISABLED             = 1 << 14; // Card ECC disabled status bit
        const ERASE_RESET                   = 1 << 13; // Erase reset status bit
        const READY_FOR_DATA                = 1 << 8;  // Ready for data status bit
        const SWITCH_ERROR                  = 1 << 7;  // Switch error status bit
        const APPLICATION_COMMAND           = 1 << 5;  // Application command enabled status bit
        const AUTHENTICATION_SEQUENCE_ERROR = 1 << 3;  // Error in the sequence of authentication process
        const ALL_ERROR_FLAG = 0xFFF90008;    // All error status bits
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MCIHostCurrentState {
    Idle = 0,
    Ready = 1,
    Identification = 2,
    Standby = 3,
    Transfer = 4,
    Data = 5,
    Receive = 6,
    Programming = 7,
    Disconnect = 8,
}

impl MCIHostCurrentState {
    pub(crate) fn current_state(state: u32) -> Self {
        let state = (state & 0x00001E00) >> 9;
        match state {
            0 => MCIHostCurrentState::Idle,
            1 => MCIHostCurrentState::Ready,
            2 => MCIHostCurrentState::Identification,
            3 => MCIHostCurrentState::Standby,
            4 => MCIHostCurrentState::Transfer,
            5 => MCIHostCurrentState::Data,
            6 => MCIHostCurrentState::Receive,
            7 => MCIHostCurrentState::Programming,
            8 => MCIHostCurrentState::Disconnect,
            _ => MCIHostCurrentState::Idle,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostOperationVoltage {
    None = 0,
    Voltage330V = 1,
    Voltage300V = 2,
    Voltage180V = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MCIHostDetectCardType {
    ByGpioCD,
    ByHostCD,
    ByHostDATA3,
}

pub(crate) const MCI_HOST_CLOCK_400KHZ: u32 = 400_000;
pub(crate) const MCI_HOST_MAX_CMD_RETRIES: u32 = 10;
pub(crate) const MCI_HOST_DEFAULT_BLOCK_SIZE: u32 = 512;
pub(crate) const MCI_HOST_MAX_BLOCK_LENGTH: u32 = 4096;

bitflags! {
    /// OCR register flags in SD card
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct MCIHostOCR: u32 {
        /// Power up busy status (bit 31)
        const POWER_UP_BUSY_FLAG = 1 << 31;

        /// Card/Host capacity status (bit 30)
        const HOST_CAPACITY_SUPPORT_FLAG = 1 << 30;
        /// Card capacity status (bit 30, same as HOST_CAPACITY_SUPPORT_FLAG)
        const CARD_CAPACITY_SUPPORT_FLAG = 1 << 30;

        /// Switch to 1.8V request (bit 24)
        const SWITCH_18_REQUEST_FLAG = 1 << 24;
        /// Switch to 1.8V accepted (bit 24, same as SWITCH_18_REQUEST_FLAG)
        const SWITCH_18_ACCEPT_FLAG = 1 << 24;

        /// VDD 2.7-2.8V (bit 15)
        const VDD_27_28 = 1 << 15;
        /// VDD 2.8-2.9V (bit 16)
        const VDD_28_29 = 1 << 16;
        /// VDD 2.9-3.0V (bit 17)
        const VDD_29_30 = 1 << 17;
        /// VDD 3.0-3.1V (bit 18)
        const VDD_30_31 = 1 << 18;
        /// VDD 3.1-3.2V (bit 19)
        const VDD_31_32 = 1 << 19;
        /// VDD 3.2-3.3V (bit 20)
        const VDD_32_33 = 1 << 20;
        /// VDD 3.3-3.4V (bit 21)
        const VDD_33_34 = 1 << 21;
        /// VDD 3.4-3.5V (bit 22)
        const VDD_34_35 = 1 << 22;
        /// VDD 3.5-3.6V (bit 23)
        const VDD_35_36 = 1 << 23;
    }
}

bitflags! {
    /// SDMMC host capability flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct MCIHostCapability: u32 {
        /// High speed capability
        const HIGH_SPEED = 1 << 0;
        /// Suspend resume capability
        const SUSPEND_RESUME = 1 << 1;
        /// 3.3V capability
        const VOLTAGE_3V3 = 1 << 2;
        /// 3.0V capability
        const VOLTAGE_3V0 = 1 << 3;
        /// 1.8V capability
        const VOLTAGE_1V8 = 1 << 4;
        /// 1.2V capability
        const VOLTAGE_1V2 = 1 << 5;
        /// 4-bit data width capability
        const BIT4_DATA_WIDTH = 1 << 6;
        /// 8-bit data width capability
        const BIT8_DATA_WIDTH = 1 << 7;
        /// DDR mode capability
        const DDR_MODE = 1 << 8;
        /// Data3 detect card capability
        const DETECT_CARD_BY_DATA3 = 1 << 9;
        /// CD detect card capability
        const DETECT_CARD_BY_CD = 1 << 10;
        /// Auto command 12 capability
        const AUTO_CMD12 = 1 << 11;
        /// SDR104 capability
        const SDR104 = 1 << 12;
        /// SDR50 capability
        const SDR50 = 1 << 13;
        /// HS200 capability
        const HS200 = 1 << 14;
        /// HS400 capability
        const HS400 = 1 << 15;
        /// Driver Type C capability
        const DRIVER_TYPE_C = 1 << 16;
        /// Set current limit capability
        const SET_CURRENT = 1 << 17;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct MCIHostCapabilityExt: u32 {
        const BIT8_WIDTH =1;
    }
}

pub const SZ_1: u64 = 0x00000001;
pub const SZ_2: u64 = 0x00000002;
pub const SZ_4: u64 = 0x00000004;
pub const SZ_8: u64 = 0x00000008;
pub const SZ_16: u64 = 0x00000010;
pub const SZ_32: u64 = 0x00000020;
pub const SZ_64: u64 = 0x00000040;
pub const SZ_128: u64 = 0x00000080;
pub const SZ_256: u64 = 0x00000100;
pub const SZ_512: u64 = 0x00000200;

pub const SZ_1K: u64 = 0x00000400;
pub const SZ_2K: u64 = 0x00000800;
pub const SZ_4K: u64 = 0x00001000;
pub const SZ_8K: u64 = 0x00002000;
pub const SZ_16K: u64 = 0x00004000;
pub const SZ_32K: u64 = 0x00008000;
pub const SZ_64K: u64 = 0x00010000;
pub const SZ_128K: u64 = 0x00020000;
pub const SZ_256K: u64 = 0x00040000;
pub const SZ_512K: u64 = 0x00080000;

pub const SZ_1M: u64 = 0x00100000;
pub const SZ_2M: u64 = 0x00200000;
pub const SZ_4M: u64 = 0x00400000;
pub const SZ_8M: u64 = 0x00800000;
pub const SZ_16M: u64 = 0x01000000;
pub const SZ_32M: u64 = 0x02000000;
pub const SZ_64M: u64 = 0x04000000;
pub const SZ_128M: u64 = 0x08000000;
pub const SZ_256M: u64 = 0x10000000;
pub const SZ_512M: u64 = 0x20000000;

pub const SZ_1G: u64 = 0x40000000;
pub const SZ_2G: u64 = 0x80000000;
pub const SZ_3G: u64 = 0xC0000000;
pub const SZ_4G: u64 = 0x100000000;
pub const SZ_8G: u64 = 0x200000000;
