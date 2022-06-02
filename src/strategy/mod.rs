pub mod color;
pub use color::ColorStrategy;

pub mod space;
pub use space::SpaceStrategy;

#[cfg(feature = "binary")]
pub mod preset;
#[cfg(feature = "binary")]
pub use preset::Preset;
