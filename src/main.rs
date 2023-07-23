extern crate separator;
extern crate flot;

use btreemultimap::BTreeMultiMap;
use cartesian::*;
use clap::{Parser, Subcommand};
use confy;
use mongodb::{bson::doc, options::{ClientOptions, ServerApi, ServerApiVersion, Credential}, sync::Client};
use semver::Op::Less;
use semver::{BuildMetadata, Prerelease, Version, VersionReq};
use separator::Separatable;
use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};
use std::collections::{BTreeMap, LinkedList};
use std::env::args;
use std::fs::File;
use std::str::FromStr;
use std::thread;
use std::thread::{JoinHandle};
use std::time::Instant;
use clap::ValueHint::CommandString;
use mongodb::options::CredentialBuilder;

use crate::sim::model::class::{Class, StreamType, sim_class::SimClass};
use crate::sim::model::system::ModelDescription;
use crate::sim::simulator::simulations_statistics::{SimStatisticsMultiV, StatisticsMultiSimulations};
use crate::sim::simulator::single_statistics::{Macrostate, StatisticsFinalized, StatisticsRunExperiment};

mod sim;

const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Parser)]
#[clap(author, version, about)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
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
    #[clap(long, default_value="uniform")]
    call_stream: Vec<String>,

    /// Service stream type
    #[clap(long, default_value="poisson")]
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
    threads_no: u32,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Save results (training data) to CSV. Each row contains following capacities: 1, 2, 3, ..., v
    OutCsv {
        /// Output filename
        #[clap(short, long, default_value="results.txt")]
        output_path: std::path::PathBuf,
    },
    ConfigureMongo {
        /// Mongo URI
        #[clap(long, default_value="mongodb://192.168.1.39")]
        mongo_uri: String,

        /// Mongo database name
        #[clap(long, default_value="anystream")]
        mongo_database: String,
    }
}

#[derive(Serialize, Deserialize)]
struct MyConfig {
    mongo_uri: String,
    mongo_database: String,
}

impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self {
            mongo_uri: "192.168.1.39".into(),
            mongo_database: "anystream".into()
        }
    }
}

struct SimulationTask {
    tr_class: SimClass,
    v : usize,
    mim_state_cntr: u32,
    sim_no: u32
}

fn mongo_open_database(mongo_uri: &String, mongo_db: &String) -> Option<mongodb::sync::Database> {
    let mut client_options = match ClientOptions::parse(mongo_uri) {
        Ok(co) => co,
        Err(e) => {
            println!("Failed to parse mongo URI \"{mongo_uri}\". Mongo DB is disabled: {e}");
            return None;
        }
    };

    //let mut credential = Credential::builder().build();
    //credential.username = Some(str!("adam"));
    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    //client_options.credential = Some(credential);
    // Create a new client and connect to the server
    let client = match Client::with_options(client_options) {
        Ok(cl) => cl,
        Err(e) => {
            println!("Failed to create mongo client: {e}. Mongo DB is disabled");
            return None;
        }
    };

    // Send a ping to confirm a successful connection
    let db = client.database(mongo_db);

    match db.run_command(doc! {"ping": 1}, None) {
        Ok(_) => {
            println!("Pinged your deployment. You successfully connected to MongoDB!");
            println!("All OK");
            Some(db)
        }
        Err(e) => {
            println!("Failed to ping DB {e}");
            None
        }
    }
}
fn read_finilized_statistics(model: &ModelDescription, db: &mongodb::sync::Database, min_no_of_events_per_state: u32) -> LinkedList<StatisticsFinalized> {
    let stats = StatisticsFinalized::read_mongo(model, db);
    stats.into_iter()
        .filter(|itm| if let Ok(_ver) = Version::parse(itm.metadata.version.as_str()) {
            let req = VersionReq::parse(">=0.3.0").unwrap();
            req.matches(&_ver)
        } else {false })
        .filter(|itm| itm.metadata.min_no_of_events_per_state >= min_no_of_events_per_state)
        .collect()
}

fn generate_ML_results(results: BTreeMultiMap<ModelDescription, StatisticsFinalized>) -> BTreeMap<Class, SimStatisticsMultiV> {
    let mut final_results: BTreeMap<Class, SimStatisticsMultiV> = BTreeMap::new();

    for (key, values) in results {
        let mut map_item = final_results.get_mut(&key.class);
        match  map_item {
            Some(itm) => {
                let stat_signel_v = StatisticsMultiSimulations::statistics_proc(&values.into_iter().collect(), key.v);
                itm.results.push_back(stat_signel_v);
            },
            None => {
                let mut aggregated_result:SimStatisticsMultiV = SimStatisticsMultiV::new(key.class);
                let stat_signel_v = StatisticsMultiSimulations::statistics_proc(&values.into_iter().collect(), key.v);
                aggregated_result.results.push_back(stat_signel_v);
                final_results.insert(key.class, aggregated_result);
            }
        }
    }
    final_results
}

fn generate_ml_csv(filename: &std::path::PathBuf, v:u32, results: &BTreeMap<Class, SimStatisticsMultiV>) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    SimStatisticsMultiV::write_header(v, &mut file);

    for (key, value) in results {
        println!("Writing to file statistics for {:?}", key);
        value.write(&mut file);
    }

    println!("Done");

    Ok(())
}

/// Prepares tasks.
/// First check if the results are available in database
fn prepare_tasks(args: &Cli, db: &Option<mongodb::sync::Database>, results: &mut BTreeMultiMap<ModelDescription, StatisticsFinalized>) -> LinkedList<SimulationTask> {
    let mut tasks: LinkedList<SimulationTask> = LinkedList::new();

    let call_streams = args.call_stream.clone();
    let serv_streams = args.serv_stream.clone();
    let cs_e2_d2_col:Vec<f64> = flot::range(args.cs_e2_d2_min, args.cs_e2_d2_max + args.cs_e2_d2_delta, args.cs_e2_d2_delta).collect();
    let ss_e2_d2_col:Vec<f64> = flot::range(args.ss_e2_d2_min, args.ss_e2_d2_max + args.ss_e2_d2_delta, args.ss_e2_d2_delta).collect();
    let a_col: Vec<f64> = flot::range(args.a_min, args.a_max + args.a_delta, args.a_delta).collect();

    let mut no_off_skipped_classes = 0;
    let mut no_off_stored_tasks_before = 0;
    let mut no_off_total_tasks_before = 0;

    for (cur_call_stream, cur_serv_stream, cs_e2_d2, ss_e2_d2, a) in
    cartesian!(call_streams.iter(), serv_streams.iter(), cs_e2_d2_col.iter(), ss_e2_d2_col.iter(), a_col.iter())
    {
        // Prepare Streams and write its params
        let call_stream = StreamType::from_str(&cur_call_stream.to_lowercase()).expect("Failed");
        let service_stream = StreamType::from_str(&cur_serv_stream.to_lowercase()).expect("Failed");

        if let Some(tr_class) = SimClass::new(
            call_stream, service_stream,
            *a, *cs_e2_d2, 1f64, *ss_e2_d2) {

            for v in 1..args.v + 1 {
                let model = ModelDescription{v:v as usize, class:tr_class.tr_class.clone()};
                let mut sim_experiments = match &db {
                    Some(_db) => read_finilized_statistics(&model, &_db, args.mim_state_cntr),
                    None => LinkedList::new()
                };

                let no_of_ready_statistics= sim_experiments.len();
                for sim_result in sim_experiments {
                    results.insert(ModelDescription{class: tr_class.tr_class, v: v as usize}, sim_result);
                }

                no_off_stored_tasks_before += no_of_ready_statistics;
                no_off_total_tasks_before += args.no_of_series;

                for sim_no in no_of_ready_statistics..args.no_of_series {
                    tasks.push_back(SimulationTask {
                        tr_class: tr_class,
                        v: v as usize,
                        mim_state_cntr: args.mim_state_cntr,
                        sim_no: (sim_no - no_of_ready_statistics) as u32
                    });
                }
            }
        } else {
            no_off_skipped_classes += 1;
        }
    }
    println!("Number od tasks to do: {}, number of stored (skipped) tasks {}", tasks.len(), no_off_stored_tasks_before);

    tasks
}

fn calculate(no_of_threads:u32, mut tasks: LinkedList<SimulationTask>, db: &mut Option<mongodb::sync::Database>, results: &mut BTreeMultiMap<ModelDescription, StatisticsFinalized>) {
    let mut workers: LinkedList <JoinHandle<(ModelDescription, StatisticsFinalized)>> = LinkedList::new();
    while !tasks.is_empty() {
        let mut task_no = 0;
        while task_no < no_of_threads && !tasks.is_empty() {
            let cur_task = tasks.pop_front().unwrap();
            workers.push_back(thread::spawn(move || {
                println!("Simulation a={}, arrival stream {}:{}, service stream {}:{}", cur_task.tr_class.get_a(), cur_task.tr_class.get_str_new_desc(), cur_task.tr_class.get_new_e2d2(), cur_task.tr_class.get_str_end_desc(), cur_task.tr_class.get_end_e2d2());
                let start = Instant::now();
                let result = sim::simulation(cur_task.v, cur_task.tr_class, cur_task.mim_state_cntr);
                let duration = start.elapsed();
                let pefromance = (result.no_of_events as f64) / duration.as_micros() as f64;
                println!("v={}: performance {:.3} events/µs, no of events : {} ", cur_task.v, pefromance, result.no_of_events);
                (ModelDescription{class:cur_task.tr_class.tr_class, v: cur_task.v }, result)
            }));
            task_no += 1;
        }

        while !workers.is_empty() {
            let single_worker = workers.pop_front().unwrap();
            let (key, value) = single_worker.join().unwrap();

            if let Some(db_val) = db {
                let str = serde_json::to_string(&value).unwrap();
                println!("Serialized: {str}");
                if let Err(err) = value.write_mongo(&key, db_val) {
                    println!("Failed to save results: {err}");
                }
            }
            results.insert(key, value);
        }
    }
}

fn main() -> std::io::Result<()>
{
    let args = Cli::parse();
    let mut cfg = match confy::load(env!("CARGO_PKG_NAME")) {
        Ok(config) => config,
        Err(e) => {
            println!("Failed to load config: {e}");
            MyConfig::default()
        }
    };

    match &args.command {
        Some(Commands::OutCsv {output_path}) => {
            let mut db: Option<mongodb::sync::Database> = mongo_open_database(&cfg.mongo_uri, &cfg.mongo_database);
            let mut results: BTreeMultiMap<ModelDescription, StatisticsFinalized> = BTreeMultiMap::new();

            let mut tasks = prepare_tasks(&args, &db, &mut results);
            calculate(args.threads_no, tasks, &mut db, &mut results);

            let final_results = generate_ML_results(results);

            generate_ml_csv(output_path, args.v, &final_results)
        }
        Some(Commands::ConfigureMongo {mongo_uri, mongo_database}) => {
            cfg.mongo_uri = mongo_uri.to_string();
            cfg.mongo_database = mongo_database.to_string();
            confy::store(env!("CARGO_PKG_NAME"), cfg)
        }
        None => {
            Ok(())
        }
    }
}

