extern crate separator;
extern crate flot;

use std::fs::File;
use std::str::FromStr;
use std::time::Instant;

use clap::Parser;
use crate::sim::model::class::{Class, StreamType};
use crate::sim::simulator::sim_result::SimResult;

use float_cmp::*;
use cartesian::*;
use separator::Separatable;
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
    #[clap(long, default_value="poisson", multiple=true)]
    call_stream: Vec<String>,

    /// Service stream type
    #[clap(long, default_value="poisson", multiple=true)]
    serv_stream: Vec<String>,

    /// Arrival stream parameters ExpectedValue²/Variance² initial value
    #[clap(long, default_value_t=3.0)]
    cs_e2_d2_min: f64,

    /// Arrival stream parameters. ExpectedValue²/Variance² end value
    #[clap(long, default_value_t=3.0)]
    cs_e2_d2_max: f64,

    /// Arrival stream parameters. ExpectedValue²/Variance² increment
    #[clap(long, default_value_t=1.0)]
    cs_e2_d2_delta: f64,

    /// Service stream parameters. ExpectedValue²/Variance² initial value
    #[clap(long, default_value_t=3.0)]
    ss_e2_d2_min: f64,

    /// Service stream parameters. ExpectedValue²/Variance² end value
    #[clap(long, default_value_t=3.0)]
    ss_e2_d2_max: f64,

    /// Service stream parameters. ExpectedValue²/Variance² increment
    #[clap(long, default_value_t=1.0)]
    ss_e2_d2_delta: f64,

    /// Minimum no of ocurrance of every state to finish simulation experiment
    #[clap(short, default_value_t=100)]
    mim_state_cntr: u32,

    /// Number of series in simulation experiment
    #[clap(short, default_value_t=3)]
    no_of_series: usize
}

fn main() -> std::io::Result<()>
{
    let args = Cli::parse();

    let mut file = File::create(args.output_path)?;
    SimResult::write_header(args.v, &mut file);

    let call_streams = args.call_stream.clone();
    let serv_streams = args.serv_stream.clone();
    let cs_e2_d2_col:Vec<f64> = flot::range(args.cs_e2_d2_min, args.cs_e2_d2_max + args.cs_e2_d2_delta, args.cs_e2_d2_delta).collect();
    let ss_e2_d2_col:Vec<f64> = flot::range(args.ss_e2_d2_min, args.ss_e2_d2_max + args.ss_e2_d2_delta, args.ss_e2_d2_delta).collect();
    let a_col: Vec<f64> = flot::range(args.a_min, args.a_max + args.a_delta, args.a_delta).collect();

    for (cur_call_stream, cur_serv_stream, cs_e2_d2, ss_e2_d2, a) in
        cartesian!(call_streams.iter(), serv_streams.iter(), cs_e2_d2_col.iter(), ss_e2_d2_col.iter(), a_col.iter())
    {
        // Prepare Streams and write its params
        let call_stream = StreamType::from_str(&cur_call_stream).expect("Failed");
        let service_stream = StreamType::from_str(&cur_serv_stream).expect("Failed");

        if (matches!(call_stream, StreamType::Poisson) && !approx_eq!(f64, *cs_e2_d2, 1f64))
        || (matches!(service_stream, StreamType::Poisson) && !approx_eq!(f64, *ss_e2_d2, 1f64)) {
            continue;
        }

        let tr_class = Class::new(call_stream, service_stream,
                                  *a, *cs_e2_d2, 1f64, *ss_e2_d2);

        // Make simulation experiments and write it
        let mut results = SimResult::new(&tr_class);

        println!("Simulation a={}, arrival stream {}:{}, service stream {}:{}", *a, &cur_call_stream, *cs_e2_d2, &cur_serv_stream, *ss_e2_d2);

        //let mut results = BTreeMap::new();
        for v in 1..args.v + 1 {
            print!("v={}", v);
            let start = Instant::now();
            let (avg, dev) = sim::simulation(v, tr_class, args.mim_state_cntr, args.no_of_series);
            let duration = start.elapsed();
            let pefromance =  (avg.no_of_events * args.no_of_series as f64) / duration.as_micros() as f64;
            println!(" performance {:.3} events/µs, no of events : {} ", pefromance, avg.no_of_events.round().separated_string());
            results.add(v, avg, dev);
        }
        results.write(&mut file);
    }

    println!("Done");
    Ok(())
}

