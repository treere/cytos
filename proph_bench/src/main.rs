use clap::{value_parser, Arg, Command};
use proph::loader::{GraphRepr, Registry};
use proph::utils::execution_time;
use proph_transformers::{AddValue, ImageDecoder, IncrementalGenerator, Rscam};
use std::fs::File;
use std::io::Read;

fn load_registry() -> Registry {
    Registry::default()
        .add("IncrementalGenerator", IncrementalGenerator::default)
        .add("AddValue", AddValue::default)
        .add("Rscam", Rscam::default)
        .add("ImageDecoder", ImageDecoder::default)
}

fn main() -> Result<(), String> {
    let loader = load_registry();

    let matches = Command::new("bench")
        .about("benchmark a configuration")
        .version("0.0.1")
        .arg_required_else_help(true)
        .author("Treere")
        .arg(Arg::new("config_file"))
        .arg(
            Arg::new("steps")
                .short('s')
                .default_value("10")
                .value_parser(value_parser!(u64)),
        )
        .get_matches();

    let steps = matches
        .get_one::<u64>("steps")
        .expect("missing steps")
        .clone();

    let configuration = {
        let filename = matches
            .get_one::<String>("config_file")
            .expect("missing file");

        let mut configuration = String::new();

        File::open(filename)
            .expect("cannot open file")
            .read_to_string(&mut configuration)
            .expect("cannot read");
        configuration
    };

    let mut orchestrator = GraphRepr::load(&configuration, &loader)?;

    orchestrator.initialize().expect("cannot initialize");

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
