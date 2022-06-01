use serde::{Deserialize, Serialize};

use hilly_sounds::strategy::{
    color::HueColorStrategy,
    space::{HilbertSpaceStrategy, LineSpaceStrategy},
    ColorStrategy, SpaceStrategy,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Preset {
    pub color: ColorPreset,
    pub space: SpacePreset,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "strategy", content = "options", rename_all = "snake_case")]
pub enum ColorPreset {
    Hue {
        #[serde(flatten)]
        options: HueColorPreset,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(default)]
pub struct HueColorPreset {
    saturation: f32,
    value: f32,
}

impl Default for HueColorPreset {
    fn default() -> Self {
        HueColorPreset {
            saturation: 1.0,
            value: 1.0,
        }
    }
}

impl ColorPreset {
    pub fn to_strategy(&self) -> Box<dyn ColorStrategy + Send> {
        use ColorPreset::*;
        match self {
            Hue { options } => Box::new(HueColorStrategy::new(
                options.saturation,
                options.value,
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "strategy", content = "options", rename_all = "snake_case")]
pub enum SpacePreset {
    Hilbert {
        // #[serde(default = "1024")]
        size: u32,
    },
    Line {
        // #[serde(default = "8192")]
        length: usize,
    },
}

impl SpacePreset {
    pub fn to_strategy(&self) -> Box<dyn SpaceStrategy + Send> {
        use SpacePreset::*;
        match self {
            Hilbert { size } => {
                Box::new(HilbertSpaceStrategy::from_size(*size))
            }
            Line { length } => Box::new(LineSpaceStrategy::new(*length)),
        }
    }
}
