use proph::{loader, transformer::AddValueConfigOutput, utils::time_execution};

fn main() -> Result<(), String> {
    let mut orchestrator = loader::Graph::load(include_str!("bench.json"))?;

    let steps = 100000000;
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
                assert_eq!(*value, 100000006);
            })
            .unwrap();
    }

    Ok(())
}
