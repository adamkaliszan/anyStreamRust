use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use clap::Parser;
use anyhow::Result;
use crate::sim::model::class::{Class, StreamType};
use crate::sim::simulator::statistics::{Macrostate, Statistics};

mod sim;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    //#[clap(long, default_value="asd")]
    //pattern: String,

    #[clap(parse(from_os_str), short, long, default_value="result.txt")]
    pub output_path: std::path::PathBuf,

    #[clap(short, default_value_t=30)]
    v: u32,

    #[clap(short, default_value_t=0.2)]
    a_min: f32,

    #[clap(short, default_value_t=2.0)]
    a_max: f32,

    #[clap(short, default_value_t=0.1)]
    a_delta: f32,

    #[clap(long, default_value="poisson")]
    call_stream: String,

    #[clap(long, default_value="poisson")]
    serv_stream: String,

    #[clap(short, default_value_t=1_f32)]
    cs_d2_e2: f32,

    #[clap(short, default_value_t=1_f32)]
    ss_d2_e2: f32,
}

fn main() -> std::io::Result<()>
{
    //env_logger::init();

    let args = Cli {
        output_path: ["./", "test.txt"].iter().collect() ,
        v:20,
        a_min: 0.5f32,
        a_max: 1.5f32,
        a_delta: 0.1f32,
        call_stream: String::from("Poisson"),
        serv_stream: String::from("Poisson"),
        cs_d2_e2 : 1f32,
        ss_d2_e2 : 1f32,
    };
    //= Cli::parse();

    let mut a = args.a_min;

    let mut file = File::open("results.txt")?;
    while a <= args.a_max
    {
        let tr_class =
            Class::new(StreamType::Poisson, StreamType::Poisson,
                       a as f64, 1f64, 1f64, 1f64);

        write_sim_par(&tr_class, &mut file);

        let mut results = HashMap::new();
        for v in 1..args.v+1 {
            let res = sim::simulation(v, tr_class, 1000, 10);
            results.insert(v, res);
        }

        for itm in &results {
            //itm.0
        }
        a+= args.a_delta;
    }

    //let content = std::fs::read_to_string(args.output_path)
    //    .with_context(|| format!("could not read file `{}`", args.output_path.into_os_string().into_string().unwrap()))?;//
    // std::fs::read_to_string(args.path);
    //let content = match result {
    //    Ok(content) => { content },
    //    Err(error) => { return Err(error.into()); }
    //};

    //for line in content.lines() {
    //    if line.contains(&args.pattern) {
    //        println!("Line contains \"{}\": \"{}\"", args.pattern, line);
    //    }
    //}
    println!("Done");
    Ok(())

}

fn write_sim_par(tr_class : &Class, output: &mut File) {
    output.write_fmt(format_args!("{}\t{}\t{}\t{}\t{}\t{}\t{}", tr_class.get_a() ,
                                   tr_class.get_str_new_id(), tr_class.get_str_new_desc(), tr_class.get_new_e2d2(),
                                   tr_class.get_str_end_id(), tr_class.get_str_end_desc(), tr_class.get_end_e2d2()));
}

fn write_sim_prob(results: &HashMap<u32, Statistics>, output: &mut File) {
    let v_max = results.iter().map(|x|x.0).max().unwrap();
    for v in 1..v_max+1 {
        match results.get_key_value(&v) {
            Option::Some(itm) => {
                let val = itm.1;

                for x in 0..val.v+1 {
                    output.write_fmt(format_args!("\t{}", val.states[x].p));
                }
            }
            None => {}
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
                    output.write_fmt(format_args!("\t{}", val.states[x].out_new));
                }
            }
            None => {}
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
                    output.write_fmt(format_args!("\t{}", val.states[x].out_end));
                }
            }
            None => {}
        }
    }
}
