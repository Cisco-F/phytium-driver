#![no_std]

extern crate alloc;

use core::{ptr::NonNull, time::Duration};

#[macro_use]
mod regs;
mod aarch;
pub mod iopad;
pub mod mci;
pub mod mci_host;
pub mod osa;
mod tools;

pub use iopad::*;
pub use mci_host::*;

pub trait Kernel {
    fn sleep(duration: Duration);
    fn mmap(virt_addr: NonNull<u8>) -> u64;
    fn flush(addr: NonNull<u8>, size: usize);
    fn invalidate(addr: core::ptr::NonNull<u8>, size: usize);
}

pub(crate) fn sleep(duration: Duration) {
    extern "Rust" {
        fn _phytium_mci_sleep(duration: Duration);
    }

    unsafe {
        _phytium_mci_sleep(duration);
    }
}

pub(crate) fn mmap(virt_addr: NonNull<u8>) -> u64 {
    extern "Rust" {
        fn _phytium_mci_map(virt_addr: NonNull<u8>) -> u64;
    }

    unsafe { _phytium_mci_map(virt_addr) }
}

pub(crate) fn flush(addr: NonNull<u8>, size: usize) {
    extern "Rust" {
        fn _phytium_mci_flush(addr: NonNull<u8>, size: usize);
    }

    unsafe {
        _phytium_mci_flush(addr, size);
    }
}

pub(crate) fn invalidate(addr: core::ptr::NonNull<u8>, size: usize) {
    extern "Rust" {
        fn _phytium_mci_invalidate(addr: core::ptr::NonNull<u8>, size: usize);
    }

    unsafe {
        _phytium_mci_invalidate(addr, size);
    }
}

#[macro_export]
macro_rules! set_impl {
    ($t: ty) => {
        #[no_mangle]
        unsafe fn _phytium_mci_sleep(duration: core::time::Duration) {
            <$t as $crate::Kernel>::sleep(duration)
        }
        #[no_mangle]
        fn _phytium_mci_map(addr: core::ptr::NonNull<u8>) -> u64 {
            <$t as $crate::Kernel>::mmap(addr)
        }
        #[no_mangle]
        fn _phytium_mci_flush(addr: core::ptr::NonNull<u8>, size: usize) {
            <$t as $crate::Kernel>::flush(addr, size)
        }
        #[no_mangle]
        fn _phytium_mci_invalidate(addr: core::ptr::NonNull<u8>, size: usize) {
            <$t as $crate::Kernel>::invalidate(addr, size)
        }
    };
}
