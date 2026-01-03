//! # Cytos Transformers
//!
//! This crate provides a collection of transformer nodes for the Cytos dataflow framework.
//! These nodes enable various functionalities including:
//!
//! - Image processing (decoding, saving, operations like blur, resize, crop)
//! - Logic operations (comparisons, boolean gates)
//! - Machine learning (face detection, YOLO object detection)
//! - Signal processing (printing, web sending)
//! - Time management (timers, rate limiters, sleep)
//! - Data sources (file reading, camera capture)
//!
//! Each module contains nodes that can be loaded into the Cytos registry for use in dataflow graphs.

#![deny(clippy::perf)]
#![deny(clippy::style)]
#![deny(clippy::complexity)]
#![deny(clippy::suspicious)]
#![deny(clippy::correctness)]
#![deny(clippy::nursery)]
#![deny(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_precision_loss)]

extern crate cytos_derive;

mod cmp;
mod imageio;
mod imageops;
mod logic;
mod ml;
mod print;
mod signal;
mod simple;
mod source;
mod time;
mod types;
mod web_sender;

use cytos::loader::DynamicLoadingRegistryWrapper;

/// Loads all transformer nodes from this crate into the provided registry.
///
/// This function registers nodes from all modules:
/// - `imageio`: Image decoding and saving
/// - `imageops`: Image manipulation operations
/// - `logic`: Boolean logic operations
/// - `cmp`: Comparison operations
/// - `ml`: Machine learning nodes (face detection, YOLO)
/// - `print`: Value printing nodes
/// - `signal`: Signal processing nodes
/// - `simple`: Basic arithmetic and generation nodes
/// - `source`: Data source nodes (files, cameras)
/// - `time`: Time-related nodes (timers, sleep)
/// - `web_sender`: HTTP request sending nodes
///
/// # Safety
/// This function is marked as unsafe due to the `no_mangle` extern "C" attribute,
/// but the implementation itself is safe.
#[unsafe(no_mangle)]
pub extern "C" fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    imageio::load_registry(registry);
    imageops::load_registry(registry);
    logic::load_registry(registry);
    cmp::load_registry(registry);
    ml::load_registry(registry);
    print::load_registry(registry);
    signal::load_registry(registry);
    simple::load_registry(registry);
    source::load_registry(registry);
    time::load_registry(registry);
    web_sender::load_registry(registry);
}
