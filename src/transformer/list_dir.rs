use std::fs::ReadDir;

use crate::architecture::{
    new_shared, InputConfiguration, OutputConfiguration, ParamId, Params, Results, SharedData,
    SharedType, Transformer,
};

#[allow(non_snake_case)]
pub mod ListDirConfigOutput {
    use crate::architecture::ParamId;

    pub const FILE: ParamId = 1;
}

pub struct ListDir {
    reader: ReadDir,
}

impl ListDir {
    pub fn new(dir: String) -> Self {
        Self {
            reader: std::fs::read_dir(dir).unwrap(),
        }
    }
}

impl InputConfiguration for ListDir {
    fn inputs(&self) -> &[ParamId] {
        &[]
    }

    fn input_default(&self, _val: ParamId) -> SharedData {
        unreachable!()
    }
}

impl SharedType for String {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl OutputConfiguration for ListDir {
    fn outputs(&self) -> &[ParamId] {
        &[ListDirConfigOutput::FILE]
    }

    fn output_default(&self, val: ParamId) -> SharedData {
        match val {
            ListDirConfigOutput::FILE => new_shared("".to_owned()),
            _ => unreachable!(),
        }
    }
}

impl Transformer for ListDir {
    fn process(&mut self, _inputs: Params, mut outputs: Results) -> Result<(), &'static str> {
        if let Some(Ok(_file)) = self.reader.next() {
            let mut output = outputs.get_mut(&(ListDirConfigOutput::FILE)).ok_or("BB")?;
            *output.as_any_mut().downcast_mut::<String>().ok_or("A")? = "pippo".to_owned();
            Ok(())
        } else {
            Err("no files")
        }
    }
}
