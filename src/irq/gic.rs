use core::arch::asm;

use bare_test::irq::IrqRegister;
use log::{error, warn};

use crate::{aarch::*, irq::err::GicError, mci::consts::MCIId};

use super::{consts::*, err::GicStatus, reg::{gic_clear_bit, gic_read_32, gic_write_32}};

#[derive(Default)]
pub struct Gic {
    config: GicConfig,
    is_ready: bool,
    redis_base: u32,
    max_spi_num: i32,
    max_lpi_num: i32,
}

#[derive(Default)]
pub struct GicConfig {
    instance_id: MCIId,
    dis_base: u32,
    its_base: u32,
}

static mut ISR_TABLE: [Option<IrqRegister>]

// lazy_static!(
//     pub static ref INTERRUPT_HANDLER_PTR: Mutex<Gic> =
//         Mutex::new(Gic::new());
// )

impl Gic {
    /// 中断初始化，调用其他中断函数之前必须调用本函数
    /// Interrupt preinitialization function. 
    /// This function is usually called in assembly code. When the user sets the default 
    /// initialization mode, this function will use CORE0 as the main CORE and initialize all components 
    /// in the interrupt driver, and other cores as slave cores will initialize the necessary components in the interrupt driver.
    pub fn interrput_early_init() -> Self {
        let mut gic = Gic::default();

        if let Err(e) = gic.interrupt_init() {
            error!("interrput_early_init failed! err: {:?}", e);
            // todo we just panic here
            panic!();
        }

        gic
    }

    /// Initializes the interrupt module
    pub fn interrupt_init(&mut self) -> GicStatus {
        let config = GicConfig::lookup_config();
        let redis_base = match self.gic_redistributor_iterate() {
            Some(base) => base,
            None => {
                error!("gic_redistributor_iterate failed!");
                return Err(GicError::RedisGetError);
            },
        };
        self.cfg_init(config, redis_base);

        // todo init exceptions table

        self.redis_init()?;

        //todo giclpissetup

        cpu_interface_init();

        interrupt_prio_group_bits_set(IRQ_GROUP_PRIORITY_4 as u32);

        // todo need_translate
        // todo gicitsinit

        Ok(())
    }

    pub fn cfg_init(&mut self, config: GicConfig, redis_base: u32) {
        self.config = config;
        self.redis_base = redis_base;
        self.is_ready = true;

        let dis_base = self.config.dis_base;
        let mut max_ints_num = gic_read_32(dis_base, FGIC_GICD_TYPER_OFFSET);
        max_ints_num &= FGIC_GICD_TYPER_ITLINESNUMBER_MASK;
        max_ints_num = ((max_ints_num + 1) << 5) - 1;
        self.max_spi_num = max_ints_num as i32;
    }

    pub fn redis_init(&mut self) -> GicStatus {
        gic_clear_bit(self.redis_base, FGIC_GICR_WAKER_OFFSET, FGIC_GICR_WAKER_PROCESSOR_SLEEP_MASK);

        let mut timeout = 0;
        while gic_read_32(self.redis_base, FGIC_GICR_WAKER_OFFSET) & FGIC_GICR_WAKER_CHILDREN_ASLEEP_MASK != 0 {
            timeout += 1;
            if timeout >= 0xFFFFFFF {
                return Err(GicError::Timeout);
            }
        }

        if let Err(e) = self.wait_rwp(WaitRwqMode::GicrCtlrRwpWait) {
            return Err(e);
        }
        let sgi_base = self.redis_base + FGIC_GICR_SGI_BASE_OFFSET;
        gic_write_32(sgi_base, FGIC_GICR_ICENABLER0_OFFSET, FGIC_GICR_ICENABLER0_DEFAULT_MASK);

        // clear pending
        gic_write_32(sgi_base, FGIC_GICR_ICPENDR0_OFFSET, FGIC_GICR_ICPENDR0_DEFAULT_MASK);

        // Set sgi ppi route to different security group
        gic_write_32(sgi_base, FGIC_GICR_IGROUPR0_OFFSET, FGIC_GICR_IGROUPR0_DEFAULT_MASK);
        gic_write_32(sgi_base, FGIC_GICR_IGRPMODR0_OFFSET, FGIC_GICR_IGRPMODR0_DEFAULT_MASK);

        // 默认所有优先级为0xA0
        let mut int_id = 0;
        loop {
            let offset = ((int_id >> 2) << 2) + FGIC_GICR_IPRIORITYR_OFFSET;
            gic_write_32(sgi_base, offset, FGIC_INT_DEFAULT_PRI_X4);

            int_id += FGIC_GICD_4_PER_REG;
            if int_id >= FGIC_SPI_START_ID as u32 {
                break;
            }
        }

        // set level-sensitive
        gic_write_32(sgi_base, FGIC_GICR_ICFGR0_OFFSET, 0);
        gic_write_32(sgi_base, FGIC_GICR_ICFGR1_OFFSET, 0);

        Ok(())
    }

    pub fn wait_rwp(&self, mode: WaitRwqMode) -> GicStatus {
        let rwp_mask = if mode == WaitRwqMode::GicdCtlrRwpWait {
            FGIC_GICD_CTLR_RWP_MASK
        } else {
            FGIC_GICR_CTLR_RWP_MASK
        };

        let mut timeout = 0;
        while gic_read_32(self.redis_base, 0) & rwp_mask != 0 {
            timeout += 1;
            if timeout >= 0xFFFFFF {
                error!("wait rwp timeout!");
                return Err(GicError::Timeout);
            }
        }

        Ok(())
    }

    pub fn set_spi_affinity_routing(&mut self, int_id: i32, route_mode: SpiRoutingMode, affinity: u64) -> GicStatus {
        if int_id <= FGIC_PPI_END_ID {
            warn!("set_spi_affinity_routing failed! int_id {} is out of range", int_id);
            return Err(GicError::CtlrSetError);
        }

        set_spi_route()
    }

    pub fn set_spi_route(spi_id: u32, route_mode: GicdIrpiterMode, affinity: u64) {
        let bank = spi_id & FGIC_GICD_IROUTER_RANGE_LIMIT;
        unsafe {
            asm!("dsb sy", options(nostack, preserves_flags));
            
        }
    }
}

impl Gic {
    pub fn gic_redistributor_iterate(&self) -> Option<u32> {
        let mpidr_aff = unsafe { get_affinity() & 0xFFF };
        let mut redis_base = GICV3_RD_BASE_ADDR;
        loop {
            let gicr_typer_aff = gic_read_32(redis_base, FGIC_GICR_TYPER_H_OFFSET);
            if gicr_typer_aff as u64 == mpidr_aff {
                return Some(redis_base);
            }

            redis_base += GICV3_RD_OFFSET;
            if redis_base >= GICV3_RD_BASE_ADDR + GICV3_RD_SIZE {
                break;
            }
        }

        error!("mpidr_aff and gicr_typer_aff is not match");
        return None;
    }

    pub fn set_prio(&mut self, int_id: i32, prio: u32) -> GicStatus {
        // let int_id = match u32::try_from(int_id) {
        //     Ok(id) => id,
        //     Err(_) => {
        //         error!("int_id out of u32 range!");
        //         return Err(GicError::InvalidIntId);
        //     },
        // };

        if int_id <= FGIC_PPI_END_ID {
            self.set_private_prio(int_id as u32, prio)?;
        } else if int_id <= self.max_spi_num {
            self.set_spi_prio(int_id as u32, prio)?;
        } else if int_id as u32 >= FGIC_LPI_START_ID && int_id < self.max_lpi_num {
            // todo if def CONFIG_ENABLE_GIC_ITS
            error!("The GIC ITS module is not enabled");
            return Err(GicError::ItsDeviceNotExist);
        } else {
            error!("int_id is over max spi num for FGicSetPriority");
            return Err(GicError::CtlrNumError);
        }

        Ok(())
    }
}

/// private api
impl Gic {
    fn set_private_prio(&mut self, int_id: u32, prio: u32) -> GicStatus {
        let offset = (int_id >> 2) << 2 + FGIC_GICR_IPRIORITYR_OFFSET;
        let mut mask = gic_read_32(self.redis_base, offset);
        // todo 没有做isize到u32越界判断
        mask &= !fgic_gicr_ipriorityr_value_mask(int_id);
        mask |= (prio & 0xFF) << (int_id % 4) * 8;
        gic_write_32(self.redis_base, offset, mask);

        Ok(())
    }
    fn set_spi_prio(&mut self, int_id: u32, prio: u32) -> GicStatus {
        let offset = FGIC_GICD_IPRIORITYR_OFFSET + (int_id >> 2) << 2;
        let mut mask = gic_read_32(self.config.dis_base, offset);
        mask &= !fgic_gicd_ipriorityr_value_mask(int_id);
        mask |= (prio & 0xFF) << (int_id % 4) * 8;
        gic_write_32(self.config.dis_base, offset, mask);

        Ok(())
    }
}

impl GicConfig {
    pub fn lookup_config() -> Self {
        Self {
            instance_id: MCIId::MCI0,
            dis_base: GICV3_DISTRIBUTOR_BASE_ADDR,
            its_base: GICV3_ITS_BASE_ADDR,
        }
    }
}

/// Initialize Cpu interface of current core
pub fn cpu_interface_init() {
    unsafe {
        // let mut reg = aarch64_read_gic_sysreg!("ICC_SRE_EL1");
        let mut reg = read_icc_sre_el1();
        
        if reg & GICC_SRE_SRE as u64 == 0 {
            reg |= (GICC_SRE_SRE | GICC_SRE_DFB | GICC_SRE_DIB) as u64;
            write_icc_sre_el1(reg); 
            reg = read_icc_sre_el1();
            if reg & GICC_SRE_SRE as u64 == 0 {
                error!("set ICC_SRE_EL1 failed!");
                panic!();
            }
        }

        write_icc_pmr(FGIC_CPU_INTERFACE_DEFAULT_FILTER as u64);
        isb();
        write_icc_igrpen1_el3(1u64);
        isb();
        write_icc_ctlr_el1(GICC_CTLR_CBPR as u64);
        isb();
    }
}

pub fn interrupt_prio_group_bits_set(bits: u32) {
    unsafe {
        write_icc_bpr1_el1(bits as u64);
    }
}
