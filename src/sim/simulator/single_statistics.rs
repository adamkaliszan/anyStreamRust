use mongodb::bson;
use mongodb::options::FindOptions;
use mongodb::bson::{Uuid, doc, Document};
use semver::{BuildMetadata, Prerelease, Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

use crate::sim::model::system::ModelDescription;


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
pub struct StatisticsRunExperiment
{
    pub states: Vec<MacrostateRaw>,
    pub time_total: f64
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StatisticsFinalizedMetadata
{
    pub min_no_of_events_per_state: u32,
    pub uuid: Uuid,
    pub version: String
}

/// Processed Statistics
/// Base on single finished simulation experiment
#[derive(Serialize, Deserialize, Clone)]
pub struct StatisticsFinalized
{
    pub states: Vec<Macrostate>,
    pub v : usize,
    pub no_of_events: u64,
    pub metadata: StatisticsFinalizedMetadata
}

#[derive(Serialize, Deserialize, Clone)]
struct StatisticsFinalizedWithInputModel {
    system: ModelDescription,
    stat: StatisticsFinalized
}

impl StatisticsFinalized {
    pub fn write_mongo(& self, model: &ModelDescription, db: &mut mongodb::sync::Database) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error>
    {
        let data: StatisticsFinalizedWithInputModel = StatisticsFinalizedWithInputModel {stat: self.clone(), system: model.clone()};

        let collection = db.collection::<StatisticsFinalizedWithInputModel>("statistics");
        collection.insert_one(data, None)
    }

    pub fn read_mongo(model: &ModelDescription, db: &mongodb::sync::Database) -> LinkedList<StatisticsFinalized> {
        let mut result= LinkedList::new();

        let collection = db.collection::<StatisticsFinalizedWithInputModel>("statistics");

        let filter = bson::to_document(model).unwrap();
        let mut final_filter: Document = bson::Document::new();
        final_filter.insert("system", filter);

        let find_options = FindOptions::builder().sort(doc! { "system": 1 }).build();
        let mut cursor = collection.find(final_filter, find_options).unwrap();

        while let Some(itm) = cursor.next() {
            match itm {
                Ok(itm_data) => {
                    result.push_back(itm_data.stat);
                }
                Err(e) => { println!("Failed to read data: {}", e);}
            };
        }
        result
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct MacrostateRaw
{
    pub duration: f64,
    pub no_out_new: usize,
    pub no_out_end: usize,
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

impl StatisticsRunExperiment {
    pub fn new(v: usize) ->Self {
        StatisticsRunExperiment {
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

impl Macrostate {
    pub fn new() -> Self {
        Macrostate {
            p: 0f64,
            out_new: 0f64,
            out_end: 0f64,
        }
    }
}