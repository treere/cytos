use crate::decoder::Image;
use proph::architecture::{InputProp, OutputProp, Result, Stepper};
use proph_derive::ProphNode;

#[derive(ProphNode, Default)]
pub struct Blur {
    input: InputProp<Image>,
    sigma: InputProp<f32>,

    output: OutputProp<Image>,
}

impl Stepper for Blur {
    fn step(&mut self) -> Result<()> {
        *self.output = Image {
            image: image::imageops::blur(&self.input.image, *self.sigma),
        };

        Ok(())
    }
}

#[derive(ProphNode, Default)]
pub struct FastBlur {
    input: InputProp<Image>,
    sigma: InputProp<f32>,

    output: OutputProp<Image>,
}

impl Stepper for FastBlur {
    fn step(&mut self) -> Result<()> {
        *self.output = Image {
            image: image::imageops::fast_blur(&self.input.image, *self.sigma),
        };

        Ok(())
    }
}

#[derive(ProphNode, Default)]
pub struct Filter3x3 {
    input: InputProp<Image>,
    kernel: InputProp<Vec<f32>>,

    output: OutputProp<Image>,
}

impl Stepper for Filter3x3 {
    fn step(&mut self) -> Result<()> {
        *self.output = Image {
            image: image::imageops::filter3x3(&self.input.image, &*self.kernel),
        };

        Ok(())
    }
}

#[derive(ProphNode, Default)]
pub struct Unsharpen {
    input: InputProp<Image>,
    sigma: InputProp<f32>,
    threshold: InputProp<i32>,

    output: OutputProp<Image>,
}

impl Stepper for Unsharpen {
    fn step(&mut self) -> Result<()> {
        *self.output = Image {
            image: image::imageops::unsharpen(&self.input.image, *self.sigma, *self.threshold),
        };

        Ok(())
    }
}
