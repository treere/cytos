use proph::{
    architecture::Graph,
    transformer::{
        AddValue, AddValueConfigInput, AddValueConfigOutput, IncrementalGenerator,
        IncrementalGeneratorConfigOutput,
    },
    utils::time_execution,
};

fn main() -> Result<(), String> {
    let mut orchestrator = Graph::new()
        .add("SOURCE", IncrementalGenerator::new())?
        .add("DOUBLER0", AddValue::new())?
        .add("DOUBLER1", AddValue::new())?
        .add("DOUBLER2", AddValue::new())?
        .add("DOUBLER3", AddValue::new())?
        .add("DOUBLER4", AddValue::new())?
        .connect(
            ("SOURCE", IncrementalGeneratorConfigOutput::OUTPUT),
            ("DOUBLER0", AddValueConfigInput::INPUT),
        )?
        .connect(
            ("DOUBLER0", AddValueConfigOutput::OUTPUT),
            ("DOUBLER1", AddValueConfigInput::INPUT),
        )?
        .connect(
            ("DOUBLER1", AddValueConfigOutput::OUTPUT),
            ("DOUBLER2", AddValueConfigInput::INPUT),
        )?
        .connect(
            ("DOUBLER2", AddValueConfigOutput::OUTPUT),
            ("DOUBLER3", AddValueConfigInput::INPUT),
        )?
        .connect(
            ("DOUBLER3", AddValueConfigOutput::OUTPUT),
            ("DOUBLER4", AddValueConfigInput::INPUT),
        )?;

    let steps = 100000000;
    println!("running {} steps", steps);
    orchestrator.step().expect("step");

    {
        let value = orchestrator.param_value("DOUBLER4").unwrap();
        value
            .output(&AddValueConfigOutput::OUTPUT)
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
        let value = orchestrator.param_value("DOUBLER4").unwrap();
        value
            .output(&AddValueConfigOutput::OUTPUT)
            .unwrap()
            .try_read::<u64>(|value| {
                println!("final value {:?}", value);
                assert_eq!(*value, 100000006);
            })
            .unwrap();
    }

    Ok(())
}
