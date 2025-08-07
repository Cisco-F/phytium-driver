#![no_std]
#![no_main]
#![feature(used_with_arg)]
#![feature(stdarch_arm_barrier)]

extern crate alloc;

#[bare_test::tests]
mod tests {
    use core::{
        arch::{
            aarch64::{__dsb, __isb, SY},
            asm,
        },
        ptr::NonNull,
        time::Duration,
    };

    use alloc::vec::Vec;
    use bare_test::{
        globals::{global_val, PlatformInfoKind},
        irq::{IrqHandleResult, IrqParam},
        mem::{mmu::iomap, PhysAddr, VirtAddr},
        platform_if::CacheOp,
        time::spin_delay,
        GetIrqConfig,
    };
    use log::*;
    use phytium_mci::{
        mci::{
            fsdif_interrupt_handler,
            regs::{MCICtrl, MCIDMACStatus, MCIIntMask, MCIRawInts, MCIReg},
        },
        sd::{init_reg_base, SdCard},
        set_impl, IoPad, Kernel, PAD_ADDRESS,
    };

    const SD_START_BLOCK: u32 = 131072;
    const SD_USE_BLOCK: u32 = 4;
    const SD_BLOCK_SIZE: u32 = 512;
    const SD_MAX_RW_BLK: u32 = 1024;

    #[test]
    fn test_work() {
        // if cfg!(feature = "irq") {
        //     compile_error!("feature irq isn't finished yet!");
        // }

        let fdt = match &global_val().platform_info {
            PlatformInfoKind::DeviceTree(fdt) => fdt.get(),
            // _ => panic!("unsupported platform"),
        };

        let mci0 = fdt.find_compatible(&["phytium,mci"]).next().unwrap();
        let reg = mci0.reg().unwrap().next().unwrap();
        info!(
            "mci0 reg: {:#x},mci0 reg size: {:#x}",
            reg.address,
            reg.size.unwrap()
        );
        let mci_reg_base = iomap((reg.address as usize).into(), reg.size.unwrap());
        clear_pending_irq(mci_reg_base);

        // 一定要初始化，不然注册中断处理函数会报错
        init_reg_base(mci_reg_base);

        let iopad_reg_base = iomap((PAD_ADDRESS as usize).into(), 0x2000);
        let iopad = IoPad::new(iopad_reg_base);

        if cfg!(feature = "irq") {
            let irq_info = mci0.irq_info().unwrap();
            IrqParam {
                intc: irq_info.irq_parent,
                cfg: irq_info.cfgs[0].clone(),
            }
            .register_builder(|_irq_num| {
                fsdif_interrupt_handler();
                IrqHandleResult::Handled
            })
            .register();
            info!(
                "registered irq {:?} for {:?}, irq_parent: {:?}, trigger: {:?}",
                irq_info.cfgs[0].irq,
                mci0.name(),
                irq_info.irq_parent,
                irq_info.cfgs[0].trigger
            );
        }

        let mut sdcard = SdCard::new(mci_reg_base, iopad);
        if let Err(err) = sdcard.init(mci_reg_base) {
            error!("Sd Card Init Fail, error = {:?}", err);
            panic!();
        }

        ////////////////////// SD card init finished //////////////////////

        // 初始化write buffer
        let mut buffer: Vec<u32> = Vec::with_capacity((SD_BLOCK_SIZE * SD_MAX_RW_BLK / 4) as usize);
        buffer.resize((SD_BLOCK_SIZE * SD_MAX_RW_BLK / 4) as usize, 0);
        for i in 0..buffer.len() {
            buffer[i] = i as u32;
        }

        sdcard
            .write_blocks(&mut buffer, SD_START_BLOCK, SD_USE_BLOCK)
            .unwrap();

        let mut receive_buf = Vec::new();

        sdcard
            .read_blocks(&mut receive_buf, SD_START_BLOCK, SD_USE_BLOCK)
            .unwrap();

        for i in 0..receive_buf.len() {
            assert_eq!(receive_buf[i], buffer[i]);
        }
        // for i in 0..receive_buf.len() {
        //     warn!("{:x},{:x},{:x},{:x}",
        //     receive_buf[i] as u8,
        //     (receive_buf[i] >> 8) as u8,
        //     (receive_buf[i] >> 16) as u8,
        //     (receive_buf[i] >> 24) as u8);
        // }
        info!("receive buffer len is {}", receive_buf.len());

        info!("test_work passed\n");
    }

    fn sleep(duration: Duration) {
        spin_delay(duration);
    }

    fn clear_pending_irq(reg_base: NonNull<u8>) {
        let reg = MCIReg::new(reg_base);
        let raw_ints = reg.read_reg::<MCIRawInts>();
        let dmac_status = reg.read_reg::<MCIDMACStatus>();
        info!("before SD card init, clear pending irq!");
        info!(
            "int_mask 0x{:x}, ctrl 0x{:x}, raw_ints 0x{:x}, dmac_status 0x{:x}",
            reg.read_reg::<MCIIntMask>(),
            reg.read_reg::<MCICtrl>(),
            reg.read_reg::<MCIRawInts>(),
            reg.read_reg::<MCIDMACStatus>()
        );
        reg.write_reg(raw_ints);
        reg.write_reg(dmac_status);
        drop(reg);
    }

    #[inline(always)]
    fn cache_line_size() -> usize {
        unsafe {
            let mut ctr_el0: u64;
            asm!("mrs {}, ctr_el0", out(reg) ctr_el0);
            let log2_cache_line_size = ((ctr_el0 >> 16) & 0xF) as usize;
            // Calculate the cache line size
            4 << log2_cache_line_size
        }
    }

    #[inline(always)]
    fn _dcache_line(op: CacheOp, addr: usize) {
        unsafe {
            match op {
                CacheOp::Invalidate => asm!("dc ivac, {0}", in(reg) addr),
                CacheOp::Clean => asm!("dc cvac, {0}", in(reg) addr),
                CacheOp::CleanAndInvalidate => asm!("dc civac, {0}", in(reg) addr),
            }
        }
    }

    #[inline(always)]
    fn dcache_range(op: CacheOp, addr: usize, size: usize) {
        let start = addr;
        let end = start + size;
        let cache_line_size = cache_line_size();

        let mut aligned_addr = addr & !(cache_line_size - 1);

        while aligned_addr < end {
            _dcache_line(op, aligned_addr);
            aligned_addr += cache_line_size;
        }

        unsafe {
            __dsb(SY);
            __isb(SY);
        }
    }

    struct KernelImpl;

    impl Kernel for KernelImpl {
        fn sleep(duration: Duration) {
            sleep(duration);
        }
        fn mmap(virt_addr: NonNull<u8>) -> u64 {
            let vaddr = VirtAddr::from(virt_addr);
            let paddr = PhysAddr::from(vaddr);
            debug!(
                "do mmap, va: {:x}, pa {:x}",
                virt_addr.as_ptr() as usize,
                paddr.as_usize()
            );
            paddr.as_usize() as _
        }
        fn flush(addr: NonNull<u8>, size: usize) {
            dcache_range(CacheOp::Clean, addr.as_ptr() as _, size);
        }
        fn invalidate(addr: core::ptr::NonNull<u8>, size: usize) {
            dcache_range(CacheOp::Invalidate, addr.as_ptr() as _, size);
        }
    }

    set_impl!(KernelImpl);
}
