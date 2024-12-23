use cytos::{Prop, Stepper};
use cytos_derive::CytosNode;

use crate::decoder::Image;

#[derive(CytosNode, Default)]
pub struct BinaryClassifier {
    #[input]
    image: Prop<Image>,

    #[output]
    prediction: Prop<f32>,
}

impl Stepper for BinaryClassifier {
    fn step(&mut self) -> cytos::Result<()> {
        Ok(())
    }
}
