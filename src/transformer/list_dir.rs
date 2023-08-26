use std::{any::Any, fs::ReadDir, rc::Rc};

use crate::architecture::{ParamId, Prop, Transformer};

#[allow(non_snake_case)]
pub mod ListDirConfigOutput {
    use crate::architecture::ParamId;

    pub const FILE: ParamId = "file";
}

pub struct ListDir {
    reader: ReadDir,
    file: Prop<&'static str>,
}

impl ListDir {
    pub fn new(dir: String) -> Self {
        Self {
            reader: std::fs::read_dir(dir).unwrap(),
            file: Prop::new(""),
        }
    }
}

impl Transformer for ListDir {
    fn step(&mut self) -> Result<(), &'static str> {
        if let Some(Ok(_file)) = self.reader.next() {
            *self.file.set() = "pippo";
            Ok(())
        } else {
            Err("no files")
        }
    }

    fn inputs_name(&self) -> &[ParamId] {
        &[]
    }

    fn input(&self, _val: ParamId) -> Rc<dyn Any> {
        unreachable!()
    }
    fn outputs_name(&self) -> &[ParamId] {
        &[ListDirConfigOutput::FILE]
    }

    fn output(&self, val: ParamId) -> Rc<dyn Any> {
        match val {
            ListDirConfigOutput::FILE => self.file.get_any(),
            _ => unreachable!(),
        }
    }

    fn set_input(&mut self, name: ParamId, val: Rc<dyn Any>) -> Result<(), &'static str> {
        match name {
            ListDirConfigOutput::FILE => self.file.change_value(val),
            _ => unreachable!(),
        }
    }
}
