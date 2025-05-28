use core::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use consts::{MAX_POOL_SIZE, OSA_SEM_HANDLE_SIZE, SDMMC_OSA_EVENT_FLAG_AND};
use err::FMempError;
use pool_buffer::PoolBuffer;
use rlsf::Tlsf;
use spin::Mutex;
use lazy_static::*;
use no_std_async::Condvar;

use crate::aarch::dsb;

mod err;
pub mod consts;
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
    pub static ref GLOBAL_FMEMP: Mutex<FMemp<'static>> = 
        Mutex::new(FMemp::new());
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

    unsafe fn alloc_aligned(&mut self, size: usize, align: usize) -> Result<PoolBuffer, FMempError> {
        let layout = Layout::from_size_align_unchecked(size, align);
        if let Some(result) = self.tlsf_ptr.allocate(layout) {
            let buffer = PoolBuffer::new(size, result);
            Ok(buffer)
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
pub fn osa_alloc(size: usize) -> Result<PoolBuffer, FMempError> {
    unsafe { GLOBAL_FMEMP.lock().alloc_aligned(size, size_of::<usize>()) }
}

/// Alloc 'size' bytes space, aligned to 'align' bytes
pub fn osa_alloc_aligned(size: usize, align: usize) -> Result<PoolBuffer, FMempError> {
    unsafe { GLOBAL_FMEMP.lock().alloc_aligned(size, align) }
}

/// Dealloc 'size' bytes space from 'addr'
pub fn osa_dealloc(addr: NonNull<u8>, size: usize) {
    unsafe { GLOBAL_FMEMP.lock().dealloc(addr, size); }
}

pub struct OSAEvent {
    mutex: Mutex<EventState>,
    condvar: Condvar,
}

#[derive(Default)]
pub struct EventState {
    event_flag: u32,
}

impl OSAEvent {
    pub fn default() -> Self {
        Self {
            mutex: Mutex::new(EventState::default()),
            condvar: Condvar::new(),
        }
    }
    pub fn osa_event_set(&mut self, event_type: u32) -> Result<(), &'static str> {
        let mut state = self.mutex.lock();
        state.event_flag |= event_type;
        self.condvar.notify_all();
        Ok(())
    }
    pub fn osa_event_wait(&self, event_type: u32, _timeout_ms: u32, event: &mut u32, _flags: u32) -> Result<(), &'static str> {
        self.condvar.wait();
        *event = self.osa_event_get();
        if *event & SDMMC_OSA_EVENT_FLAG_AND != 0 {
            if *event == event_type {
                return Ok(());
            }
        } else {
            if *event & event_type != 0 {
                return Ok(())
            }
        }

        Err("event wait failed")
    }
    pub fn osa_event_get(&self) -> u32 {
        self.mutex.lock().event_flag
    }
    pub fn osa_event_clear(&mut self, event_type: u32) {
        let mut state = self.mutex.lock();
        state.event_flag &= !event_type;
    }
}

// pub fn osa_event_set(event_handle: &mut OSAEvent, event_type: u32) {
//     // let osa_current_sr = 0;
//     // event_handle.event_flag |= event_type;
//     // unsafe { dsb(); }
    
// }

// pub fn osa_event_wait()