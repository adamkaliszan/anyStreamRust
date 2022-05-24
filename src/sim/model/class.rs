use rand::prelude::Distribution;
use rand::Rng;
use rand_distr::{Exp, Uniform, Gamma, Pareto};
//use crate::sim::model::class::utils::*;

mod utils;

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

    pub fn new(new_stream_type: StreamType, end_stream_type: StreamType,
               new_int: f64, new_e2_d2: f64,
               end_int: f64, end_e2_d2: f64 ) -> Self {

        let result = Self {
            arrival_stream       : match new_stream_type {
                StreamType::Poisson =>
                    StreamOfEvents::Poisson(Exp::new(new_int).unwrap()),
                StreamType::Uniform => {
                    let (min, max) = utils::uniform_gen_min_max(new_int, new_e2_d2);
                    StreamOfEvents::Uniform(Uniform::new(min, max))
                },
                StreamType::Gamma => {
                    let (scale, shape) = utils::gamma_get_scale_shape(new_int, new_e2_d2);
                    StreamOfEvents::Gamma(Gamma::new(scale, shape).unwrap())
                },
                StreamType::Pareto => {
                    let (scale, shape) = utils::pareto_get_scale_shape(new_int, new_e2_d2);
                    StreamOfEvents::Pareto(Pareto::new(scale, shape).unwrap())
                }
            },
            service_stream       : match end_stream_type {
                StreamType::Poisson =>
                    StreamOfEvents::Poisson(Exp::new(end_int).unwrap()),
                StreamType::Uniform => {
                    let (min, max) = utils::uniform_gen_min_max(end_int, end_e2_d2);
                    StreamOfEvents::Uniform(Uniform::new(min, max))
                },
                StreamType::Gamma => {
                    let (scale, shape) = utils::gamma_get_scale_shape(end_int, end_e2_d2);
                    StreamOfEvents::Gamma(Gamma::new(scale, shape).unwrap())
                },
                StreamType::Pareto => {
                    let (scale, shape) = utils::pareto_get_scale_shape(end_int, end_e2_d2);
                    StreamOfEvents::Pareto(Pareto::new(scale, shape).unwrap())
                }
            },
            a : new_int / end_int,
            arrival_stream_type: new_stream_type,
            arrival_e2d2       : new_e2_d2,
            service_stream_type: end_stream_type,
            service_e2d2       : end_e2_d2,
        };
        result
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