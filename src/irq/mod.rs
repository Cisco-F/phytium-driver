use cpu::get_cpu_id;
use err::GicStatus;
use log::error;

use crate::mci::MCI;

mod consts;
mod cpu;
pub mod gic;
mod err;
mod reg;
pub mod interrupt;

impl MCI {
    pub fn setup_irq(&mut self) -> GicStatus {
        let cpu_id = get_cpu_id();
        let cpu_id = match cpu_id {
            Err(e) => {
                error!("get cpu id failed! err: {:?}", e);
                panic!();
            },
            Ok(id) => id,
        };

        cpu_id.set_target_cpus_interrupt(self.config().irq_num() as isize);
        let irq_num = self.config().irq_num() as i32;
        self.gic_handler_mut().set_prio(irq_num, 0xC)?;


        Ok(())
    }

}

