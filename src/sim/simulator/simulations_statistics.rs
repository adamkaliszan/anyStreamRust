use std::collections::{LinkedList};
use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};
use mongodb::bson::{doc, Uuid};

use crate::sim::model::class::{Class};
use crate::sim::simulator::single_statistics::{StatisticsFinalized, Macrostate};

#[derive(Serialize, Deserialize)]
pub struct SimStatisticsMultiV
{
    pub tr_class: Class,
    pub results: LinkedList<StatisticsMultiSimulations>
}

/// Processed Statistics
/// Base on many simulation series
/// Holds average value or standard deviation
#[derive(Serialize, Deserialize)]
pub struct StatisticsMultiSimulations
{
    pub uuids: Vec<Uuid>,
    pub v: usize,
    pub states_avarage: Vec<Macrostate>,
    pub states_deviation: Vec<Macrostate>,
    pub no_of_events_avg: f64,
    pub no_of_events_dev: f64
}

impl StatisticsMultiSimulations {
    pub fn statistics_proc(statistics: &LinkedList<StatisticsFinalized>, v:usize) -> Self {
        let no_of_series = statistics.len();
        let mut result = StatisticsMultiSimulations {
            uuids: statistics.into_iter().map(|x|x.metadata.uuid).collect(),
            v: v,
            states_avarage: vec![Macrostate::new(); v+1],
            states_deviation: vec![Macrostate::new(); v+1],
            no_of_events_avg: 0.0,
            no_of_events_dev: 0.0
        };

        for stat_ser in statistics {
            for (idx, stat_macr) in stat_ser.states.iter().enumerate() {
                result.states_avarage[idx].p += stat_macr.p;
                result.states_avarage[idx].out_new += stat_macr.out_new;
                result.states_avarage[idx].out_end += stat_macr.out_end;
            }
            result.no_of_events_avg+= stat_ser.no_of_events as f64;
        }

        for res_st in &mut result.states_avarage {
            res_st.p /= no_of_series as f64;
            res_st.out_new /= no_of_series as f64;
            res_st.out_end /= no_of_series as f64;
        }
        result.no_of_events_avg /= no_of_series as f64;

        for stat_ser in statistics {
            for (idx, stat_macr) in stat_ser.states.iter().enumerate() {
                result.states_deviation[idx].p += (result.states_avarage[idx].p - stat_macr.p).powi(2);
                result.states_deviation[idx].out_new += (result.states_avarage[idx].out_new - stat_macr.out_new).powi(2);
                result.states_deviation[idx].out_end += (result.states_avarage[idx].out_end - stat_macr.out_end).powi(2);
            }
            result.no_of_events_dev += (result.no_of_events_avg - stat_ser.no_of_events as f64).powi(2);
        }

        for res_st in &mut result.states_deviation {
            res_st.p = (res_st.p / no_of_series as f64).sqrt();
            res_st.out_new = (res_st.out_new / no_of_series as f64).sqrt();
            res_st.out_end = (res_st.out_end / no_of_series as f64).sqrt();
        }
        result.no_of_events_dev = (result.no_of_events_dev / no_of_series as f64).sqrt();
        result
    }
}

impl SimStatisticsMultiV {
    pub fn new(tr_class :Class) -> SimStatisticsMultiV {
        SimStatisticsMultiV {
            tr_class: tr_class,
            results: LinkedList::new()
        }
    }

    pub fn write_header(v_max :u32, output: &mut File)
    {
        output.write_fmt(format_args!("#A\tArrival Id\tArrival desc\tE²/D²\tServ Id\tServ desc\tE²/D²")).
            expect("Write header filed");

        for v in 1..v_max+1 {
            for n in 0..v+1 {
                output.write_fmt(format_args!("\tp[{}]_{}", n, v)).
                    expect("Write header filed");
            }
        }

        for v in 1..v_max+1 {
            for n in 0..v+1 {
                output.write_fmt(format_args!("\tλ[{}]_{}", n, v)).
                    expect("Write header filed");
            }
        }

        for v in 1..v_max+1 {
            for n in 0..v+1 {
                output.write_fmt(format_args!("\tµ[{}]_{}", n, v)).
                    expect("Write header filed");
            }
        }

        for v in 1..v_max+1 {
            for n in 0..v+1 {
                output.write_fmt(format_args!("\tδ p[{}]_{}", n, v)).
                    expect("Write header filed");
            }
        }

        for v in 1..v_max+1 {
            for n in 0..v+1 {
                output.write_fmt(format_args!("\tδ λ[{}]_{}", n, v)).
                    expect("Write header filed");
            }
        }

        for v in 1..v_max+1 {
            for n in 0..v+1 {
                output.write_fmt(format_args!("\tδ µ[{}]_{}", n, v)).
                    expect("Write header filed");
            }
        }

        output.write_fmt(format_args!("\n")).
            expect("Write header filed");
    }

    pub fn write_mongo(& self, db: &mut mongodb::sync::Database) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error>
    {
        let collection = db.collection::<Self>("results");
        collection.insert_one(self, None)
    }

    pub fn write(& self, output: &mut File)
    {
        SimStatisticsMultiV::write_sim_par(&self.tr_class, output);
        self.write_sim_prob(output);
        self.write_new_int(output);
        self.write_end_int(output);
        self.write_sim_prob_dev(output);
        self.write_new_int_dev(output);
        self.write_end_int_dev(output);
        output.write_fmt(format_args!("\n")).expect("I/O error");
    }

    fn write_sim_prob(& self, output: &mut File) {

        let mut systems_v :LinkedList<StatisticsMultiSimulations>;
        let v_max = self.results.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            //systems_v.append(StatisticsMultiSimulations::statistics_proc(
            //    self.systems.iter().filter(|item| item.v == v).map(|x| x.clone()).collect()));
        }


        for v in 1..v_max+1 {
            let item = self.results.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val_avg = &itm.states_avarage;

                    for x in 0..v+1 {
                        output.write_fmt(format_args!("\t{}", val_avg[x].p)).
                            expect("write_sim_prob failed");
                    }
                }
                None => {
                    for _ in 0..v + 1 {
                        output.write_fmt(format_args!("\t")).
                            expect("write_sim_prob failed");
                    }
                }
            }
        }
    }

    fn write_sim_prob_dev(& self, output: &mut File) {
        let v_max = self.results.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            let item = self.results.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val_dev = &itm.states_deviation;

                    for x in 0..v+1 {
                        output.write_fmt(format_args!("\t{}", val_dev[x].p)).
                            expect("write_sim_prob_dev failed");
                    }
                }
                None => {
                    for _ in 0..v + 1 {
                        output.write_fmt(format_args!("\t")).
                            expect("write_sim_prob_dev failed");
                    }
                }
            }
        }
    }

    fn write_new_int(& self, output: &mut File) {
        let v_max = self.results.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            let item = self.results.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val = &itm.states_avarage;

                    for x in 0..v+1 {
                        output.write_fmt(format_args!("\t{}", val[x].out_new)).
                            expect("Write New intensities failed");
                    }
                }
                None => {
                    for _ in 0..v + 1 {
                        output.write_fmt(format_args!("\t")).
                            expect("Write New intensities failed");
                    }
                }
            }
        }
    }

    fn write_new_int_dev(& self, output: &mut File) {
        let v_max = self.results.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            let item = self.results.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val = &itm.states_deviation;

                    for x in 0..v+1 {
                        output.write_fmt(format_args!("\t{}", val[x].out_new)).
                            expect("write_new_int_dev failed");
                    }
                }
                None => {
                    for _ in 0..v + 1 {
                        output.write_fmt(format_args!("\t")).
                            expect("write_new_int_dev failed");
                    }
                }
            }
        }
    }

    fn write_end_int(& self, output: &mut File) {
        let v_max = self.results.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            let item = self.results.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val = &itm.states_avarage;

                    for x in 0..v+1 {
                        output.write_fmt(format_args!("\t{}", val[x].out_end)).
                            expect("Write End intensities failed");
                    }
                }
                None => {
                    for _ in 0..v+1 {
                        output.write_fmt(format_args!("\t")).
                            expect("Write End intensities failed");
                    }
                }
            }
        }
    }

    fn write_end_int_dev(& self, output: &mut File) {
        let v_max = self.results.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            let item = self.results.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val = &itm.states_deviation;

                    for x in 0..v+1 {
                        output.write_fmt(format_args!("\t{}", val[x].out_end)).
                            expect("write_end_int_dev failed");
                    }
                }
                None => {
                    for _ in 0..v+1 {
                        output.write_fmt(format_args!("\t")).
                            expect("write_end_int_dev failed");
                    }
                }
            }
        }
    }

    fn write_sim_par(tr_class : &Class, output: &mut File) {
        output.write_fmt(format_args!("{:0.4}\t{}\t{}\t{}\t{}\t{}\t{}", tr_class.get_a(),
                                      tr_class.get_str_new_id(), tr_class.get_str_new_desc(), tr_class.get_new_e2d2(),
                                      tr_class.get_str_end_id(), tr_class.get_str_end_desc(), tr_class.get_end_e2d2())).
            expect("Write write_sim_par filed");
    }
}
