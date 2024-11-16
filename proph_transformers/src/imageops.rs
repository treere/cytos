use crate::decoder::Image;
use proph::architecture::{InputProp, OutputProp, Result, Stepper};
use proph_derive::ProphNode;

macro_rules! define_function {
    ($struct_name:ident, $image_ops_func:path, $param_type:ty) => {
        #[derive(ProphNode, Default)]
        pub struct $struct_name {
            input: InputProp<Image>,
            param: InputProp<$param_type>,

            output: OutputProp<Image>,
        }

        impl Stepper for $struct_name {
            fn step(&mut self) -> Result<()> {
                *self.output = Image {
                    image: ($image_ops_func)(&self.input.image, *self.param),
                };

                Ok(())
            }
        }
    };

    ($struct_name:ident, $image_ops_func:path, $param1_type:ty, $param2_type:ty) => {
        #[derive(ProphNode, Default)]
        pub struct $struct_name {
            input: InputProp<Image>,
            param1: InputProp<$param1_type>,
            param2: InputProp<$param2_type>,

            output: OutputProp<Image>,
        }

        impl Stepper for $struct_name {
            fn step(&mut self) -> Result<()> {
                *self.output = Image {
                    image: ($image_ops_func)(&self.input.image, *self.param1, *self.param2),
                };

                Ok(())
            }
        }
    };
}

define_function!(Blur, image::imageops::blur, f32);
define_function!(FastBlur, image::imageops::fast_blur, f32);
define_function!(Brighten, image::imageops::brighten, i32);
define_function!(Contrast, image::imageops::contrast, f32);
define_function!(Unsharpen, image::imageops::unsharpen, f32, i32);

#[derive(ProphNode, Default)]
pub struct Filter3x3 {
    input: InputProp<Image>,
    kernel: InputProp<Vec<f32>>,

    output: OutputProp<Image>,
}

impl Stepper for Filter3x3 {
    fn step(&mut self) -> Result<()> {
        *self.output = Image {
            image: image::imageops::filter3x3(&self.input.image, &self.kernel),
        };

        Ok(())
    }
}
