use janus_plugin::JanusPlugin;
use nih_plug::prelude::*;

fn main() {
    nih_export_standalone::<JanusPlugin>();
}
