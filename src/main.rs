extern crate separator;

use std::fs::File;
use std::str::FromStr;
use std::time::{Duration, Instant};

use clap::Parser;
use crate::sim::model::class::{Class, StreamType};
use crate::sim::simulator::sim_result::SimResult;

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
    SimResult::write_header(args.v, &mut file);

    for cur_call_stream in &args.call_stream
    {
        for cur_serv_stream in &args.serv_stream
        {
            while a <= args.a_max
            {
                // Prepare Streams and write its params
                let tr_class =
                    Class::new(StreamType::from_str(&cur_call_stream).expect("Failed"),
                               StreamType::from_str(&cur_serv_stream).expect("Failed"),
                               a, args.cs_e2_d2, 1f64, args.ss_e2_d2);

                // Make simulation experiments and write it

                let mut results = SimResult::new(&tr_class);

                //let mut results = BTreeMap::new();
                for v in 1..args.v + 1 {
                    print!("Simulation a={} v={}", a, v);
                    let start = Instant::now();
                    let no_of_ser = 3;
                    let (avg, dev) = sim::simulation(v, tr_class, 1000, no_of_ser);
                    let duration = start.elapsed();
                    let pefromance =  (avg.no_of_events * no_of_ser as f64) / duration.as_micros() as f64;
                    println!(" performance {:.3} events/µs, no of events : {} ", pefromance, avg.no_of_events.round().separated_string());
                    results.add(v, avg, dev);
                }
                results.write(&mut file);

                a += args.a_delta;
            }
        }
    }

    println!("Done");
    Ok(())
}


