#![allow(unused_variables)]
mod architecture;

mod data;
mod map;
mod transformer;

use architecture::{NodeId, ParamId};

use data::Data;

use std::time::Instant;

use crate::{
    architecture::{Orchestrator, Path},
    transformer::{AddOne, IncrementalGenerator},
};

pub const INPUT: ParamId = 0;
pub const OUTPUT: ParamId = 1;

pub const SOURCE: NodeId = 1;
pub const DOUBLER0: NodeId = 2;
pub const DOUBLER1: NodeId = 3;
pub const DOUBLER2: NodeId = 4;
pub const DOUBLER3: NodeId = 5;
pub const DOUBLER4: NodeId = 6;

fn main() -> Result<(), ()> {
    let mut orchestrator = Orchestrator::new()
        .add(SOURCE, IncrementalGenerator::Module::new())?
        .add(DOUBLER0, AddOne::Module::new())?
        .connect(
            Path::new(SOURCE, IncrementalGenerator::Config::OUTPUT),
            Path::new(DOUBLER0, AddOne::Config::INPUT),
        )?
        .add(DOUBLER1, AddOne::Module::new())?
        .connect(
            Path::new(DOUBLER0, AddOne::Config::OUTPUT),
            Path::new(DOUBLER1, AddOne::Config::INPUT),
        )?
        .add(DOUBLER2, AddOne::Module::new())?
        .connect(
            Path::new(DOUBLER1, AddOne::Config::OUTPUT),
            Path::new(DOUBLER2, AddOne::Config::INPUT),
        )?
        .add(DOUBLER3, AddOne::Module::new())?
        .connect(
            Path::new(DOUBLER2, AddOne::Config::OUTPUT),
            Path::new(DOUBLER3, AddOne::Config::INPUT),
        )?
        .add(DOUBLER4, AddOne::Module::new())?
        .connect(
            Path::new(DOUBLER3, AddOne::Config::OUTPUT),
            Path::new(DOUBLER4, AddOne::Config::INPUT),
        )?;

    let steps = 100000000;
    println!("running {} steps", steps);
    orchestrator.step().expect("step");

    {
        let value = orchestrator.value(DOUBLER4, OUTPUT);
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
        let value = orchestrator.value(DOUBLER4, OUTPUT);
        println!("final value {:?}", *value);
        match *value {
            Data::U64(100000005) => (),
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
