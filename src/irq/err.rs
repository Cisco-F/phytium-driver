use crate::regs::RegError;

pub enum GicError {
    CtlrTypeError,
    CtlrNumError,
    CtlrSetError,
    CtlrGetError,
    Timeout,
    ItsQuietCendError,
    ItsGetCmdError,

}

impl RegError for GicError {
    fn timeout() -> Self {
        GicError::Timeout
    }
}

pub type GicStatus = Result<(), GicError>;