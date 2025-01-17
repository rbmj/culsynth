#[cfg(not(target_family = "wasm"))]
fn main() {
    nih_plug::prelude::nih_export_standalone::<culsynth_plugin::nih::CulSynthPlugin>();
}

#[cfg(target_family = "wasm")]
fn main() {
    //TODO
}
