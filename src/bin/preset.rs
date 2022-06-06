use serde::{Deserialize, Serialize};

use hilly_sounds::strategy::{
    color::HueColorStrategy,
    space::{HilbertSpaceStrategy, LineSpaceStrategy, SpaceStrategyAdapter},
    ColorStrategy, SpaceStrategy,
};

#[derive(Serialize, Deserialize, Debug, Default)]
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

impl Default for ColorPreset {
    fn default() -> ColorPreset {
        ColorPreset::Hue {
            options: Default::default(),
        }
    }
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
    Hilbert { size: u32 },
    Line { length: usize },
}

impl SpacePreset {
    pub fn to_strategy(&self) -> Box<dyn SpaceStrategy<2> + Send> {
        use SpacePreset::*;
        match self {
            Hilbert { size } => {
                Box::new(HilbertSpaceStrategy::from_size(*size))
            }
            Line { length } => Box::new(SpaceStrategyAdapter::new(
                LineSpaceStrategy::new(*length),
            )),
        }
    }
}

impl Default for SpacePreset {
    fn default() -> SpacePreset {
        SpacePreset::Hilbert { size: 512 }
    }
}
