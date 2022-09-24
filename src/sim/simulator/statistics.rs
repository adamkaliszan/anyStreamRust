#[derive(Clone)]
pub enum EventType
{
    NewCall,
    EndCall,
    LostCall
}

#[derive(Clone)]
pub struct StatisticsRaw
{
    pub states: Vec<MacrostateRaw>,
    pub time_total: f64
}

pub struct Statistics
{
    pub v: usize,
    pub states: Vec<Macrostate>
}

#[derive(Clone, Copy)]
pub struct MacrostateRaw
{
    pub duration: f64,
    pub no_out_new: u32,
    pub no_out_end: u32,
}

#[derive(Clone, Copy)]
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
            states: vec![Macrostate::new(); v + 1]
        }
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