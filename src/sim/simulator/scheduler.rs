use std::collections::BinaryHeap;

//use crate::sim::model::class::*;
//use crate::sim::simulator::system::*;
use crate::sim::simulator::process::*;

//use std::cmp::Ordering;
//use rand_distr::num_traits::ToPrimitive;
//use self::Ordering::*;

#[derive(Clone)]
pub struct Scheduler<'a>
{
    offset: f64,
    agenda: BinaryHeap<SimProcess<'a>>,
}

impl <'a>Scheduler<'a>
{
    pub fn new() -> Scheduler<'a> {
        Scheduler {
            offset: 0f64, agenda: BinaryHeap::new()
        }
    }

    pub fn get_process(&mut self) -> SimProcess<'a>
    {
        let mut result = self.agenda.pop().unwrap();
        result.time-= self.offset;
        self.offset+= result.time;
        if self.offset > 1024f64*1024f64 {
            self.clear_offset();
        }
        result
    }

    pub fn add_process(&mut self, mut event: SimProcess<'a>) {
        event.time+= self.offset;
        self.agenda.push(event)
    }

    fn clear_offset(&mut self) {
        let new_items : BinaryHeap<SimProcess> = self.agenda.iter().map(|itm|
            SimProcess {state: itm.state, time: itm.time-self.offset, class:itm.class}).collect();
        self.offset = 0f64;
        self.agenda = new_items;
    }
}
