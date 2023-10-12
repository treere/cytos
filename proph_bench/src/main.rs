use proph::loader::Loader;
use proph::{loader, utils::execution_time};
use proph_transformers::{AddValue, IncrementalGenerator};
use std::env;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), String> {
    let configuration = {
        let filename = env::args().nth(1).expect("missing file");

        let mut configuration = String::new();

        File::open(filename)
            .expect("cannot open file")
            .read_to_string(&mut configuration)
            .expect("cannot read");
        configuration
    };

    let loader = Loader::default()
        .add("IncrementalGenerator", IncrementalGenerator::default)
        .add("AddValue", AddValue::default);

    let mut orchestrator = loader::GraphRepr::load(&configuration, &loader)?;

    let steps = 2000000000;
    println!("running {} steps", steps);
    orchestrator.step().expect("step");

    let seconds = execution_time(|| {
        for _ in 0..steps {
            orchestrator.step().unwrap()
        }
    });

    println!("{} seconds.", seconds);
    println!("{} step/seconds", steps as f64 / seconds);
    println!("{} seconds/steps", seconds / steps as f64);

    Ok(())
}
