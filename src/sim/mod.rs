pub mod simulator;
pub mod model;

use crate::sim::model::class::*;

use crate::sim::simulator::Simulator;
use crate::sim::simulator::statistics::Statistics;

pub fn simulation(v: u32, tr_class:Class, total_lost: u32, no_of_ser: usize) -> Statistics
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

    statistics_proc(&statistics)
}

pub fn statistics_proc(statistics: &Vec<Statistics>) -> Statistics {
    let mut result = Statistics::new(statistics[0].v);
    for stat_ser in statistics {
        for (idx, stat_macr) in stat_ser.states.iter().enumerate() {
            result.states[idx].p += stat_macr.p;
            result.states[idx].out_new += stat_macr.out_new;
            result.states[idx].out_end += stat_macr.out_end;
        }
    }
    for res_st in &mut result.states {
        res_st.p /= statistics.len() as f64;
        res_st.out_new /= statistics.len() as f64;
        res_st.out_end /= statistics.len() as f64;
    }
    result
}