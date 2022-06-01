use std::{path::PathBuf, thread, time::Duration};

use clap::{arg, command};
use hilbert::fast_hilbert::hilbert_axes;
use nannou::{
    color::{Hsv, Srgb},
    event::Update,
    image::{ImageBuffer, Rgba},
    wgpu::{Texture, TextureBuilder, TextureUsages},
    App, Frame,
};
use num_bigint::ToBigUint;

pub const SIZE_EXP: usize = 9;
pub const SIZE: u32 = 2_u32.pow(SIZE_EXP as u32);

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    index: u32,
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    texture: Texture,
    samples: Box<dyn Iterator<Item = i16>>,
}

fn model(app: &App) -> Model {
    let matches = command!()
        .propagate_version(true)
        .color(clap::ColorChoice::Never)
        .arg(arg!(<INPUT_FILE> "Input WAV file"))
        .get_matches();

    let input_file: PathBuf = matches.value_of_t("INPUT_FILE").unwrap();

    let reader =
        hound::WavReader::open(input_file).expect("could not read WAV file");
    let samples = Box::new(reader.into_samples().map_while(Result::ok));

    let _window = app
        .new_window()
        .size(SIZE, SIZE)
        .resizable(false)
        .view(view)
        .build()
        .unwrap();

    let window = app.main_window();

    let image = ImageBuffer::new(SIZE, SIZE);

    let texture = TextureBuilder::new()
        .size([SIZE, SIZE])
        .usage(TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING)
        .build(window.device());

    Model {
        index: 0,
        image,
        texture,
        samples,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    // match event {
    //     Event::Update(update) => model.index += 1,
    //     _ => {}
    // }

    for _ in 0..(8 * SIZE) {
        if model.index >= SIZE * SIZE {
            model
                .image
                .save_with_format("out.png", nannou::image::ImageFormat::Png)
                .expect("could not save image");
            thread::sleep(Duration::from_secs(16));
            break;
        }

        if let Some(_sample) = model.samples.next() {
            // const EXP: f32 = 1. / 32.;

            // let val = sample as f32 / (2u32.pow(15) as f32);
            // let val = val.abs().powf(EXP) * val.signum() / 2. + 0.5;
            let val = (model.index as f32 / 8192.).sin() / 2. + 0.5;

            // let val = (sample as f32 + 2u32.pow(15) as f32) / (2u32.pow(16) as f32);

            // let hue = hue.log10();

            // println!("{sample} -> {hue:.6}");
            let color = Hsv::new(240., 0.9, val);
            let rgb: Srgb = color.into();

            let coords = hilbert_axes(
                &model.index.to_biguint().unwrap(),
                SIZE_EXP + 2,
                2,
            );
            model.image.put_pixel(
                coords[0],
                coords[1],
                Rgba([
                    (rgb.red * 255.) as u8,
                    (rgb.green * 255.) as u8,
                    (rgb.blue * 255.) as u8,
                    255,
                ]),
            );
            model.index += 1;
        }
    }

    // while t < 800 && model.index < SIZE * SIZE {
    //     let coords =
    //         hilbert_axes(&model.index.to_biguint().unwrap(), SIZE_EXP + 2, 2);
    //     model.image.put_pixel(
    //         coords[0],
    //         coords[1],
    //         Rgba([
    //             (model.index % 256) as u8,
    //             0,
    //             (255. * model.index as f32 / (SIZE * SIZE) as f32) as u8,
    //             255,
    //         ]),
    //     );
    //     model.index += 1;
    //     t += 1;
    // }

    // model
    //     .image
    //     .put_pixel(coords[0], coords[1], Rgb([255, 0, 0]));
    // model.index += 1;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let samples = model.image.as_flat_samples();
    model.texture.upload_data(
        app.main_window().device(),
        &mut frame.command_encoder(),
        samples.as_slice(),
    );

    let draw = app.draw();
    // draw.background().color(MIDNIGHTBLUE);
    draw.texture(&model.texture);
    draw.to_frame(app, &frame).unwrap();
}
