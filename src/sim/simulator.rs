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
    pub total_lost: u32,
    pub total_serv: u32,

    pub min_occurrance: u32,
    pub analyze_states: bool,
    pub check_cntr: i32
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
            total_lost: 0,
            total_serv: 0,
            min_occurrance: 0,
            analyze_states: false,
            check_cntr: 100
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

        self.min_occurrance = total_lost;
        self.analyze_states = true;

        loop
        {
            let evnt = self.scheduler.get_process();
            evnt.execute(self);

            if self.end_simulation()
            {
                break;
            }
        }
    }
    pub fn prepare_statistics(&self) -> Statistics {
        self.group.statistics_preview()
    }

    pub fn end_simulation(&mut self) -> bool {
        let mut result = false;
        if self.analyze_states
        {
            self.check_cntr -= 1;
            if self.check_cntr <= 0
            {
                let min_ocur;
                self.check_cntr = 100;
                min_ocur = self.group.min_state_occurance();
                result = min_ocur >= self.min_occurrance;
            }
        }
        else { result =  self.no_of_lost_calls > 100000;}

        result
    }
}
