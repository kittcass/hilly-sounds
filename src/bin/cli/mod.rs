use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;
use clap::{ArgEnum, Parser, Subcommand};
use hound::{WavReader, WavSpec, WavWriter};
use nannou::image::{self, ImageFormat};

use hilly_sounds::{
    decode_image, encode_image,
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

        /// The number of channels to output to.
        #[clap(short, long, default_value_t = 2)]
        channels: u16,

        /// The sample rate to output to.
        #[clap(short, long, default_value_t = 48000)]
        sample_rate: u32,
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
    /// Dump the current preset to the standard output.
    DumpPreset {
        /// The format to output to.
        #[clap(arg_enum, short, long, default_value_t = DumpFormat::Toml)]
        format: DumpFormat,

        /// Whether or not to pretty print outputs.
        #[clap(short, long)]
        pretty: bool,
    },
}

#[derive(ArgEnum, Copy, Clone)]
enum DumpFormat {
    Debug,
    Toml,
    #[cfg(feature = "json")]
    Json,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // TODO handle validation errors
    let preset = if let Some(preset_path) = args.preset_path {
        let preset_toml = fs::read_to_string(preset_path)
            .context("failed to read preset file")?;
        let preset: Preset = toml::from_str(&preset_toml)
            .context("failed to parse TOML in preset file")?;
        preset
    } else {
        todo!("default preset")
    };

    let (color_strategy, space_strategy) =
        (preset.color.to_strategy(), preset.space.to_strategy());

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
            )
            .context("failed to run encoder")?;
        }
        Command::Decode {
            input_file,
            output_path,
            channels,
            sample_rate,
        } => {
            let output_file =
                resolve_output_file(input_file, output_path, "wav");
            let wav_spec = WavSpec {
                channels: *channels,
                sample_rate: *sample_rate,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            decode(
                input_file,
                &output_file,
                wav_spec,
                color_strategy,
                space_strategy,
            )
            .context("failed to run deocder")?;
        }
        Command::DecodePlay {
            input_file: _,
            device: _,
            list_devices: _,
        } => todo!(),
        Command::DumpPreset { format, pretty } => {
            dump_preset(&preset, *format, *pretty)
                .context("failed to dump preset")?;
        }
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
) -> anyhow::Result<()> {
    let mut reader = WavReader::open(input_file)?;

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
        opener::open(output_file)?;
    }

    Ok(())
}

fn decode(
    input_file: &Path,
    output_file: &Path,
    wav_spec: WavSpec,
    color_strategy: Box<dyn ColorStrategy>,
    space_strategy: Box<dyn SpaceStrategy>,
) -> anyhow::Result<()> {
    let image = image::io::Reader::open(input_file)?.decode()?.to_rgba8();
    let mut writer = WavWriter::create(output_file, wav_spec)?;
    decode_image(image, &mut writer, color_strategy, space_strategy)?;
    writer.finalize()?;

    Ok(())
}

fn dump_preset(
    preset: &Preset,
    format: DumpFormat,
    pretty: bool,
) -> anyhow::Result<()> {
    if pretty {
        match format {
            DumpFormat::Debug => println!("{:#?}", preset),
            DumpFormat::Toml => {
                print!("{}", toml::to_string_pretty(&preset)?)
            }
            #[cfg(feature = "json")]
            DumpFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&preset)?)
            }
        }
    } else {
        match format {
            DumpFormat::Debug => println!("{:?}", preset),
            DumpFormat::Toml => {
                print!("{}", toml::to_string(&preset)?)
            }
            #[cfg(feature = "json")]
            DumpFormat::Json => {
                println!("{}", serde_json::to_string(&preset)?)
            }
        }
    }

    Ok(())
}
