#[cfg(feature = "binary")]
use clap::Parser;

use hilbert::fast_hilbert::hilbert_axes;
use nannou::{
    color::{hsv, Rgba},
    image,
    prelude::{One, ToPrimitive, Zero},
};
use num_bigint::BigUint;

// TODO should we separate pixel data encoding/decoding from the hilbert curve?
// so far, this algorithm operates independently of the curve, so maybe it
// should be decoupled? much to think about...

// TODO maybe pass sizes as actual values and infer exp instead, validating that
// they are power of two

#[derive(Copy, Clone)]
#[cfg_attr(feature = "binary", derive(Parser))]
pub struct Options {
    /// The exponent of the side length (2^n).
    #[cfg_attr(feature = "binary", clap(long, default_value_t = 9))]
    pub size_exp: usize,
}

impl Options {
    pub fn side_len(&self) -> u32 {
        2u32.pow(self.size_exp as u32)
    }
}

pub struct Encoder<S, I>
where
    S: hound::Sample + SampleConvert,
    I: Iterator<Item = S>,
{
    iter: I,
    options: Options,
    index: BigUint,
}

impl<S, I> Encoder<S, I>
where
    S: hound::Sample + SampleConvert,
    I: Iterator<Item = S>,
{
    pub fn new(iter: I, options: Options) -> Self {
        Encoder {
            iter,
            options,
            index: BigUint::zero(),
        }
    }

    pub fn options(&self) -> &Options {
        &self.options
    }
}

impl<S, I> Iterator for Encoder<S, I>
where
    S: hound::Sample + SampleConvert,
    I: Iterator<Item = S>,
{
    type Item = ([u32; 2], image::Rgba<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        // TODO don't do this every iteration?
        let side_len = self.options.side_len();
        if self.index.to_u32().unwrap() >= side_len * side_len {
            return None;
        }

        if let Some(sample) = self.iter.next() {
            let coords =
                hilbert_axes(&self.index, self.options.size_exp + 2, 2);
            self.index += BigUint::one();

            let hue = (sample.convert_to_f32() + 2u32.pow(15) as f32)
                / (2u32.pow(16) as f32);

            let rgb: Rgba = hsv(hue, 1.0, 1.0).into();

            Some((
                [coords[0], coords[1]],
                image::Rgba([
                    (255. * rgb.red) as u8,
                    (255. * rgb.green) as u8,
                    (255. * rgb.blue) as u8,
                    255,
                ]),
            ))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // TODO also take into account size from options
        self.iter.size_hint()
    }
}

pub fn encode_image<S, I>(
    iter: I,
    options: Options,
) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>>
where
    S: hound::Sample + SampleConvert,
    I: Iterator<Item = S>,
{
    let side_len = options.side_len();
    let mut image = image::ImageBuffer::new(side_len, side_len);

    let mut encoder = Encoder::new(iter, options);

    while let Some(([x, y], color)) = encoder.next() {
        image.put_pixel(x, y, color);
    }

    image
}

pub trait SampleConvert {
    fn convert_to_i16(self) -> i16;
    fn convert_to_f32(self) -> f32;
}

impl SampleConvert for i16 {
    fn convert_to_i16(self) -> i16 {
        self
    }

    fn convert_to_f32(self) -> f32 {
        self as f32 / 32768.0
    }
}

impl SampleConvert for f32 {
    fn convert_to_i16(self) -> i16 {
        (self * 32768.0) as i16
    }

    fn convert_to_f32(self) -> f32 {
        self
    }
}
