use std::path::PathBuf;

use clap::{arg, command};
use hilbert::fast_hilbert::hilbert_axes;
use nannou::{
    color::{named::BLACK, Hsv, Srgb},
    event::Update,
    image::{ImageBuffer, Rgba},
    wgpu::{Texture, TextureBuilder, TextureUsages},
    App, Frame, LoopMode,
};
use nannou_egui::{egui, Egui};
use num_bigint::ToBigUint;

pub const SIZE_EXP: usize = 9;
pub const SIZE: u32 = 2_u32.pow(SIZE_EXP as u32);

fn main() {
    nannou::app(model)
        .loop_mode(LoopMode::RefreshSync)
        .update(update)
        .run();
}

struct Model {
    index: u32,
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    texture: Texture,
    samples: Box<dyn Iterator<Item = i16>>,
    egui: Egui,
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
        .raw_event(raw_window_event)
        .build()
        .unwrap();

    let window = app.main_window();

    let image = ImageBuffer::new(SIZE, SIZE);

    let texture = TextureBuilder::new()
        .size([SIZE, SIZE])
        .usage(TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING)
        .build(window.device());

    let egui = Egui::from_window(&window);

    Model {
        index: 0,
        image,
        texture,
        samples,
        egui,
    }
}

fn raw_window_event(
    _app: &App,
    model: &mut Model,
    event: &nannou::winit::event::WindowEvent,
) {
    model.egui.handle_raw_event(event);
}

fn update(_app: &App, model: &mut Model, update: Update) {
    for _ in 0..64 {
        if model.index >= SIZE * SIZE {
            break;
        }

        if let Some(sample) = model.samples.next() {
            const EXP: f32 = 1. / 32.;

            let val = sample as f32 / (2u32.pow(15) as f32);
            let val = val.abs().powf(EXP) * val.signum() / 2. + 0.5;

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

    let egui = &mut model.egui;
    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();
    egui::Window::new("Hilly Sounds").show(&ctx, |ui| {
        ui.heading("Status");
        ui.label(format!("Index: {}", model.index));
        ui.add(egui::ProgressBar::new(
            model.index as f32 / (SIZE * SIZE) as f32,
        ));
        if ui.button("hello...").clicked() {
            println!("world!");
        }
    });
}

fn view(app: &App, model: &Model, frame: Frame) {
    let samples = model.image.as_flat_samples();
    model.texture.upload_data(
        app.main_window().device(),
        &mut frame.command_encoder(),
        samples.as_slice(),
    );

    let draw = app.draw();
    draw.background().color(BLACK);
    draw.texture(&model.texture);
    draw.to_frame(app, &frame).unwrap();

    model.egui.draw_to_frame(&frame).unwrap();
}
