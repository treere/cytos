extern crate proph_derive;

mod add_one;
mod decoder;
mod incremental;
mod mean;
mod print;
mod source;
mod transform;

pub use add_one::AddValue;
pub use decoder::{ImageDecoder, ZuneImageDecoder};
pub use incremental::IncrementalGenerator;
pub use mean::Mean;
pub use print::Print;
use proph::loader::Registry;
pub use source::Rscam;
pub use transform::GrayScale;

#[no_mangle]
pub extern "C" fn load_registry(registry: &mut Registry) {
    registry
        .add("IncrementalGenerator", IncrementalGenerator::default)
        .add("AddValue", AddValue::default)
        .add("Rscam", Rscam::default)
        .add("ImageDecoder", ImageDecoder::default)
        .add("ZuneImageDecoder", ZuneImageDecoder::default)
        .add("ImageGrayScale", GrayScale::default)
        .add("ImageMean", Mean::default)
        .add("PrintU64", Print::<u64>::default)
        .add("PrintF64", Print::<f64>::default);
}
