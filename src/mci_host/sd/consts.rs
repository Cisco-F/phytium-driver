use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SdTimingMode {
    SDR12DefaultMode = 0,
    SDR25HighSpeedMode = 1,
    SDR50Mode = 2,
    SDR104Mode = 3,
    DDR50Mode = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SdDriverStrength {
    TypeB = 0,
    TypeA = 1,
    TypeC = 2,
    TypeD = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SdMaxCurrent {
    Limit200mA = 0,
    Limit400mA = 1,
    Limit600mA = 2,
    Limit800mA = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SdIoVoltageCtrlType {
    NotSupport = 0,
    ByHost = 1,
    ByGpio = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdCmd {
    SendRelativeAddress = 3,    // Send Relative Address
    Switch = 6,                 // Switch Function
    SendInterfaceCondition = 8, // Send Interface Condition
    VoltageSwitch = 11,         // Voltage Switch
    SpeedClassControl = 20,     // Speed Class control
    SendTuningBlock = 19,       // Send Tuning Block
    EraseWriteBlockStart = 32,  // Write Block Start
    EraseWriteBlockEnd = 33,    // Write Block End
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SdAppCmd {
    SetBusWdith = 6,              /* Set Bus Width */
    Status = 13,                  /* Send SD status */
    SendNumberWriteBlocks = 22,   /* Send Number Of Written Blocks */
    SetWriteBlockEraseCount = 23, /* Set Write Block Erase Count */
    SendOperationCondition = 41,  /* Send Operation Condition */
    SetClearCardDetect = 42,      /* Set Connect/Disconnect pull up on detect pin */
    SendScr = 51,                 /* Send Scr */
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SdSwitchMode {
    Check = 0,
    Set = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SdGroupNum {
    TimingMode = 0,
    CommandSystem = 1,
    DriverStrength = 2,
    CurrentLimit = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SdTimingFuncNum {
    SDR12Default = 0,
    SDR25HighSpeed = 1,
    SDR50 = 2,
    SDR104 = 3,
    DDR50 = 4,
}

bitflags! {
    /// SD card flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SdCardFlag: u32 {
        /// Support high capacity
        const SupportHighCapacity = 1 << 1;
        /// Support 4-bit data width
        const Support4BitWidth = 1 << 2;
        /// Card is SDHC
        const SupportSdhc = 1 << 3;
        /// Card is SDXC
        const SupportSdxc = 1 << 4;
        /// Card supports 1.8v voltage
        const SupportVoltage180v = 1 << 5;
        /// Card supports CMD23 (SET_BLOCK_COUNT)
        const SupportSetBlockCountCmd = 1 << 6;
        /// Card supports speed class control
        const SupportSpeedClassControlCmd = 1 << 7;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SdSpecificationVersion {
    Version1_0 = 1 << 0,
    Version1_1 = 1 << 1,
    Version2_0 = 1 << 2,
    Version3_0 = 1 << 3,
}

pub(crate) const SD_POWER_ON_DELAY_MS: u32 = 400;
pub(crate) const SD_POWER_OFF_DELAY_MS: u32 = 100;

pub(crate) const SD_CLOCK_25MHZ: u32 = 25_000_000;
pub(crate) const SD_CLOCK_50MHZ: u32 = 50_000_000;
pub(crate) const SD_CLOCK_100MHZ: u32 = 100_000_000;
pub(crate) const SD_CLOCK_208MHZ: u32 = 208_000_000;

pub(crate) const SD_CMD13_RETRY_TIMES: u32 = 10;
pub(crate) const SD_PRODUCT_NAME_BYTES: usize = 5;
pub(crate) const SD_MAX_RW_BLK: usize = 1024;
pub(crate) const SD_BLOCK_SIZE: usize = 512;

pub(crate) const SD_CARD_ACCESS_WAIT_IDLE_TIMEOUT: u32 = 600;
