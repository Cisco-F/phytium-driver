use core::arch::asm;

use log::warn;

use crate::{irq::err::GicError, mci::consts::MCIId};

use super::{consts::*, err::GicStatus, reg::GicReg};

struct Gic {
    config: GicConfig,
    is_ready: bool,
    gic_reg: GicReg,
}

struct GicConfig {
    mci_id: MCIId,

}

impl Gic {
    pub fn set_spi_affinity_routing(&mut self, int_id: isize, route_mode: SpiRoutingMode, affinity: u64) -> GicStatus {
        if int_id <= GIC_PPI_END_ID {
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