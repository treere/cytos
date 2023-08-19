#![allow(unused_variables)]
mod architecture;
mod consts;
use architecture::*;
use consts::*;

use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Instant};

struct ZerosGenerator;

impl ZerosGenerator {
    fn new() -> Self {
        ZerosGenerator
    }
}

impl Transformer for ZerosGenerator {
    fn inputs(&self) -> &[u64] {
        &[]
    }

    fn outputs(&self) -> &[u64] {
        &[OUTPUT]
    }

    fn process(
        &self,
        _inputs: &HashMap<u64, Rc<RefCell<Data>>>,
        outputs: &mut HashMap<u64, Rc<RefCell<Data>>>,
    ) -> Result<(), ()> {
        let p = outputs.get_mut(&OUTPUT).unwrap();
        let mut data = (**p).borrow_mut();

        *data = Data::U8(0);
        Ok(())
    }
}

struct AddOne;

impl AddOne {
    fn new() -> Self {
        AddOne
    }
}

impl Transformer for AddOne {
    fn inputs(&self) -> &[u64] {
        &[INPUT]
    }

    fn outputs(&self) -> &[u64] {
        &[OUTPUT]
    }

    fn process(
        &self,
        inputs: &HashMap<u64, Rc<RefCell<Data>>>,
        outputs: &mut HashMap<u64, Rc<RefCell<Data>>>,
    ) -> Result<(), ()> {
        match *inputs[&INPUT].borrow() {
            Data::None => Err(()),
            Data::U8(v) => {
                let p = outputs.get_mut(&OUTPUT).unwrap();
                let mut data = (**p).borrow_mut();

                *data = Data::U8(v + 1);

                Ok(())
            }
        }
    }
}

fn main() -> Result<(), ()> {
    let mut orchestrator = Orchestrator::new()
        .add(SOURCE, ZerosGenerator::new())?
        .add(DOUBLER0, AddOne::new())?
        .connect(Path::new(SOURCE, OUTPUT), Path::new(DOUBLER0, INPUT))?
        .add(DOUBLER1, AddOne::new())?
        .connect(Path::new(DOUBLER0, OUTPUT), Path::new(DOUBLER1, INPUT))?
        .add(DOUBLER2, AddOne::new())?
        .connect(Path::new(DOUBLER1, OUTPUT), Path::new(DOUBLER2, INPUT))?
        .add(DOUBLER3, AddOne::new())?
        .connect(Path::new(DOUBLER2, OUTPUT), Path::new(DOUBLER3, INPUT))?
        .add(DOUBLER4, AddOne::new())?
        .connect(Path::new(DOUBLER3, OUTPUT), Path::new(DOUBLER4, INPUT))?;

    let steps = 10000000;
    println!("running {} steps", steps);

    let now = Instant::now();

    let _result = (0..steps)
        .map(|_| orchestrator.step().expect("step"))
        .count();

    let elapsed_time = now.elapsed();
    println!("{} seconds.", elapsed_time.as_secs_f64());

    println!("{} step/seconds", steps as f64 / elapsed_time.as_secs_f64());
    println!(
        "{} seconds/steps",
        elapsed_time.as_secs_f64() / steps as f64
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_success() {
        assert!(Orchestrator::new()
            .add(SOURCE1, ZerosGenerator::new())
            .expect("cannot insert")
            .add(SOURCE2, ZerosGenerator::new())
            .is_ok())
    }

    #[test]
    fn test_add_same_name() {
        assert!(Orchestrator::new()
            .add(SOURCE, ZerosGenerator::new())
            .expect("cannot insert")
            .add(SOURCE, ZerosGenerator::new())
            .is_err())
    }

    #[test]
    fn test_connect_success() {
        assert!(Orchestrator::new()
            .add(SOURCE, ZerosGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new(SOURCE, OUTPUT), Path::new(DOUBLER, INPUT))
            .is_ok())
    }

    #[test]
    fn test_connect_missing_destination_source() {
        assert!(Orchestrator::new()
            .add(SOURCE, ZerosGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new(SOURCE, OUTPUT), Path::new(PIPPO, PIPPO))
            .is_err())
    }

    #[test]
    fn test_connect_missing_destination_value() {
        assert!(Orchestrator::new()
            .add(SOURCE, ZerosGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new(SOURCE, OUTPUT), Path::new(DOUBLER, PIPPO))
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_source() {
        assert!(Orchestrator::new()
            .add(SOURCE, ZerosGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new(PIPPO, PIPPO), Path::new(DOUBLER, INPUT))
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_value() {
        assert!(Orchestrator::new()
            .add(SOURCE, ZerosGenerator::new())
            .expect("cannot add source")
            .add(DOUBLER, AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new(SOURCE, PIPPO), Path::new(DOUBLER, INPUT))
            .is_err())
    }
}
