extern crate proph_derive;

mod add_one;
mod decoder;
mod incremental;
mod list_files;
mod mean;
mod print;
mod source;
mod timer;
mod transform;

use add_one::AddValue;
use decoder::ImageDecoder;
use incremental::IncrementalGenerator;
use list_files::ListFiles;
use mean::Mean;
use print::Print;
use proph::loader::Registry;
use source::Rscam;
use timer::Timer;
use transform::GrayScale;

#[no_mangle]
pub extern "C" fn load_registry(registry: &mut Registry) {
    registry
        .add("IncrementalGenerator", IncrementalGenerator::default)
        .add("AddValue", AddValue::default)
        .add("Rscam", Rscam::default)
        .add("ImageDecoder", ImageDecoder::default)
        .add("ImageGrayScale", GrayScale::default)
        .add("ImageMean", Mean::default)
        .add("PrintU64", Print::<u64>::default)
        .add("PrintF64", Print::<f64>::default)
        .add("PrintString", Print::<String>::default)
        .add("ListFiles", ListFiles::default)
        .add("Timer", Timer::default);
}
