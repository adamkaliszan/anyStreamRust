pub mod simulator;
pub mod model;

use rand_distr::num_traits::Pow;
use crate::sim::model::class::*;

use crate::sim::simulator::Simulator;
use crate::sim::simulator::statistics::Statistics;

pub fn simulation(v: u32, tr_class:Class, total_lost: u32, no_of_ser: usize)
    -> (Statistics, Statistics)
{
    let mut systems = vec![Simulator::new(&tr_class, v as usize); no_of_ser];

    let mut statistics: Vec<Statistics> = Vec::new();

    for system in &mut systems {
        system.prepare_simulation();
    }

    for system in &mut systems {
        system.simulate_with_statistics(total_lost);
        statistics.push(system.prepare_statistics());
    }

    Statistics::statistics_proc(&statistics)
}
