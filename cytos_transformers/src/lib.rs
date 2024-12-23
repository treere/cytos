extern crate cytos_derive;

mod add_one;
mod classifier;
mod decoder;
mod encoder;
mod face_detection;
mod imageops;
mod incremental;
mod mean;
mod print;
mod sleep;
mod source;
mod timer;
mod types;
mod web_sender;

use cytos::loader::DynamicLoadingRegistryWrapper;

#[no_mangle]
pub extern "C" fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add(
            "IncrementalGenerator",
            incremental::IncrementalGenerator::default,
        )
        .add("BinaryClassifier", classifier::BinaryClassifier::default)
        .add("AddValue", add_one::AddValue::default)
        .add("RscamSource", source::Rscam::default)
        .add("FileSource", source::File::default)
        .add("ImageDecoder", decoder::ImageDecoder::default)
        .add("ImageMean", mean::Mean::default)
        .add("PrintU64", print::Print::<u64>::default)
        .add("PrintF64", print::Print::<f64>::default)
        .add("PrintString", print::Print::<String>::default)
        .add("Timer", timer::Timer::default)
        .add("Blur", imageops::Blur::default)
        .add("FastBlur", imageops::FastBlur::default)
        .add("Brighten", imageops::Brighten::default)
        .add("Contrast", imageops::Contrast::default)
        .add("Filter3x3", imageops::Filter3x3::default)
        .add("Unsharpen", imageops::Unsharpen::default)
        .add("FaceDetection", face_detection::FaceDetection::default)
        .add("Sleep", sleep::Sleep::default)
        .add("Save", imageops::Unsharpen::default)
        .add("WebSenderU64", web_sender::WebSender::<u64>::default);
}
