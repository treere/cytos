use proph::architecture::{InputProp, Result, Stepper};
use proph_derive::ProphNode;

use crate::decoder::Image;

#[derive(ProphNode, Default)]
pub struct Save {
    input: InputProp<Image>,
    filename: InputProp<String>,
}

impl Stepper for Save {
    fn step(&mut self) -> Result<()> {
        self.input
            .image
            .save(&*self.filename)
            .map_err(|x| x.to_string())?;
        Ok(())
    }
}
