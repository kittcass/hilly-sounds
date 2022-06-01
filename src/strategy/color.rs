use nannou::{
    color::{self, hsv},
    image,
};

pub trait ColorStrategy {
    fn sample_to_color(&self, sample: i16) -> image::Rgba<u8>;

    fn color_to_sample(&self, color: image::Rgba<u8>) -> i16;
}

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
        let rgb: color::Rgb = hsv(hue, self.saturation, self.value).into();
        image::Rgba([
            (255. * rgb.red) as u8,
            (255. * rgb.green) as u8,
            (255. * rgb.blue) as u8,
            255,
        ])
    }

    fn color_to_sample(&self, _color: image::Rgba<u8>) -> i16 {
        unimplemented!()
    }
}
