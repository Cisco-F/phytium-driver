use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull, sync::atomic::{AtomicU32, Ordering}};

use alloc::{boxed::Box, sync::Arc};
use consts::{MAX_POOL_SIZE, SDMMC_OSA_EVENT_FLAG_AND};
use err::FMempError;
use log::{error, info};
use rlsf::Tlsf;
use semaphore::Semaphore;
use spin::Mutex;
use lazy_static::*;

mod err;
pub mod consts;
pub mod pool_buffer;
pub mod semaphore;

/// Memory menaged by Tlsf pool
static mut POOL: [MaybeUninit<u8>; MAX_POOL_SIZE] = [MaybeUninit::uninit(); MAX_POOL_SIZE];

/// Tlsf controller
pub struct FMemp<'a> {
    tlsf_ptr: Tlsf<'a, u32, u32, 32, 32>,
    is_ready: bool,
}

lazy_static! {
    /// Global memory pool manager
    pub static ref GLOBAL_FMEMP: Mutex<Box<FMemp<'static>>> = 
        Mutex::new(Box::new(FMemp::new()));
}

impl<'a> FMemp<'a> {
    /// Constructor
    pub fn new() -> Self {
        Self {
            tlsf_ptr: Tlsf::new(),
            is_ready: false,
        }
    }

    unsafe fn init(&mut self) {
        self.tlsf_ptr.insert_free_block(&mut POOL[..]);
        self.is_ready = true;
    }

    unsafe fn alloc_aligned(&mut self, size: usize, align: usize) -> Result<NonNull<u8>, FMempError> {
        let layout = Layout::from_size_align_unchecked(size, align);
        if let Some(result) = self.tlsf_ptr.allocate(layout) {
            Ok(result)
        } else {
            Err(FMempError::BadMalloc)
        }
    }

    unsafe fn dealloc(&mut self, addr: NonNull<u8>, size: usize) {
        self.tlsf_ptr.deallocate(addr, size);
    }
}  

/// Init memory pool with size of ['MAX_POOL_SIZE']
pub fn osa_init() {
    unsafe { GLOBAL_FMEMP.lock().init(); }
}

/// Alloc 'size' bytes space, aligned to 64 KiB by default
pub fn osa_alloc(size: usize) -> Result<NonNull<u8>, FMempError> {
    unsafe { GLOBAL_FMEMP.lock().alloc_aligned(size, size_of::<usize>()) }
}

/// Alloc 'size' bytes space, aligned to 'align' bytes
pub fn osa_alloc_aligned(size: usize, align: usize) -> Result<NonNull<u8>, FMempError> {
    unsafe { GLOBAL_FMEMP.lock().alloc_aligned(size, align) }
}

/// Dealloc 'size' bytes space from 'addr'
pub fn osa_dealloc(addr: NonNull<u8>, size: usize) {
    unsafe { GLOBAL_FMEMP.lock().dealloc(addr, size); }
}

pub struct OSAEvent {
    event_flag: AtomicU32,
    handle: Semaphore,
}

impl OSAEvent {
    pub fn default() -> Self {
        Self {
            event_flag: AtomicU32::new(0),
            handle: Semaphore::new(0),
        }
    }
    pub fn osa_event_set(&self, event_type: u32) {
        self.event_flag.fetch_or(event_type, Ordering::SeqCst);
        self.handle.up();
    }
    pub fn osa_event_wait(&self, event_type: u32, _timeout_ms: u32, event: &mut u32, flags: u32) -> Result<(), &'static str> {
        info!("waiting event");

        self.handle.down();
        *event = self.osa_event_get();
        if flags & SDMMC_OSA_EVENT_FLAG_AND != 0 {
            if *event == event_type {
                return Ok(());
            }
        } else {
            if *event & event_type != 0 {
                return Ok(());
            }
        }

        error!("event wait failed");
        Err("event wait failed")
    }
    pub fn osa_event_get(&self) -> u32 {
        self.event_flag.load(Ordering::SeqCst)
    }
    pub fn osa_event_clear(&self, event_type: u32) {
        self.event_flag.fetch_and(!event_type, Ordering::SeqCst);
    }
}

lazy_static! {
    /// Global event handler
    pub static ref OSA_EVENT: Arc<OSAEvent> = 
        Arc::new(OSAEvent::default());
}

pub fn osa_event_set(event_type: u32) {
    OSA_EVENT.osa_event_set(event_type);
}

pub fn osa_event_wait(event_type: u32, _timeout_ms: u32, event: &mut u32, flags: u32) -> Result<(), &'static str> {
    OSA_EVENT.osa_event_wait(event_type, _timeout_ms, event, flags)
}

pub fn osa_event_get() -> u32 {
    OSA_EVENT.osa_event_get()
}

pub fn osa_event_clear(event_type: u32) {
    OSA_EVENT.osa_event_clear(event_type);
}