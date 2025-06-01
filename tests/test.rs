#![no_std]
#![no_main]
#![feature(used_with_arg)]

extern crate alloc;

#[bare_test::tests]
mod tests {
    use core::time::Duration;

    use alloc::vec::Vec;
    use bare_test::{
        globals::{global_val, PlatformInfoKind},
        mem::mmu::iomap,
        time::spin_delay, GetIrqConfig,
    };
    use log::*;
    use phytium_mci::{iopad::PAD_ADDRESS, sd::SdCard, *};

    const SD_START_BLOCK: u32 = 131072;
    const SD_USE_BLOCK: u32 = 10;
    const SD_BLOCK_SIZE: u32 = 512;
    const SD_MAX_RW_BLK: u32 = 1024;

    #[test]
    fn test_work() {
        let fdt = match &global_val().platform_info {
            PlatformInfoKind::DeviceTree(fdt) => fdt.get(),
            // _ => panic!("unsupported platform"),
        };

        let mci0 = fdt.find_compatible(&["phytium,mci"]).next().unwrap();
        let intr = mci0.interrupts().unwrap().next().unwrap().next().unwrap();
        let prop = mci0.propertys().next().unwrap();
        let name = prop.name;
        info!("intr {}", intr);
        info!("propety name {}", name);

        let info= mci0.irq_info().unwrap();
        info!("irq parent {:?}", info.irq_parent);
        
        let parent = mci0.interrupt_parent().unwrap().node;
        let p_name = parent.name;
        let a = parent.interrupts().unwrap().next().unwrap().next().unwrap();
        info!("p name {}", p_name);
        info!("p intr {}", a);

        let reg = mci0.reg().unwrap().next().unwrap();
        let p_reg = parent.reg().unwrap().next().unwrap();

        info!(
            "mci0 reg: {:#x},mci0 reg size: {:#x}",
            reg.address,
            reg.size.unwrap()
        );
        info!(
            "parent reg: {:#x}, reg size: {:#x}",
            p_reg.address,
            p_reg.size.unwrap()
        );

        let mci_reg_base = iomap((reg.address as usize).into(), reg.size.unwrap());

        let iopad_reg_base = iomap((PAD_ADDRESS as usize).into(), 0x2000);

        let iopad = IoPad::new(iopad_reg_base);
    
        let mut sdcard = SdCard::new(mci_reg_base,iopad);

        ////////////////////// SD card init finished //////////////////////

        // 初始化write buffer
        let mut buffer: Vec<u32> = Vec::with_capacity((SD_BLOCK_SIZE * SD_MAX_RW_BLK / 4) as usize);
        buffer.resize((SD_BLOCK_SIZE * SD_MAX_RW_BLK / 4) as usize, 0);
        for i in 0..buffer.len() {
            buffer[i] = i as u32;
        }

        sdcard.write_blocks(&mut buffer, SD_START_BLOCK, SD_USE_BLOCK).unwrap();

        let mut receive_buf = Vec::new();

        sdcard.read_blocks(&mut receive_buf, SD_START_BLOCK, SD_USE_BLOCK).unwrap();

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

    struct KernelImpl;

    impl Kernel for KernelImpl {
        fn sleep(duration: Duration) {
            sleep(duration);
        }
    }

    set_impl!(KernelImpl);
}
