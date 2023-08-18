#![allow(unused_variables)]
mod architecture;
use architecture::*;

use std::collections::HashMap;

struct ZerosGenerator;

impl ZerosGenerator {
    fn new() -> Self {
        ZerosGenerator
    }
}

struct ZerosGeneratorProps;

impl TryFrom<&HashMap<&str, &Data>> for ZerosGeneratorProps {
    type Error = ();

    fn try_from(value: &HashMap<&str, &Data>) -> Result<Self, Self::Error> {
        Ok(ZerosGeneratorProps)
    }
}

struct ZerosGeneratorOutput(u8);

impl TryFrom<ZerosGeneratorOutput> for HashMap<&str, Data> {
    type Error = ();

    fn try_from(value: ZerosGeneratorOutput) -> Result<Self, Self::Error> {
        Ok(HashMap::from([("output", Data::U8(0))]))
    }
}

impl ZerosGenerator {
    fn run(&self, props: ZerosGeneratorProps) -> Result<ZerosGeneratorOutput, ()> {
        Ok(ZerosGeneratorOutput(0))
    }
}

impl Transformer for ZerosGenerator {
    fn inputs(&self) -> &[&str] {
        &[]
    }

    fn outputs(&self) -> &[&str] {
        &["output"]
    }

    fn process(&self, val: &HashMap<&str, &Data>) -> Result<HashMap<&str, Data>, ()> {
        self.run(val.try_into()?)?.try_into()
    }
}

struct AddOne;

struct AddOneProps(u8);

impl TryFrom<&HashMap<&str, &Data>> for AddOneProps {
    type Error = ();

    fn try_from(value: &HashMap<&str, &Data>) -> Result<Self, Self::Error> {
        match value["input"] {
            Data::U8(val) => Ok(AddOneProps(*val)),
        }
    }
}

struct AddOneOutput(u8);

impl TryFrom<AddOneOutput> for HashMap<&str, Data> {
    type Error = ();

    fn try_from(value: AddOneOutput) -> Result<Self, Self::Error> {
        Ok(HashMap::from([("output", Data::U8(value.0))]))
    }
}

impl AddOne {
    fn run(&self, props: AddOneProps) -> Result<AddOneOutput, ()> {
        Ok(AddOneOutput(props.0 + 1))
    }
}

impl AddOne {
    fn new() -> Self {
        AddOne
    }
}

impl Transformer for AddOne {
    fn inputs(&self) -> &[&str] {
        &["input"]
    }

    fn outputs(&self) -> &[&str] {
        &["output"]
    }

    fn process(&self, val: &HashMap<&str, &Data>) -> Result<HashMap<&str, Data>, ()> {
        self.run(val.try_into()?)?.try_into()
    }
}

fn main() -> Result<(), ()> {
    let mut orchestrator = Orchestrator::new()
        .add("source", ZerosGenerator::new())?
        .add("doubler0", AddOne::new())?
        .connect(
            Path::new("source", "output"),
            Path::new("doubler0", "input"),
        )?
        .add("doubler1", AddOne::new())?
        .connect(
            Path::new("doubler0", "output"),
            Path::new("doubler1", "input"),
        )?
        .add("doubler2", AddOne::new())?
        .connect(
            Path::new("doubler1", "output"),
            Path::new("doubler2", "input"),
        )?
        .add("doubler3", AddOne::new())?
        .connect(
            Path::new("doubler2", "output"),
            Path::new("doubler3", "input"),
        )?
        .add("doubler4", AddOne::new())?
        .connect(
            Path::new("doubler3", "output"),
            Path::new("doubler4", "input"),
        )?;

    let result = (0..1000000)
        .map(|_| orchestrator.step().expect("step"))
        .count();

    println!("{:?}", result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_success() {
        assert!(Orchestrator::new()
            .add("source1", ZerosGenerator::new())
            .expect("cannot insert")
            .add("source2", ZerosGenerator::new())
            .is_ok())
    }

    #[test]
    fn test_add_same_name() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot insert")
            .add("source", ZerosGenerator::new())
            .is_err())
    }

    #[test]
    fn test_connect_success() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot add source")
            .add("doubler", AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new("source", "output"), Path::new("doubler", "input"))
            .is_ok())
    }

    #[test]
    fn test_connect_missing_destination_source() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot add source")
            .add("doubler", AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new("source", "output"), Path::new("pippo", "pippo"))
            .is_err())
    }

    #[test]
    fn test_connect_missing_destination_value() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot add source")
            .add("doubler", AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new("source", "output"), Path::new("doubler", "pippo"))
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_source() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot add source")
            .add("doubler", AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new("pippo", "pippo"), Path::new("doubler", "input"))
            .is_err())
    }

    #[test]
    fn test_connect_missing_source_value() {
        assert!(Orchestrator::new()
            .add("source", ZerosGenerator::new())
            .expect("cannot add source")
            .add("doubler", AddOne::new())
            .expect("cannot add doubler")
            .connect(Path::new("source", "pippo"), Path::new("doubler", "input"))
            .is_err())
    }
}
