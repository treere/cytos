//! Image processing transformer nodes for Cytos.
//!
//! This module provides various image manipulation operations including:
//! - Blurring (standard and fast blur)
//! - Brightness adjustment
//! - Unsharp masking
//! - 3x3 kernel filtering
//! - Mean calculation
//! - Resizing (standard and exact)
//! - Cropping
//!
//! These nodes operate on the [`Image`] type from the `imageio` module
//! and use the `image` crate for underlying operations.

use crate::imageio::Image;
use cytos::{Prop, Result, Stepper, loader::DynamicLoadingRegistryWrapper, props::Ownable};
use cytos_derive::CytosNode;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

macro_rules! define_function {
    ($struct_name:ident, $image_ops_func:path, $param_type:ty) => {
        #[derive(CytosNode, Default)]
        struct $struct_name {
            #[cytos(input)]
            input: Prop<Image>,
            #[cytos(input)]
            param: Prop<$param_type>,

            #[cytos(output)]
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
            #[cytos(input)]
            input: Prop<Image>,
            #[cytos(input)]
            param1: Prop<$param1_type>,
            #[cytos(input)]
            param2: Prop<$param2_type>,

            #[cytos(output)]
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
    #[cytos(input)]
    input: Prop<Image>,
    #[cytos(input)]
    kernel: Prop<Vec<f32>>,
    #[cytos(output)]
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
    #[cytos(input)]
    input: Prop<Image>,

    #[cytos(output)]
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
    #[cytos(input)]
    input: Prop<Image>,

    #[cytos(input)]
    width: Prop<u32>,

    #[cytos(input)]
    height: Prop<u32>,

    #[cytos(input)]
    filter: Prop<FilterTypeDef>,

    #[cytos(output)]
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

#[derive(Default, Clone, Copy, Debug, PartialEq, Hash, Serialize, Deserialize)]
enum FilterTypeDef {
    #[default]
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
    #[cytos(input)]
    input: Prop<Image>,

    #[cytos(input)]
    width: Prop<u32>,

    #[cytos(input)]
    height: Prop<u32>,

    #[cytos(input)]
    filter: Prop<FilterTypeDef>,

    #[cytos(output)]
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
    #[cytos(input)]
    input: Prop<Image>,

    #[cytos(input)]
    rect: Prop<Vec<u32>>,

    #[cytos(output)]
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

/// Node that converts an RGB image to grayscale (luminance).
#[derive(CytosNode, Default)]
struct Grayscale {
    #[cytos(input)]
    input: Prop<Image>,
    #[cytos(output)]
    output: Prop<Image>,
}

impl Stepper for Grayscale {
    fn step(&mut self) -> Result<()> {
        *self.output = Image {
            image: DynamicImage::ImageLuma8(self.input.image.to_luma8()),
        };
        Ok(())
    }
}

/// Node that converts an image to a flat vector of f32 values.
/// Each pixel is normalized to [0, 1] range.
#[derive(CytosNode, Default)]
struct ImageToVec {
    #[cytos(input)]
    input: Prop<Image>,
    #[cytos(output)]
    output: Prop<Vec<f32>>,
}

impl Stepper for ImageToVec {
    fn step(&mut self) -> Result<()> {
        let gray = self.input.image.to_luma8();
        let output: Vec<f32> = gray.pixels().map(|p| f32::from(p[0]) / 255.0).collect();
        *self.output = output;
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
        .add("Crop", Crop::default)
        .add("Grayscale", Grayscale::default)
        .add("ImageToVec", ImageToVec::default);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grayscale() {
        let img = image::DynamicImage::new_rgb8(2, 2);
        let mut grayscale = Grayscale::default();
        *grayscale.input = Image { image: img };

        grayscale.step().unwrap();

        let output = &grayscale.output.image;
        assert_eq!(output.width(), 2);
        assert_eq!(output.height(), 2);
    }

    #[test]
    fn test_image_to_vec() {
        let img = image::DynamicImage::new_luma8(2, 3);
        let mut image_to_vec = ImageToVec::default();
        *image_to_vec.input = Image { image: img };

        image_to_vec.step().unwrap();

        let output = &*image_to_vec.output;
        assert_eq!(output.len(), 6);
        for &v in output {
            assert!((0.0..=1.0).contains(&v));
        }
    }
}
