#[cfg(feature = "pw-standalone")]
mod pwapp;

#[cfg(not(target_family = "wasm"))]
fn main() {
    #[cfg(feature = "nih-standalone")]
    nih_plug::prelude::nih_export_standalone::<culsynth_plugin::nih::CulSynthPlugin>();
    #[cfg(feature = "pw-standalone")]
    pwapp::run();
}

#[cfg(target_family = "wasm")]
fn main() {
    //TODO
}
