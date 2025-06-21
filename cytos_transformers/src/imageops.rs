use crate::imageio::Image;
use cytos::{Prop, Result, Stepper, loader::DynamicLoadingRegistryWrapper, props::Ownable};
use cytos_derive::CytosNode;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

macro_rules! define_function {
    ($struct_name:ident, $image_ops_func:path, $param_type:ty) => {
        #[derive(CytosNode, Default)]
        struct $struct_name {
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
        struct $struct_name {
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

define_function!(Blur, DynamicImage::blur, f32);
define_function!(FastBlur, DynamicImage::fast_blur, f32);
define_function!(Brighten, DynamicImage::brighten, i32);
define_function!(Unsharpen, DynamicImage::unsharpen, f32, i32);

#[derive(CytosNode, Default)]
struct Filter3x3 {
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
            image: self.input.image.filter3x3(&self.kernel),
        };

        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct Mean {
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
            .to_luma8()
            .pixels()
            .fold(0u64, |a, b| u64::from(b[0]) + a) as f64;

        *self.output = sum / f64::from(self.input.image.width() * self.input.image.height());
        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct Resize {
    #[input]
    input: Prop<Image>,

    #[input]
    width: Prop<u32>,

    #[input]
    height: Prop<u32>,

    #[input]
    filter: Prop<FilterTypeDef>,

    #[output]
    output: Prop<Image>,
}

impl Stepper for Resize {
    fn step(&mut self) -> Result<()> {
        *self.output = Image {
            image: self
                .input
                .image
                .resize(*self.width, *self.height, (*self.filter).into()),
        };
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Serialize, Deserialize)]
enum FilterTypeDef {
    /// Nearest Neighbor
    Nearest,

    /// Linear Filter
    Triangle,

    /// Cubic Filter
    CatmullRom,

    /// Gaussian Filter
    Gaussian,

    /// Lanczos with window 3
    Lanczos3,
}

impl Default for FilterTypeDef {
    fn default() -> Self {
        Self::Nearest
    }
}

impl Ownable for FilterTypeDef {
    type Value = Self;

    fn to_ownable(&self) -> Self::Value {
        *self
    }

    fn from_owned(v: &Self::Value) -> Self {
        *v
    }
}

impl From<FilterTypeDef> for image::imageops::FilterType {
    fn from(value: FilterTypeDef) -> Self {
        match value {
            FilterTypeDef::Nearest => Self::Nearest,
            FilterTypeDef::Triangle => Self::Triangle,
            FilterTypeDef::CatmullRom => Self::CatmullRom,
            FilterTypeDef::Gaussian => Self::Gaussian,
            FilterTypeDef::Lanczos3 => Self::Lanczos3,
        }
    }
}

#[derive(CytosNode, Default)]
struct ResizeExact {
    #[input]
    input: Prop<Image>,

    #[input]
    width: Prop<u32>,

    #[input]
    height: Prop<u32>,

    #[input]
    filter: Prop<FilterTypeDef>,

    #[output]
    output: Prop<Image>,
}

impl Stepper for ResizeExact {
    fn step(&mut self) -> Result<()> {
        *self.output = Image {
            image: self
                .input
                .image
                .resize_exact(*self.width, *self.height, (*self.filter).into()),
        };
        Ok(())
    }
}

#[derive(CytosNode, Default)]
struct Crop {
    #[input]
    input: Prop<Image>,

    #[input]
    rect: Prop<Vec<u32>>,

    #[output]
    output: Prop<Image>,
}

impl Stepper for Crop {
    fn step(&mut self) -> Result<()> {
        *self.output = Image {
            image: self
                .input
                .image
                .crop(self.rect[0], self.rect[1], self.rect[2], self.rect[3]),
        };
        Ok(())
    }
}

pub fn load_registry(registry: &mut DynamicLoadingRegistryWrapper) {
    registry
        .add("Blur", Blur::default)
        .add("FastBlur", FastBlur::default)
        .add("Brighten", Brighten::default)
        .add("Filter3x3", Filter3x3::default)
        .add("Unsharpen", Unsharpen::default)
        .add("Resize", Resize::default)
        .add("ResizeExact", ResizeExact::default)
        .add("ImageMean", Mean::default)
        .add("Crop", Crop::default);
}
