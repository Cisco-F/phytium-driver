use super::consts::*;
use super::mci_data::MCIData;

#[derive(Debug, Clone)]
pub struct MCICmdData {
    cmdidx: u32,
    cmdarg: u32,
    response: [u32; 4],
    flag: MCICmdFlag,
    data: Option<MCIData>,
    success: bool,
}

#[allow(unused)]
impl MCICmdData {
    pub(crate) fn new() -> Self {
        MCICmdData {
            cmdidx: 0,
            cmdarg: 0,
            response: [0; 4],
            flag: MCICmdFlag::empty(),
            data: None,
            success: false,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.cmdidx = 0;
        self.cmdarg = 0;
        self.response = [0; 4];
        self.flag = MCICmdFlag::empty();
        self.data = None;
        self.success = false;
    }

    pub(crate) fn success(&self) -> bool {
        self.success
    }

    pub(crate) fn success_set(&mut self, success: bool) {
        self.success = success;
    }

    pub(crate) fn cmdidx(&self) -> u32 {
        self.cmdidx
    }

    pub(crate) fn cmdidx_set(&mut self, cmdidx: u32) {
        self.cmdidx = cmdidx;
    }

    pub(crate) fn cmdarg(&self) -> u32 {
        self.cmdarg
    }

    pub(crate) fn cmdarg_set(&mut self, cmdarg: u32) {
        self.cmdarg = cmdarg;
    }

    pub(crate) fn flag(&self) -> &MCICmdFlag {
        &self.flag
    }

    pub(crate) fn flag_set(&mut self, flag: MCICmdFlag) {
        self.flag = flag
    }

    pub(crate) fn flag_mut(&mut self) -> &mut MCICmdFlag {
        &mut self.flag
    }

    pub(crate) fn get_response(&self) -> &[u32] {
        self.response.as_ref()
    }

    pub(crate) fn get_mut_response(&mut self) -> &mut [u32] {
        self.response.as_mut()
    }

    pub(crate) fn get_mut_data(&mut self) -> Option<&mut MCIData> {
        self.data.as_mut()
    }

    pub(crate) fn get_data(&self) -> Option<&MCIData> {
        self.data.as_ref()
    }

    pub(crate) fn set_data(&mut self, data: Option<MCIData>) {
        self.data = data
    }
}
