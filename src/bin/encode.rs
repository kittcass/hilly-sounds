use std::path::PathBuf;

use clap::Parser;
use hound::WavReader;
use nannou::image::ImageFormat;

use hilly_sounds::{encode_image, Options};

#[derive(Parser)]
#[clap(version, color = clap::ColorChoice::Never)]
struct Args {
    /// Input WAV file path.
    input_file: PathBuf,

    /// Output PNG file path, if different.
    output_file: Option<PathBuf>,

    #[clap(flatten)]
    options: Options,
}

fn main() {
    let args = Args::parse();

    let output_file = args
        .output_file
        .unwrap_or_else(|| args.input_file.with_extension("png"));

    let mut reader =
        WavReader::open(args.input_file).expect("could not read WAV file");

    // TODO handle errors
    let image = match reader.spec().sample_format {
        hound::SampleFormat::Float => encode_image(
            reader.samples::<f32>().map_while(Result::ok),
            args.options,
        ),
        hound::SampleFormat::Int => encode_image(
            reader.samples::<i16>().map_while(Result::ok),
            args.options,
        ),
    };
    image
        .save_with_format(output_file, ImageFormat::Png)
        .expect("could not save PNG");
}
