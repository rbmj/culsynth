use nih_plug::prelude::*;
use janus_plugin::JanusPlugin;

fn main() {
    nih_export_standalone::<JanusPlugin>();
}
