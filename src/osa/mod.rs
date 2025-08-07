use core::{
    alloc::Layout,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    time::Duration,
};

use alloc::{boxed::Box, sync::Arc};
use consts::{MAX_POOL_SIZE, SDMMC_OSA_EVENT_FLAG_AND};
use err::FMempError;
use lazy_static::*;
use rlsf::Tlsf;
use spin::Mutex;

use crate::{mci, sleep};

pub mod consts;
mod err;
pub mod pool_buffer;

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

    unsafe fn alloc_aligned(
        &mut self,
        size: usize,
        align: usize,
    ) -> Result<NonNull<u8>, FMempError> {
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
    unsafe {
        GLOBAL_FMEMP.lock().init();
    }
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
    unsafe {
        GLOBAL_FMEMP.lock().dealloc(addr, size);
    }
}

pub struct OSAEvent {
    event_flag: AtomicU32,
    notification: AtomicBool,
}

impl Default for OSAEvent {
    fn default() -> Self {
        Self::new()
    }
}

impl OSAEvent {
    pub const fn new() -> Self {
        Self {
            event_flag: AtomicU32::new(0),
            notification: AtomicBool::new(false),
        }
    }

    pub fn osa_event_set(&self, event_type: u32) {
        self.event_flag.fetch_or(event_type, Ordering::SeqCst);
        self.notification.store(true, Ordering::Release);
    }

    pub fn osa_event_wait(&self, event_type: u32, timeout_ticks: u32) -> Result<u32, &'static str> {
        let mut ticks = 0;

        loop {
            if self.notification.load(Ordering::Acquire) {
                let events = self.event_flag.load(Ordering::SeqCst);
                if events & event_type != 0 {
                    self.notification.store(false, Ordering::Release);
                    return Ok(events);
                }
            }

            if ticks >= timeout_ticks {
                return Err("timeout");
            }

            ticks += 1;
            #[cfg(feature = "pio")]
            sleep(Duration::from_millis(2));

            core::hint::spin_loop();
        }
    }

    pub fn osa_event_clear(&self, event_type: u32) {
        self.event_flag.fetch_and(!event_type, Ordering::SeqCst);

        if self.event_flag.load(Ordering::SeqCst) == 0 {
            self.notification.store(false, Ordering::Release);
        }
    }

    pub fn osa_event_get(&self) -> u32 {
        self.event_flag.load(Ordering::SeqCst)
    }
}

lazy_static! {
    static ref OSA_EVENT: OSAEvent = OSAEvent::new();
}

pub fn osa_event_set(event_type: u32) {
    OSA_EVENT.osa_event_set(event_type);
}

pub fn osa_event_wait(event_type: u32, timeout: u32) -> Result<(), &'static str> {
    OSA_EVENT.osa_event_wait(event_type, timeout).map(|_| ())
}

pub fn osa_event_get() -> u32 {
    OSA_EVENT.osa_event_get()
}

pub fn osa_event_clear(event_type: u32) {
    OSA_EVENT.osa_event_clear(event_type);
}
