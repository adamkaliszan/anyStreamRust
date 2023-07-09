use mongodb::bson::Uuid;

use crate::sim::simulator::single_statistics::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
pub struct Group
{
    pub v: usize,
    v_free: usize,
    statistics: Option<StatisticsRunExperiment>
}

impl Group {
    pub fn new(capacity: usize) -> Self {
        Self {
            v: capacity,
            v_free: capacity,
            statistics: None
        }
    }

    pub fn call_add(&mut self, time_delta: f64) -> bool {
        let result;
        let old_state: usize = (self.v - self.v_free) as usize;
        let new_state: usize;
        let event_type: EventType;

        if self.v_free > 0 {
            event_type = EventType::NewCall;
            new_state = old_state+1;
            self.v_free -= 1;
            result = true;
        }
        else {
            event_type = EventType::LostCall;
            new_state = old_state;
            result = false;
        }
        if let Some(stat) = &mut self.statistics {
            stat.update(event_type, old_state, new_state, time_delta);
        }
        result
    }

    pub fn call_end(&mut self, time_delta: f64) {

        assert!(self.v_free <= self.v);

        let event_type = EventType::EndCall;
        let old_state: usize = (self.v - self.v_free) as usize;
        self.v_free += 1;
        let new_state: usize = (self.v - self.v_free) as usize;

        if let Some(stat) = &mut self.statistics {
            stat.update(event_type, old_state, new_state, time_delta);
        }
    }

    pub fn statistics_init(&mut self) {
        match &mut self.statistics {
            None => self.statistics = Option::Some(StatisticsRunExperiment::new(self.v)),
            Some(stat) => stat.clear()
        }
    }

    pub fn statistics_preview(&self, no_of_events:u64, min_no_of_events_per_state: u32) -> StatisticsFinalized {
        if let Some(stat) = &self.statistics {
            let total_time = stat.time_total;

            return StatisticsFinalized {
                states : stat.states.iter().map(
                    |x| x.get_macrostate_statistics(total_time)
                ).collect(),
                v: self.v,
                no_of_events: no_of_events,
                metadata : StatisticsFinalizedMetadata {
                    min_no_of_events_per_state: min_no_of_events_per_state,
                    uuid: Uuid::new(),
                    version: VERSION.to_string()
                }
            };
        }
        panic!("No raw statistics");
    }

    pub fn min_state_occurance(&self) -> usize {
        match &self.statistics {
            Some(a) => {
                match &a.states.iter().min_by(
                    |&itm1, &itm2|
                        (itm1.no_out_new + itm1.no_out_end).cmp(&(itm2.no_out_new + itm2.no_out_end)))
                {
                    Some(b) => b.no_out_new + b.no_out_end,
                    None => 0
                }
            },
            None => 0
        }
    }
}