use std::fs::ReadDir;

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

    fn output(&self, val: ParamId) -> Option<GenericProp> {
        match val {
            ListDirConfigOutput::FILE => Some(self.file.as_generic()),
            _ => None,
        }
    }

    fn link(&mut self, _name: ParamId, _val: GenericProp) -> Result<(), &'static str> {
        Err("missing param")
    }
}
