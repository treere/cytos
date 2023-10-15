use clap::Command;
use easy_repl::{command, CommandStatus, Repl};
use proph::loader::Registry;

use proph_transformers::{
    AddValue, GrayScale, ImageDecoder, IncrementalGenerator, Mean, Print, Rscam,
};

fn load_registry() -> Registry {
    Registry::default()
        .add("IncrementalGenerator", IncrementalGenerator::default)
        .add("AddValue", AddValue::default)
        .add("Rscam", Rscam::default)
        .add("ImageDecoder", ImageDecoder::default)
        .add("ImageGrayScale", GrayScale::default)
        .add("ImageMean", Mean::default)
        .add("PrintU64", Print::<u64>::default)
        .add("PrintF64", Print::<f64>::default)
}

struct Status {
    x: u8,
}

fn main() -> Result<(), String> {
    let _loader = load_registry();

    let mut status = Status { x: 1 };

    let _matches = Command::new("repl")
        .about("proph repl")
        .version("0.0.1")
        .author("Treere")
        .get_matches();

    Repl::builder()
        .add(
            "hello",
            command! {
                "Say hello",
                (name: String) => |name| {
                    status.x += 1;
                    println!("Hello {}! -- {}", name, status.x);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .add(
            "add",
            command! {
                "Add X to Y",
                (X:i32, Y:i32) => |x, y| {
                    status.x += 1;
                    println!("{} + {} = {} -- {}", x, y, x + y, status.x);
                    Ok(CommandStatus::Done)
                }
            },
        )
        .build()
        .expect("Failed to create repl")
        .run()
        .expect("Critical REPL error");

    Ok(())
}
