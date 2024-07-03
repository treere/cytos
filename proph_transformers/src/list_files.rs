use std::fs::ReadDir;

use proph::architecture::{InputProp, OutputProp, Stepper};
use proph_derive::TransFn;

#[derive(TransFn, Default)]
pub struct ListFiles {
    input: InputProp<String>,
    output: OutputProp<String>,
    read_dir: Option<ReadDir>,
}

impl Stepper for ListFiles {
    fn step(&mut self) -> proph::architecture::Result<()> {
        if let Some(entry) = self.read_dir.as_mut().and_then(|r| r.next()) {
            let file_name = entry.unwrap().file_name();

            *self.output.set() = file_name.to_str().unwrap().to_owned();
            Ok(())
        } else {
            Err("cannot list dir")
        }
    }

    fn initialize(&mut self) -> proph::architecture::Result<()> {
        let r = std::fs::read_dir(self.input.get()).or(Err(""))?;
        self.read_dir.replace(r);
        Ok(())
    }
}
