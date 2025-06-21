//! A managed memory buffer for aligned allocations.
//!
//! Provides [`PoolBuffer`] - a safe wrapper around pooled memory
use core::{ptr::{copy_nonoverlapping, write_bytes, NonNull}, slice::{from_raw_parts, from_raw_parts_mut}};

use alloc::vec::Vec;

use super::{err::FMempError, osa_dealloc};

/// PoolBuffer definition
pub struct PoolBuffer {
    size: usize,
    addr: NonNull<u8>,
    align: usize
}

impl PoolBuffer {
    /// Construct a PoolBuffer
    pub fn new(size: usize, addr: NonNull<u8>, align: usize) -> Self {
        Self {
            size,
            addr,
            align
        }
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