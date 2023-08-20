#![allow(unused_variables)]
mod architecture;

mod data;
mod map;
mod transformer;

use architecture::NodeId;

use data::Data;

use std::time::Instant;

use crate::{
    architecture::Orchestrator,
    transformer::{
        AddOne, AddOneConfigInput, AddOneConfigOutput, IncrementalGenerator,
        IncrementalGeneratorConfigOutput,
    },
};

pub const SOURCE: NodeId = 1;
pub const DOUBLER0: NodeId = 2;
pub const DOUBLER1: NodeId = 3;
pub const DOUBLER2: NodeId = 4;
pub const DOUBLER3: NodeId = 5;
pub const DOUBLER4: NodeId = 6;

fn main() -> Result<(), ()> {
    let mut orchestrator = Orchestrator::new()
        .add(SOURCE, IncrementalGenerator::new())?
        .add(DOUBLER0, AddOne::new())?
        .add(DOUBLER1, AddOne::new())?
        .add(DOUBLER2, AddOne::new())?
        .add(DOUBLER3, AddOne::new())?
        .add(DOUBLER4, AddOne::new())?
        .connect(
            (SOURCE, IncrementalGeneratorConfigOutput::OUTPUT),
            (DOUBLER0, AddOneConfigInput::INPUT),
        )?
        .connect(
            (DOUBLER0, AddOneConfigOutput::OUTPUT),
            (DOUBLER1, AddOneConfigInput::INPUT),
        )?
        .connect(
            (DOUBLER1, AddOneConfigOutput::OUTPUT),
            (DOUBLER2, AddOneConfigInput::INPUT),
        )?
        .connect(
            (DOUBLER2, AddOneConfigOutput::OUTPUT),
            (DOUBLER3, AddOneConfigInput::INPUT),
        )?
        .connect(
            (DOUBLER3, AddOneConfigOutput::OUTPUT),
            (DOUBLER4, AddOneConfigInput::INPUT),
        )?;

    let steps = 10000000;
    println!("running {} steps", steps);
    orchestrator.step().expect("step");

    {
        let value = orchestrator
            .value(DOUBLER4, AddOneConfigOutput::OUTPUT)
            .unwrap();
        println!("first step value {:?}", *value);
        match *value {
            Data::U64(5) => (),
            _ => unreachable!("error here"),
        }
    }

    let now = Instant::now();

    let _result = (0..steps)
        .map(|_| orchestrator.step().expect("step"))
        .count();

    let elapsed_time = now.elapsed();
    println!("{} seconds.", elapsed_time.as_secs_f64());

    {
        let value = orchestrator
            .value(DOUBLER4, AddOneConfigOutput::OUTPUT)
            .unwrap();
        println!("final value {:?}", *value);
        match *value {
            Data::U64(10000005) => (),
            _ => unreachable!("error here"),
        }
    }

    println!("{} step/seconds", steps as f64 / elapsed_time.as_secs_f64());
    println!(
        "{} seconds/steps",
        elapsed_time.as_secs_f64() / steps as f64
    );

    Ok(())
}
