use crate::sim::simulator::statistics::*;

#[derive(Clone)]
pub struct Group
{
    v: usize,
    v_free: usize,
    statistics: Option<StatisticsRaw>
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
            None => self.statistics = Option::Some(StatisticsRaw::new(self.v)),
            Some(stat) => stat.clear()
        }
    }

    pub fn statistics_preview(&self) -> Statistics {
        if let Some(stat) = &self.statistics {
            let total_time = stat.time_total;

            return Statistics {
                v: self.v as usize,
                states : stat.states.iter().map(
                    |x| x.get_macrostate_statistics(total_time)
                ).collect()
            };
        }
        panic!("No raw statistics");
    }
}