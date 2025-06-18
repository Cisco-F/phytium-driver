#![allow(dead_code)]
use core::arch::asm;

#[inline(always)]
pub unsafe fn dsb() {
    core::arch::asm!("dsb sy");
    core::arch::asm!("isb sy");
}

#[inline(always)]
pub unsafe fn isb() {
    core::arch::asm!("isb", options(nostack, preserves_flags));
}

// #[inline(always)]
// pub unsafe fn flush(addr: *const u8, size: usize) {
//     let mut addr = addr as usize;
//     let end = addr + size;
//     while addr < end {
//         asm!("dc civac, {0}", in(reg) addr, options(nostack, preserves_flags));
//         addr += 64;
//     }
//     dsb();
// }

// #[inline(always)]
// pub unsafe fn invalidate(addr: *const u8, size: usize) {
//     let mut addr = addr as usize;
//     let end = addr + size;
//     while addr < end {
//         asm!("dc ivac, {0}", in(reg) addr, options(nostack, preserves_flags));
//         addr += core::mem::size_of::<u32>();
//     }
//     asm!("dsb sy");
// }

#[inline(always)]
pub unsafe fn get_affinity() -> u64 {
    let mpidr: u64;
    asm!("mrs {0}, MPIDR_EL1", out(reg) mpidr);
    mpidr
}

macro_rules! define_read_gic {
    ($fn_name:ident, $reg_name:literal) => {
        #[inline(always)]
        pub unsafe fn $fn_name() -> u64 {
            let val: u64;
            asm!(concat!("mrs {0}, ", $reg_name), out(reg) val, options(nostack, preserves_flags));
            val
        }
    };
}

macro_rules! define_write_gic {
    ($fn_name:ident, $reg_name:literal) => {
        #[inline(always)]
        pub unsafe fn $fn_name(val: u64) {
            asm!(concat!("msr ", $reg_name, ", {0}"), in(reg) val, options(nostack, preserves_flags));
        }
    };
}

define_read_gic!(read_icc_sre_el1, "ICC_SRE_EL1");
define_read_gic!(read_icc_sre_el2, "ICC_SRE_EL2");
define_read_gic!(read_icc_sre_el3, "ICC_SRE_EL3");
define_read_gic!(read_icc_pmr, "ICC_PMR_EL1");
define_read_gic!(read_icc_rpr, "ICC_RPR_EL1");

define_write_gic!(write_icc_sre_el1, "ICC_SRE_EL1");
define_write_gic!(write_icc_sre_el2, "ICC_SRE_EL2");
define_write_gic!(write_icc_sre_el3, "ICC_SRE_EL3");
define_write_gic!(write_icc_pmr, "ICC_PMR_EL1");
define_write_gic!(write_icc_rpr, "ICC_RPR_EL1");
define_write_gic!(write_icc_igrpen1_el3, "ICC_IGRPEN1_EL3");
define_write_gic!(write_icc_ctlr_el1, "ICC_CTLR");
define_write_gic!(write_icc_bpr1_el1, "ICC_BPR1_EL1");



// // #[inline(always)]
// // pub unsafe fn aarch64_read_gic_sysreg(reg: &str) -> u64 {
// //     let mut val: u64;

// //     match reg {
// //         "ICC_SRE_EL1" => {
// //             asm!(
// //                 "mrs {val}, S3_0_C12_C12_5",
// //                 val = out(reg) val,
// //                 options(nostack, preserves_flags)
// //             );
// //         }
// //         "ICC_SRE_EL2" => {
// //             asm!(
// //                 "mrs {val}, S3_4_C12_C9_5",
// //                 val = out(reg) val,
// //                 options(nostack, preserves_flags)
// //             );
// //         }
// //         "ICC_SRE_EL3" => {
// //             asm!(
// //                 "mrs {val}, S3_6_C12_C12_5",
// //                 val = out(reg) val,
// //                 options(nostack, preserves_flags)
// //             );
// //         }
// //         _ => panic!("Unsupported register: {}", reg),
// //     }

// //     val
// // }

// #[inline(always)]
// pub unsafe fn aarch64_write_gic_sysreg(reg: &str, bits: u64) {
//     match reg {
//         "ICC_SRE_EL1" => {
//             asm!(
//                 "msr S3_0_C12_C12_5, {bits}",
//                 bits = in(reg) bits,
//                 options(nostack, preserves_flags)
//             );
//         }
//         "ICC_SRE_EL2" => {
//             asm!(
//                 "msr S3_4_C12_C9_5, {bits}",
//                 bits = in(reg) bits,
//                 options(nostack, preserves_flags)
//             );
//         }
//         "ICC_SRE_EL3" => {
//             asm!(
//                 "msr S3_6_C12_C12_5, {bits}",
//                 bits = in(reg) bits,
//                 options(nostack, preserves_flags)
//             );
//         }
//         _ => panic!("Unsupported register: {}", reg),
//     }
// }