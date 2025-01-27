extern crate cytos_derive;

mod imageio;
mod imageops;
mod ml;
mod print;
mod simple;
mod source;
mod time;
mod types;
mod web_sender;

use cytos::loader::DynamicLoadingRegistryWrapper;

#[no_mangle]
pub extern "C" fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    source::load_registry(registry);
    imageops::load_registry(registry);
    print::load_registry(registry);
    web_sender::load_registry(registry);
    time::load_registry(registry);
    simple::load_registry(registry);
    ml::load_registry(registry);
    imageio::load_registry(registry);
}
