use std::fs::ReadDir;

use crate::architecture::{GenericInputProp, GenericOutputProp, OutputProp, ParamId, Transformer};

#[allow(non_snake_case)]
pub mod ListDirConfigOutput {

    pub const FILE: &str = "file";
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

    fn output(&self, val: ParamId) -> Option<GenericOutputProp> {
        match val.as_str() {
            ListDirConfigOutput::FILE => Some(self.file.as_generic()),
            _ => None,
        }
    }

    fn input(&self, _val: ParamId) -> Option<GenericInputProp> {
        None
    }

    fn link(&mut self, _name: ParamId, _val: GenericOutputProp) -> Result<(), &'static str> {
        Err("missing param")
    }

    fn output_names(&self) -> Vec<ParamId> {
        vec![ListDirConfigOutput::FILE.to_owned()]
    }
}
