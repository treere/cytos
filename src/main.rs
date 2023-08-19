#![allow(unused_variables)]
mod architecture;
mod consts;
use architecture::*;
use consts::*;

use std::collections::HashMap;

struct ZerosGenerator;

impl ZerosGenerator {
    fn new() -> Self {
        ZerosGenerator
    }
}

struct ZerosGeneratorProps;

impl TryFrom<&HashMap<u64, &Data>> for ZerosGeneratorProps {
    type Error = ();

    fn try_from(value: &HashMap<u64, &Data>) -> Result<Self, Self::Error> {
        Ok(ZerosGeneratorProps)
    }
}

struct ZerosGeneratorOutput(u8);

impl TryFrom<ZerosGeneratorOutput> for HashMap<u64, Data> {
    type Error = ();

    fn try_from(value: ZerosGeneratorOutput) -> Result<Self, Self::Error> {
        Ok(HashMap::from([(OUTPUT, Data::U8(0))]))
    }
}

impl ZerosGenerator {
    fn run(&self, props: ZerosGeneratorProps) -> Result<ZerosGeneratorOutput, ()> {
        Ok(ZerosGeneratorOutput(0))
    }
}

impl Transformer for ZerosGenerator {
    fn inputs(&self) -> &[u64] {
        &[]
    }

    fn outputs(&self) -> &[u64] {
        &[OUTPUT]
    }

    fn process(&self, val: &HashMap<u64, &Data>) -> Result<HashMap<u64, Data>, ()> {
        self.run(val.try_into()?)?.try_into()
    }
}

struct AddOne;

struct AddOneProps(u8);

impl TryFrom<&HashMap<u64, &Data>> for AddOneProps {
    type Error = ();

    fn try_from(value: &HashMap<u64, &Data>) -> Result<Self, Self::Error> {
        match value[&INPUT] {
            Data::U8(val) => Ok(AddOneProps(*val)),
        }
    }
}

struct AddOneOutput(u8);

impl TryFrom<AddOneOutput> for HashMap<u64, Data> {
    type Error = ();

    fn try_from(value: AddOneOutput) -> Result<Self, Self::Error> {
        Ok(HashMap::from([(OUTPUT, Data::U8(value.0))]))
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
    fn inputs(&self) -> &[u64] {
        &[INPUT]
    }

    fn outputs(&self) -> &[u64] {
        &[OUTPUT]
    }

    fn process(&self, val: &HashMap<u64, &Data>) -> Result<HashMap<u64, Data>, ()> {
        self.run(val.try_into()?)?.try_into()
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

    let result = (0..1).map(|_| orchestrator.step().expect("step")).count();

    println!("steps done {:?}", result);

    let v = vec![
        std::rc::Rc::new(std::cell::RefCell::new(1)),
        std::rc::Rc::new(std::cell::RefCell::new(2)),
    ];
    {
        let h = HashMap::from([("a", v[0].borrow())]);
        let mut p = HashMap::from([("b", v[1].borrow_mut())]);

        let q = p.get_mut("b").unwrap();

        **q += 1;
    }

    {
        let h = HashMap::from([("a", v[0].borrow())]);
        let mut p = HashMap::from([("b", v[1].borrow_mut())]);

        let q = p.get_mut("b").unwrap();

        **q += 1;
    }

    for p in v.iter() {
        println!("{:?}", p);
    }

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
