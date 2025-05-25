use alloc::sync::Arc;
use bare_test::{driver::intc::{IrqConfig, Trigger}, irq::{IrqHandleResult, IrqParam}};
use cpu::get_cpu_id;
use err::GicStatus;
use log::error;
use spin::Mutex;

use crate::mci::MCI;

mod consts;
pub mod cpu;
pub mod gic;
mod err;
mod reg;
pub mod interrupt;


impl MCI {
    // pub fn setup_irq(&mut self) -> GicStatus {
    //     let cpu_id = get_cpu_id();
    //     let cpu_id = match cpu_id {
    //         Err(e) => {
    //             error!("get cpu id failed! err: {:?}", e);
    //             panic!();
    //         },
    //         Ok(id) => id,
    //     };

    //     cpu_id.set_target_cpus_interrupt(self.config().irq_num() as isize);
    //     let irq_num = self.config().irq_num() as i32;
    //     self.gic_handler_mut().set_prio(irq_num, 0xC)?;

    //     let self_arc = Arc::new(Mutex::new(self.take));
    //     IrqParam {
    //         intc: 0.into(),
    //         cfg: IrqConfig {
    //             irq: (irq_num as usize).into(),
    //             trigger: Trigger::LevelHigh,
    //         }
    //     }
    //     .register_builder({
    //         let self_arc = Arc::clone(&self_arc);
    //         move |_irq| {
    //             let mut self_locked = self_arc.lock().unwrap();
    //             self_locked.fsdif_interrupt_handler();
    //             IrqHandleResult::Handled
    //         }
    //     })
    //     .register();

    //     Ok(())
    // }

}

