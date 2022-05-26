use std::path::PathBuf;

use clap::{arg, command};
use hound::WavReader;
use nannou::image::ImageFormat;

use hilly_sounds::{encode_image, Options};

fn main() {
    let matches = command!()
        .propagate_version(true)
        .color(clap::ColorChoice::Never)
        .arg(arg!(<INPUT_FILE> "Input WAV file"))
        .arg(arg!([OUTPUT_FILE] "Output PNG file"))
        .get_matches();

    let input_file: PathBuf = matches.value_of_t("INPUT_FILE").unwrap();
    let output_file: PathBuf = matches
        .value_of_t("OUTPUT_FILE")
        .unwrap_or_else(|_| input_file.clone().with_extension("png"));

    let mut reader =
        WavReader::open(input_file).expect("could not read WAV file");
    let image = encode_image(
        reader.samples().map_while(Result::ok),
        Options::default(),
    );
    image
        .save_with_format(output_file, ImageFormat::Png)
        .expect("could not save PNG");
}
