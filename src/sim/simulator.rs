pub mod scheduler;
pub mod process;
pub mod system;
pub mod statistics;

use system::Group;
use rand::rngs::ThreadRng;
use scheduler::Scheduler;
use crate::sim::model::class::Class;
use crate::sim::simulator::statistics::Statistics;

#[derive(Clone)]
pub struct Simulator<'a> {
    pub group: Group,
    pub scheduler: Scheduler<'a>,
    pub rng: ThreadRng,
    pub no_of_lost_calls: u32,
    pub tr_class: &'a Class,
    pub total_lost: u32
}

impl <'a>Simulator<'a>
{
    pub fn new(tr_class:&'a Class, v:usize) -> Simulator<'a> {
        Simulator {
            group: Group::new(v),
            scheduler: Scheduler::new(),
            rng: ThreadRng::default(),
            no_of_lost_calls: 0,
            tr_class: tr_class,
            total_lost: 0
        }
    }

    pub fn prepare_simulation(&mut self) {
        let first_process = process::SimProcess::new(& self.tr_class);
        self.scheduler.add_process(first_process);

        let mut no_of_proc_call =10_000;

        while no_of_proc_call > 0 {
            let evnt = self.scheduler.get_process();
            evnt.execute( self);
            no_of_proc_call-=1;
        }
    }

    pub fn simulate_with_statistics(&mut self, total_lost: u32) {
        self.group.statistics_init();
        self.total_lost = 0;
        loop
        {
            let evnt = self.scheduler.get_process();
            evnt.execute(self);

            //let mut next_check = 0;

            if self.total_lost >= total_lost {
                break;
            }
        }
    }
    pub fn prepare_statistics(&self) -> Statistics {
        self.group.statistics_preview()
    }

}
