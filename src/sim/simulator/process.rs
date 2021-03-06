use std::cmp::Ordering;

use crate::sim::simulator::Simulator;
use crate::sim::model::class::Class;

//use self::Ordering::*;
//use std::collections::BinaryHeap;
//use rand_distr::num_traits::ToPrimitive;
//use std::rc::{Rc, Weak};
//use rand::rngs::ThreadRng;
//use crate::sim::simulator::system::*;
//use crate::sim::simulator::scheduler::*;


#[derive(Clone, Copy)]
pub enum State {
    WaitForNew,
    WaitForService
}

#[derive(Clone)]
pub struct SimProcess<'a>//: Ord
{
    pub state: State,
    pub time: f64,
    pub class: &'a Class,
}

impl Ord for SimProcess<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.time < other.time {
            Ordering::Greater
        }
        else if self.time > other.time {
            Ordering::Less
        }
        else {
            Ordering::Equal
        }
    }
}

impl <'a> SimProcess<'a>  {
    pub fn new(class: &'a Class) -> SimProcess<'a> {
        SimProcess {state:State::WaitForNew, time: 0.1f64, class:class}
    }
    pub fn execute(mut self, system: &mut Simulator<'a>) -> bool
    {
        let last_time = self.time;
        if match self.state {
            State::WaitForNew => {
                if system.group.call_add(last_time) {
                    let time = self.class.get_time_end_call(&mut system.rng);
                    system.scheduler.add_process(SimProcess { state: State::WaitForService, time: time, class: &self.class });
                }
                else {
                    system.total_lost += 1;
                }
                self.time = self.class.get_time_new_call(&mut system.rng);
                true
            },
            State::WaitForService => {
                system.group.call_end(last_time);
                false
            }
        } {
            system.scheduler.add_process(self);
        }
        true
    }
}

impl PartialOrd for SimProcess<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Eq for SimProcess<'_> {
    fn assert_receiver_is_total_eq(&self) { }
}

impl PartialEq for SimProcess<'_> {
    fn eq(&self, other: &Self) -> bool { self.time.eq(&other.time) }
    fn ne(&self, other: &Self) -> bool { self.time.ne(&other.time) }
}