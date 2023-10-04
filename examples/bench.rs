use proph::transformer::{AddValue, IncrementalGenerator, Loader};
use proph::{loader, utils::time_execution};
use std::env;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), String> {
    let configuration = {
        let filename = env::args().nth(1).expect("missing file");

        let mut configuration = String::new();
        File::open(&filename)
            .expect("cannot open file")
            .read_to_string(&mut configuration)
            .expect("cannot read");
        configuration
    };

    let loader = Loader::new()
        .add("IncrementalGenerator", IncrementalGenerator::new)
        .add("AddValue", AddValue::new);

    let mut orchestrator = loader::Graph::load(&configuration, &loader)?;

    let steps = 2000000000;
    println!("running {} steps", steps);
    orchestrator.step().expect("step");

    let seconds = time_execution(|| {
        for _ in 0..steps {
            orchestrator.step().unwrap()
        }
    });

    println!("{} seconds.", seconds);
    println!("{} step/seconds", steps as f64 / seconds);
    println!("{} seconds/steps", seconds / steps as f64);

    Ok(())
}
