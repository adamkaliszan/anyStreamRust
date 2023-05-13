extern crate separator;
extern crate flot;

use std::collections::LinkedList;
use std::fs::File;
use std::str::FromStr;
use std::thread;
use std::thread::{JoinHandle};
use std::time::Instant;

use clap::Parser;
use crate::sim::model::class::{Class, StreamType};
use crate::sim::simulator::sim_result::SimResult;

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
    #[clap(long, default_value="uniform", multiple=true)]
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
    #[clap(long, default_value_t=1.0)]
    ss_e2_d2_min: f64,

    /// Service stream parameters. ExpectedValue²/Variance² end value
    #[clap(long, default_value_t=1.0)]
    ss_e2_d2_max: f64,

    /// Service stream parameters. ExpectedValue²/Variance² increment
    #[clap(long, default_value_t=1.0)]
    ss_e2_d2_delta: f64,

    /// Minimum no of occurrence of every state to finish simulation experiment
    #[clap(short, default_value_t=100)]
    mim_state_cntr: u32,

    /// Number of series in simulation experiment
    #[clap(short, default_value_t=3)]
    no_of_series: usize,

    /// Number of threads
    #[clap(short, default_value_t=8)]
    threads_no: usize
}

struct SimulationTask {
    tr_class: Class,
    v : u32
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

    let mut no_off_skipped_tasks = 0;

    let mut tasks:LinkedList<SimulationTask> = LinkedList::new();

    for (cur_call_stream, cur_serv_stream, cs_e2_d2, ss_e2_d2, a) in
        cartesian!(call_streams.iter(), serv_streams.iter(), cs_e2_d2_col.iter(), ss_e2_d2_col.iter(), a_col.iter())
    {
        // Prepare Streams and write its params
        let call_stream = StreamType::from_str(&cur_call_stream.to_lowercase()).expect("Failed");
        let service_stream = StreamType::from_str(&cur_serv_stream.to_lowercase()).expect("Failed");

        if let Some(tr_class) = Class::new(
            call_stream, service_stream,
            *a, *cs_e2_d2, 1f64, *ss_e2_d2) {
            tasks.push_back(SimulationTask { tr_class: tr_class, v: args.v });
        } else {
            no_off_skipped_tasks += 1;
        }
    }
    println!("Number od tasks to do: {}, number of skipped tasks {}", tasks.len(), no_off_skipped_tasks);

    let mut workers: LinkedList <JoinHandle<SimResult>> = LinkedList::new();
    while !tasks.is_empty() {
        let mut task_no = 0;
        while task_no < args.threads_no && !tasks.is_empty() {
            let cur_task = tasks.pop_front().unwrap();
            workers.push_back(thread::spawn(move || {
                let mut results = SimResult::new(cur_task.tr_class.clone());

                println!("Simulation a={}, arrival stream {}:{}, service stream {}:{}", cur_task.tr_class.get_a(), cur_task.tr_class.get_str_new_desc(), cur_task.tr_class.get_new_e2d2(), cur_task.tr_class.get_str_end_desc(), cur_task.tr_class.get_end_e2d2());

                for v in 1..cur_task.v + 1 {
                    let start = Instant::now();
                    let (avg, dev) = sim::simulation(v, cur_task.tr_class, args.mim_state_cntr, args.no_of_series);
                    let duration = start.elapsed();
                    let pefromance = (avg.no_of_events * args.no_of_series as f64) / duration.as_micros() as f64;
                    println!("v={}: performance {:.3} events/µs, no of events : {} ", v, pefromance, avg.no_of_events.round().separated_string());
                    results.add(v, avg, dev);
                }
                results
            }));
            task_no += 1;
        }

        while !workers.is_empty() {
            let single_worker = workers.pop_front().unwrap();
            let result = single_worker.join().unwrap();
            result.write(&mut file);
        }
    }


    /*

        let sim_task = thread::spawn(move || {
            let mut results = SimResult::new(cur_task.tr_class.clone());

            println!("Simulation a={}, arrival stream {}:{}, service stream {}:{}", cur_task.tr_class.get_a(), cur_task.tr_class.get_str_new_desc(), cur_task.tr_class.get_new_e2d2(), cur_task.tr_class.get_str_end_desc(), cur_task.tr_class.get_end_e2d2());

            for v in 1..cur_task.v + 1 {
                print!("v={}", v);
                let start = Instant::now();
                let (avg, dev) = sim::simulation(v, cur_task.tr_class, args.mim_state_cntr, args.no_of_series);
                let duration = start.elapsed();
                let pefromance =  (avg.no_of_events * args.no_of_series as f64) / duration.as_micros() as f64;
                println!(" performance {:.3} events/µs, no of events : {} ", pefromance, avg.no_of_events.round().separated_string());
                results.add(v, avg, dev);
            }
            results
        });
        let result = sim_task.join().unwrap();
        result.write(&mut file);
    for cur_task in &tasks
    {
        // Make simulation experiments and write it
        let mut results = SimResult::new(&cur_task.tr_class);

        println!("Simulation a={}, arrival stream {}:{}, service stream {}:{}", cur_task.tr_class.get_a(), cur_task.tr_class.get_str_new_desc(), cur_task.tr_class.get_new_e2d2(), cur_task.tr_class.get_str_end_desc(), cur_task.tr_class.get_end_e2d2());

        //let mut results = BTreeMap::new();
        for v in 1..cur_task.v + 1 {
            print!("v={}", v);
            let start = Instant::now();
            let (avg, dev) = sim::simulation(v, cur_task.tr_class, args.mim_state_cntr, args.no_of_series);
            let duration = start.elapsed();
            let pefromance =  (avg.no_of_events * args.no_of_series as f64) / duration.as_micros() as f64;
            println!(" performance {:.3} events/µs, no of events : {} ", pefromance, avg.no_of_events.round().separated_string());
            results.add(v, avg, dev);
        }
        results.write(&mut file);
    }
    */

    println!("Done");
    Ok(())
}

