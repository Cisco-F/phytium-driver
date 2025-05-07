use log::error;

use crate::{aarch::get_affinity, CORE0_AFF, CORE1_AFF, CORE2_AFF, CORE3_AFF};

use super::consts::INTERRUPT_CPU_TARGET_ALL_SET;



pub struct CpuId(u32);

impl CpuId {
    pub fn set_target_cpus_interrupt(&self, int_id: isize) {
        if self.0 == INTERRUPT_CPU_TARGET_ALL_SET {

        } else {
            let cluster = get_cpu_affinity(self.0);
            let mut cluster = match cluster {
                Err(e) => {
                    error!("get cpu affinity failed! err: {:?}", e);
                    panic!("cpu num [{}] is not in board!", self.0);
                },
                Ok(cluster) => cluster,
            };

            let temp_cluster = (cluster >> 24) & 0xFF;
            cluster &= !(0xFF << 24);
            cluster |= temp_cluster;
            // fgicsetspiaffinityrouting
        }

        
    }
}

pub fn get_cpu_id() -> Result<CpuId, &'static str> {
    let affinity = unsafe { get_affinity() };
    let id = match affinity {
        CORE0_AFF => 0,
        CORE1_AFF => 1,
        CORE2_AFF => 2,
        CORE3_AFF => 3,
        _ => return Err("Unknown CPU affinity"),
    };

    Ok(CpuId(id))
}

pub fn get_cpu_affinity(cpu_id: u32) -> Result<u64, &'static str> {
    let affinity_level = match cpu_id {
        0 => CORE0_AFF,
        1 => CORE1_AFF,
        2 => CORE2_AFF,
        3 => CORE3_AFF,
        _ => return Err("Unknown CPU affinity"),
    };

    Ok(affinity_level)
}