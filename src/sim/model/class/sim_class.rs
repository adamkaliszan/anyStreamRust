use std::cmp::Ordering;
use rand::prelude::Distribution;
use rand_distr::{Exp, Uniform, Gamma, Pareto};
use rand::Rng;
use approx::relative_ne;
use std::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::ser::SerializeStruct;
use crate::sim::model::class;

use crate::sim::model::class::{Class, StreamType, StreamOfEvents};
use crate::sim::model::class::utils;

#[derive(Clone, Copy)]
pub struct SimClass {
    pub tr_class: Class,
    arrival_stream   : StreamOfEvents,
    service_stream   : StreamOfEvents,
}

impl SimClass {
    pub fn new(new_stream_type: StreamType, end_stream_type: StreamType,
               new_int: f64, new_e2_d2: f64,
               end_int: f64, end_e2_d2: f64 ) -> Option<Self> {
        let (arrival_mean, arrival_variance) = utils::get_e_d(new_int, new_e2_d2);
        let (service_mean, service_variance) = utils::get_e_d(end_int, end_e2_d2);

        let arrival_str_opt = Self::try_get_stream(new_stream_type, arrival_mean, arrival_variance);
        let service_str_opt = Self::try_get_stream(end_stream_type, service_mean, service_variance);

        match (arrival_str_opt, service_str_opt) {
            (Some(arrival_str), Some(service_str)) =>
                Some (SimClass {
                    tr_class: Class::new(new_stream_type, end_stream_type, new_int, new_e2_d2, end_int, end_e2_d2),
                    arrival_stream: arrival_str,
                    service_stream: service_str,
                }),
            _ => None
        }
    }

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

    #[allow(dead_code)]
    pub fn get_a(&self) -> f64 {
        self.tr_class.a
    }

    #[allow(dead_code)]
    pub fn get_str_new_id(&self) -> u32  {
        Class::get_str_id(&self.tr_class.arrival_stream_type)
    }

    #[allow(dead_code)]
    pub fn get_str_new_desc(&self) -> &str {
        Class::get_str_desc(&self.tr_class.arrival_stream_type)
    }

    #[allow(dead_code)]
    pub fn get_new_e2d2(&self) -> f64 {
        self.tr_class.arrival_e2d2
    }

    #[allow(dead_code)]
    pub fn get_str_end_id(&self) -> u32 {
        Class::get_str_id(&self.tr_class.service_stream_type)
    }

    #[allow(dead_code)]
    pub fn get_str_end_desc(&self) -> &str {
        Class::get_str_desc(&self.tr_class.service_stream_type)
    }

    #[allow(dead_code)]
    pub fn get_end_e2d2(&self) -> f64 {
        self.tr_class.service_e2d2
    }
}

impl Serialize for SimClass {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("TrClass", 5)?;
        state.serialize_field("a", &self.tr_class.a)?;
        state.serialize_field("arrival_str_type", &self.tr_class.arrival_stream_type)?;
        state.serialize_field("arrival_e2d2", &self.tr_class.arrival_e2d2)?;
        state.serialize_field("service_stream_type", &self.tr_class.service_stream_type)?;
        state.serialize_field("service_e2d2", &self.tr_class.service_e2d2)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for SimClass {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        enum Field { A, NewStreamType, EndStreamType, NewE2D2, EndE2D2 }
        // This part could also be generated independently by:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        //    enum Field { Secs, Nanos }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where
                    D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`a`, `New_stream_type`, `End_stream_type`, `New_e2_d2`, `End_e2_d2`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where
                            E: de::Error,
                    {
                        match value {
                            "a" => Ok(Field::A),
                            "New_stream_type" => Ok(Field::NewStreamType),
                            "End_stream_type" => Ok(Field::EndStreamType),
                            "New_e2_d2" => Ok(Field::NewE2D2),
                            "End_e2_d2" => Ok(Field::EndE2D2),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ClassVisitor;

        impl<'de> Visitor<'de> for ClassVisitor {
            type Value = SimClass;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Class")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<SimClass, V::Error>
                where
                    V: SeqAccess<'de>,
            {
                let a = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let arrival_stream_type = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let arrival_e2d2 = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let service_stream_type = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let service_e2d2 = seq.next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                Ok(SimClass::new(arrival_stream_type, service_stream_type, a, arrival_e2d2, 1.0, service_e2d2).
                    ok_or_else(|| de::Error::custom(format_args!("Failed to create object"))))?
            }

            fn visit_map<V>(self, mut map: V) -> Result<SimClass, V::Error>
                where
                    V: MapAccess<'de>,
            {
                let mut a = None;
                let mut new_stream_type = None;
                let mut end_stream_type = None;
                let mut new_e2_d2 = None;
                let mut end_e2_d2 = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::A => {
                            if a.is_some() {
                                return Err(de::Error::duplicate_field("a"));
                            }
                            a = Some(map.next_value()?);
                        }
                        Field::NewStreamType => {
                            if new_stream_type.is_some() {
                                return Err(de::Error::duplicate_field("new_stream_type"));
                            }
                            new_stream_type = Some(map.next_value()?);
                        }
                        Field::EndStreamType => {
                            if end_stream_type.is_some() {
                                return Err(de::Error::duplicate_field("end_stream_type"));
                            }
                            end_stream_type = Some(map.next_value()?);
                        }
                        Field::NewE2D2 => {
                            if new_e2_d2.is_some() {
                                return Err(de::Error::duplicate_field("new_e2_d2"));
                            }
                            new_e2_d2 = Some(map.next_value()?);
                        }
                        Field::EndE2D2 => {
                            if end_e2_d2.is_some() {
                                return Err(de::Error::duplicate_field("end_e2_d2"));
                            }
                            end_e2_d2 = Some(map.next_value()?);
                        }
                    }
                }
                let a: f64 = a.ok_or_else(|| de::Error::missing_field("a"))?;
                let new_stream_type: StreamType = new_stream_type.ok_or_else(|| de::Error::missing_field("new_stream_type"))?;
                let end_stream_type: StreamType = end_stream_type.ok_or_else(|| de::Error::missing_field("end_stream_type"))?;
                let new_e2_d2: f64 = new_e2_d2.ok_or_else(|| de::Error::missing_field("new_e2_d2"))?;
                let end_e2_d2: f64 = end_e2_d2.ok_or_else(|| de::Error::missing_field("end_e2_d2"))?;

                Ok(SimClass::new(new_stream_type, end_stream_type, a, new_e2_d2, 1.0, end_e2_d2).
                    ok_or_else(|| de::Error::custom(format_args!("Failed to create object"))))?
            }
        }

        const FIELDS: &'static [&'static str] = &["a", "new_stream_type", "end_stream_type", "new_e2_d2", "end_e2_d2"];
        deserializer.deserialize_struct("SimClass", FIELDS, ClassVisitor)
    }
}
