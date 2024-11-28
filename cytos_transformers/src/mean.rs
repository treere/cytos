use cytos::{Prop, Result, Stepper};
use cytos_derive::CytosNode;

use crate::decoder::Image;

#[derive(CytosNode, Default)]
pub struct Mean {
    #[input]
    input: Prop<Image>,

    #[output]
    output: Prop<f64>,
}

impl Stepper for Mean {
    fn step(&mut self) -> Result<()> {
        let sum = self
            .input
            .image
            .pixels()
            .fold(0u64, |a, b| u64::from(b[0]) + a) as f64;

        *self.output = sum / f64::from(self.input.image.width() * self.input.image.height());
        Ok(())
    }
}
