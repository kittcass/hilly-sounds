#![feature(int_log)]

use std::io;

use hound::WavWriter;
use nannou::image::{self, RgbaImage};
use strategy::{ColorStrategy, SpaceStrategy};

pub mod strategy;

// TODO should we separate pixel data encoding/decoding from the hilbert curve?
// so far, this algorithm operates independently of the curve, so maybe it
// should be decoupled? much to think about...

// TODO maybe pass sizes as actual values and infer exp instead, validating that
// they are power of two

pub type PixelData = ([u32; 2], image::Rgba<u8>);

pub struct Encoder<S, I>
where
    S: hound::Sample + SampleConvert,
    I: Iterator<Item = S>,
{
    index: usize,
    iter: I,
    color_strategy: Box<dyn ColorStrategy + Send>,
    space_strategy: Box<dyn SpaceStrategy + Send>,
}

impl<S, I> Encoder<S, I>
where
    S: hound::Sample + SampleConvert,
    I: Iterator<Item = S>,
{
    pub fn new(
        iter: I,
        color_strategy: Box<dyn ColorStrategy + Send>,
        space_strategy: Box<dyn SpaceStrategy + Send>,
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
    type Item = PixelData;

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
    color_strategy: Box<dyn ColorStrategy + Send>,
    space_strategy: Box<dyn SpaceStrategy + Send>,
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

pub struct Decoder {
    index: usize,
    image: RgbaImage,
    color_strategy: Box<dyn ColorStrategy + Send>,
    space_strategy: Box<dyn SpaceStrategy + Send>,
}

impl Decoder {
    pub fn new(
        image: RgbaImage,
        color_strategy: Box<dyn ColorStrategy + Send>,
        space_strategy: Box<dyn SpaceStrategy + Send>,
    ) -> Self {
        assert!(image.width() == space_strategy.width());
        assert!(image.height() == space_strategy.height());

        Decoder {
            index: 0,
            image,
            color_strategy,
            space_strategy,
        }
    }
}

impl Iterator for Decoder {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.space_strategy.size() {
            return None;
        }

        let [x, y] = self
            .space_strategy
            .index_to_coord(self.index)
            .expect("could not get coordinate from index");
        self.index += 1;

        let color = self.image.get_pixel(x, y);

        let sample = self.color_strategy.color_to_sample(color);

        Some(sample)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.space_strategy.size()))
    }
}

pub fn decode_image<W>(
    image: RgbaImage,
    writer: &mut WavWriter<W>,
    color_strategy: Box<dyn ColorStrategy + Send>,
    space_strategy: Box<dyn SpaceStrategy + Send>,
) -> hound::Result<()>
where
    W: io::Write + io::Seek,
{
    let decoder = Decoder::new(image, color_strategy, space_strategy);

    for sample in decoder {
        writer.write_sample(sample)?;
    }

    Ok(())
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
