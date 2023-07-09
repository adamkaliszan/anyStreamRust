use std::cmp::Ordering;
use approx::relative_ne;
use rand_distr::{Exp, Gamma, Pareto, Uniform};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use float_cmp::{ApproxEq, F64Margin};

pub mod utils;
pub mod sim_class;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
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
#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct Class
{
    /// Traffic offered per system, capacity doesn't matter
    a : f64,

    /// Service intensity of single call
    mu: f64,

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

impl PartialOrd for Class {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        if self.arrival_stream_type > other.arrival_stream_type {
            return Some(Ordering::Greater);
        }
        if self.arrival_stream_type < other.arrival_stream_type {
            return Some(Ordering::Less);
        }

        if self.service_stream_type > other.service_stream_type {
            return Some(Ordering::Greater);
        }
        if self.service_stream_type < other.service_stream_type {
            return Some(Ordering::Less);
        }

        if self.arrival_e2d2 > other.arrival_e2d2 {
            return Some(Ordering::Greater);
        }
        if self.arrival_e2d2 < other.arrival_e2d2 {
            return Some(Ordering::Less);
        }

        if self.service_e2d2 > other.service_e2d2 {
            return Some(Ordering::Greater);
        }
        if self.service_e2d2 < other.service_e2d2 {
            return Some(Ordering::Less);
        }

        if self.a > other.a {
            return Some(Ordering::Greater);
        }
        if self.a < other.a {
            return Some(Ordering::Less);
        }

        if self.eq(other) {
            return Some(Ordering::Equal);
        }
        None
    }
}

impl Ord for Class {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            Some(res) => res,
            None => Ordering::Equal
        }
    }

    fn max(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Equal => self,
            Ordering::Less => other,
            Ordering::Greater => self
        }
    }

    fn min(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Equal => self,
            Ordering::Less => self,
            Ordering::Greater => other
        }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        if self.cmp(&min) == Ordering::Less {
            return min;
        }
        if self.cmp(&max) == Ordering::Greater {
            return max;
        }
        self
    }
}

impl Eq for Class {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        if self.arrival_stream_type != other.arrival_stream_type { return false; }
        if self.service_stream_type != other.service_stream_type { return false; }

        if !self.a.approx_eq(other.a, F64Margin::default()) { return false; }
        if !self.arrival_e2d2.approx_eq(other.arrival_e2d2, F64Margin::default()) { return false; }
        if !self.service_e2d2.approx_eq(other.service_e2d2, F64Margin::default()) { return false; }

        true
    }

    fn ne(&self, other: &Self) -> bool {
        self.arrival_stream_type != other.arrival_stream_type ||
        self.service_stream_type != other.service_stream_type ||
        !self.a.approx_eq(other.a, F64Margin::default()) ||
        !self.arrival_e2d2.approx_eq(other.arrival_e2d2, F64Margin::default()) ||
        !self.service_e2d2.approx_eq(other.service_e2d2, F64Margin::default())
    }
}

impl Class {
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

    #[allow(dead_code)]
    pub fn try_get_stream(str_type: StreamType, mean: f64, variance: f64) -> Option<StreamOfEvents> {
        let new_int = 1f64 / mean;
        return match str_type {
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
               end_int: f64, end_e2_d2: f64 ) -> Self {
        Class {
            a: new_int / end_int,
            mu: end_int,
            arrival_stream_type: new_stream_type,
            arrival_e2d2: new_e2_d2,
            service_stream_type: end_stream_type,
            service_e2d2: end_e2_d2
        }
    }

    pub fn get_a(&self) -> f64 {
        self.a
    }

    pub fn get_new_intensity(&self) -> f64 { self.a/self.mu }

    pub fn get_end_intensity(&self) -> f64 { self.mu }

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
