use super::constants::MCIHostDetectCardType;

#[allow(unused)]
pub struct MCIHostCardDetect {
    pub(crate) typ: MCIHostDetectCardType,
    pub(crate) cd_debounce_ms: u32,
    // todo Function type that takes a boolean and a user data pointer
    pub(crate) card_detected: Option<MCIHostCdStatusFn>,
    pub(crate) dat3_pull_func: Option<MCIHostDat3PullFn>,
    // todo user data
}

type MCIHostCdStatusFn = fn() -> bool;
type MCIHostDat3PullFn = fn(pull_status: u32);

impl MCIHostCardDetect {
    pub fn new() -> Self {
        MCIHostCardDetect {
            typ: MCIHostDetectCardType::ByGpioCD,
            cd_debounce_ms: 0,
            card_detected: None,
            dat3_pull_func: None,
        }
    }
}
