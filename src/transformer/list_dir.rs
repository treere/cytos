use std::{fs::ReadDir, rc::Rc};

use crate::architecture::{GenericProp, OutputProp, ParamId, Transformer};

#[allow(non_snake_case)]
pub mod ListDirConfigOutput {
    use crate::architecture::ParamId;

    pub const FILE: ParamId = "file";
}

pub struct ListDir {
    reader: ReadDir,
    file: OutputProp<&'static str>,
}

impl ListDir {
    pub fn new(dir: String) -> Self {
        Self {
            reader: std::fs::read_dir(dir).unwrap(),
            file: OutputProp::new(""),
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

    fn input(&self, _val: ParamId) -> Option<Rc<GenericProp>> {
        None
    }

    fn output(&self, val: ParamId) -> Option<Rc<GenericProp>> {
        match val {
            ListDirConfigOutput::FILE => Some(self.file.get_any()),
            _ => None,
        }
    }

    fn set_input(&mut self, name: ParamId, val: Rc<GenericProp>) -> Result<(), &'static str> {
        match name {
            ListDirConfigOutput::FILE => self.file.change_value(val),
            _ => unreachable!(),
        }
    }
}
