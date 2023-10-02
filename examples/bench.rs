use proph::{loader, transformer::AddValueConfigOutput, utils::time_execution};
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

    let mut orchestrator = loader::Graph::load(&configuration)?;

    let steps = 1000000000;
    println!("running {} steps", steps);
    orchestrator.step().expect("step");

    {
        let value = orchestrator.param_value("DOUBLER4".to_owned()).unwrap();
        value
            .output(AddValueConfigOutput::OUTPUT.to_owned())
            .unwrap()
            .try_read::<u64>(|value| {
                println!("first step value {:?}", value);
                assert_eq!(*value, 6);
            })
            .unwrap();
    }

    let seconds = time_execution(|| {
        for _ in 0..steps {
            orchestrator.step().unwrap()
        }
    });

    println!("{} seconds.", seconds);
    println!("{} step/seconds", steps as f64 / seconds);
    println!("{} seconds/steps", seconds / steps as f64);

    {
        let value = orchestrator.param_value("DOUBLER4".to_owned()).unwrap();
        value
            .output(AddValueConfigOutput::OUTPUT.to_owned())
            .unwrap()
            .try_read::<u64>(|value| {
                println!("final value {:?}", value);
                assert_eq!(*value, 1000000006);
            })
            .unwrap();
    }

    Ok(())
}
