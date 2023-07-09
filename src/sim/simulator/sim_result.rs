use std::collections::{BTreeMap, LinkedList};
use std::fs::File;
use std::io::prelude::*;
use serde::{Deserialize, Serialize};
use mongodb::{bson::doc, results, error};

use crate::sim::model::class::{Class};
use crate::sim::simulator::statistics::Statistics;

#[derive(Serialize, Deserialize)]
pub struct SimResultSingleV {
    v: u32,
    avg: Statistics,
    dev: Statistics
}

#[derive(Serialize, Deserialize)]
pub struct SimResult
{
    pub tr_class: Class,
    pub systems: LinkedList<SimResultSingleV>
}

impl SimResult {
    pub fn new(tr_class :Class) -> SimResult {
        SimResult {
            tr_class: tr_class,
            systems: LinkedList::new()
        }
    }
    pub fn add(&mut self, v: u32, avg : Statistics, dev: Statistics) {
        self.systems.push_back(SimResultSingleV{v: v, avg: avg, dev: dev});
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
        SimResult::write_sim_par(&self.tr_class, output);
        self.write_sim_prob(output);
        self.write_new_int(output);
        self.write_end_int(output);
        self.write_sim_prob_dev(output);
        self.write_new_int_dev(output);
        self.write_end_int_dev(output);
        output.write_fmt(format_args!("\n")).expect("I/O error");
    }

    fn write_sim_prob(& self, output: &mut File) {
        let v_max = self.systems.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            let item = self.systems.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val_avg = &itm.avg;

                    for x in 0..val_avg.v+1 {
                        output.write_fmt(format_args!("\t{}", val_avg.states[x].p)).
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
        let v_max = self.systems.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            let item = self.systems.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val_dev = &itm.dev;

                    for x in 0..val_dev.v+1 {
                        output.write_fmt(format_args!("\t{}", val_dev.states[x].p)).
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
        let v_max = self.systems.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            let item = self.systems.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val = &itm.avg;

                    for x in 0..val.v+1 {
                        output.write_fmt(format_args!("\t{}", val.states[x].out_new)).
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
        let v_max = self.systems.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            let item = self.systems.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val = &itm.dev;

                    for x in 0..val.v+1 {
                        output.write_fmt(format_args!("\t{}", val.states[x].out_new)).
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
        let v_max = self.systems.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            let item = self.systems.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val = &itm.avg;

                    for x in 0..val.v+1 {
                        output.write_fmt(format_args!("\t{}", val.states[x].out_end)).
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
        let v_max = self.systems.iter().map(|x|x.v).max().unwrap();
        for v in 1..v_max+1 {
            let item = self.systems.iter().find(|&x| x.v == v);
            match item {
                Option::Some(itm) => {
                    let val = &itm.dev;

                    for x in 0..val.v+1 {
                        output.write_fmt(format_args!("\t{}", val.states[x].out_end)).
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
