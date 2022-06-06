//! Mapping strategies between colors and samples.

use nannou::{
    color::{hsv, Hsv, Rgb},
    image,
};

/// A color strategy which represents a mapping between sound samples and
/// colors.
///
/// These functions should accept any value of the types [`i16`] or
/// [`image::Rgba<u8>`] without panicking.
///
/// ## Mapping
///
/// Ideally, each function is the inverse of each other. However, because there
/// are more possible colors than samples, the mapping between colors and
/// samples cannot be bijective.
///
/// One consequence of this fact is that not all algorithms cover the entire
/// domain of these types. In this case, it is best if the implementation
/// follows some sort of "best guess" so that unexpected artifacts do not
/// appear.
pub trait ColorStrategy {
    /// Convert a sample to a color.
    ///
    /// This should accept any value for `sample` without panicking.
    fn sample_to_color(&self, sample: i16) -> image::Rgba<u8>;

    /// Convert a sample to a color.
    ///
    /// This should accept any value for `color` without panicking.
    fn color_to_sample(&self, color: &image::Rgba<u8>) -> i16;
}

/// A [`ColorStrategy`] which maps operates based on hue.
pub struct HueColorStrategy {
    saturation: f32,
    value: f32,
}

impl HueColorStrategy {
    pub fn new(saturation: f32, value: f32) -> Self {
        HueColorStrategy { saturation, value }
    }
}

impl ColorStrategy for HueColorStrategy {
    fn sample_to_color(&self, sample: i16) -> image::Rgba<u8> {
        let hue = (sample as f32 + 2u32.pow(15) as f32) / (2u32.pow(16) as f32);
        let rgb: Rgb = hsv(hue, self.saturation, self.value).into();
        image::Rgba([
            (255. * rgb.red) as u8,
            (255. * rgb.green) as u8,
            (255. * rgb.blue) as u8,
            255,
        ])
    }

    fn color_to_sample(&self, color: &image::Rgba<u8>) -> i16 {
        let [r, g, b, _] = color.0;
        let hsv: Hsv =
            Rgb::new(r as f32 / 255., g as f32 / 255., b as f32 / 255.).into();
        let sample = (hsv.hue.to_positive_radians() / std::f32::consts::TAU)
            * (2u32.pow(16) as f32)
            - (2u32.pow(15) as f32);
        sample as i16
    }
}
