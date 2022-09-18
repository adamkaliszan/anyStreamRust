use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;

use clap::Parser;
use rand_distr::num_traits::ToPrimitive;
use crate::sim::model::class::{Class, StreamType};
use crate::sim::simulator::statistics::{Macrostate, Statistics};

mod sim;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {

    /// Output filename
    #[clap(parse(from_os_str), short, long, default_value="results.txt")]
    pub output_path: std::path::PathBuf,

    /// System capacity
    #[clap(short, default_value_t=10)]
    v: u32,

    /// Start value of offered traffic per system
    #[clap(long, default_value_t=8.0)]
    a_min: f64,

    /// End value of offered traffic per system
    #[clap(long, default_value_t=12.0)]
    a_max: f64,

    /// Increment traffic per experiment
    #[clap(long, default_value_t=1.0)]
    a_delta: f64,

    /// Arrival stream type
    #[clap(long, default_value="poisson")]
    call_stream: String,

    /// Service stream type
    #[clap(long, default_value="poisson")]
    serv_stream: String,

    /// Arrival stream parameters. ExpectedValue²/Variance²
    #[clap(long, default_value_t=1.0)]
    cs_e2_d2: f64,

    /// Service stream parameters. ExpectedValue²/Variance²
    #[clap(long, default_value_t=1.0)]
    ss_e2_d2: f64,

    /// Minimum no of ocurrance of every state to finish simulation experiment
    #[clap(short, default_value_t=100)]
    nim_state_cntr: u32,
}

fn main() -> std::io::Result<()>
{
    let args = Cli::parse();
    let mut a = args.a_min;

    let mut file = File::create(args.output_path)?;
    write_header(&mut file, args.v);

    while a <= args.a_max
    {
        // Prepare Streams and write its params
        let tr_class =
            Class::new( StreamType::from_str(&args.call_stream).expect("Failed"),
                        StreamType::from_str(&args.serv_stream).expect("Failed"),
                        a, args.cs_e2_d2, 1f64, args.ss_e2_d2);
        write_sim_par(&tr_class, &mut file);

        // Make simulation experiments and write it
        let mut results = HashMap::new();
        for v in 1..args.v+1 {
            let res = sim::simulation(v, tr_class, 1000, 3);
            results.insert(v, res);
        }
        write_sim_prob(&results, &mut file);
        write_new_int(&results, &mut file);
        write_end_int(&results, &mut file);
        file.write_fmt(format_args!("\n")).expect("I/O error");
        a+= args.a_delta;
    }

    println!("Done");
    Ok(())

}

fn write_header(output: &mut File, v_max :u32)
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

fn write_sim_par(tr_class : &Class, output: &mut File) {
    output.write_fmt(format_args!("{:0.4}\t{}\t{}\t{}\t{}\t{}\t{}", tr_class.get_a(),
        tr_class.get_str_new_id(), tr_class.get_str_new_desc(), tr_class.get_new_e2d2(),
        tr_class.get_str_end_id(), tr_class.get_str_end_desc(), tr_class.get_end_e2d2())).
        expect("Write write_sim_par filed");
}

fn write_sim_prob(results: &HashMap<u32, Statistics>, output: &mut File) {
    let v_max = results.iter().map(|x|x.0).max().unwrap();
    for v in 1..v_max+1 {
        match results.get_key_value(&v) {
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

fn write_new_int(results: &HashMap<u32, Statistics>, output: &mut File) {
    let v_max = results.iter().map(|x|x.0).max().unwrap();
    for v in 1..v_max+1 {
        match results.get_key_value(&v) {
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

fn write_end_int(results: &HashMap<u32, Statistics>, output: &mut File) {
    let v_max = results.iter().map(|x|x.0).max().unwrap();
    for v in 1..v_max+1 {
        match results.get_key_value(&v) {
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
