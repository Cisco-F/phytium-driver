use crate::regs::RegError;

#[derive(Debug, Clone, Copy)]
pub enum GicError {
    CtlrTypeError,
    CtlrNumError,
    CtlrSetError,
    CtlrGetError,
    RedisGetError,
    Timeout,
    ItsQuietCendError,
    ItsGetCmdError,
    ItsDeviceNotExist,

    InvalidIntId,
}

impl RegError for GicError {
    fn timeout() -> Self {
        GicError::Timeout
    }
}

pub type GicStatus = Result<(), GicError>;