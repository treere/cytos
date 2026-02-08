//! Signal processing transformer nodes for Cytos.
//!
//! This module provides nodes for signal filtering and processing operations.
//! Currently includes a Markov filter for threshold-based signal smoothing.

use cytos::{Prop, Result, Stepper, loader::DynamicLoadingRegistryWrapper, props::Ownable};
use cytos_derive::CytosNode;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Display;

/// A Markov filter node for threshold-based signal smoothing.
///
/// This node implements a hysteresis filter that helps prevent rapid
/// oscillations when a signal hovers near a threshold. The output stays
/// `true` until the input drops below `low`, and stays `false` until
/// the input rises above `high`.
///
/// This is useful for debouncing signals or creating stable on/off
/// transitions from noisy analog inputs.
#[derive(CytosNode, Default)]
struct MarkovFilter<T>
where
    T: Ownable + Display + Default + DeserializeOwned + PartialOrd + Serialize + 'static,
{
    /// The input signal value to filter
    #[cytos(input)]
    input: Prop<T>,

    /// The lower threshold for switching to `false`
    #[cytos(input)]
    low: Prop<T>,

    /// The upper threshold for switching to `true`
    #[cytos(input)]
    high: Prop<T>,

    /// The filtered boolean output
    #[cytos(output)]
    output: Prop<bool>,
}

impl<T: Ownable + Display + Default + DeserializeOwned + Serialize + PartialOrd + 'static> Stepper
    for MarkovFilter<T>
{
    fn step(&mut self) -> Result<()> {
        if *self.output {
            *self.output = *self.input > *self.high;
        } else {
            *self.output = *self.low < *self.input;
        }
        Ok(())
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry.add("MarkovFilterF32", MarkovFilter::<f32>::default);
}
