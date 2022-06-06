use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use clap::{ArgEnum, Parser, Subcommand, ValueHint};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device,
};
use hound::{WavReader, WavSpec, WavWriter};
use nannou::image::{self, ImageFormat};

#[cfg(feature = "completion")]
use clap::CommandFactory;
#[cfg(feature = "completion")]
use clap_complete::{generate, Generator, Shell};

use hilly_sounds::{
    decode_image, encode_image,
    strategy::{ColorStrategy, Preset, SpaceStrategy},
    Decoder,
};

mod util;
use util::*;

#[derive(Parser)]
#[clap(name = "hscli", version, color = clap::ColorChoice::Never)]
struct Args {
    /// Path to a TOML preset file, containing color and space strategies.
    #[clap(name = "preset", env = "PRESET", short, long, value_hint = ValueHint::FilePath)]
    preset_path: Option<PathBuf>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
#[non_exhaustive]
enum Command {
    /// Encode a WAV file into a PNG file.
    Encode {
        /// Path to the input WAV file.
        #[clap(validator = validate_is_file, value_hint = ValueHint::FilePath)]
        input_file: PathBuf,

        /// Output path for the WAV file, either a file or directory.
        ///
        /// By default, the file name is the same as the input with the .png
        /// extension instead (e.g. example.wav to example.png). This is used
        /// both when no output path is specified and when only a directory is
        /// provided.
        #[clap(value_hint = ValueHint::AnyPath)]
        output_path: Option<PathBuf>,

        /// Number of sections to skip.
        #[clap(long, default_value_t = 0)]
        skip: usize,

        /// Open the file using the default system application for the file type
        /// after it has been saved.
        #[clap(short, long)]
        open: bool,
    },
    /// Decode a PNG file into a WAV file.
    Decode {
        /// Path to the input PNG file.
        #[clap(validator = validate_is_file, value_hint = ValueHint::FilePath)]
        input_file: PathBuf,

        /// Output path for the WAV file, either a file or directory.
        ///
        /// By default, the file name is the same as the input with the .wav
        /// extension instead (e.g. example.png to example.wav). This is used
        /// both when no output path is specified and when only a directory is
        /// provided.
        #[clap(value_hint = ValueHint::AnyPath)]
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
        #[clap(value_hint = ValueHint::FilePath)]
        input_file: PathBuf,

        /// The number of channels to output to.
        #[clap(short, long, default_value_t = 2)]
        channels: u16,

        /// The sample rate to output to.
        #[clap(short, long, default_value_t = 48000)]
        sample_rate: u32,

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
        #[clap(long)]
        pretty: bool,
    },
    /// Generate shell completions for a given shell.
    #[cfg(feature = "completion")]
    GenerateCompletions {
        #[clap(arg_enum, conflicts_with = "preset")]
        shell: Shell,
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

    #[cfg(feature = "completion")]
    if let Command::GenerateCompletions { shell } = args.command {
        print_completions(shell, &mut Args::command());
        return Ok(());
    }

    // TODO handle validation errors
    let preset = if let Some(preset_path) = args.preset_path {
        let preset_toml = fs::read_to_string(preset_path)
            .context("failed to read preset file")?;
        toml::from_str(&preset_toml)
            .context("failed to parse TOML in preset file")?
    } else {
        Preset::default()
    };

    let (color_strategy, space_strategy) =
        (preset.color.to_strategy(), preset.space.to_strategy());

    match &args.command {
        Command::Encode {
            input_file,
            output_path,
            skip,
            open,
        } => {
            let output_file =
                resolve_output_file(input_file, output_path, "png");
            encode(
                input_file,
                &output_file,
                *skip,
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
            input_file,
            channels,
            sample_rate,
            device,
            list_devices,
        } => {
            let host = cpal::default_host();

            if *list_devices {
                for device in host.output_devices()? {
                    println!("{}", device.name()?);
                }
            } else {
                let device = if let Some(device_name) = device {
                    host.output_devices()?.find(|d| {
                        d.name()
                            .map(|name| name == *device_name)
                            .unwrap_or_default()
                    })
                } else {
                    host.default_output_device()
                }
                .context("failed to find output device")?;

                let _config: cpal::SupportedStreamConfig = device
                    .default_output_config()
                    .context("failed to fo find default output config")?;
                let config = cpal::StreamConfig {
                    channels: *channels,
                    sample_rate: cpal::SampleRate(*sample_rate),
                    buffer_size: cpal::BufferSize::Default,
                };

                decode_play(
                    input_file,
                    &device,
                    &config,
                    color_strategy,
                    space_strategy,
                )?;
            }
        }
        Command::DumpPreset { format, pretty } => {
            dump_preset(&preset, *format, *pretty)
                .context("failed to dump preset")?;
        }
        #[cfg(feature = "completion")]
        _ => unreachable!(),
    }

    Ok(())
}

#[cfg(feature = "completion")]
fn print_completions<G: Generator>(gen: G, cmd: &mut clap::Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
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
    skip: usize,
    open: bool,
    color_strategy: Box<dyn ColorStrategy + Send>,
    space_strategy: Box<dyn SpaceStrategy + Send>,
) -> anyhow::Result<()> {
    let mut reader = WavReader::open(input_file)?;

    let skip = skip * space_strategy.size();
    let image = match reader.spec().sample_format {
        hound::SampleFormat::Float => match reader.spec().bits_per_sample {
            16 => encode_image(
                reader.samples::<f32>().skip(skip).map_while(Result::ok),
                color_strategy,
                space_strategy,
            ),
            bps => bail!("unsupported number of bits per sample: {}", bps),
        },
        hound::SampleFormat::Int => match reader.spec().bits_per_sample {
            16 => encode_image(
                reader.samples::<i16>().skip(skip).map_while(Result::ok),
                color_strategy,
                space_strategy,
            ),
            32 => encode_image(
                reader.samples::<i32>().skip(skip).map_while(Result::ok),
                color_strategy,
                space_strategy,
            ),
            bps => bail!("unsupported number of bits per sample: {}", bps),
        },
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
    color_strategy: Box<dyn ColorStrategy + Send>,
    space_strategy: Box<dyn SpaceStrategy + Send>,
) -> anyhow::Result<()> {
    let image = image::io::Reader::open(input_file)?.decode()?.to_rgba8();
    let mut writer = WavWriter::create(output_file, wav_spec)?;
    decode_image(image, &mut writer, color_strategy, space_strategy)?;
    writer.finalize()?;

    Ok(())
}

fn decode_play(
    input_file: &Path,
    device: &Device,
    config: &cpal::StreamConfig,
    color_strategy: Box<dyn ColorStrategy + Send>,
    space_strategy: Box<dyn SpaceStrategy + Send>,
) -> anyhow::Result<()> {
    let image = image::io::Reader::open(input_file)?.decode()?.to_rgba8();

    let mut decoder = Decoder::new(image, color_strategy, space_strategy);

    let err_fn = |err| eprintln!("an error occurred while streaming: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            // TODO better way to play until done?
            data.fill_with(|| {
                decoder.next().unwrap_or_else(|| std::process::exit(0))
            });
        },
        err_fn,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::MAX);

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
