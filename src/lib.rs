#![feature(int_log)]

use nannou::image;
use strategy::{ColorStrategy, SpaceStrategy};

pub mod strategy;

// TODO should we separate pixel data encoding/decoding from the hilbert curve?
// so far, this algorithm operates independently of the curve, so maybe it
// should be decoupled? much to think about...

// TODO maybe pass sizes as actual values and infer exp instead, validating that
// they are power of two

pub struct Encoder<S, I>
where
    S: hound::Sample + SampleConvert,
    I: Iterator<Item = S>,
{
    index: usize,
    iter: I,
    color_strategy: Box<dyn ColorStrategy>,
    space_strategy: Box<dyn SpaceStrategy>,
}

impl<S, I> Encoder<S, I>
where
    S: hound::Sample + SampleConvert,
    I: Iterator<Item = S>,
{
    pub fn new(
        iter: I,
        color_strategy: Box<dyn ColorStrategy>,
        space_strategy: Box<dyn SpaceStrategy>,
    ) -> Self {
        Encoder {
            index: 0,
            iter,
            color_strategy,
            space_strategy,
        }
    }
}

impl<S, I> Iterator for Encoder<S, I>
where
    S: hound::Sample + SampleConvert,
    I: Iterator<Item = S>,
{
    type Item = ([u32; 2], image::Rgba<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.space_strategy.size() {
            return None;
        }

        if let Some(sample) = self.iter.next() {
            let coords = self
                .space_strategy
                .index_to_coord(self.index)
                .expect("could not get coordinate from index");
            self.index += 1;

            let color =
                self.color_strategy.sample_to_color(sample.convert_to_i16());

            Some((coords, color))
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
    color_strategy: Box<dyn ColorStrategy>,
    space_strategy: Box<dyn SpaceStrategy>,
) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>>
where
    S: hound::Sample + SampleConvert,
    I: Iterator<Item = S>,
{
    let mut image = image::ImageBuffer::new(
        space_strategy.width(),
        space_strategy.height(),
    );

    let encoder = Encoder::new(iter, color_strategy, space_strategy);

    for ([x, y], color) in encoder {
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
