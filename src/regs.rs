#![allow(unused)]

use crate::sleep;
use bitflags::{bitflags, Flags};
use core::{marker::PhantomData, ops, ptr::NonNull, time::Duration};
use log::info;

/*
 * 为所有的 bitflag! 实现一个 BitsOps trait
 * 方便后续为所有的 bitflag! 实现一些通用的操作
 * 原理是所有的 bitflag! 都是一个结构体，而结构体都是实现了 ops::BitOr 等操作的
 * 这时候为实现了 ops::BitOr 的结构体实现一个 BitsOps trait
 * 这样所有的 bitflag! 都可以识别为实现了 BitsOps trait
*/
pub trait BitsOps:
    ops::BitOr<Output = Self>
    + ops::BitAnd<Output = Self>
    + ops::Not<Output = Self>
    + ops::BitXor<Output = Self>
    + Sized
{
}
impl<T> BitsOps for T where
    T: ops::BitOr<Output = Self>
        + ops::BitAnd<Output = Self>
        + ops::Not<Output = Self>
        + ops::BitXor<Output = Self>
{
}

/*
 * Create a contiguous bitmask starting at bit position @l and ending at
 * position @h. For example
 * GENMASK_ULL(39, 21) gives us the 64bit vector 0x000000ffffe00000.
 */
#[macro_export]
macro_rules! genmask {
    ($h:expr, $l:expr) => {
        (((!0u32) - (1u32 << $l) + 1) & ((!0u32) >> (32 - 1 - $h)))
    };
}

#[macro_export]
macro_rules! genmask_ull {
    ($h:expr, $l:expr) => {
        (((!0u64) - (1u64 << $l) + 1) & ((!0u64) >> (64 - 1 - $h)))
    };
}

/* set 32-bit register [a:b] as x, where a is high bit, b is low bit, x is setting/getting value */
#[macro_export]
macro_rules! get_reg32_bits {
    ($reg:expr, $a:expr, $b:expr) => {
        ($reg & genmask!($a, $b)) >> $b
    };
}

#[macro_export]
macro_rules! set_reg32_bits {
    ($reg:expr, $a:expr, $b:expr) => {
        (($reg << $b) & genmask!($a, $b))
    };
}

#[derive(Debug)]
pub struct Reg<E: RegError> {
    pub addr: NonNull<u8>,
    _marker: PhantomData<E>,
}

impl<E: RegError> Reg<E> {
    pub fn new(addr: NonNull<u8>) -> Self {
        Self {
            addr,
            _marker: PhantomData,
        }
    }

    pub fn read_32(&self, reg: u32) -> u32 {
        unsafe {
            let ptr = self.addr.add(reg as _);
            ptr.cast().read_volatile()
        }
    }

    pub fn write_32(&self, reg: u32, val: u32) {
        unsafe {
            let ptr = self.addr.add(reg as _);
            ptr.cast().write_volatile(val);
        }
    }

    pub fn read_reg<F: FlagReg>(&self) -> F {
        F::from_bits_retain(self.read_32(F::REG))
    }

    pub fn write_reg<F: FlagReg>(&self, val: F) {
        self.write_32(F::REG, val.bits())
    }

    pub fn modify_reg<F: FlagReg>(&self, f: impl Fn(F) -> F) {
        let old = self.read_reg::<F>();
        self.write_reg(f(old));
    }

    pub fn clear_reg<F: FlagReg + Copy + BitsOps>(&self, val: F) {
        self.modify_reg(|old| !val & old)
    }

    pub fn set_reg<F: FlagReg + Copy + BitsOps>(&self, val: F) {
        self.modify_reg(|old| val | old)
    }

    pub fn get_base_addr(&self) -> NonNull<u8> {
        self.addr
    }

    pub fn wait_for<R: FlagReg, F: Fn(R) -> bool>(
        &self,
        f: F,
        interval: Duration,
        try_count: Option<usize>,
    ) -> Result<(), E> {
        for _ in 0..try_count.unwrap_or(usize::MAX) {
            if f(self.read_reg::<R>()) {
                return Ok(());
            }

            sleep(interval);
        }
        Err(E::timeout())
    }

    pub fn retry_for<R: FlagReg, F: Fn(R) -> bool>(
        &self,
        f: F,
        try_count: Option<usize>,
    ) -> Result<(), E> {
        for _ in 0..try_count.unwrap_or(usize::MAX) {
            if f(self.read_reg::<R>()) {
                return Ok(());
            }
        }
        Err(E::timeout())
    }
}

impl<E: RegError> PartialEq for Reg<E> {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

pub trait RegError {
    fn timeout() -> Self;
}

pub trait FlagReg: Flags<Bits = u32> {
    const REG: u32;
}

#[macro_export]
macro_rules! BitsOpsForU32 {
    ($name:ident) => {
        impl ops::BitOr<u32> for $name {
            type Output = Self;
            fn bitor(self, rhs: u32) -> Self {
                self | Self::from_bits_truncate(rhs)
            }
        }
        impl ops::BitAnd<u32> for $name {
            type Output = Self;
            fn bitand(self, rhs: u32) -> Self {
                self & Self::from_bits_truncate(rhs)
            }
        }
        impl From<u32> for $name {
            fn from(val: u32) -> Self {
                Self::from_bits_truncate(val)
            }
        }
    };
}
