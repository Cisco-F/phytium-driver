use crate::regs::RegError;

#[derive(Debug)]
pub enum FioPadError {
    InvalParam,
    NotReady,
    NotNotSupport,
    Timeout,
}

impl RegError for FioPadError {
    fn timeout() -> Self {
        FioPadError::Timeout
    }
}

pub type FioPadResult<T = ()> = Result<T, FioPadError>;
