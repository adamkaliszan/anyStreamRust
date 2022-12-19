pub mod simulator;
pub mod model;

use crate::sim::model::class::*;

use crate::sim::simulator::Simulator;
use crate::sim::simulator::statistics::Statistics;

//#![feature(map_first_last)]

pub fn simulation(v: u32, tr_class:Class, min_state_cntr: u32, no_of_ser: usize)
    -> (Statistics, Statistics)
{
    let mut systems = vec![Simulator::new(&tr_class, v as usize); no_of_ser];

    let mut statistics: Vec<Statistics> = Vec::new();

    for system in &mut systems {
        system.prepare_simulation();
    }

    for system in &mut systems {
        system.simulate_with_statistics(min_state_cntr);
        statistics.push(system.prepare_statistics());
    }

    Statistics::statistics_proc(&statistics)
}
