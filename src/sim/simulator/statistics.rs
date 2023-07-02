use serde::{Deserialize, Serialize};
use std::cmp::Ord;
use std::cmp::Ordering;


#[derive(Clone)]
pub enum EventType
{
    NewCall,
    EndCall,
    LostCall
}

/// Single simulation statistics, before processing.
/// New events can be added in order to increase single simulations'
/// experiment accuracy if only add events before ware accounted to
/// the statistics.
#[derive(Serialize, Deserialize, Clone)]
pub struct StatisticsRaw
{
    pub states: Vec<MacrostateRaw>,
    pub time_total: f64
}

/// Processed Statistics
/// Base on many simulation series
/// Holds average value or standard deviation
#[derive(Serialize, Deserialize)]
pub struct Statistics
{
    pub v: usize,
    pub states: Vec<Macrostate>,
    pub no_of_events: f64
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct MacrostateRaw
{
    pub duration: f64,
    pub no_out_new: u32,
    pub no_out_end: u32,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Macrostate
{
    pub p: f64,
    pub out_new: f64,
    pub out_end: f64,
}

impl MacrostateRaw {
    pub fn new() -> Self {
        Self { duration:0f64, no_out_new: 0, no_out_end: 0 }
    }

    pub fn clear_statistics(&mut self) {
        self.no_out_end = 0;
        self.no_out_new = 0;
        self.duration = 0.0;
    }

    pub fn get_macrostate_statistics(&self, total_time: f64) -> Macrostate {
        Macrostate{
            p: self.duration/total_time,
            out_new: self.no_out_new as f64 / self.duration,
            out_end: self.no_out_end as f64 / self.duration
        }
    }
}

impl StatisticsRaw {
    pub fn new(v: usize) ->Self {
        StatisticsRaw {
            states : vec![MacrostateRaw::new(); v+1],
            time_total:0_f64
        }
    }

    pub fn clear(&mut self)
    {
        for ref mut itm in &mut self.states {
            itm.clear_statistics();
        }
        self.time_total = 0f64;
    }

    pub fn update(&mut self, event_type: EventType, old_state:usize, _new_state:usize, time:f64) {
        self.time_total += time;
        self.states[old_state].duration += time;
        match event_type {
            EventType::NewCall => {
                self.states[old_state].no_out_new +=1;
            }
            EventType::LostCall => {
                self.states[old_state].no_out_new +=1;
            }
            EventType::EndCall => {
                self.states[old_state].no_out_end +=1;
            }
        }
    }
}

impl Statistics {
    pub fn new(v: usize) -> Self {
        Statistics {
            v,
            states: vec![Macrostate::new(); v + 1],
            no_of_events: 0.0
        }
    }

    pub fn statistics_proc(statistics: &Vec<Statistics>) -> (Statistics, Statistics) {
        let mut res_avg = Statistics::new(statistics[0].v);
        let mut res_dev = Statistics::new(statistics[0].v);
        for stat_ser in statistics {
            for (idx, stat_macr) in stat_ser.states.iter().enumerate() {
                res_avg.states[idx].p += stat_macr.p;
                res_avg.states[idx].out_new += stat_macr.out_new;
                res_avg.states[idx].out_end += stat_macr.out_end;
            }
            res_avg.no_of_events+= stat_ser.no_of_events;
        }
        for res_st in &mut res_avg.states {
            res_st.p /= statistics.len() as f64;
            res_st.out_new /= statistics.len() as f64;
            res_st.out_end /= statistics.len() as f64;
        }
        res_avg.no_of_events = res_avg.no_of_events / statistics.len() as f64;

        for stat_ser in statistics {
            for (idx, stat_macr) in stat_ser.states.iter().enumerate() {
                res_dev.states[idx].p += (res_avg.states[idx].p - stat_macr.p).powi(2);
                res_dev.states[idx].out_new += (res_avg.states[idx].out_new - stat_macr.out_new).powi(2);
                res_dev.states[idx].out_end += (res_avg.states[idx].out_end - stat_macr.out_end).powi(2);
            }
            res_dev.no_of_events += (res_avg.no_of_events - stat_ser.no_of_events).powi(2);
        }

        for res_st in &mut res_dev.states {
            res_st.p = (res_st.p / statistics.len() as f64).sqrt();
            res_st.out_new = (res_st.out_new / statistics.len() as f64).sqrt();
            res_st.out_end = (res_st.out_end /statistics.len() as f64).sqrt();
        }
        res_dev.no_of_events = (res_dev.no_of_events / statistics.len() as f64).sqrt();
        (res_avg, res_dev)
    }
}

impl Macrostate {
    pub fn new() -> Self {
        Macrostate {
            p: 0f64,
            out_new: 0f64,
            out_end: 0f64,
        }
    }
}