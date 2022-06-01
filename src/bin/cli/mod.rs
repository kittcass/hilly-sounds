use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use hound::WavReader;
use nannou::image::ImageFormat;

use hilly_sounds::{
    encode_image,
    strategy::{ColorStrategy, SpaceStrategy},
};

mod preset;
use preset::Preset;

#[derive(Parser)]
#[clap(version, color = clap::ColorChoice::Never)]
struct Args {
    /// Path to a TOML preset file, containing color and space strategies.
    #[clap(name = "preset", short, long, global = true)]
    preset_path: Option<PathBuf>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Encode a WAV file into a PNG file.
    Encode {
        /// Path to the input WAV file.
        #[clap(validator = validate_is_file)]
        input_file: PathBuf,

        /// Output path for the WAV file, either a file or directory.
        ///
        /// By default, the file name is the same as the input with the .png
        /// extension instead (e.g. example.wav to example.png). This is used
        /// both when no output path is specified and when only a directory is
        /// provided.
        output_path: Option<PathBuf>,

        /// Open the file using the default system application for the file type
        /// after it has been saved.
        #[clap(short, long)]
        open: bool,
    },
    /// Decode a PNG file into a WAV file.
    Decode {
        /// Path to the input PNG file.
        #[clap(validator = validate_is_file)]
        input_file: PathBuf,

        /// Output path for the WAV file, either a file or directory.
        ///
        /// By default, the file name is the same as the input with the .wav
        /// extension instead (e.g. example.png to example.wav). This is used
        /// both when no output path is specified and when only a directory is
        /// provided.
        output_path: Option<PathBuf>,
    },
    /// Decode a PNG file and play it.
    DecodePlay {
        /// Path to the input PNG file..
        input_file: PathBuf,

        /// The output audio device with which to play the file.
        #[clap(short, long)]
        device: Option<String>,

        /// List the available output audio devices.
        #[clap(short, long)]
        list_devices: bool,
    },
    /// Dump the current configuration values.
    DumpConfig,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // TODO default preset

    // TODO handle validation errors
    let (color_strategy, space_strategy) =
        if let Some(preset_path) = args.preset_path {
            let preset_toml = fs::read_to_string(preset_path)?;
            let preset: Preset = toml::from_str(&preset_toml)?;
            (preset.color.to_strategy(), preset.space.to_strategy())
        } else {
            todo!()
        };

    match &args.command {
        Command::Encode {
            input_file,
            output_path,
            open,
        } => {
            let output_file =
                resolve_output_file(input_file, output_path, "png");
            encode(
                input_file,
                &output_file,
                *open,
                color_strategy,
                space_strategy,
            );
        }
        Command::Decode {
            input_file: _,
            output_path: _output_file,
        } => {
            todo!()
        }
        Command::DecodePlay {
            input_file: _,
            device: _,
            list_devices: _,
        } => todo!(),
        Command::DumpConfig => todo!(),
    }

    Ok(())
}

fn validate_is_file(arg: &str) -> Result<(), String> {
    if !PathBuf::from(arg).is_file() {
        Err(String::from("not a file"))
    } else {
        Ok(())
    }
}

fn resolve_output_file(
    input_file: &Path,
    output_path: &Option<PathBuf>,
    extension: &str,
) -> PathBuf {
    assert!(input_file.is_file());

    match output_path {
        Some(dir) if dir.is_dir() => {
            let renamed = input_file.with_extension(extension);
            let file_name = renamed.file_name().unwrap();
            dir.join(Path::new(file_name))
        }
        Some(file) => file.clone(),
        None => input_file.with_extension(extension),
    }
}

fn encode(
    input_file: &Path,
    output_file: &Path,
    open: bool,
    color_strategy: Box<dyn ColorStrategy>,
    space_strategy: Box<dyn SpaceStrategy>,
) {
    let mut reader =
        WavReader::open(input_file).expect("could not read WAV file");

    let image = match reader.spec().sample_format {
        hound::SampleFormat::Float => encode_image(
            reader.samples::<f32>().map_while(Result::ok),
            color_strategy,
            space_strategy,
        ),
        hound::SampleFormat::Int => encode_image(
            reader.samples::<i16>().map_while(Result::ok),
            color_strategy,
            space_strategy,
        ),
    };

    image
        .save_with_format(output_file, ImageFormat::Png)
        .expect("could not save PNG");

    if open {
        // TODO open file using opener crate
    }
}
