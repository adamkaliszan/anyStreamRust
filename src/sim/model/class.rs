use rand::prelude::Distribution;
use rand::Rng;
use rand_distr::{Exp, Uniform, Gamma, Pareto};
//use crate::sim::model::class::utils::*;

mod utils;

use std::str::FromStr;
use approx::relative_ne;
use float_cmp::*;
use crate::sim::model::class::StreamType::Poisson;

#[derive(Clone, Copy)]
pub enum StreamType {
    Poisson,
    Uniform,
    Gamma,
    Pareto
}

#[derive(Clone, Copy)]
pub enum StreamOfEvents {
    Poisson (Exp<f64>),
    Uniform (Uniform<f64>),
    Gamma (Gamma<f64>),
    Pareto (Pareto<f64>),
}

#[derive(Clone, Copy)]
pub struct Class
{
    arrival_stream   : StreamOfEvents,
    service_stream   : StreamOfEvents,
    a : f64,
    arrival_stream_type: StreamType,
    arrival_e2d2       : f64,
    service_stream_type: StreamType,
    service_e2d2       : f64
}

impl std::fmt::Debug for StreamOfEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Class{
    pub fn get_str_id(stram_type : &StreamType) -> u32 {
        match stram_type {
            StreamType::Poisson => 0,
            StreamType::Uniform => 1,
            StreamType::Gamma   => 2,
            StreamType::Pareto  => 3
        }
    }

    pub fn get_str_desc(stram_type : &StreamType) -> &str {
        match stram_type {
            StreamType::Poisson => "Poisson",
            StreamType::Uniform => "Uniform",
            StreamType::Gamma   => "Gamma",
            StreamType::Pareto  => "Pareto"
        }
    }

    pub fn try_get_stream(strType: StreamType, mean: f64, variance: f64) -> Option<StreamOfEvents> {
        let new_int = 1f64 / mean;
        return match strType {
            StreamType::Poisson => {
                if relative_ne!(variance, 1f64/(new_int*new_int)) { return None; }

                let distrib = Exp::new(new_int);
                match distrib {
                    Ok(some_distrib) => Some(StreamOfEvents::Poisson(some_distrib)),
                    Err(e) => {
                        println!("{}: failed to create Poisson stream with λ = {}", e, new_int);
                        None
                    }
                }
            },
            StreamType::Uniform => {
                let (min, max) = utils::uniform_gen_min_max(new_int, variance);
                if min < 0f64 {
                    println!("Wrong parameters in Uniform distribution. Minimum value is < 0");
                    None
                }
                else {
                    Some(StreamOfEvents::Uniform(Uniform::new(min, max)))
                }
            },
            StreamType::Gamma => {
                let (scale, shape) = utils::gamma_get_scale_shape(new_int, variance);
                match Gamma::new(scale, shape) {
                    Ok(some_distrib) => Some(StreamOfEvents::Gamma(some_distrib)),
                    Err(e) => {
                        println!("{}: failed to create Gamma with Ex = {} and D = {}", e, mean, variance);
                        None
                    }
                }
            },
            StreamType::Pareto => {
                let (scale, shape) = utils::pareto_get_scale_shape(new_int, variance);
                match Pareto::new(scale, shape) {
                    Ok(some_distrib) => Some(StreamOfEvents::Pareto(some_distrib)),
                    Err(e) => {
                        println!("{}: failed to create Pareto with Ex = {} and D = {}", e, mean, variance);
                        None
                    }
                }
            }
        }
    }

    pub fn new(new_stream_type: StreamType, end_stream_type: StreamType,
                   new_int: f64, new_e2_d2: f64,
                   end_int: f64, end_e2_d2: f64 ) -> Option<Self> {
        let (arrival_mean, arrival_variance) = utils::get_e_d(new_int, new_e2_d2);
        let (service_mean, service_variance) = utils::get_e_d(end_int, end_e2_d2);

        let arrival_str_opt = Self::try_get_stream(new_stream_type, arrival_mean, arrival_variance);
        let service_str_opt = Self::try_get_stream(end_stream_type, service_mean, service_variance);

        match (arrival_str_opt, service_str_opt) {
            (Some(arrival_str), Some(service_str)) =>
                Some (Class {
                    arrival_stream: arrival_str,
                    service_stream: service_str,
                    a: new_int / end_int,
                    arrival_stream_type: new_stream_type,
                    arrival_e2d2: new_e2_d2,
                    service_stream_type: end_stream_type,
                    service_e2d2: end_e2_d2
                }),
            _ => None
        }
    }

    pub fn get_a(&self) -> f64 {
        self.a
    }

    pub fn get_str_new_id(&self) -> u32 {
        Class::get_str_id(&self.arrival_stream_type)
    }

    pub fn get_str_new_desc(&self) -> &str {
        Class::get_str_desc(&self.arrival_stream_type)
    }

    pub fn get_new_e2d2(&self) -> f64 {
        self.arrival_e2d2
    }

    pub fn get_str_end_id(&self) -> u32 {
        Class::get_str_id(&self.service_stream_type)
    }

    pub fn get_str_end_desc(&self) -> &str {
        Class::get_str_desc(&self.service_stream_type)
    }

    pub fn get_end_e2d2(&self) -> f64 {
        self.service_e2d2
    }

    pub fn get_time_new_call<R: Rng + ?Sized>(&self,rng: &mut R) -> f64 {
        match &self.arrival_stream {
            StreamOfEvents::Poisson(distr)=> distr.sample(rng),
            StreamOfEvents::Uniform(distr)=> distr.sample(rng),
            StreamOfEvents::Gamma(distr)=> distr.sample(rng),
            StreamOfEvents::Pareto(distr)=> distr.sample(rng),
            //_ => panic!("Not supported distraibution {:?}", self.arrival_stream)
        }
    }

    pub fn get_time_end_call<R: Rng + ?Sized>(&self,rng: &mut R) -> f64 {
        match &self.service_stream {
            StreamOfEvents::Poisson(distr)=> distr.sample(rng),
            StreamOfEvents::Uniform(distr)=> distr.sample(rng),
            StreamOfEvents::Gamma(distr)=> distr.sample(rng),
            StreamOfEvents::Pareto(distr)=> distr.sample(rng),
            //_ => panic!("Not supported distraibution {:?}", self.service_stream)
        }
    }
}

impl FromStr for StreamType {
    type Err = ();
    fn from_str(input: &str) -> Result<StreamType, Self::Err> {
        match input {
            "poisson" => Ok(StreamType::Poisson),
            "uniform" => Ok(StreamType::Uniform),
            "gamma"   => Ok(StreamType::Gamma),
            "pareto"  => Ok(StreamType::Pareto),
            _         => Err(()),
        }
    }
}