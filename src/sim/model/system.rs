use core::cmp::Ordering;
use serde::{Deserialize, Serialize};
use crate::sim::model::class::Class;

#[derive(PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct ModelDescription {
    pub class: Class,
    pub v: usize
}

impl Ord for ModelDescription {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.partial_cmp(other) {
            None => Ordering::Less,
            Some(result) => result
        }
    }
    fn max(self, other: Self) -> Self {
        match self.cmp(&other) {
            Ordering::Equal => self,
            Ordering::Greater => self,
            Ordering::Less => other
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
        if self.cmp(&min) != Ordering::Greater {
            min
        }
        else if self.cmp(&max) != Ordering::Less {
            max
        }
        else {
            self
        }
    }
}

impl PartialOrd for ModelDescription {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match self.class.partial_cmp(&other.class) {
            Some(val) => match val {
                Ordering::Equal => Some(self.v.cmp(&other.v)),
                _ => Some(val)
            }
            None => None
        }
    }
}