use cpu::get_cpu_id;
use log::error;

use crate::mci::MCI;

mod consts;
mod cpu;
mod gic;
mod err;
mod reg;

impl MCI {
    pub fn setup_irq(&self) {
        let cpu_id = get_cpu_id();
        let cpu_id = match cpu_id {
            Err(e) => {
                error!("get cpu id failed! err: {:?}", e);
                panic!();
            },
            Ok(id) => id,
        };

        cpu_id.set_target_cpus_interrupt(self.config().irq_num() as isize);
    }

}