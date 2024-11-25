use clap::{value_parser, Arg, Command};
use cytos::loader::Registry;
use cytos::repr::GraphRepr;
use cytos::utils::execution_time;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), String> {
    let mut loader = Registry::default();

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
        .arg(Arg::new("library").short('l'))
        .get_matches();

    if let Some(library) = matches.get_one::<String>("library") {
        loader
            .load_library(dbg!(library))
            .map_err(|e| e.to_string())?;
    }

    let steps = *matches.get_one::<u64>("steps").expect("missing steps");

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

    let graph = GraphRepr::from_json(&configuration).map_err(|r| r.to_string())?;
    let mut graph = graph.into_graph(&loader).map_err(|r| r.to_string())?;

    graph.initialize().map_err(|e| e.to_string())?;

    println!("running {steps} steps");
    graph.step().map_err(|e| e.to_string())?;

    let seconds = execution_time(|| {
        for _ in 0..steps {
            graph.step().expect("step");
        }
    });

    println!("{seconds} seconds.");
    println!("{} step/seconds", steps as f64 / seconds);
    println!("{} seconds/steps", seconds / steps as f64);

    Ok(())
}
