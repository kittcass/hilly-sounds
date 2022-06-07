# Hilly Sounds

Hilly Sounds is a collaboration of Kitt Zwovic and Cassaundra Smith for credit in CS 410 Intro to Comp. Photography and CS 410P Music, Sound & Computers.

The idea behind this project is to convert a sequence of sound samples into an image representation, using a Hilbert curve mapping, perform various image filters and effects to manipulate the data, and then decode that visual data back into sound.

Our implementation consists of a command-line interface built on top of a library.

## Usage

Requirements:
- Your favorite flavor of Linux with up-to-date audio software
- The Rust [nightly toolchain](https://rust-lang.github.io/rustup/concepts/channels.html)

```bash
git clone git@gitlab.cecs.pdx.edu:hilly-sounds/hilly-sounds.git
cd hilly-sounds
cargo build --bin hscli --features=binary,completion,cpal,json,toml --release
```

The program will be located in `target/release/hscli`.
Try running the program with `--help` to learn how to use it, or skip to the [examples](#Examples).

### Examples

The following example uses SoX for playback and feh for image preview.

```bash
# use a provided configuration preset
cat presets/example.toml
export PRESET=presets/example.toml

# make a directory to dump output files into
mkdir output

# listen to the sample you want to encode
play samples/sounds/anxiety_moozic.wav

# now encode it!
hscli encode samples/sounds/anxiety_moozic.wav output/

# view the output
feh output/anxiety_moozic.png

# decode the image and play the sound directly
# check out the --device and --list-device flags if you have trouble with playback
hscli decode-play samples/sounds/anxiety_moozic.wav

# if you want, you can also decode the image to a WAV file
# in the same manner as encoding from before
hscli decode samples/sounds/anxiety_moozic.wav output/
```

Check out the files in the [samples directory](samples/) for inspiration.

### Shell completion

The `hscli` binary supports shell completion for some shells (including bash, zsh, fish, and PowerShell).
With the `completion` feature enabled, run `hscli generate-completions <shell>`, directing the output to wherever your shell reads completion scripts.

## Presentation

You can view the slides by hosting the `presentation/` directory on a server and viewing the `index.html` file.

One easy way to do this might be running the following script and opening http://localhost:8000/ in your browser:
```bash
cd presentation/
python -m http.server
```
