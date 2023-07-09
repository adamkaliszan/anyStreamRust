pub mod simulator;
pub mod model;

use std::collections::LinkedList;
use crate::sim::model::class::sim_class::SimClass;
use crate::sim::simulator::Simulator;
use crate::sim::simulator::single_statistics::{StatisticsFinalized};
use crate::sim::simulator::simulations_statistics::StatisticsMultiSimulations;


//#![feature(map_first_last)]

pub fn simulation_all_series(v: usize, tr_class:SimClass, min_state_cntr: u32, no_of_ser: usize)
                           -> StatisticsMultiSimulations
{
    let mut systems = vec![Simulator::new(&tr_class, v); no_of_ser];

    let mut statistics: LinkedList<StatisticsFinalized> = LinkedList::new();

    for system in &mut systems {
        system.prepare_simulation();
    }

    for system in &mut systems {
        system.simulate_with_statistics(min_state_cntr);
        statistics.push_back(system.prepare_statistics());
    }
    //TODO mongo write to database

    StatisticsMultiSimulations::statistics_proc(&statistics, v)
}

pub fn simulation(v: usize, tr_class:SimClass, min_state_cntr: u32)
                           -> StatisticsFinalized
{
    let mut system = Simulator::new(&tr_class, v);

    system.prepare_simulation();
    system.simulate_with_statistics(min_state_cntr);
    system.prepare_statistics()
}