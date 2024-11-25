extern crate cytos_derive;

mod add_one;
mod decoder;
mod encoder;
mod imageops;
mod incremental;
mod mean;
mod print;
mod source;
mod timer;
mod types;

use add_one::AddValue;
use cytos::loader::DynamicLoadingRegistryWrapper;
use decoder::ImageDecoder;
use incremental::IncrementalGenerator;
use mean::Mean;
use print::Print;
use source::Rscam;
use timer::Timer;

#[no_mangle]
pub extern "C" fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("IncrementalGenerator", IncrementalGenerator::default)
        .add("AddValue", AddValue::default)
        .add("Rscam", Rscam::default)
        .add("ImageDecoder", ImageDecoder::default)
        .add("ImageMean", Mean::default)
        .add("PrintU64", Print::<u64>::default)
        .add("PrintF64", Print::<f64>::default)
        .add("PrintString", Print::<String>::default)
        .add("Timer", Timer::default)
        .add("Blur", imageops::Blur::default)
        .add("FastBlur", imageops::FastBlur::default)
        .add("Brighten", imageops::Brighten::default)
        .add("Contrast", imageops::Contrast::default)
        .add("Filter3x3", imageops::Filter3x3::default)
        .add("Unsharpen", imageops::Unsharpen::default)
        .add("Save", imageops::Unsharpen::default);
}
