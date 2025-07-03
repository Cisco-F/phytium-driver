//! A managed memory buffer for aligned allocations.
//!
//! Provides [`PoolBuffer`] - a safe wrapper around pooled memory
use core::{ptr::{copy_nonoverlapping, write_bytes, NonNull}, slice::{from_raw_parts, from_raw_parts_mut}};

use alloc::vec::Vec;
use log::error;

use super::{err::FMempError, osa_alloc_aligned, osa_dealloc};

/// PoolBuffer definition
pub struct PoolBuffer {
    size: usize,
    addr: NonNull<u8>,
    align: usize
}

impl PoolBuffer {
    /// Alloc a PoolBuffer, where size is buffer size in bytes
    pub fn new(size: usize, align: usize) -> Result<Self, &'static str> {
        let ptr = match osa_alloc_aligned(size, align) {
            Err(_) => return Err("osa alloc failed!"),
            Ok(ptr) => ptr,
        };
        Ok(Self {
            size,
            addr: ptr,
            align
        })
    }

    /// Construct from &[T]
    pub fn copy_from_slice<T: Copy>(&mut self, src: &[T]) -> Result<(), &'static str> {
        let len = src.len() * size_of::<T>();
        if self.size < len {
            return Err("Too small to receive data!");
        }

        unsafe {
            // equivalent to memcpy in C
            copy_nonoverlapping(
                src.as_ptr() as *mut u8,
                self.addr.as_ptr(),
                len
            );
        }

        Ok(())
    }

    /// Construct a &[T] from self
    pub fn as_slice<T>(&self) -> Result<&[T], FMempError> {
        let size = size_of::<T>();
        if self.size() % size != 0 {
            return Err(FMempError::SizeNotAligned);
        }

        unsafe {
            let result = from_raw_parts(self.addr.as_ptr() as *const T, self.size() / size);
            Ok(result)
        }
    }

    pub fn as_slice_in_len<T>(&self, len: usize) -> Result<&[T], FMempError> {
        if len * size_of::<T>() > self.size {
            error!("Acquiring length to big for this PoolBuffer");
            return Err(FMempError::NotEnoughSpace);
        }

        unsafe {
            let result = from_raw_parts(self.addr.as_ptr() as *const T, len);
            Ok(result)
        }
    }

    /// Construct a &mut [T] from self
    pub fn as_slice_mut<T>(&self) -> Result<&[T], FMempError> {
        let size = size_of::<T>();
        if self.size() % size != 0 {
            return Err(FMempError::SizeNotAligned);
        }

        unsafe {
            let result = from_raw_parts_mut(self.addr.as_ptr() as *mut T, self.size() / size);
            Ok(result)
        }
    }

    /// Construct a Vec<u32> from self
    pub fn to_vec<T: Clone>(&self) -> Result<Vec<T>, FMempError> {
        let slice = self.as_slice::<T>()?;
        Ok(slice.to_vec())
    }

    pub fn to_vec_in_len<T: Clone>(&self, len: usize) -> Result<Vec<T>, FMempError> {
        let slice = self.as_slice_in_len::<T>(len)?;
        Ok(slice.to_vec())
    }

    /// Clear buffer, leaving 0s at original places
    pub fn clear(&mut self) {
        unsafe { write_bytes(self.addr.as_ptr(), 0, self.size); }
    }

    /// Get size
    pub fn size(&self) -> usize {
        self.size
    }

    /// Get addr
    pub fn addr(&self) -> NonNull<u8> {
        self.addr.clone()
    }
}

impl Drop for PoolBuffer {
    fn drop(&mut self) {
        osa_dealloc(self.addr, self.align);
    }
}

impl Into<Vec<u32>> for PoolBuffer {
    fn into(self) -> Vec<u32> {
        unsafe {
           let slice = from_raw_parts(self.addr.as_ptr() as *const u32, self.size / 4);
           slice.to_vec()
        }
    }
}