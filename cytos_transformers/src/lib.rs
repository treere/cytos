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

#[no_mangle]
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
