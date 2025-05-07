use crate::mci_host::constants::MCIHostOperationVoltage;

use super::consts::SdIoVoltageCtrlType;

pub(crate) struct SdIoVoltage {
    typ: SdIoVoltageCtrlType,
    func: Option<SdIoVoltageFn>,
}

type SdIoVoltageFn = fn(MCIHostOperationVoltage);

impl SdIoVoltage {
    pub(crate) fn new() -> Self {
        SdIoVoltage {
            typ: SdIoVoltageCtrlType::NotSupport,
            func: None,
        }
    }
    
    pub(crate) fn typ(&self) -> SdIoVoltageCtrlType {
        self.typ
    }

    pub(crate) fn typ_set(&mut self, typ: SdIoVoltageCtrlType) {
        self.typ = typ;
    }

    pub(crate) fn func(&self) -> Option<&SdIoVoltageFn> {
        self.func.as_ref()
    }

    pub(crate) fn set_func(&mut self, func: Option<SdIoVoltageFn>) {
        self.func = func;
    }
}