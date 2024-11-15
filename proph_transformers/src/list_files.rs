use std::fs::ReadDir;

use proph::architecture::{InputProp, OutputProp, Stepper};
use proph_derive::ProphNode;

#[derive(ProphNode, Default)]
pub struct ListFiles {
    input: InputProp<String>,
    output: OutputProp<String>,
    read_dir: Option<ReadDir>,
}

impl Stepper for ListFiles {
    fn step(&mut self) -> proph::architecture::Result<()> {
        if let Some(entry) = self.read_dir.as_mut().and_then(std::iter::Iterator::next) {
            let file_name = entry.map_err(|c| c.to_string())?.file_name();

            file_name
                .to_str()
                .ok_or("cannot convert to string")?
                .clone_into(&mut self.output);
            Ok(())
        } else {
            Err("cannot list dir".into())
        }
    }

    fn initialize(&mut self) -> proph::architecture::Result<()> {
        let r = std::fs::read_dir(&*self.input).or(Err(""))?;
        self.read_dir.replace(r);
        Ok(())
    }
}
