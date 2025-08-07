#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MCIHostError {
    Fail,
    ReadOnly,
    OutOfRange,
    InvalidArgument,
    Timeout,
    NoTransferInProgress,
    Busy,
    NoData,
    // SD/MMC card API's running status.
    NotSupportYet,                     // Haven't supported
    TransferFailed,                    // Send command failed
    SetCardBlockSizeFailed,            // Set block size failed
    HostNotSupport,                    // Host doesn't support
    CardNotSupport,                    // Card doesn't support
    AllSendCidFailed,                  // Send CID failed
    SendRelativeAddressFailed,         // Send relative address failed
    SendCsdFailed,                     // Send CSD failed
    SelectCardFailed,                  // Select card failed
    SendScrFailed,                     // Send SCR failed
    SetDataBusWidthFailed,             // Set bus width failed
    GoIdleFailed,                      // Go idle failed
    HandShakeOperationConditionFailed, // Send Operation Condition failed
    SendApplicationCommandFailed,      // Send application command failed
    SwitchFailed,                      // Switch command failed
    StopTransmissionFailed,            // Stop transmission failed
    WaitWriteCompleteFailed,           // Wait write complete failed
    SetBlockCountFailed,               // Set block count failed
    SetRelativeAddressFailed,          // Set relative address failed
    SwitchBusTimingFailed,             // Switch high speed failed
    SendExtendedCsdFailed,             // Send EXT_CSD failed
    ConfigureBootFailed,               // Configure boot failed
    ConfigureExtendedCsdFailed,        // Configure EXT_CSD failed
    EnableHighCapacityEraseFailed,     // Enable high capacity erase failed
    SendTestPatternFailed,             // Send test pattern failed
    ReceiveTestPatternFailed,          // Receive test pattern failed
    SdioResponseError,                 // SDIO response error
    SdioInvalidArgument,               // SDIO invalid argument response error
    SdioSendOperationConditionFail,    // SDIO send operation condition fail
    InvalidVoltage,                    // Invalid voltage
    SdioSwitchHighSpeedFail,           // Switch to high speed fail
    SdioReadCISFail,                   // Read CIS fail
    SdioInvalidCard,                   // Invalid SDIO card
    TuningFail,                        // Tuning fail
    SwitchVoltageFail,                 // Switch voltage fail
    SwitchVoltage18VFail33VSuccess,    // Switch voltage fail
    ReTuningRequest,                   // Retuning request
    SetDriverStrengthFail,             // Set driver strength fail
    SetPowerClassFail,                 // Set power class fail
    HostNotReady,                      // Host controller not ready
    CardDetectFailed,                  // Card detect failed
    AuSizeNotSetProperly,              // AU size not set properly
    PollingCardIdleFailed,             // Polling card idle status failed
    DeselectCardFailed,                // Deselect card failed
    CardStatusIdle,                    // Card idle
    CardStatusBusy,                    // Card busy
    CardInitFailed,                    // Card init failed
    IrqInitFailed,                     // init irq failed
}

pub type MCIHostStatus<T = ()> = Result<T, MCIHostError>;
