use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;

use crate::sim::model::class::{Class};
use crate::sim::simulator::statistics::Statistics;

pub struct SimResult<'a>
{
    pub tr_class: &'a Class,
    pub items: BTreeMap<u32, Statistics>
}

impl <'a> SimResult<'a> {
    pub fn new(tr_class :&'a Class) -> SimResult<'a> {
        SimResult {
            tr_class: tr_class,
            items: BTreeMap::new()
        }
    }

    pub fn write_header(v_max :u32, output: &mut File)
    {
        output.write_fmt(format_args!("#A\tArrival Id\tArrival desc\tD²/E²\tServ Id\tServ desc\tD²/E²")).
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
        output.write_fmt(format_args!("\n")).
            expect("Write header filed");

    }

    pub fn write(& self, output: &mut File)
    {
        SimResult::write_sim_par(self.tr_class, output);
        self.write_sim_prob(output);
        self.write_new_int(output);
        self.write_end_int(output);
        output.write_fmt(format_args!("\n")).expect("I/O error");
    }

    fn write_sim_prob(& self, output: &mut File) {
        let v_max = self.items.iter().map(|x|x.0).max().unwrap();
        for v in 1..v_max+1 {
            match self.items.get_key_value(&v) {
                Option::Some(itm) => {
                    let val = itm.1;

                    for x in 0..val.v+1 {
                        output.write_fmt(format_args!("\t{}", val.states[x].p)).
                            expect("Write probabilities failed");

                    }
                }
                None => {
                    for _ in 0..v + 1 {
                        output.write_fmt(format_args!("\t")).
                            expect("Write probsbilities failed");
                    }
                }
            }
        }
    }

    fn write_new_int(& self, output: &mut File) {
        let v_max = self.items.iter().map(|x|x.0).max().unwrap();
        for v in 1..v_max+1 {
            match self.items.get_key_value(&v) {
                Option::Some(itm) => {
                    let val = itm.1;

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

    fn write_end_int(& self, output: &mut File) {
        let v_max = self.items.iter().map(|x|x.0).max().unwrap();
        for v in 1..v_max+1 {
            match self.items.get_key_value(&v) {
                Option::Some(itm) => {
                    let val = itm.1;

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

    fn write_sim_par(tr_class : &Class, output: &mut File) {
        output.write_fmt(format_args!("{:0.4}\t{}\t{}\t{}\t{}\t{}\t{}", tr_class.get_a(),
                                      tr_class.get_str_new_id(), tr_class.get_str_new_desc(), tr_class.get_new_e2d2(),
                                      tr_class.get_str_end_id(), tr_class.get_str_end_desc(), tr_class.get_end_e2d2())).
            expect("Write write_sim_par filed");
    }
}
