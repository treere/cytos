use clap::{value_parser, Arg, Command};
use proph::architecture::graph::GraphRepr;
use proph::loader::Registry;
use proph::utils::execution_time;
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), String> {
    let loader = Registry::default();

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
    let (_, mut graph) = graph.to_graph(&loader).map_err(|r| r.to_string())?;

    graph.initialize().expect("cannot initialize");

    println!("running {steps} steps");
    graph.step().expect("step");

    let seconds = execution_time(|| {
        for _ in 0..steps {
            graph.step().unwrap();
        }
    });

    println!("{seconds} seconds.");
    println!("{} step/seconds", steps as f64 / seconds);
    println!("{} seconds/steps", seconds / steps as f64);

    Ok(())
}
