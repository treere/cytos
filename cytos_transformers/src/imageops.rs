use crate::decoder::Image;
use cytos::{Prop, Result, Stepper};
use cytos_derive::CytosNode;

macro_rules! define_function {
    ($struct_name:ident, $image_ops_func:path, $param_type:ty) => {
        #[derive(CytosNode, Default)]
        pub struct $struct_name {
            #[input]
            input: Prop<Image>,
            #[input]
            param: Prop<$param_type>,

            #[output]
            output: Prop<Image>,
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
        #[derive(CytosNode, Default)]
        pub struct $struct_name {
            #[input]
            input: Prop<Image>,
            #[input]
            param1: Prop<$param1_type>,
            #[input]
            param2: Prop<$param2_type>,

            #[output]
            output: Prop<Image>,
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

#[derive(CytosNode, Default)]
pub struct Filter3x3 {
    #[input]
    input: Prop<Image>,
    #[input]
    kernel: Prop<Vec<f32>>,
    #[output]
    output: Prop<Image>,
}

impl Stepper for Filter3x3 {
    fn step(&mut self) -> Result<()> {
        *self.output = Image {
            image: image::imageops::filter3x3(&self.input.image, &self.kernel),
        };

        Ok(())
    }
}
