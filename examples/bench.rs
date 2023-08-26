use proph::{
    architecture::{Graph, NodeId},
    transformer::{
        AddValue, AddValueConfigInput, AddValueConfigOutput, IncrementalGenerator,
        IncrementalGeneratorConfigOutput,
    },
    utils::time_execution,
};

pub const SOURCE: NodeId = 1;
pub const DOUBLER0: NodeId = 2;
pub const DOUBLER1: NodeId = 3;
pub const DOUBLER2: NodeId = 4;
pub const DOUBLER3: NodeId = 5;
pub const DOUBLER4: NodeId = 6;

fn main() -> Result<(), String> {
    let mut orchestrator = Graph::new()
        .add(SOURCE, IncrementalGenerator::new())?
        .add(DOUBLER0, AddValue::new())?
        .add(DOUBLER1, AddValue::new())?
        .add(DOUBLER2, AddValue::new())?
        .add(DOUBLER3, AddValue::new())?
        .add(DOUBLER4, AddValue::new())?
        .connect(
            (SOURCE, IncrementalGeneratorConfigOutput::OUTPUT),
            (DOUBLER0, AddValueConfigInput::INPUT),
        )?
        .connect(
            (DOUBLER0, AddValueConfigOutput::OUTPUT),
            (DOUBLER1, AddValueConfigInput::INPUT),
        )?
        .connect(
            (DOUBLER1, AddValueConfigOutput::OUTPUT),
            (DOUBLER2, AddValueConfigInput::INPUT),
        )?
        .connect(
            (DOUBLER2, AddValueConfigOutput::OUTPUT),
            (DOUBLER3, AddValueConfigInput::INPUT),
        )?
        .connect(
            (DOUBLER3, AddValueConfigOutput::OUTPUT),
            (DOUBLER4, AddValueConfigInput::INPUT),
        )?;

    let steps = 100000000;
    println!("running {} steps", steps);
    orchestrator.step().expect("step");

    {
        // let value = orchestrator.param_value(DOUBLER4).unwrap();
        // let value = value.get::<u64>(&AddValueConfigOutput::OUTPUT).unwrap();

        // println!("first step value {:?}", value);
        // assert_eq!(*value, 5);
    }

    let seconds = time_execution(|| {
        for _ in 0..steps {
            orchestrator.step().expect("")
        }
    });

    println!("{} seconds.", seconds);
    println!("{} step/seconds", steps as f64 / seconds);
    println!("{} seconds/steps", seconds / steps as f64);

    {
        // let value = orchestrator.param_value(DOUBLER4).unwrap();
        // let value = value.get::<u64>(&AddValueConfigOutput::OUTPUT).unwrap();

        // println!("first step value {:?}", value);
        // assert_eq!(*value, 100000005);
    }

    Ok(())
}
