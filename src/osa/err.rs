#[derive(Debug)]
pub enum FMempError {
    InvalidBuf,
    InitTlsfError,
    BadMalloc,
    // PoolBuffer related errors
    NotEnoughSpace, // PoolBuffer size too small to copy contents from a slice
    SizeNotAligned, // PoolBuffer size isn't aligned to size::T
}

#[allow(unused)]
pub type FMempStatus<T = ()> = Result<T, FMempError>;
