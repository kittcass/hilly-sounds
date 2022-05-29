use std::path::PathBuf;

use clap::Parser;
use hilly_sounds::Options;

#[derive(Parser)]
#[clap(version, color = clap::ColorChoice::Never)]
struct Args {
    /// Input PNG file path, if named differently than the input file path.
    input_file: PathBuf,

    /// Output WAV file path, if different.
    output_file: Option<PathBuf>,

    #[clap(flatten)]
    options: Options,
}

fn main() {
    let args = Args::parse();

    let _output_file = args.output_file.unwrap_or_else(|| {
        args.input_file.with_extension("wav")
    });

    unimplemented!()
}
