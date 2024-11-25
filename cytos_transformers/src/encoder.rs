use cytos::architecture::{Prop, Result, Stepper};
use cytos_derive::CytosNode;

use crate::decoder::Image;

#[derive(CytosNode, Default)]
pub struct Save {
    #[input]
    input: Prop<Image>,

    #[input]
    filename: Prop<String>,
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
